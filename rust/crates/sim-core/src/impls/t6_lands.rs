//! Tier 6 lands batch (rust/agents/archive/t6-batches.md, `lands`): utility lands of the Tier 6 decks. Cards only: what
//! needs a new engine feature is listed in rust/agents/engine-needs-lands.md.
//!
//! Lands are not permanents: their abilities are offered through the `land_options` / `hand_options` slots, their
//! mana through `land_cols`, `dyn_mana_land` and `on_tap_land` (see impls/lands.rs).

use super::lands::{can_pay_without, pay_without};
use crate::engine::repl;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable, discard_worst, on_cast};
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window_card, counter_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{etgh, pval, threat, untargetable};
use crate::engine::zones::max_by;
use crate::engine::zones::{Tokens, die, make_tokens, min_by};
use crate::flow::Res;
use crate::hooks::Registry;
use crate::hooks::{Action, Opt};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

fn name_of(g: &Game, l: PermId) -> &str {
    &g.db.get(g.perm(l).card()).name
}

// ================================================================== City of Brass
/// **City of Brass**: {T}: any colour (tag c=A); whenever it becomes tapped it deals 1 damage to you.
fn brass_tap(g: &mut Game, p: PlayerId, _l: PermId, _n: u32) -> Res {
    lose_life(g, p, 1, Some(p), "other", Some(true))
}

