//! Karn deck (key `karn`): the card draw and engines. Card verdicts are in data/card_overrides.toml; each card's
//! doc comment says how it is modeled and what is left out.
//!
//! - **Karn, Scion of Urza** (walker framework): +1 reveals the top two cards, an opponent chooses which one you get and
//!   the other is exiled with a silver counter (`Player::silver`); -1 returns a silver-counter card; -2 makes a Construct.
//! - **Mazemind Tome**, **Urza's Tome**: activated abilities written as options (page counters; the Tome's discard-unless-
//!   exile-a-historic-card).
//! - **Mirrorworks**: an artifact trigger that pays {2} for a token copy.
//! - **Mystic Forge**: the top card is cast through the main-phase options; the exile ability digs past a land.
//! - **Urza's Blueprints**: the echo upkeep payment (the draw ability is compiled).

use crate::cards::{CardDb, Types};
use crate::engine::cast::discard_worst;
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::stack::{ability_window, ability_window_card, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{epow, pval};
use crate::engine::zones::Zone;
use crate::engine::zones::{Tokens, die, draw, enter_token_copy, leave, make_tokens, max_by, min_by, to_zone_card};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{WalkerAb, walker};
use crate::state::{Ctx, DataKey, Game, PermData, Val};
use crate::tag::Tag;

pub const KARN_SCION: &str = "Karn, Scion of Urza";
pub const MAZEMIND: &str = "Mazemind Tome";
pub const MIRRORWORKS: &str = "Mirrorworks";
pub const FORGE: &str = "Mystic Forge";
pub const BLUEPRINTS: &str = "Urza's Blueprints";
pub const TOME: &str = "Urza's Tome";

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

fn pname(g: &Game, p: PlayerId) -> String {
    g.player(p).name.to_string()
}

fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

fn art_card(g: &Game, c: CardId) -> bool {
    g.db.get(c).types.has(Types::ARTIFACT)
}

/// a historic card: an artifact, a legendary card or a Saga
pub fn historic(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.types.has(Types::ARTIFACT) || d.tag(Tag::Leg) || d.has_subtype("saga")
}

fn artifacts(g: &Game, p: PlayerId) -> i32 {
    g.player(p).perms.iter().filter(|&&x| live(g, x) && crate::impls::t5::is_artifact(g, x)).count() as i32
}

/// an ability that isn't a mana ability is stopped (Pithing Needle, Collector Ouphe ...)
fn locked(g: &Game, src: PermId, p: PlayerId) -> bool {
    crate::cardcode::ability_locked(g, src, p)
}

// ================================================================== Karn, Scion of Urza
/// the worth of a card to p's AI, a land counting only while it is wanted
fn worth(g: &Game, p: PlayerId, c: CardId) -> f64 {
    card_worth(g, p, c, false)
}

fn karn_plus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    (!g.player(p).library.is_empty()).then_some(3.4)
}

/// +1: reveal the top two cards; an opponent chooses one for your hand, the other is exiled with a silver counter.
/// The opponent's AI gives you the card you want less (`card_worth`). With one card left it is the one chosen.
fn karn_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let mut top: Vec<CardId> = (0..2).filter_map(|_| g.player_mut(p).library.pop()).collect();
    if top.is_empty() {
        return Ok(());
    }
    top.reverse(); // the top card first
    let give =
        if top.len() == 2 { if worth(g, p, top[1]) < worth(g, p, top[0]) { top[1] } else { top[0] } } else { top[0] };
    for &c in &top {
        if c == give {
            let pl = g.player_mut(p);
            pl.hand.push(c);
            pl.seen.insert(c);
            crate::glog!(g, "    Karn: an opponent gives {} the {}", pname(g, p), g.db.get(c).name);
        } else {
            let pl = g.player_mut(p);
            pl.exile.push(c);
            pl.silver.push(c);
            crate::glog!(g, "    Karn: {} is exiled with a silver counter", g.db.get(c).name);
        }
    }
    Ok(())
}

fn best_silver(g: &Game, p: PlayerId) -> Option<CardId> {
    let cs: Vec<CardId> = g.player(p).silver.iter().copied().filter(|c| g.player(p).exile.contains(c)).collect();
    max_by(&cs, |c| worth(g, p, c))
}

fn karn_minus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    let c = best_silver(g, p)?;
    if g.perm(src).loyalty.unwrap_or(0) < 2 {
        return None;
    }
    let hand = g.player(p).hand.len() as f64;
    Some((2.0 + worth(g, p, c) / 20.0 - 0.3 * (hand - 3.0).max(0.0)).max(0.5))
}

