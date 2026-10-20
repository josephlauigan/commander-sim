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
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::zones::{Tokens, die, draw, enter_token_copy, leave, make_tokens, max_by, min_by, to_zone_card};
use crate::engine::zones::Zone;
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
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
    let give = if top.len() == 2 {
        if worth(g, p, top[1]) < worth(g, p, top[0]) { top[1] } else { top[0] }
    } else {
        top[0]
    };
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
    let mut u = 1.1 + if free { 0.5 } else { -0.3 } + if hand <= 2.0 { 1.0 } else { 0.0 } - 0.25 * (hand - 4.0).max(0.0);
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
    if !trigger_window(g, o, Some(src), "pay {2}: a token copy of the artifact", None)? || !can_pay(g, o, 2, "", false) {
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
            let ok = can_pay(g, p, cg, &cp, false) && (!g.cast_checks() || crate::engine::cast::castable(g, p, top, "lib"));
            g.pay_for = None;
            if ok {
                out.push(Opt {
                    utility: u + 0.6,
                    label: format!("Mystic Forge: cast {}", d.name),
                    act: Some(Action::Plan { f: forge_cast, arg: top.0 as i64 }),
                });
            }
        }
    } else if d.land && !g.perm(src).tapped && !locked(g, src, p) && g.player(p).life > 8 && g.player(p).library.len() > 12 {
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

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    walker(r, db, KARN_SCION, &KARN_ABILITIES)?;
    r.card(db, MAZEMIND)?.options = Some(mazemind_options);
    r.card(db, TOME)?.options = Some(urzas_tome_options);
    r.card(db, MIRRORWORKS)?.etb = Some(mirror_etb);
    r.card(db, FORGE)?.options = Some(forge_options);
    r.card(db, BLUEPRINTS)?.upkeep = Some(blueprints_upkeep);
    let _ = Event::Etb;
    Ok(())
}