// ================================================================== City of Traitors
/// **City of Traitors**: {T}: {C}{C} (tag amt=2). When you play another land, sacrifice it. The engine calls
/// `land_etb` for the land just played, so every land without code of its own gets this check (`traitors_check`).
pub(crate) fn traitors_check(g: &mut Game, p: PlayerId, l: PermId) -> Res {
    let mine: Vec<PermId> =
        g.player(p).lands.iter().copied().filter(|&x| x != l && name_of(g, x) == "City of Traitors").collect();
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
/// clause (begin the game with it on the battlefield with a luck counter, exiling a card) is
/// `card_models::caverns_opening`, run by `turn::setup_game`. The luck counter is `DataKey::Counters` (an Int) on
/// the land.
fn caverns_cols(g: &Game, p: PlayerId, l: PermId) -> Colors {
    if g.perm(l).data.int(DataKey::Counters) > 0 { g.player(p).ident } else { Colors::NONE }
}

// ================================================================== the "{1},{T}: any colour" lands
/// **The Mycosynth Gardens, Talon Gates of Madara, Abstergo Entertainment**: {T}: Add {C}. {1},{T}: Add one mana of any
/// colour. A filter for one mana: the land makes any colour only when another untapped land (not one of these) pays
/// the {1}; the count of mana stays right (the land is one mana, the payer another, as if the payer made colourless
/// mana and the land its own colourless mana). With nothing else to pay, colourless only
const ANY_FILTER: [&str; 3] = ["The Mycosynth Gardens", "Talon Gates of Madara", "Abstergo Entertainment"];

fn any_filter_cols(g: &Game, p: PlayerId, l: PermId) -> Colors {
    let payer = g.player(p).lands.iter().any(|&x| x != l && !g.perm(x).tapped && !ANY_FILTER.contains(&name_of(g, x)));
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

fn abstergo(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post == Some(false) || g.perm(l).tapped || !can_pay_without(g, p, l, 3, "") {
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

pub(crate) fn abstergo_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    if !g.player(p).lands.contains(&l) || g.perm(l).tapped || !pay_without(g, p, l, 3, "")? {
        return Ok(false);
    }
    let cd = g.perm(l).card();
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
    if ability_window_card(g, who, c, "4 damage to an attacking or blocking creature", None, Some(t))?
        && g.perm(t).on_bf
    {
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
fn land_untap_gain(g: &mut Game, p: PlayerId, t: PermId) -> i32 {
    let before = total_mana(g, p, false) as i32;
    g.perm_mut(t).tapped = false;
    let after = total_mana(g, p, false) as i32;
    g.perm_mut(t).tapped = true;
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
fn temple(g: &mut Game, l: PermId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(l).tapped || !can_pay_without(g, p, l, 1, "") {
        return Ok(vec![]);
    }
    let mut best: Option<(PermId, i32)> = None;
    for t in g.player(p).lands.clone() {
        if t == l || !g.perm(t).tapped {
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
    let (l, t) = (PermId(l), PermId(t));
    if !g.player(p).lands.contains(&l) || !g.player(p).lands.contains(&t) || g.perm(l).tapped || !g.perm(t).tapped {
        return Ok(false);
    }
    if !pay_without(g, p, l, 1, "")? {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    crate::glog!(g, "  {} uses Deserted Temple: untaps {}", g.player(p).name, name_of(g, t));
    if ability_window_card(g, p, g.perm(l).card(), "untap a land", None, None)? && g.player(p).lands.contains(&t) {
        g.perm_mut(t).tapped = false;
    }
    Ok(true)
}

// ================================================================== Minamo, School at Water's Edge
/// **Minamo, School at Water's Edge**: {T}: Add {U}. {U},{T}: untap target legendary permanent (a legendary land or
/// permanent). The AI untaps a tapped legendary land or permanent when that gives at least 3 more mana (the {U} and
/// Minamo's own mana are the cost): Gaea's Cradle, a legendary mana creature. Arg: Minamo's land id, then the
/// target's kind (1 land, 2 permanent) and id packed above it
fn minamo(g: &mut Game, l: PermId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(l).tapped || !can_pay_without(g, p, l, 0, "U") {
        return Ok(vec![]);
    }
    let mut best: Option<(i64, i32, String)> = None; // (arg, gain, label)
    for t in g.player(p).lands.clone() {
        if t != l && g.perm(t).tapped && g.db.get(g.perm(t).card()).tag(Tag::Leg) {
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
    let l = PermId(l);
    if !g.player(p).lands.contains(&l) || g.perm(l).tapped || !pay_without(g, p, l, 0, "U")? {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    let perm = t & (1 << 31) != 0;
    let t = t & !(1 << 31);
    crate::glog!(g, "  {} uses Minamo: untaps a legendary permanent", g.player(p).name);
    if ability_window_card(g, p, g.perm(l).card(), "untap a legendary permanent", None, None)? {
        if perm {
            let m = PermId(t);
            if g.perm(m).on_bf && g.perm(m).owner == p {
                g.perm_mut(m).tapped = false;
                g.bf_ver += 1;
            }
        } else if g.player(p).lands.contains(&PermId(t)) {
            g.perm_mut(PermId(t)).tapped = false;
        }
    }
    Ok(true)
}

// ================================================================== Emergence Zone
/// the player's one-turn flag that "you may cast spells this turn as though they had flash" (Emergence Zone); the AI's
/// end-of-turn casting must read it (`cast::grants_flash` reads it)
pub const FLASH_FLAG: &str = "as_flash_turn";

/// kept as a switch for the land's AI option
const EMERGENCE_LIVE: bool = true;

/// **Emergence Zone**: {T}: Add {C}. {1},{T}, sacrifice it: you may cast spells this turn as though they had flash.
/// The AI uses it at the end of an opponent's turn when it holds a creature spell it can pay for after the cost
fn emergence(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !EMERGENCE_LIVE || post.is_some() || g.active == Some(p) || g.perm(l).tapped || !can_pay_without(g, p, l, 1, "")
    {
        return Ok(vec![]);
    }
    let spare = total_mana(g, p, false).saturating_sub(2);
    let has = g.player(p).hand.iter().any(|&c| {
        let d = g.db.get(c);
        d.creature && !d.land && {
            let (gn, pips) = crate::engine::mana::cost_of(g, p, c);
            gn + pips.chars().count() as u32 <= spare
        }
    });
    if !has {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0,
        label: "Emergence Zone".into(),
        act: Some(Action::Plan { f: emergence_go, arg: l.0 as i64 }),
    }])
}

pub fn emergence_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    if !g.player(p).lands.contains(&l) || g.perm(l).tapped || !pay_without(g, p, l, 1, "")? {
        return Ok(false);
    }
    let cd = g.perm(l).card();
    super::lands::sac_land(g, p, l)?; // {T} and sacrifice are costs
    crate::glog!(g, "  {} sacrifices Emergence Zone: spells can be cast as though they had flash", g.player(p).name);
    if ability_window_card(g, p, cd, "cast spells as though they had flash", None, None)? {
        let st = g.turn_stamp();
        g.player_mut(p).flag_turn.insert(crate::sym::intern(FLASH_FLAG), st);
    }
    Ok(true)
}

// ================================================================== Forbidden Orchard
/// **Forbidden Orchard**: {T}: one mana of any colour (tag c=A). Whenever you tap it for mana, target opponent creates a
/// 1/1 colorless Spirit creature token: the AI gives it to the opponent who threatens it least
fn orchard_tap(g: &mut Game, p: PlayerId, _l: PermId, _n: u32) -> Res {
    let opps: Vec<PlayerId> = g.opps(p).filter(|&q| g.player(q).alive).collect();
    let Some(q) = min_by(&opps, |q| threat(g, p, q)) else { return Ok(()) };
    crate::glog!(g, "    Forbidden Orchard: {} creates a 1/1 Spirit", g.player(q).name);
    make_tokens(g, q, Tokens { color: Some(Colors::NONE), types: vec!["spirit".into()], ..Tokens::new(1, 1) })?;
    Ok(())
}

// ================================================================== Petrified Hamlet
/// **Petrified Hamlet**: when it enters, choose a land card name. Activated abilities of sources with that name can't
/// be activated unless they are mana abilities; lands with that name have "{T}: Add {C}". The AI names the land of
/// an opponent's that has the most activated abilities of its own (a land with card code in its `land_options`, or a
/// utility land), most copies first. The chosen card is kept on the Hamlet (`DataKey::Imprint`, a `Val::Card`).
/// Nothing yet stops the named lands' abilities, or gives them {T}: Add {C}: engine-needs-lands.md
fn hamlet_etb(g: &mut Game, p: PlayerId, l: PermId) -> Res {
    traitors_check(g, p, l)?;
    let mut seen: Vec<(CardId, u32)> = vec![];
    for q in g.opps(p).collect::<Vec<_>>() {
        for &x in &g.player(q).lands {
            let cd = g.perm(x).card();
            if g.registry
                .get(cd)
                .is_some_and(|i| i.land_options.is_some() || i.land_defend.is_some() || i.land_upkeep.is_some())
            {
                match seen.iter_mut().find(|s| s.0 == cd) {
                    Some(s) => s.1 += 1,
                    None => seen.push((cd, 1)),
                }
            }
        }
    }
    let Some(c) = max_by(&seen, |s| s.1 as f64).map(|s| s.0) else { return Ok(()) };
    g.perm_mut(l).data.set(DataKey::Imprint, Val::Card(c));
    crate::glog!(g, "    Petrified Hamlet names {}", g.db.get(c).name);
    Ok(())
}

/// the land card name p's Petrified Hamlets have chosen (activated abilities of sources with that name can't be
/// activated unless mana abilities), for the engine to read
pub fn hamlet_names(g: &Game) -> Vec<CardId> {
    let mut v = vec![];
    for pl in &g.players {
        if !pl.alive {
            continue;
        }
        for &l in &pl.lands {
            if name_of(g, l) == "Petrified Hamlet"
                && let Some(Val::Card(c)) = g.perm(l).data.get(DataKey::Imprint)
            {
                v.push(*c);
            }
        }
    }
    v
}

// ================================================================== Fomori Vault
/// the artifacts p controls (cards with the artifact type, and Treasure / Clue / Food / Blood tokens)
fn artifacts(g: &Game, p: PlayerId) -> u32 {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            let card_art = x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT));
            let token_art =
                x.token && x.ttypes.iter().any(|t| ["treasure", "clue", "food", "blood", "artifact"].contains(&&**t));
            x.on_bf && (card_art || token_art)
        })
        .count() as u32
}

/// **Fomori Vault**: {T}: Add {C}. {3},{T}, discard a card: look at the top X cards, X the number of artifacts you
/// control; put one into your hand and the rest on the bottom in a random order. The AI does it with 3 spare mana at
/// the end of the turn or after combat, when X is at least 3 (or 2 with a land in hand to throw away)
fn vault_options(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post == Some(false) || g.perm(l).tapped || !can_pay_without(g, p, l, 3, "") || g.player(p).hand.is_empty() {
        return Ok(vec![]);
    }
    let x = artifacts(g, p);
    let spare_land = g.player(p).hand.iter().any(|&c| g.db.get(c).land);
    if x < 3 && !(x == 2 && spare_land) || g.player(p).library.len() < x as usize {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.3 * x as f64,
        label: "Fomori Vault".into(),
        act: Some(Action::Plan { f: vault_go, arg: l.0 as i64 }),
    }])
}

fn vault_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    if !g.player(p).lands.contains(&l)
        || g.perm(l).tapped
        || g.player(p).hand.is_empty()
        || !pay_without(g, p, l, 3, "")?
    {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    discard_worst(g, p, 1)?; // the cost
    crate::glog!(g, "  {} uses Fomori Vault", g.player(p).name);
    if !ability_window_card(g, p, g.perm(l).card(), "look at the top cards, one to hand", None, None)? {
        return Ok(true);
    }
    let n = (artifacts(g, p) as usize).min(g.player(p).library.len());
    let top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
    let Some(pick) = max_by(&top, |c| crate::ai::tutor_value(g, p, c)) else { return Ok(true) };
    let mut rest = top.clone();
    rest.remove(rest.iter().position(|&c| c == pick).unwrap()); // one copy of the pick
    g.rng.shuffle(&mut rest);
    let pl = g.player_mut(p);
    pl.library.splice(0..0, rest);
    pl.hand.push(pick);
    crate::glog!(g, "    Fomori Vault: {} to hand, the rest to the bottom", g.db.get(pick).name);
    Ok(true)
}

// ================================================================== Mount Doom
/// **Mount Doom**: {T}, pay 1 life: Add {B} or {R} (tag c=BR; the life is `mount_tap`, and it is not tapped for mana
/// at 1 life). {1}{B}{R},{T}: 1 damage to each opponent (the AI pings with spare mana at the end of the turn).
/// {5}{B}{R},{T}, sacrifice it and a legendary artifact: choose up to two creatures, destroy the rest (sorcery speed;
/// the AI keeps its own two best creatures and does it when that kills at least 8 more worth of opposing creatures
/// than it loses, after the artifact)
fn mount_mana(g: &Game, p: PlayerId, _l: PermId) -> u32 {
    (g.player(p).life > 1) as u32
}

fn mount_tap(g: &mut Game, p: PlayerId, _l: PermId, _n: u32) -> Res {
    lose_life(g, p, 1, Some(p), "other", None)
}

/// the legendary artifact the wipe sacrifices: the cheapest in value
fn mount_artifact(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            x.on_bf && x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT) && g.db.get(c).tag(Tag::Leg))
        })
        .collect();
    min_by(&cs, |m| pval(g, m))
}