/// -1: put a card you own with a silver counter on it from exile into your hand (the best one)
fn karn_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(c) = best_silver(g, p) {
        let pl = g.player_mut(p);
        pl.silver.retain(|&x| x != c);
        if let Some(i) = pl.exile.iter().position(|&x| x == c) {
            pl.exile.remove(i);
        }
        pl.hand.push(c);
        crate::glog!(g, "    Karn returns {} from exile", g.db.get(c).name);
    }
    Ok(())
}

fn karn_construct_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    if g.perm(src).loyalty.unwrap_or(0) < 3 {
        return None;
    }
    Some(0.9 + 0.35 * (artifacts(g, p) + 1) as f64)
}

/// the Construct token: 0/0 colourless artifact creature, +1/+1 for each artifact you control (t5's `construct_bonus`)
pub fn make_construct(g: &mut Game, p: PlayerId) -> Res {
    g.selfpt = true; // before the token exists: a 0/0 is checked at once
    let mut data = PermData::default();
    data.set(DataKey::Construct, Val::Bool(true));
    data.set(DataKey::Artifact, Val::Bool(true));
    let spec = Tokens {
        tgh: Some(0),
        color: Some(crate::cards::Colors::NONE),
        types: vec!["construct"],
        data,
        ..Tokens::new(1, 0)
    };
    make_tokens(g, p, spec)?;
    Ok(())
}

/// -2: a Construct token
fn karn_construct(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    make_construct(g, p)
}

static KARN_ABILITIES: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "reveal two, an opponent chooses", val: karn_plus_val, eff: karn_plus },
    WalkerAb { delta: -1, label: "return a silver-counter card", val: karn_minus_val, eff: karn_minus },
    WalkerAb { delta: -2, label: "Construct token", val: karn_construct_val, eff: karn_construct },
];

// ================================================================== Mazemind Tome
fn pages(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Counters)
}

/// the fourth page counter: exile the Tome and gain 4 life
fn tome_page(g: &mut Game, src: PermId, p: PlayerId) -> Res {
    let n = pages(g, src) + 1;
    g.perm_mut(src).data.set(DataKey::Counters, Val::Int(n));
    if n >= 4 && live(g, src) {
        leave(g, src)?;
        to_zone_card(g, src, Zone::Exile);
        crate::glog!(g, "    Mazemind Tome is exiled: {} gains 4 life", pname(g, p));
        gain(g, p, 4)?;
    }
    Ok(())
}

/// {T}, put a page counter: scry 1. The AI scrys only when it can't draw (the draw is the better use of a page).
/// {2}, {T}, put a page counter: draw a card, a stop at the fourth page counter (exile, gain 4).
fn mazemind_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if owner(g, src) != p || !live(g, src) || g.perm(src).tapped || locked(g, src, p) {
        return Ok(out);
    }
    let hand = g.player(p).hand.len() as f64;
    let lib = g.player(p).library.len();
    if can_pay(g, p, 2, "", false) && lib > 8 {
        let u = 1.3 + if hand <= 2.0 { 1.0 } else { 0.0 } - 0.25 * (hand - 4.0).max(0.0);
        let u = if post.is_none() { u + 0.6 } else { u };
        out.push(ability(u, "Mazemind Tome: draw".into(), src, mazemind_go, 1));
    } else if lib > 8 && pages(g, src) < 3 && post.is_some() {
        out.push(ability(0.2, "Mazemind Tome: scry".into(), src, mazemind_go, 0));
    }
    Ok(out)
}

fn mazemind_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    if arg == 1 {
        if !can_pay(g, p, 2, "", false) || !pay(g, p, 2, "", false)? {
            return Ok(false);
        }
    }
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), if arg == 1 { "draw a card" } else { "scry 1" }, None, None)? {
        return Ok(true);
    }
    if arg == 1 {
        draw(g, p, 1, false)?;
    } else {
        crate::cardcode::scry(g, p, 1, false)?;
    }
    tome_page(g, src, p)?;
    Ok(true)
}

// ================================================================== Urza's Tome
/// {3}, {T}: draw a card, then discard a card unless you exile a historic card from your graveyard. The AI exiles
/// the historic card it would miss least (not a creature a Portal to Phyrexia could reanimate, if it has another).
fn urzas_tome_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if owner(g, src) != p || !live(g, src) || g.perm(src).tapped || locked(g, src, p) {
        return Ok(out);
    }
    if g.player(p).library.len() <= 8 || !can_pay(g, p, 3, "", false) {
        return Ok(out);
    }
    let hand = g.player(p).hand.len() as f64;
    let free = g.player(p).gy.iter().any(|&c| historic(g, c));
    let mut u =
        1.1 + if free { 0.5 } else { -0.3 } + if hand <= 2.0 { 1.0 } else { 0.0 } - 0.25 * (hand - 4.0).max(0.0);
    if post.is_none() {
        u += 0.6;
    }
    out.push(ability(u, "Urza's Tome: draw".into(), src, urzas_tome_go, 0));
    Ok(out)
}