/// (the creatures to keep, the net worth of the wipe) for p: its two best creatures stay
fn mount_wipe_plan(g: &Game, p: PlayerId, art: PermId) -> (Vec<PermId>, f64) {
    let mut mine: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && g.perm(m).on_bf).collect();
    mine.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
    let keep: Vec<PermId> = mine.iter().copied().take(2).collect();
    let lost: f64 = mine.iter().filter(|m| !keep.contains(m)).map(|&m| pval(g, m)).sum();
    let killed: f64 = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && g.perm(m).on_bf && !g.perm(m).phased)
        .map(|m| pval(g, m))
        .sum();
    (keep, killed - lost - pval(g, art))
}

fn mount_options(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if g.perm(l).tapped {
        return Ok(out);
    }
    if g.active == Some(p)
        && post.is_some()
        && let Some(art) = mount_artifact(g, p)
        && can_pay_without(g, p, l, 5, "BR")
    {
        let (_, net) = mount_wipe_plan(g, p, art);
        if net >= 8.0 {
            out.push(Opt {
                utility: net / 4.0,
                label: "Mount Doom destroys all but two creatures".into(),
                act: Some(Action::Plan { f: mount_wipe_go, arg: l.0 as i64 }),
            });
        }
    }
    if post.is_none() && can_pay_without(g, p, l, 1, "BR") {
        let opps = g.opps(p).filter(|&q| g.player(q).alive).count();
        out.push(Opt {
            utility: 0.3 + 0.1 * opps as f64,
            label: "Mount Doom pings".into(),
            act: Some(Action::Plan { f: mount_ping_go, arg: l.0 as i64 }),
        });
    }
    Ok(out)
}