fn urzas_tome_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped || !can_pay(g, p, 3, "", false) {
        return Ok(false);
    }
    if !pay(g, p, 3, "", false)? {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "draw a card, then discard unless you exile a historic card", None, None)? {
        return Ok(true);
    }
    draw(g, p, 1, false)?;
    let hist: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| historic(g, c)).collect();
    if hist.is_empty() {
        discard_worst(g, p, 1)?;
        return Ok(true);
    }
    // keep reanimation targets (creatures) in the graveyard when something else will do
    let pick = min_by(&hist, |c| {
        let d = g.db.get(c);
        d.cmc as f64 + if d.creature { 10.0 } else { 0.0 }
    })
    .unwrap();
    let pl = g.player_mut(p);
    if let Some(i) = pl.gy.iter().position(|&x| x == pick) {
        pl.gy.remove(i);
    }
    pl.exile.push(pick);
    crate::glog!(g, "    Urza's Tome: {} exiles {} from the graveyard", pname(g, p), g.db.get(pick).name);
    Ok(true)
}

// ================================================================== Mirrorworks
/// What a token copy of artifact `m` is worth ({2} is paid for it): rocks and creatures, not another legend or a
/// Walking Ballista / Hangarback Walker that enters with no counters.
fn mirror_worth(g: &Game, m: PermId) -> f64 {
    let Some(c) = g.perm(m).cd else { return 0.0 };
    let d = g.db.get(c);
    if d.tag(Tag::Leg) || matches!(&*d.name, "Walking Ballista" | "Hangarback Walker" | "Mishra's Self-Replicator") {
        return 0.0;
    }
    let mut v = 0.0;
    if d.tag(Tag::Rock) {
        v += 2.0;
    }
    if d.creature {
        v += 1.0 + d.pow.max(0) as f64 * 0.4;
    }
    v += d.cmc as f64 * 0.3 + 1.0;
    v
}

/// Whenever another nontoken artifact you control enters, you may pay {2}. If you do, create a token that's a copy of
/// that artifact. (Nontoken artifact lands aren't spells' permanents here: Darksteel Citadel is not copied.)
fn mirror_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m == src
        || !live(g, src)
        || owner(g, m) != o
        || g.perm(m).token
        || !g.perm(m).on_bf
        || g.perm(src).data.truthy(DataKey::Act)
    {
        return Ok(());
    }
    let Some(c) = g.perm(m).cd else { return Ok(()) };
    if !art_card(g, c) || mirror_worth(g, m) < 1.0 || !can_pay(g, o, 2, "", false) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "pay {2}: a token copy of the artifact", None)? || !can_pay(g, o, 2, "", false)
    {
        return Ok(());
    }
    if !pay(g, o, 2, "", false)? {
        return Ok(());
    }
    crate::glog!(g, "    Mirrorworks copies {}", g.db.get(c).name);
    g.perm_mut(src).data.set(DataKey::Act, Val::Bool(true)); // the copy entering is not copied
    let r = enter_token_copy(g, o, c);
    g.perm_mut(src).data.set(DataKey::Act, Val::Bool(false));
    r?;
    Ok(())
}

// ================================================================== Mystic Forge
/// the top card of p's library, if the Forge lets p cast it: an artifact or colourless nonland card
fn forge_top(g: &Game, p: PlayerId) -> Option<CardId> {
    let top = *g.player(p).library.last()?;
    let d = g.db.get(top);
    (!d.land && (d.types.has(Types::ARTIFACT) || d.pips.is_empty())).then_some(top)
}

fn forge_up(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == FORGE))
}

/// You may cast artifact spells and colorless spells from the top of your library: the AI casts the top card when it
/// would cast it from hand (`card_utility`, plus a little for the free card). X spells, removal and wipes (which need
/// targets chosen by their own options) and counterspells are left to the hand. {T}, pay 1 life: exile the top card of
/// your library: the AI digs past a land it can't use (a land on top is never cast).
fn forge_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if owner(g, src) != p || !live(g, src) || forge_up(g, p) != Some(src) {
        return Ok(out); // one option set per player, from the first Forge
    }
    let Some(&top) = g.player(p).library.last() else { return Ok(out) };
    let d = g.db.get(top);
    if let Some(top) = forge_top(g, p)
        && post.is_some()
        && !(d.tag(Tag::Rem) || d.tag(Tag::Wipe) || d.tag(Tag::X) || d.tag(Tag::Tokx) || d.tag(Tag::Ctr))
        && !(d.tag(Tag::Rem) && !d.has_dsl())
    {
        let s = crate::ai::brain::Situation::new(g, p);
        if let Some(u) = crate::ai::brain::card_utility(g, p, &s, top) {
            let (cg, cp) = cost_of(g, p, top);
            g.pay_for = Some(top);
            let ok =
                can_pay(g, p, cg, &cp, false) && (!g.cast_checks() || crate::engine::cast::castable(g, p, top, "lib"));
            g.pay_for = None;
            if ok {
                out.push(Opt {
                    utility: u + 0.6,
                    label: format!("Mystic Forge: cast {}", d.name),
                    act: Some(Action::Plan { f: forge_cast, arg: top.0 as i64 }),
                });
            }
        }
    } else if d.land
        && !g.perm(src).tapped
        && !locked(g, src, p)
        && g.player(p).life > 8
        && g.player(p).library.len() > 12
    {
        let pl = g.player(p);
        let in_hand = pl.hand.iter().filter(|&&c| g.db.get(c).land).count();
        // lands are only drawn, never cast from the top: skip one when the mana is there and a land waits in the hand
        if pl.lands.len() + in_hand >= 5 && post.is_some() {
            out.push(ability(0.9, "Mystic Forge: exile the top land".into(), src, forge_exile, 0));
        }
    }
    Ok(out)
}

fn forge_cast(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let top = CardId(arg as u16);
    if g.player(p).library.last() != Some(&top) || forge_up(g, p).is_none() {
        return Ok(false);
    }
    g.player_mut(p).library.pop();
    crate::glog!(g, "  {} casts {} from the top of the library (Mystic Forge)", pname(g, p), g.db.get(top).name);
    let ok = crate::ai::brain::do_cast_with(g, p, top, Some("lib"), Ctx::default())?;
    if !ok {
        g.player_mut(p).library.push(top);
    }
    Ok(ok)
}

fn forge_exile(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || g.perm(src).tapped || g.player(p).library.is_empty() || g.player(p).life <= 1 {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    lose_life(g, p, 1, Some(p), "life", None)?;
    if !ability_window(g, p, Some(src), "exile the top card of your library", None, None)? {
        return Ok(true);
    }
    if let Some(c) = g.player_mut(p).library.pop() {
        g.player_mut(p).exile.push(c);
        crate::glog!(g, "    Mystic Forge exiles {}", g.db.get(c).name);
    }
    Ok(true)
}

// ================================================================== Urza's Blueprints
/// Echo {6}: at your first upkeep after it came under your control, sacrifice it unless you pay {6}. The AI pays when
/// the mana is there and a card a turn is worth the turn's mana (it has few cards in hand, or the mana is spare). The
/// card itself is the compiled `{T}: draw a card`, used the turn it is cast whatever the echo does.
fn blueprints_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !live(g, src) || g.perm(src).data.get(DataKey::Echo) == Some(&Val::Str("done")) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "echo {6}: sacrifice it unless you pay", Some(2.0))? {
        g.perm_mut(src).data.set(DataKey::Echo, Val::Str("done"));
        return Ok(());
    }
    let total = total_mana(g, p, false);
    let hand = g.player(p).hand.len();
    let pays = can_pay(g, p, 6, "", false) && g.player(p).library.len() > 12 && (total >= 9 || hand <= 2);
    g.perm_mut(src).data.set(DataKey::Echo, Val::Str("done"));
    if pays && pay(g, p, 6, "", false)? {
        crate::glog!(g, "    {} pays {{6}} for Urza's Blueprints' echo", pname(g, p));
    } else if live(g, src) {
        crate::glog!(g, "    {} doesn't pay Urza's Blueprints' echo", pname(g, p));
        die(g, src, "sac")?;
    }
    Ok(())
}

// ================================================================== land search (Urza's Cave, Expedition Map)
const TRON: [&str; 3] = ["Urza's Mine", "Urza's Power Plant", "Urza's Tower"];

fn lname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// how much p's AI wants land card c from a search (battlefield and hand count as held): the missing Tron piece first
/// (the last one is worth the most), then Mishra's Workshop, Ancient Tomb, Urza's Workshop / Saga, a third Wastes for
/// Endless Atlas, any other land
pub fn land_want(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let pl = g.player(p);
    let held: Vec<&str> =
        pl.lands.iter().map(|&l| lname(g, g.land(l).cd)).chain(pl.hand.iter().map(|&h| lname(g, h))).collect();
    let name = lname(g, c);
    let n = |x: &str| held.iter().filter(|&&h| h == x).count();
    if TRON.contains(&name) {
        if n(name) > 0 {
            return 1.0;
        }
        let have = TRON.iter().filter(|&&t| n(t) > 0).count();
        return if have >= 2 { 12.0 } else { 7.0 };
    }
    match name {
        "Mishra's Workshop" if n(name) == 0 => 8.0,
        "Ancient Tomb" if n(name) == 0 => 6.0,
        "Urza's Workshop" | "Urza's Saga" if n(name) == 0 => 5.0,
        "Mishra's Factory" | "Mishra's Foundry" if n(name) == 0 => 4.0,
        "Wastes" => 3.0 + if n("Wastes") == 2 { 1.0 } else { 0.0 },
        _ => 2.0,
    }
}