fn mount_ping_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    if !g.player(p).lands.contains(&l) || g.perm(l).tapped || !pay_without(g, p, l, 1, "BR")? {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    crate::glog!(g, "  {} uses Mount Doom: 1 damage to each opponent", g.player(p).name);
    if ability_window_card(g, p, g.perm(l).card(), "1 damage to each opponent", None, None)? {
        for q in g.opps(p).filter(|&q| g.player(q).alive).collect::<Vec<_>>() {
            lose_life(g, q, 1, Some(p), "burn", None)?;
        }
    }
    Ok(true)
}

fn mount_wipe_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    let Some(art) = mount_artifact(g, p) else { return Ok(false) };
    if !g.player(p).lands.contains(&l) || g.perm(l).tapped || !pay_without(g, p, l, 5, "BR")? {
        return Ok(false);
    }
    let (keep, _) = mount_wipe_plan(g, p, art);
    let cd = g.perm(l).card();
    super::lands::sac_land(g, p, l)?; // {T} and sacrifice are costs
    crate::glog!(g, "  {} sacrifices Mount Doom and {}", g.player(p).name, g.perm(art).name);
    die(g, art, "sac")?;
    if ability_window_card(g, p, cd, "destroy all creatures but two", None, None)? {
        crate::glog!(
            g,
            "    Mount Doom: all creatures but {} destroyed",
            keep.iter().map(|&m| g.perm(m).name.to_string()).collect::<Vec<_>>().join(", ")
        );
        let all: Vec<PermId> = g
            .players
            .iter()
            .flat_map(|pl| pl.perms.iter().copied())
            .filter(|&m| g.is_creature(m) && g.perm(m).on_bf && !keep.contains(&m))
            .collect();
        for m in all {
            if g.perm(m).on_bf {
                apply_removal(g, Some(p), m, "destroy", None)?;
            }
        }
    }
    Ok(true)
}