/// search p's library for a land card (any land, not only a basic) and put it onto the battlefield (tapped or not) or
/// into the hand, then shuffle; the AI takes the one it wants most (`land_want`). The card, if one was found.
pub fn fetch_land(g: &mut Game, p: PlayerId, to_hand: bool, tapped: bool) -> Res<Option<CardId>> {
    let c = if crate::human::is_human(g, p) {
        let got = crate::human::choices::search(
            g,
            p,
            &|g: &Game, c: CardId| g.db.get(c).land,
            1,
            "Search for a land card",
            true,
        )?;
        got.first().copied()
    } else {
        let cands: Vec<CardId> =
            crate::engine::zones::searchable(g, p).into_iter().filter(|&c| g.db.get(c).land).collect();
        let pick = max_by(&cands, |c| land_want(g, p, c));
        if let Some(c) = pick {
            let lib = &mut g.player_mut(p).library;
            if let Some(i) = lib.iter().position(|&x| x == c) {
                lib.remove(i);
            }
        }
        pick
    };
    let Some(c) = c else {
        crate::engine::tutors::shuffle_library(g, p);
        return Ok(None);
    };
    if let Some(a) = crate::engine::zones::agent_for(g, p) {
        crate::engine::zones::agent_take(g, a, p, c); // Opposition Agent
    } else if to_hand {
        let pl = g.player_mut(p);
        pl.hand.push(c);
        pl.seen.insert(c);
    } else {
        g.add_land_entering(p, c, tapped);
        crate::engine::zones::landfall(g, p)?;
    }
    crate::engine::tutors::shuffle_library(g, p);
    crate::glog!(g, "    {} searches for {}", pname(g, p), g.db.get(c).name);
    Ok(Some(c))
}

fn best_land_in_library(g: &Game, p: PlayerId) -> f64 {
    let cands = crate::engine::zones::searchable(g, p);
    cands.into_iter().filter(|&c| g.db.get(c).land).map(|c| land_want(g, p, c)).fold(0.0, f64::max)
}

fn land_ok(g: &Game, l: crate::ids::LandId, p: PlayerId) -> bool {
    g.land(l).on_bf
        && g.player(p).lands.contains(&l)
        && !g.land(l).tapped
        && !crate::impls::t6_stax::needle_stops(g, lname(g, g.land(l).cd))
}

/// {3}, {T}, sacrifice Urza's Cave: a land card onto the battlefield tapped. The AI does it for a Tron piece, Mishra's
/// Workshop or Ancient Tomb (a want of 6 or more): trading the Cave for a land that is better than a Cave.
fn cave_options(g: &mut Game, l: crate::ids::LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !land_ok(g, l, p) || !crate::impls::lands::can_pay_without(g, p, l, 3, "") {
        return Ok(vec![]);
    }
    let w = best_land_in_library(g, p);
    if w < 6.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.2 + w / 6.0,
        label: "Urza's Cave: fetch a land".into(),
        act: Some(Action::Plan { f: cave_go, arg: l.0 as i64 }),
    }])
}

fn cave_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = crate::ids::LandId(arg as u32);
    if !land_ok(g, l, p) || !crate::impls::lands::pay_without(g, p, l, 3, "")? {
        return Ok(false);
    }
    let cd = g.land(l).cd;
    crate::impls::lands::sac_land(g, p, l)?;
    if ability_window_card(g, p, cd, "search for a land", None, None)? {
        fetch_land(g, p, false, true)?;
    }
    Ok(true)
}

/// {2}, {T}, sacrifice Expedition Map: a land card (any land) into your hand. The AI cracks it for a land worth fetching.
fn map_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || g.perm(src).tapped || locked(g, src, p) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    let w = best_land_in_library(g, p);
    if w < 6.0 && !(w >= 3.0 && g.player(p).lands.len() < 4 && !g.player(p).hand.iter().any(|&c| g.db.get(c).land)) {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0 + w / 6.0, "Expedition Map: find a land".into(), src, map_go, 0)])
}

fn map_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || g.perm(src).tapped || !can_pay(g, p, 2, "", false) || !pay(g, p, 2, "", false)? {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    die(g, src, "sac")?;
    if ability_window(g, p, Some(src), "search for a land", None, None)? {
        fetch_land(g, p, true, false)?;
    }
    Ok(true)
}

// ================================================================== Inventors' Fair
fn artifact_total(g: &Game, p: PlayerId) -> i32 {
    artifacts(g, p)
        + g.player(p).lands.iter().filter(|&&l| g.db.get(g.land(l).cd).types.has(Types::ARTIFACT)).count() as i32
}

/// At the beginning of your upkeep, if you control three or more artifacts, you gain 1 life.
fn fair_upkeep(g: &mut Game, l: crate::ids::LandId, p: PlayerId) -> Res {
    if g.land(l).owner != p || artifact_total(g, p) < 3 {
        return Ok(());
    }
    if crate::engine::stack::trigger_window_card(g, p, g.land(l).cd, "gain 1 life")? {
        gain(g, p, 1)?;
    }
    Ok(())
}

/// {4}, {T}, sacrifice Inventors' Fair: search for an artifact card (to the hand). Only with three or more artifacts.
/// The AI cracks it with spare mana (a second land of use is not needed), for the best artifact left (`tutor_named`).
fn fair_options(g: &mut Game, l: crate::ids::LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none()
        || !land_ok(g, l, p)
        || artifact_total(g, p) < 3
        || !crate::impls::lands::can_pay_without(g, p, l, 4, "")
    {
        return Ok(vec![]);
    }
    if !crate::engine::zones::searchable(g, p).iter().any(|&c| art_card(g, c)) {
        return Ok(vec![]);
    }
    let hand = g.player(p).hand.len() as f64;
    Ok(vec![Opt {
        utility: 1.4 + if hand <= 2.0 { 0.8 } else { 0.0 },
        label: "Inventors' Fair: search for an artifact".into(),
        act: Some(Action::Plan { f: fair_go, arg: l.0 as i64 }),
    }])
}

fn fair_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = crate::ids::LandId(arg as u32);
    if !land_ok(g, l, p) || artifact_total(g, p) < 3 || !crate::impls::lands::pay_without(g, p, l, 4, "")? {
        return Ok(false);
    }
    let cd = g.land(l).cd;
    crate::impls::lands::sac_land(g, p, l)?;
    if ability_window_card(g, p, cd, "search for an artifact", None, None)? {
        crate::impls::t1::tutor_named(g, p, &|g, c| art_card(g, c), 1, "hand")?;
    }
    Ok(true)
}

// ================================================================== Rogue's Passage
/// {4}, {T}: target creature can't be blocked this turn. The AI uses it in the first main phase on its biggest ready
/// attacker (power 4 or more) when an opponent has an untapped creature to block with (combat.rs reads `DataKey::Unbl`,
/// as for a person using it).
fn passage_options(g: &mut Game, l: crate::ids::LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !land_ok(g, l, p) || !crate::impls::lands::can_pay_without(g, p, l, 4, "") {
        return Ok(vec![]);
    }
    let st = g.turn_stamp();
    let atk = g.player(p).perms.iter().copied().filter(|&m| {
        let x = g.perm(m);
        live(g, m)
            && g.is_creature(m)
            && !x.sick
            && !x.tapped
            && !x.noatk
            && epow(g, m) >= 4
            && x.data.get(DataKey::Unbl) != Some(&Val::Stamp(st))
    });
    let Some(best) = max_by(&atk.collect::<Vec<_>>(), |m| epow(g, m) as f64) else { return Ok(vec![]) };
    let blockers =
        g.opps(p).any(|q| g.player(q).perms.iter().any(|&m| live(g, m) && g.is_creature(m) && !g.perm(m).tapped));
    if !blockers {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.2 + 0.15 * epow(g, best) as f64,
        label: format!("Rogue's Passage: {} can't be blocked", g.perm(best).name),
        act: Some(Action::Plan { f: passage_go, arg: l.0 as i64 | (best.0 as i64) << 32 }),
    }])
}

fn passage_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = crate::ids::LandId((arg & 0xFFFF_FFFF) as u32);
    let m = PermId((arg >> 32) as u32);
    if !land_ok(g, l, p) || !live(g, m) || !crate::impls::lands::pay_without(g, p, l, 4, "")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    if ability_window_card(g, p, g.land(l).cd, "can't be blocked", None, Some(m))? && live(g, m) {
        let st = g.turn_stamp();
        g.perm_mut(m).data.set(DataKey::Unbl, Val::Stamp(st));
        crate::glog!(g, "    {} can't be blocked this turn (Rogue's Passage)", g.perm(m).name);
    }
    Ok(true)
}

// ================================================================== Endless Atlas
/// {2}, {T}: draw a card. Activate only if you control three or more lands with the same name.
fn atlas_ready(g: &Game, p: PlayerId) -> bool {
    let names: Vec<&str> = g.player(p).lands.iter().map(|&l| lname(g, g.land(l).cd)).collect();
    names.iter().any(|n| names.iter().filter(|&m| m == n).count() >= 3)
}