// ================================================================== modal double-faced spell // land
/// The back faces of Shatterskull Smashing and Razorgrass Ambush are lands, but the card data has them as spells (the
/// engine's land play only takes cards with the land flag). So the AI plays them as lands through `hand_options`: a
/// land option when it still has its land drop, holds no other land and has fewer than 4 lands. The land comes in
/// untapped only if the AI pays 3 life (`turn::pays_for_land`, as for any pay-3-life land); the colour comes from
/// `land_cols`. See engine-needs-lands.md for the missing engine side (the normal land play, the human's choices).
const SHATTERSKULL: [&str; 2] = ["Shatterskull Smashing", "Shatterskull Smashing // Shatterskull, the Hammer Pass"];
const RAZORGRASS: &str = "Razorgrass Ambush // Razorgrass Field";

pub(crate) fn mdfc_cols(g: &Game, _p: PlayerId, l: PermId) -> Colors {
    Colors::from_letters(if name_of(g, l) == RAZORGRASS { "W" } else { "R" })
}

pub(crate) fn mdfc_land_opt(g: &Game, c: CardId, p: PlayerId, post: Option<bool>) -> Option<Opt> {
    let pl = g.player(p);
    let allowed = 1 + crate::cardcode::extra_lands(g, p) + pl.extra_land_now;
    if post.is_none() || g.active != Some(p) || pl.lands_played >= allowed || !pl.hand.contains(&c) {
        return None;
    }
    if pl.lands.len() >= 4 || pl.hand.iter().any(|&x| g.db.get(x).land) {
        return None;
    }
    Some(Opt {
        utility: 2.5,
        label: format!("play {} as a land", g.db.get(c).name),
        act: Some(Action::Plan { f: mdfc_land_go, arg: c.0 as i64 }),
    })
}

pub(crate) fn mdfc_land_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let allowed = 1 + crate::cardcode::extra_lands(g, p) + g.player(p).extra_land_now;
    let Some(i) = g.player(p).hand.iter().position(|&x| x == c) else { return Ok(false) };
    if g.player(p).lands_played >= allowed {
        return Ok(false);
    }
    g.player_mut(p).hand.remove(i);
    // "As this land enters, you may pay 3 life. If you don't, it enters tapped."
    let paid = crate::engine::turn::pays_for_land(g, p, c, 3);
    if paid {
        lose_life(g, p, 3, Some(p), "other", None)?;
    }
    crate::engine::turn::play_land_card(g, p, c, "plays")?;
    if let Some(&l) = g.player(p).lands.last().filter(|&&l| g.perm(l).card() == c) {
        if !paid {
            g.perm_mut(l).tapped = true;
        }
        traitors_check(g, p, l)?;
    }
    Ok(true)
}