fn atlas_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || !live(g, src) || g.perm(src).tapped || locked(g, src, p) || !atlas_ready(g, p) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "", false) || g.player(p).library.len() <= 8 {
        return Ok(vec![]);
    }
    let hand = g.player(p).hand.len() as f64;
    let u = 1.3 + if hand <= 2.0 { 1.0 } else { 0.0 } - 0.25 * (hand - 4.0).max(0.0)
        + if post.is_none() { 0.6 } else { 0.0 };
    Ok(vec![ability(u, "Endless Atlas: draw".into(), src, atlas_go, 0)])
}

fn atlas_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src)
        || g.perm(src).tapped
        || !atlas_ready(g, p)
        || !can_pay(g, p, 2, "", false)
        || !pay(g, p, 2, "", false)?
    {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "draw a card", None, None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

// ================================================================== Hangarback Walker
/// the X the AI casts Hangarback Walker for ({X}{X}): a third of its mana, one to four, none if it cannot pay for one
fn hangar_x(g: &Game, p: PlayerId) -> i32 {
    let total = total_mana(g, p, false) as i32;
    if total < 2 { 0 } else { (total / 3).clamp(1, 4) }
}

/// {X}{X}: two generic mana per X (the data has no mana cost for it); uncastable when it cannot pay for X = 1
fn hangar_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match hangar_x(g, p) {
        0 => 99,
        x => 2 * x,
    }
}

fn hangar_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if hangar_x(g, p) == 0 { 0 } else { 44 }
}

/// enters with X +1/+1 counters, X being half the mana paid for it: read by `green_cards::enters_with` (`zones::enter`)
/// before the creature check, which would kill a 0/0 (a copy, or a card put onto the battlefield, enters with none and
/// dies as the Oracle says)
pub fn enters_with(g: &Game, cd: CardId, was_cast: bool) -> i32 {
    match &*g.db.get(cd).name {
        "Hangarback Walker" if was_cast => (g.entering_spent / 2) as i32,
        "Walking Ballista" if was_cast => g.last_x.div_euclid(2), // {X}{X}: the X mana its cast paid (`last_x`)
        _ => 0,
    }
}

/// When it dies, create a 1/1 colorless Thopter artifact creature token with flying for each +1/+1 counter on it.
fn hangar_dies(g: &mut Game, _src: Src, m: PermId, _cause: crate::sym::Sym) -> Res {
    let n = g.perm(m).plus;
    let o = g.perm(m).orig;
    if n <= 0 || !trigger_window(g, o, Some(m), "create Thopters", None)? {
        return Ok(());
    }
    let mut data = PermData::default();
    data.set(DataKey::Artifact, Val::Bool(true));
    let spec = Tokens {
        fly: true,
        color: Some(crate::cards::Colors::NONE),
        types: vec!["thopter"],
        data,
        ..Tokens::new(n as u32, 1)
    };
    make_tokens(g, o, spec)?;
    Ok(())
}

// ================================================================== Batterskull
/// Living weapon: when it enters, create a 0/0 black Phyrexian Germ token, then attach it. (The Germ is made with 1
/// toughness and set to 0/0 once attached, so the token check does not kill it before the +4/+4.)
fn skull_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "living weapon: a Germ token", None)? {
        return Ok(());
    }
    let spec = Tokens {
        tgh: Some(1),
        color: Some(crate::cards::Colors::from_letters("B")),
        types: vec!["phyrexian", "germ"],
        ..Tokens::new(1, 0)
    };
    for germ in make_tokens(g, p, spec)? {
        g.perm_mut(germ).tgh = 0;
        if live(g, src) {
            g.perm_mut(src).attached = Some(germ);
        } else {
            die(g, germ, "sba")?;
        }
    }
    Ok(())
}

/// {3}: return Batterskull to its owner's hand. The AI does it when no creature carries it and it can recast it (a new
/// Germ) for {5} after the {3}: eight mana. (It does not bounce in response to removal.)
fn skull_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || locked(g, src, p) {
        return Ok(vec![]);
    }
    let carried = g.perm(src).attached.is_some_and(|t| live(g, t) && owner(g, t) == p);
    if carried || total_mana(g, p, false) < 8 {
        return Ok(vec![]);
    }
    Ok(vec![ability(2.4, "Batterskull: return it to hand".into(), src, skull_go, 0)])
}

fn skull_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || !can_pay(g, p, 3, "", false) || !pay(g, p, 3, "", false)? {
        return Ok(false);
    }
    if ability_window(g, p, Some(src), "return it to its owner's hand", None, None)? && live(g, src) {
        crate::engine::zones::bounce(g, src)?;
    }
    Ok(true)
}