// ================================================================== Razorgrass Ambush
/// **Razorgrass Ambush** ({1}{W} instant): 3 damage to target attacking or blocking creature. Cast as the defender
/// against an attacker, or as the attacker against a blocker, when 3 damage kills a creature worth at least 4 (the
/// same rule as Eiganjo's channel)
fn razorgrass_cast(g: &mut Game, c: CardId, who: PlayerId, t: PermId) -> Res<bool> {
    if !g.player(who).hand.contains(&c) || !castable(g, who, c, "hand") {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, who, c);
    if !can_pay(g, who, gn, &pips, false) || !pay(g, who, gn, &pips, false)? {
        return Ok(false);
    }
    let i = g.player(who).hand.iter().position(|&x| x == c).unwrap();
    g.player_mut(who).hand.remove(i);
    crate::glog!(g, "  {} casts Razorgrass Ambush: 3 damage to {}", g.player(who).name, g.perm(t).name);
    on_cast(g, who, c)?;
    if !counter_window(g, who, c, 4.0, vec![])? {
        g.player_mut(who).gy.push(c);
        return Ok(true);
    }
    if g.perm(t).on_bf {
        apply_removal(g, Some(who), t, "dmg3", Some(c))?;
    }
    g.player_mut(who).gy.push(c);
    Ok(true)
}

fn razorgrass_defend(
    g: &mut Game,
    c: CardId,
    d: PlayerId,
    p: PlayerId,
    atk: &mut Vec<PermId>,
    _assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    let (gn, pips) = cost_of(g, d, c);
    if !can_pay(g, d, gn, &pips, false) {
        return Ok(());
    }
    let mine: Vec<PermId> = atk.iter().copied().filter(|&a| g.perm(a).owner == p).collect();
    let Some(t) = shot_target(g, d, &mine, 3) else { return Ok(()) };
    if razorgrass_cast(g, c, d, t)? && !g.perm(t).on_bf {
        atk.retain(|&a| a != t);
    }
    Ok(())
}

fn razorgrass_blocks(
    g: &mut Game,
    c: CardId,
    p: PlayerId,
    _atk: &[PermId],
    _d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn, &pips, false) {
        return Ok(());
    }
    let blockers: Vec<PermId> = assign.iter().map(|x| x.1).collect();
    let Some(t) = shot_target(g, p, &blockers, 3) else { return Ok(()) };
    if razorgrass_cast(g, c, p, t)? && !g.perm(t).on_bf {
        assign.retain(|x| x.1 != t);
    }
    Ok(())
}

fn razorgrass_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(mdfc_land_opt(g, c, p, post).into_iter().collect())
}

// ================================================================== Shatterskull Smashing
/// **Shatterskull Smashing** ({X}{R}{R} sorcery): X damage divided as you choose among up to two target creatures
/// and/or planeswalkers; if X is 6 or more, twice X instead. The AI picks the X that kills the most valuable one or
/// two opposing creatures / planeswalkers (the smallest X that reaches the best total value), and divides the damage
/// as exactly lethal to each; with no kill worth 5 in value it keeps the card (or plays it as a land when short of lands)
pub fn smash_damage(x: i32) -> i32 {
    if x >= 6 { 2 * x } else { x }
}

/// the damage a creature (its toughness) or planeswalker (its loyalty) needs to die, if damage can hurt it
fn smash_need(g: &Game, m: PermId) -> Option<i32> {
    let x = g.perm(m);
    if !x.on_bf || x.phased || untargetable(g, m) {
        return None;
    }
    if g.is_creature(m) {
        return if repl::no_damage(g, m) { None } else { Some(etgh(g, m).max(1)) };
    }
    x.loyalty.map(|l| l.max(1))
}