// ================================================================== Liquimetal Torque
/// {T}: target nonland permanent becomes an artifact in addition to its other types until end of turn (the mark is
/// `DataKey::Artifact` holding the turn: `t5::is_artifact` reads it). The AI never uses it (it would give up the
/// Torque's mana for nothing it can measure); a person can.
fn torque_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || g.perm(src).tapped || locked(g, src, p) {
        return Ok(vec![]);
    }
    let t = max_by(
        &g.player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| live(g, m) && !crate::impls::t5::is_artifact(g, m))
            .collect::<Vec<_>>(),
        |m| pval(g, m),
    );
    let Some(t) = t else { return Ok(vec![]) };
    Ok(vec![ability(
        -5.0,
        format!("Liquimetal Torque: {} becomes an artifact", g.perm(t).name),
        src,
        torque_go,
        t.0 as i64,
    )])
}

fn torque_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !live(g, src) || g.perm(src).tapped || !live(g, t) {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "target permanent becomes an artifact", None, Some(t))? && live(g, t) {
        let st = g.turn_stamp();
        g.perm_mut(t).data.set(DataKey::Artifact, Val::Stamp(st));
    }
    Ok(true)
}

// ================================================================== Staff of Domination
/// Basalt Monolith and Rings of Brighthearth on the battlefield: unbounded colourless mana (the Rings copy of Basalt's
/// untap costs {2}; {3}+{2} out, {3} in, again and again), once the loop is started with five mana (or the Monolith
/// untapped and two more)
pub fn infinite_mana(g: &Game, p: PlayerId) -> bool {
    let has = |n: &str| {
        g.player(p).perms.iter().any(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == n))
    };
    has("Basalt Monolith") && has("Rings of Brighthearth") && total_mana(g, p, false) >= 2
}

/// How many cards the AI leaves in its library when infinite mana draws it down through Staff of Domination ({1}: untap
/// it, {5}, {T}: draw, again and again): enough that it cannot lose to the draw step, the Ballista and the rest of the
/// win need no further draws.
pub const SAFE_LIBRARY: usize = 3;

/// Draw cards with Staff of Domination's loop until a Walking Ballista (the win) is in the hand or on the battlefield,
/// or only `SAFE_LIBRARY` cards remain. The cards drawn.
pub fn staff_draw_down(g: &mut Game, p: PlayerId) -> Res<u32> {
    let mut n = 0;
    let have_ballista = |g: &Game| {
        let pl = g.player(p);
        pl.hand.iter().any(|&c| &*g.db.get(c).name == "Walking Ballista")
            || pl
                .perms
                .iter()
                .any(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Walking Ballista"))
    };
    while g.player(p).library.len() > SAFE_LIBRARY && !have_ballista(g) && !g.over {
        draw(g, p, 1, false)?;
        n += 1;
    }
    Ok(n)
}

fn staff_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || locked(g, src, p) {
        return Ok(vec![]);
    }
    if !infinite_mana(g, p) || g.player(p).library.len() <= SAFE_LIBRARY {
        return Ok(vec![]);
    }
    Ok(vec![ability(12.0, "Staff of Domination: draw with infinite mana".into(), src, staff_go, 0)])
}

fn staff_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || !infinite_mana(g, p) {
        return Ok(false);
    }
    if !ability_window(g, p, Some(src), "draw cards (infinite mana)", Some(8.0), None)? {
        return Ok(true);
    }
    let n = staff_draw_down(g, p)?;
    crate::glog!(g, "    {} draws {n} cards with Staff of Domination and infinite mana", pname(g, p));
    Ok(true)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    walker(r, db, KARN_SCION, &KARN_ABILITIES)?;
    r.card(db, MAZEMIND)?.options = Some(mazemind_options);
    r.card(db, TOME)?.options = Some(urzas_tome_options);
    r.card(db, MIRRORWORKS)?.etb = Some(mirror_etb);
    r.card(db, FORGE)?.options = Some(forge_options);
    r.card(db, BLUEPRINTS)?.upkeep = Some(blueprints_upkeep);
    // the cards that were already in the engine
    r.card(db, "Urza's Cave")?.land_options = Some(cave_options);
    r.card(db, "Expedition Map")?.options = Some(map_options);
    let c = r.card(db, "Inventors' Fair")?;
    c.land_upkeep = Some(fair_upkeep);
    c.land_options = Some(fair_options);
    r.card(db, "Rogue's Passage")?.land_options = Some(passage_options);
    r.card(db, "Endless Atlas")?.options = Some(atlas_options);
    let c = r.card(db, "Hangarback Walker")?;
    c.self_cost = Some(hangar_cost);
    c.prio = Some(hangar_prio);
    c.self_dies = Some(hangar_dies);
    let c = r.card(db, "Batterskull")?;
    c.etb = Some(skull_etb);
    c.options = Some(skull_options);
    r.card(db, "Liquimetal Torque")?.options = Some(torque_options);
    r.card(db, "Staff of Domination")?.options = Some(staff_options);
    Ok(())
}