/// (target, damage it gets, value): the best one or two opposing targets that `dmg` kills, worth at least 3 each
fn smash_targets(g: &Game, p: PlayerId, dmg: i32) -> Vec<(PermId, i32, f64)> {
    let cs: Vec<(PermId, i32, f64)> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter_map(|m| smash_need(g, m).map(|n| (m, n, pval(g, m))))
        .filter(|&(_, n, v)| n <= dmg && v >= 3.0)
        .collect();
    let mut best: Vec<(PermId, i32, f64)> = vec![];
    let mut best_v = 0.0;
    for (i, &a) in cs.iter().enumerate() {
        if a.2 > best_v {
            best = vec![a];
            best_v = a.2;
        }
        for &b in &cs[i + 1..] {
            if a.1 + b.1 <= dmg && a.2 + b.2 > best_v {
                best = vec![a, b];
                best_v = a.2 + b.2;
            }
        }
    }
    best
}

fn smash_plan(g: &Game, p: PlayerId) -> Option<(i32, f64)> {
    let spare = total_mana(g, p, false) as i32 - 2;
    let mut best: Option<(i32, f64)> = None;
    for x in 1..=spare.max(0) {
        if !can_pay(g, p, x as u32, "RR", false) {
            break;
        }
        let score: f64 = smash_targets(g, p, smash_damage(x)).iter().map(|t| t.2).sum();
        if score >= 5.0 && best.is_none_or(|b| score > b.1 + 0.01) {
            best = Some((x, score));
        }
    }
    best
}

fn smash_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut o: Vec<Opt> = mdfc_land_opt(g, c, p, post).into_iter().collect();
    if post.is_some()
        && g.active == Some(p)
        && let Some((x, score)) = smash_plan(g, p)
    {
        o.push(Opt {
            utility: 1.5 + 0.2 * score.min(30.0),
            label: format!("Shatterskull Smashing (X = {x})"),
            act: Some(Action::Plan { f: smash_go, arg: c.0 as i64 }),
        });
    }
    Ok(o)
}

fn smash_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    let Some((x, _)) = smash_plan(g, p) else { return Ok(false) };
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn + x as u32, &pips, false) || !pay(g, p, gn + x as u32, &pips, false)? {
        return Ok(false);
    }
    cast_card(g, p, c, "hand", Ctx { x, ..Ctx::default() })?;
    Ok(true)
}

fn smash_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let dmg = smash_damage(ctx.x.max(0));
    let tg = smash_targets(g, p, dmg);
    crate::glog!(g, "    Shatterskull Smashing (X = {}, {dmg} damage)", ctx.x);
    for (m, need, _) in tg {
        if !g.perm(m).on_bf {
            continue;
        }
        if g.is_creature(m) {
            apply_removal(g, Some(p), m, &format!("dmg{need}"), None)?;
        } else {
            let l = g.perm(m).loyalty.unwrap_or(0) - need;
            g.perm_mut(m).loyalty = Some(l.max(0));
            if l <= 0 {
                die(g, m, "sba")?;
            }
        }
    }
    crate::engine::life::check_state(g)?;
    Ok("gy")
}

// ================================================================== Talon Gates of Madara
/// **Talon Gates of Madara**: when it enters, up to one target creature phases out (the AI phases out the most
/// valuable opposing creature, worth at least 3: it is gone until its controller's next untap). {T}: Add {C}.
/// {1},{T}: any colour (`any_filter_cols`). {4}: put it from your hand onto the battlefield (instant speed): the AI
/// does it in its own main phase when the land drop is spent (to phase out a creature before combat), and as the
/// defender against an attacker worth at least 5 (phasing it out of the attack)
fn gates_phase(g: &mut Game, p: PlayerId, l: PermId) -> Res<Option<PermId>> {
    let Some(t) = super::common::best_opp_creature(g, p, |m| pval(g, m) >= 3.0) else { return Ok(None) };
    if ability_window_card(g, p, g.perm(l).card(), "target creature phases out", None, Some(t))? && g.perm(t).on_bf {
        g.perm_mut(t).phased = true;
        g.bf_ver += 1;
        crate::glog!(g, "    Talon Gates of Madara: {} phases out", g.perm(t).name);
        return Ok(Some(t));
    }
    Ok(None)
}

/// played from hand (the land drop): the City of Traitors check and the enter trigger
fn gates_etb(g: &mut Game, p: PlayerId, l: PermId) -> Res {
    traitors_check(g, p, l)?;
    gates_phase(g, p, l)?;
    Ok(())
}

/// {4}: put Gates onto the battlefield (not a land play: no land drop, no "play another land" trigger); its enter trigger
/// phases out `target` when it is still the best choice, else the best opposing creature
fn gates_put(g: &mut Game, p: PlayerId, c: CardId) -> Res<Option<PermId>> {
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 4, "", false) || !pay(g, p, 4, "", false)? {
        return Ok(None);
    }
    let i = g.player(p).hand.iter().position(|&x| x == c).unwrap();
    g.player_mut(p).hand.remove(i);
    let l = g.add_land(p, c, false);
    crate::glog!(g, "  {} puts Talon Gates of Madara onto the battlefield", g.player(p).name);
    let t = gates_phase(g, p, l)?;
    crate::engine::zones::landfall(g, p)?;
    Ok(t)
}

fn gates_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let pl = g.player(p);
    let allowed = 1 + crate::cardcode::extra_lands(g, p) + pl.extra_land_now;
    if post != Some(false) || g.active != Some(p) || pl.lands_played < allowed || !pl.hand.contains(&c) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 4, "", false) || super::common::best_opp_creature(g, p, |m| pval(g, m) >= 3.0).is_none() {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0,
        label: "put Talon Gates onto the battlefield".into(),
        act: Some(Action::Plan { f: gates_put_go, arg: c.0 as i64 }),
    }])
}

fn gates_put_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    Ok(gates_put(g, p, CardId(arg as u16))?.is_some() || !g.player(p).hand.contains(&CardId(arg as u16)))
}

fn gates_defend(
    g: &mut Game,
    c: CardId,
    d: PlayerId,
    p: PlayerId,
    atk: &mut Vec<PermId>,
    _assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if !g.player(d).hand.contains(&c) || !can_pay(g, d, 4, "", false) {
        return Ok(());
    }
    let mine: Vec<PermId> =
        atk.iter().copied().filter(|&a| g.perm(a).owner == p && g.perm(a).on_bf && !untargetable(g, a)).collect();
    let Some(big) = max_by(&mine, |a| pval(g, a)).filter(|&a| pval(g, a) >= 5.0) else { return Ok(()) };
    if !pay(g, d, 4, "", false)? {
        return Ok(());
    }
    let i = g.player(d).hand.iter().position(|&x| x == c).unwrap();
    g.player_mut(d).hand.remove(i);
    let l = g.add_land(d, c, false);
    crate::glog!(g, "  {} puts Talon Gates of Madara onto the battlefield", g.player(d).name);
    if ability_window_card(g, d, c, "target creature phases out", None, Some(big))? && g.perm(big).on_bf {
        g.perm_mut(big).phased = true;
        g.bf_ver += 1;
        atk.retain(|&a| a != big);
        crate::glog!(g, "    Talon Gates of Madara: {} phases out", g.perm(big).name);
    }
    let _ = l;
    crate::engine::zones::landfall(g, d)
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
    r.card(db, "Emergence Zone")?.land_options = Some(emergence);
    r.card(db, "Forbidden Orchard")?.on_tap_land = Some(orchard_tap);
    r.card(db, "Petrified Hamlet")?.land_etb = Some(hamlet_etb);
    r.card(db, "Fomori Vault")?.land_options = Some(vault_options);
    let m = r.card(db, "Mount Doom")?;
    m.dyn_mana_land = Some(mount_mana);
    m.on_tap_land = Some(mount_tap);
    m.land_options = Some(mount_options);
    let rz = r.card(db, RAZORGRASS)?;
    rz.land_cols = Some(mdfc_cols);
    rz.hand_options = Some(razorgrass_options);
    rz.hand_defend = Some(razorgrass_defend);
    rz.hand_blocks = Some(razorgrass_blocks);
    for n in SHATTERSKULL {
        let s = r.card(db, n)?;
        s.land_cols = Some(mdfc_cols);
        s.hand_options = Some(smash_options);
        s.resolve = Some(smash_resolve);
    }
    let tg = r.card(db, "Talon Gates of Madara")?;
    tg.land_etb = Some(gates_etb);
    tg.hand_options = Some(gates_options);
    tg.hand_defend = Some(gates_defend);
    // REGISTER-HERE
    Ok(())
}
