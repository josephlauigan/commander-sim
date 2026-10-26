//! Tier 2 pool decks `brago-azorius-blink-control` and `meren-golgari-recursion` (EDHREC's bracket-3 average lists):
//! the cards the ability language doesn't cover (rust/agents/t12-brago-meren.md). Wight of the Reliquary's "+1/+1 for
//! each creature card in your graveyard" is compiled (dsl_parse.rs, `// --- Tiers 1/2 (brago-meren)`); the rest is
//! here.
//!
//! Brago:
//! - Agent of Treachery: enters: gain control of the best opposing permanent it can target (for good); your end step:
//!   draw three if you control three or more permanents you don't own. Left out: stealing a land (lands are kept
//!   apart from permanents).
//! - Cryogen Relic: draws when it enters (the engine's draw tag) and when it leaves; {1}{U}, sacrifice it: a stun
//!   counter on an opponent's tapped creature (the AI uses it at the end of an opponent's turn on its best tapped
//!   creature, or for the card with two or fewer in hand).
//! - Dour Port-Mage: one or more other creatures you control leave without dying: draw a card (once per batch: the
//!   blinks of one effect, Brago's included, draw one card). Its {1}{U} bounce is compiled.
//! - Flickerwisp: enters: exile another target permanent; it returns under its owner's control at the beginning of the
//!   next end step (a token is gone for good). The AI takes an opposing token, else its own best enters-the-battlefield
//!   permanent (or Reality Acid, whose host is then sacrificed), else an opposing permanent for the turn.
//! - Ghostly Flicker: exile two of your artifacts and/or creatures and return them (the two best enters-the-battlefield
//!   ones), cast for value or in response to removal on a creature (pool::PROTECTORS). Left out: lands.
//! - Reality Acid: an Aura on the best opposing permanent; vanishing 3 (a time counter off each of your upkeeps, then
//!   it is sacrificed); when it leaves the battlefield (any way: Brago's blink too), that permanent's controller
//!   sacrifices it.
//! - Riptide Gearhulk: enters: each opponent's best nonland permanent goes into its owner's library third from the
//!   top. Double strike and prowess are the engine's keywords.
//! - Starfield Vocalist: a permanent entering makes your triggered abilities (enters and landfall) trigger an
//!   additional time (`trigger_copies`, as Panharmonicon). Left out: warp (the engine has no casting from exile later).
//! - Strionic Resonator: {2}, {T}: copy a triggered ability you control (`stack::ai_respond` asks `resonator_wants`;
//!   the copy runs through `stack::copy_trigger`, as Peter Parker's Camera). The AI copies a trigger worth 4 or more
//!   (Brago's, Agent of Treachery's, an enters trigger of a good blink target). Left out: new targets for the copy (its
//!   effect makes its own choices again as it resolves).
//! - Time Wipe: return your best creature to its owner's hand (your commander first: it goes back without tax), then
//!   destroy all creatures (the engine's board wipe).
//! - Yorion, Sky Nomad: enters: exile your other nonland permanents worth blinking (and Reality Acid); they return at
//!   the beginning of the next end step. Left out: companion (Yorion is in the 99).
//!
//! Meren:
//! - Buried Alive: up to three creature cards from your library into your graveyard (the best reanimation targets), in
//!   a main phase or at the end of an opponent's turn. JD's decks keep their own plan (ais.seph_fill_resolve).
//! - Journey to Eternity // Atzal, Cave of Eternity: an Aura on your best creature; when it dies, it returns to the
//!   battlefield under your control, and Journey returns transformed: Atzal, a land that taps for any colour of your
//!   identity, with {3}{B}{G}, {T}: return the best creature card from your graveyard to the battlefield.
//! - Twilight Diviner: its surveil 2 is compiled; whenever other creatures you control enter from a graveyard (Meren,
//!   undying, reanimation, Atzal), a token copy of the best of them, once each turn. "Entered from a graveyard" is the
//!   game's record of creature cards put into graveyards (`Game::t12_gy`: deaths, mills, discards, surveils, searches
//!   such as Entomb and Buried Alive), read by `zones::enter`. Left out: a card returned to the library and put onto the
//!   battlefield from there later counts too.
//!
//! Combos (chained into combos::all): Peregrine Drake + Deadeye Navigator (a two-card infinite: mana and blinks), and
//! Peregrine Drake + Archaeomancer + Ghostly Flicker; both win through Agent of Treachery (every permanent stolen).

use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable};
use crate::engine::control::{Duration, gain_control};
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::removal::{apply_removal, apply_wipe};
use crate::engine::stack::{ability_window, ability_window_card, flush_triggers, queue_triggers, trigger_window};
use crate::engine::tutors::shuffle_library;
use crate::engine::values::{epow, pval, untargetable};
use crate::engine::zones::{
    Enter, agent_for, agent_take, bounce, die, draw, enter, enter_token_copy, exile_perm, landfall, max_by, min_by,
};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Src, TrigAct, Trigger};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready};
use crate::impls::common::{AURA, AuraSpec, aura};
use crate::impls::t2::{blink, blink_value};
use crate::state::{Ctx, DataKey, Game, StackItem, StackKind, Val};
use crate::sym::Sym;

const AGENT: &str = "Agent of Treachery";
const RELIC: &str = "Cryogen Relic";
const PORT_MAGE: &str = "Dour Port-Mage";
const WISP: &str = "Flickerwisp";
const FLICKER: &str = "Ghostly Flicker";
const ACID: &str = "Reality Acid";
const GEARHULK: &str = "Riptide Gearhulk";
const VOCALIST: &str = "Starfield Vocalist";
const RESONATOR: &str = "Strionic Resonator";
const TIME_WIPE: &str = "Time Wipe";
const YORION: &str = "Yorion, Sky Nomad";
const BURIED: &str = "Buried Alive";
const JOURNEY: &str = "Journey to Eternity // Atzal, Cave of Eternity";
const DIVINER: &str = "Twilight Diviner";
const DRAKE: &str = "Peregrine Drake";
const DEADEYE: &str = "Deadeye Navigator";
const ARCH: &str = "Archaeomancer";

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn is_named(g: &Game, m: PermId, name: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| cname(g, c) == name)
}

/// p's live permanent of this name
fn mine(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && is_named(g, m, name))
}

fn held(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| cname(g, c) == name)
}

fn plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

/// what taking (or losing) m is worth: its value, at least a creature token's power and a card's mana value (pval
/// rates tokens low and most noncreature permanents 0)
fn worth(g: &Game, m: PermId) -> f64 {
    let v = pval(g, m);
    let x = g.perm(m);
    if x.token {
        return if g.is_creature(m) { v.max(0.5 + 0.5 * epow(g, m) as f64) } else { v };
    }
    let Some(c) = x.cd else { return v };
    let d = g.db.get(c);
    let rock = if d.tag(crate::tag::Tag::Rock) { 1.5 } else { 0.0 };
    v.max(0.4 * d.cmc as f64 + rock)
}

/// what a creature card in the graveyard is worth to bring back (as Meren picks: enters value, bomb or power, card
/// code), and a little for its mana value
fn creature_card_worth(g: &Game, c: CardId) -> f64 {
    let d = g.db.get(c);
    let code = g.registry.get(c).is_some_and(|i| i.live());
    crate::dsl::etb_value(g, c) as f64
        + (d.bomb as f64).max(d.pow as f64)
        + if code { 3.0 } else { 0.0 }
        + 0.3 * d.cmc as f64
}

/// the opponents' live permanents p could target
fn opp_targets(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).filter(|&m| live(g, m) && !untargetable(g, m)).collect()
}

/// effects that move several permanents at once: the triggers they cause wait until all have moved (so "one or more"
/// triggers see one batch: Dour Port-Mage)
fn together(g: &mut Game, f: impl FnOnce(&mut Game) -> Res) -> Res {
    g.resolving += 1;
    let r = f(g);
    g.resolving -= 1;
    r?;
    if g.resolving == 0 && !g.trig_queue.is_empty() && !g.over {
        flush_triggers(g)?;
    }
    Ok(())
}

/// pay a hand card's cost and cast it
fn pay_and_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    let (cg, cp) = cost_of(g, p, c);
    if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") || !can_pay(g, p, cg, &cp, false) {
        return Ok(false);
    }
    g.pay_for = Some(c);
    let ok = pay(g, p, cg, &cp, false)?;
    g.pay_for = None;
    if !ok {
        return Ok(false);
    }
    cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

// ================================================================== the graveyard record (Twilight Diviner)
/// a card went to a graveyard: a creature card is recorded (zones::mill, discard, kefka::surveil, Entomb ...)
pub fn note_gy(g: &mut Game, c: CardId) {
    if g.db.get(c).creature && !g.t12_gy.contains(&c) {
        g.t12_gy.push(c);
    }
}

/// zones::die: nontoken creature m died (its card goes to the graveyard, or comes straight back from it: undying,
/// persist, Kaya's Ghostform, Journey to Eternity)
pub fn note_died(g: &mut Game, m: PermId) {
    let x = g.perm(m);
    if x.token {
        return;
    }
    if let Some(c) = x.phys.or(x.cd) {
        note_gy(g, c);
    }
}

/// cast::cast_card: a card cast from anywhere but a graveyard has left it (a creature regrown to hand, then cast)
pub fn note_cast(g: &mut Game, c: CardId, zone: Sym) {
    if zone != "gy" {
        g.t12_gy.retain(|&x| x != c);
    }
}

/// zones::enter: card cd is entering; true if it came from a graveyard (the record forgets it)
pub fn from_gy(g: &mut Game, cd: CardId) -> bool {
    match g.t12_gy.iter().position(|&x| x == cd) {
        Some(i) => {
            g.t12_gy.swap_remove(i);
            true
        }
        None => false,
    }
}

/// zones::enter: the permanent m came from a graveyard (marked for the turn)
pub fn mark_from_gy(g: &mut Game, m: PermId) {
    let st = g.turn_stamp();
    g.perm_mut(m).data.set(DataKey::T12FromGy, Val::Stamp(st));
}

fn entered_from_gy_now(g: &Game, m: PermId) -> bool {
    g.perm(m).data.get(DataKey::T12FromGy) == Some(&Val::Stamp(g.turn_stamp()))
}

// ================================================================== Agent of Treachery
/// the permanent Agent takes: the opponents' best it can target
fn agent_pick(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs = opp_targets(g, p);
    max_by(&cs, |m| worth(g, m)).filter(|&m| worth(g, m) > 0.0)
}

/// When this creature enters, gain control of target permanent.
fn agent_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    if agent_pick(g, o).is_none() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "gain control of target permanent", Some(6.0))? {
        return Ok(());
    }
    if let Some(t) = agent_pick(g, o) {
        gain_control(g, o, t, Duration::Permanent);
    }
    Ok(())
}

/// p's live permanents p doesn't own
fn not_owned(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| live(g, m) && g.perm(m).orig != p).count()
}

/// At the beginning of your end step, if you control three or more permanents you don't own, draw three cards.
fn agent_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !live(g, src) || not_owned(g, p) < 3 {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "draw three cards", Some(5.0))? && not_owned(g, p) >= 3 {
        draw(g, p, 3, false)?;
    }
    Ok(())
}

// ================================================================== Cryogen Relic
/// When this artifact leaves the battlefield, draw a card (it draws as it enters through the engine's draw tag).
fn relic_leaves(g: &mut Game, src: Src, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "draw a card", Some(3.0))? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// an opponent's tapped creature worth a stun counter
fn relic_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> =
        opp_targets(g, p).into_iter().filter(|&m| g.is_creature(m) && g.perm(m).tapped && epow(g, m) > 0).collect();
    max_by(&cs, |m| worth(g, m))
}

/// {1}{U}, Sacrifice this artifact: Put a stun counter on up to one target tapped creature. At the end of an
/// opponent's turn: on their best tapped creature (it stays tapped through your turn and theirs), or for the card
fn relic_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.active == Some(p) || owner(g, src) != p || !live(g, src) || !can_pay(g, p, 1, "U", false) {
        return Ok(vec![]);
    }
    if let Some(t) = relic_target(g, p).filter(|&t| worth(g, t) >= 3.0) {
        let label = format!("{RELIC}: stun {}", g.perm(t).name);
        return Ok(vec![ability(1.0 + 0.3 * worth(g, t), label, src, relic_go, t.0 as i64)]);
    }
    if g.player(p).hand.len() <= 2 {
        return Ok(vec![ability(0.6, format!("{RELIC}: sacrifice for a card"), src, relic_go, -1)]);
    }
    Ok(vec![])
}

fn relic_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    if !live(g, src) || !pay(g, p, 1, "U", false)? {
        return Ok(false);
    }
    let t = (arg >= 0).then(|| PermId(arg as u32));
    die(g, src, "sac")?;
    if ability_window(g, p, Some(src), "put a stun counter on a tapped creature", None, t)?
        && let Some(t) = t.filter(|&t| live(g, t) && g.perm(t).tapped)
    {
        let n = g.perm(t).data.int(DataKey::Stun);
        g.perm_mut(t).data.set(DataKey::Stun, Val::Int(n + 1));
        crate::glog!(g, "    {} gets a stun counter ({RELIC})", g.perm(t).name);
    }
    Ok(true)
}

// ================================================================== Dour Port-Mage
/// zones::leave: creature m left the battlefield. Dour Port-Mage: "whenever one or more other creatures you control
/// leave the battlefield without dying, draw a card": one trigger per Port-Mage while one is waiting (a batch).
pub fn creature_left(g: &mut Game, m: PermId) -> Res {
    if g.hooks.is_empty() || g.dying == Some(m) || !g.is_creature(m) {
        return Ok(());
    }
    let p = owner(g, m);
    let mages: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&s| s != m && live(g, s) && !g.perm(s).neutered && is_named(g, s, PORT_MAGE))
        .collect();
    for s in mages {
        let waiting = g
            .trig_queue
            .iter()
            .any(|t| t.src == Some(s) && matches!(t.act, TrigAct::Hook { event: Event::Leaves, .. }));
        if waiting {
            continue;
        }
        let t = Trigger {
            controller: p,
            src: Some(s),
            act: TrigAct::Hook { event: Event::Leaves, call: Call::Leaves { m } },
            name: None,
            known: false,
            imp: None,
            cast_etb: false,
        };
        queue_triggers(g, vec![t])?;
    }
    Ok(())
}

/// the Port-Mage's trigger (`creature_left`); its own leaving does nothing
fn mage_leaves(g: &mut Game, src: Src, m: PermId) -> Res {
    if m == src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "draw a card", Some(3.0))? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ================================================================== Flickerwisp
/// what exiling m until the end step is worth to p (a token is gone for good)
fn wisp_worth(g: &Game, p: PlayerId, src: PermId, m: PermId) -> f64 {
    let x = g.perm(m);
    if x.owner == p {
        if x.token || x.orig != p {
            return -5.0; // a token is gone; a stolen permanent returns to its owner
        }
        if is_named(g, m, ACID) {
            return x.attached.filter(|&h| live(g, h)).map_or(0.0, |h| pval(g, h) + 1.0);
        }
        return blink_value(g, p, m) as f64 - if x.is_cmd { 2.0 } else { 0.0 } - if m == src { 99.0 } else { 0.0 };
    }
    if untargetable(g, m) {
        return f64::NEG_INFINITY;
    }
    if x.token { worth(g, m) } else { 0.3 * worth(g, m) }
}

/// "another target permanent": the best for p, or the least bad (the target isn't optional)
fn wisp_pick(g: &Game, p: PlayerId, src: PermId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| m != src && live(g, m) && wisp_worth(g, p, src, m) > f64::NEG_INFINITY)
        .collect();
    max_by(&cs, |m| wisp_worth(g, p, src, m))
}

/// When this creature enters, exile another target permanent. Return that card to the battlefield under its owner's
/// control at the beginning of the next end step.
fn wisp_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    if wisp_pick(g, o, src).is_none() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "exile another target permanent until the end step", Some(4.0))? {
        return Ok(());
    }
    let Some(t) = wisp_pick(g, o, src) else { return Ok(()) };
    exile_until_end(g, o, t)
}

/// exile m; its card returns under its owner's control at the beginning of the next end step (`galadriel::eot_returns`)
fn exile_until_end(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let x = g.perm(m);
    let (q, card, token) = (x.orig, x.phys.or(x.cd), x.token);
    if x.owner == p {
        exile_perm(g, m)?;
    } else {
        apply_removal(g, Some(p), m, "exile", None)?;
    }
    if let Some(c) = card
        && !token
        && !g.perm(m).on_bf
        && g.player(q).exile.contains(&c)
    {
        g.eot_returns.push((q, c));
        crate::glog!(g, "    {} is exiled until the end step ({WISP})", cname(g, c));
    }
    Ok(())
}

// ================================================================== Ghostly Flicker
/// p's artifacts and creatures worth exiling and returning, best first (a tapped mana rock comes back untapped)
fn flicker_picks(g: &Game, p: PlayerId) -> Vec<(f64, PermId)> {
    let mut v: Vec<(f64, PermId)> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            live(g, m)
                && !x.token
                && x.orig == p
                && (g.is_creature(m) || x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)))
        })
        .map(|m| {
            let x = g.perm(m);
            let rock = x.tapped && x.cd.is_some_and(|c| g.db.get(c).tag(crate::tag::Tag::Rock));
            (blink_value(g, p, m) as f64 + if rock { 1.5 } else { 0.0 }, m)
        })
        .filter(|x| x.0 >= 1.0)
        .collect();
    v.sort_by(|a, b| b.0.total_cmp(&a.0));
    v.truncate(2);
    v
}

/// Exile two target artifacts, creatures, and/or lands you control, then return those cards to the battlefield under
/// your control.
fn flicker_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let ms: Vec<PermId> = flicker_picks(g, p).into_iter().map(|x| x.1).collect();
    together(g, |g| {
        for m in ms {
            if live(g, m) {
                blink(g, p, m)?;
            }
        }
        Ok(())
    })?;
    Ok("gy")
}

/// cast it for value: two good enters-the-battlefield permanents (a main phase, or the end of an opponent's turn)
fn flicker_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if crate::human::is_human(g, p) || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    if post.is_some() && g.active != Some(p) {
        return Ok(vec![]);
    }
    let (cg, cp) = cost_of(g, p, c);
    if !castable(g, p, c, "hand") || !can_pay(g, p, cg, &cp, false) {
        return Ok(vec![]);
    }
    let v: f64 = flicker_picks(g, p).iter().map(|x| x.0).sum();
    if v < 5.0 {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.0 + 0.4 * v, format!("{FLICKER} (value)"), flicker_go, c.0 as i64)])
}

fn flicker_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    pay_and_cast(g, p, CardId(arg as u16))
}

// ================================================================== Reality Acid
/// the permanents whose sacrifice is already waiting (a Reality Acid that left, its trigger not yet resolved)
fn doomed(g: &Game, m: PermId) -> bool {
    g.trig_queue.iter().any(|t| {
        matches!(t.act, TrigAct::Hook { event: Event::Leaves, .. })
            && t.src.is_some_and(|s| is_named(g, s, ACID) && g.perm(s).attached == Some(m))
    })
}

/// what Reality Acid enchants: the opponents' best permanent (one it can target if cast), else p's least valuable
fn acid_pick(g: &Game, p: PlayerId, a: Option<PermId>) -> Option<PermId> {
    let cast = g.last_cast_etb && !g.aura_put;
    let theirs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && Some(m) != a && !doomed(g, m) && !(cast && untargetable(g, m)))
        .collect();
    if let Some(m) = max_by(&theirs, |m| worth(g, m)) {
        return Some(m);
    }
    let ours: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| live(g, m) && Some(m) != a && !doomed(g, m)).collect();
    min_by(&ours, |m| pval(g, m))
}

fn acid_host(g: &mut Game, p: PlayerId, a: PermId) -> Res<Option<PermId>> {
    Ok(acid_pick(g, p, Some(a)))
}

/// vanishing 3: it enters with three time counters
fn acid_counters(g: &mut Game, _p: PlayerId, a: PermId, _host: PermId) -> Res {
    g.perm_mut(a).data.set(DataKey::T12, Val::Int(3));
    Ok(())
}

static ACID_SPEC: AuraSpec =
    AuraSpec { target: "opp", host_pick: Some(acid_host), on_etb: Some(acid_counters), ..AURA };

/// At the beginning of your upkeep, remove a time counter from it. When the last is removed, sacrifice it.
fn acid_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !live(g, src) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "remove a time counter", Some(1.0))? || !live(g, src) {
        return Ok(());
    }
    let n = g.perm(src).data.int(DataKey::T12) - 1;
    g.perm_mut(src).data.set(DataKey::T12, Val::Int(n.max(0)));
    if n <= 0 {
        crate::glog!(g, "    {ACID}: the last time counter is removed");
        die(g, src, "sac")?;
    }
    Ok(())
}

/// When this Aura leaves the battlefield, enchanted permanent's controller sacrifices it.
fn acid_leaves(g: &mut Game, src: Src, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let Some(h) = g.perm(src).attached.filter(|&h| live(g, h)) else { return Ok(()) };
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "the enchanted permanent is sacrificed", Some(5.0))? && live(g, h) {
        crate::glog!(g, "    {} sacrifices {} ({ACID})", g.player(owner(g, h)).name, g.perm(h).name);
        die(g, h, "sac")?;
    }
    Ok(())
}

/// cast it on an opposing permanent worth the wait
fn acid_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let best = opp_targets(g, p).into_iter().map(|m| worth(g, m)).fold(0.0, f64::max);
    if best >= 3.0 { 45 } else { 0 }
}

// ================================================================== Riptide Gearhulk
/// each opponent's best nonland permanent it can target
fn gearhulk_picks(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.opps(p)
        .filter_map(|q| {
            let cs: Vec<PermId> =
                g.player(q).perms.iter().copied().filter(|&m| live(g, m) && !untargetable(g, m)).collect();
            max_by(&cs, |m| worth(g, m)).filter(|&m| worth(g, m) >= 1.0)
        })
        .collect()
}

/// When this creature enters, for each opponent, put up to one target nonland permanent that player controls into its
/// owner's library third from the top.
fn gearhulk_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    if gearhulk_picks(g, o).is_empty() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "each opponent's best permanent third from the top", Some(6.0))? {
        return Ok(());
    }
    for t in gearhulk_picks(g, o) {
        third_from_top(g, o, t)?;
    }
    Ok(())
}

/// m into its owner's library third from the top (a commander goes to the command zone, a token is gone)
fn third_from_top(g: &mut Game, actor: PlayerId, m: PermId) -> Res {
    let x = g.perm(m);
    let (q, card) = (x.orig, x.phys.or(x.cd));
    apply_removal(g, Some(actor), m, "top", None)?;
    let Some(c) = card else { return Ok(()) };
    if g.perm(m).on_bf {
        return Ok(());
    }
    let lib = &mut g.player_mut(q).library;
    if lib.last() == Some(&c) {
        lib.pop();
        let n = lib.len();
        lib.insert(n.saturating_sub(2), c);
    }
    Ok(())
}

// ================================================================== Starfield Vocalist
/// If a permanent entering the battlefield causes a triggered ability of a permanent you control to trigger, that
/// ability triggers an additional time (enters triggers and landfall).
fn vocalist_copies(g: &Game, src: Src, p: PlayerId, kind: Sym, _x: Option<PermId>) -> i32 {
    let s = g.perm(src);
    (matches!(kind, "etb" | "landfall") && p == s.owner && s.on_bf && !s.phased && !s.neutered) as i32
}

// ================================================================== Strionic Resonator
/// p's untapped Strionic Resonator, with {2} to pay
pub fn resonator_ready(g: &Game, p: PlayerId) -> bool {
    resonator(g, p).is_some() && can_pay(g, p, 2, "", false)
}

fn resonator(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .find(|&m| live(g, m) && !g.perm(m).tapped && !g.perm(m).neutered && is_named(g, m, RESONATOR))
}

/// stack::ai_respond: q's own triggered ability on top of the stack is worth a Resonator copy (4 or more: its
/// importance, or the blink value of the permanent it comes from)
pub fn resonator_wants(g: &Game, q: PlayerId, it: &StackItem) -> bool {
    if it.controller != q || it.kind != StackKind::Trigger || it.countered || it.ctx.trigger.is_none() {
        return false;
    }
    if !resonator_ready(g, q) {
        return false;
    }
    let from = it.ctx.source.filter(|&m| g.perm(m).owner == q && !is_named(g, m, RESONATOR));
    let bv = from.map_or(0.0, |m| blink_value(g, q, m) as f64);
    it.imp.max(bv) >= 4.0
}

/// {2}, {T}: Copy target triggered ability you control (cardcode::copy_ability). True if a copy resolved.
pub fn resonator_copy(g: &mut Game, q: PlayerId, item: u32) -> Res<bool> {
    let Some(r) = resonator(g, q) else { return Ok(false) };
    let Some(it) = g.stack.iter().find(|x| x.id == item).cloned() else { return Ok(false) };
    if !resonator_wants(g, q, &it) || !pay(g, q, 2, "", false)? {
        return Ok(false);
    }
    g.perm_mut(r).tapped = true;
    crate::glog!(g, "    {} activates {RESONATOR}: copies {}", g.player(q).name, it.name);
    if !ability_window(g, q, Some(r), "copy target triggered ability", Some(3.0), None)? {
        return Ok(false);
    }
    crate::engine::stack::copy_trigger(g, q, item)
}

// ================================================================== Time Wipe
/// the creature Time Wipe returns: the commander (back to hand, no tax), else the best one to keep
fn time_wipe_pick(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
    max_by(&cs, |m| {
        let x = g.perm(m);
        if x.token || x.orig != p {
            return -10.0 + pval(g, m) * 0.01; // a token ceases to exist; a stolen creature goes to its owner
        }
        0.6 * pval(g, m) + blink_value(g, p, m) as f64 + if x.is_cmd { 4.0 } else { 0.0 }
    })
}

/// Return a creature you control to its owner's hand, then destroy all creatures.
fn time_wipe_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(m) = time_wipe_pick(g, p) {
        crate::glog!(g, "    {} returns {} to hand ({TIME_WIPE})", g.player(p).name, g.perm(m).name);
        bounce(g, m)?;
    }
    apply_wipe(g, p, "destroy", Some(c), ctx)?;
    Ok("gy")
}

// ================================================================== Yorion, Sky Nomad
/// p's other nonland permanents Yorion exiles until the end step: those worth entering again, Reality Acid (its host is
/// sacrificed), never the commander
fn yorion_picks(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            m != src
                && live(g, m)
                && !x.token
                && x.orig == p
                && !x.is_cmd
                && (blink_value(g, p, m) >= 3 || is_named(g, m, ACID) && x.attached.is_some_and(|h| live(g, h)))
        })
        .collect()
}

/// When Yorion enters, exile any number of other nonland permanents you own and control. Return those cards to the
/// battlefield at the beginning of the next end step.
fn yorion_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    if yorion_picks(g, o, src).is_empty() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "exile your permanents until the end step", Some(5.0))? {
        return Ok(());
    }
    let ms = yorion_picks(g, o, src);
    together(g, |g| crate::impls::galadriel::interlude(g, o, &ms))
}

// ================================================================== Buried Alive
const REANIMATION: [&str; 6] = ["Reanimate", "Animate Dead", "Necromancy", "Victimize", "Dread Return", "Living Death"];
const RETURNERS: [&str; 3] = ["Meren of Clan Nel Toth", "Sheoldred, Whispering One", "Cauldron of Essence"];

fn buried_worth(g: &Game, _p: PlayerId, c: CardId) -> f64 {
    creature_card_worth(g, c)
}

/// the (up to three) creature cards Buried Alive puts into p's graveyard, best first
fn buried_picks(g: &Game, p: PlayerId) -> Vec<CardId> {
    let mut lib: Vec<CardId> = g.player(p).library.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    lib.sort_by(|&a, &b| buried_worth(g, p, b).total_cmp(&buried_worth(g, p, a)));
    lib.dedup();
    lib.truncate(3);
    lib
}

/// p can bring a creature card back from its graveyard: a reanimation spell in hand, Meren or another returner out
/// (or Meren in the command zone), Atzal
fn can_return(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    pl.hand.iter().any(|&c| REANIMATION.contains(&cname(g, c)))
        || RETURNERS.iter().any(|n| mine(g, p, n).is_some())
        || pl.commanders().any(|c| cname(g, c) == RETURNERS[0] && pl.cmd_zone_of(c))
        || pl.lands.iter().any(|&l| cname(g, g.land(l).cd) == JOURNEY)
}

fn buried_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if crate::ai::is_main(g.player(p).key) || crate::human::is_human(g, p) || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    if post.is_some() && g.active != Some(p) {
        return Ok(vec![]);
    }
    let (cg, cp) = cost_of(g, p, c);
    if !castable(g, p, c, "hand") || !can_pay(g, p, cg, &cp, false) || !can_return(g, p) {
        return Ok(vec![]);
    }
    let picks = buried_picks(g, p);
    let Some(&best) = picks.first() else { return Ok(vec![]) };
    if buried_worth(g, p, best) < 4.0 {
        return Ok(vec![]);
    }
    let u = if post.is_none() { 2.6 } else { 1.6 };
    Ok(vec![plan(u, format!("{BURIED} ({})", cname(g, best)), buried_go, c.0 as i64)])
}

fn buried_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    pay_and_cast(g, p, CardId(arg as u16))
}

/// Search your library for up to three creature cards, put them into your graveyard, then shuffle.
fn buried_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if crate::ai::is_main(g.player(p).key) || crate::human::is_human(g, p) {
        crate::ai::seph_fill_resolve(g, p, "buried", ctx)?; // JD's decks: their own plan
        return Ok("gy");
    }
    for c in buried_picks(g, p) {
        let lib = &mut g.player_mut(p).library;
        let Some(i) = lib.iter().position(|&x| x == c) else { continue };
        lib.remove(i);
        if let Some(a) = agent_for(g, p) {
            agent_take(g, a, p, c); // Opposition Agent takes what they search for
            continue;
        }
        g.player_mut(p).gy.push(c);
        note_gy(g, c);
        crate::glog!(g, "    {} puts {} into the graveyard ({BURIED})", g.player(p).name, cname(g, c));
    }
    shuffle_library(g, p);
    Ok("gy")
}

// ================================================================== Journey to Eternity // Atzal, Cave of Eternity
/// the creature Journey enchants: p's best nontoken creature
fn journey_host(g: &mut Game, p: PlayerId, a: PermId) -> Res<Option<PermId>> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != a && live(g, m) && g.is_creature(m) && !g.perm(m).token && g.perm(m).orig == p)
        .collect();
    Ok(max_by(&cs, |m| pval(g, m)))
}

/// When enchanted creature dies, return it to the battlefield under your control (the mark Kaya's Ghostform uses),
/// then return this card to the battlefield transformed under your control: Atzal, Cave of Eternity, a land.
fn journey_dies(g: &mut Game, p: PlayerId, a: PermId, host: PermId) -> Res {
    let Some(jc) = g.perm(a).cd else { return Ok(()) };
    if !g.perm(host).token {
        g.perm_mut(host).data.set(DataKey::Ghostform, Val::Player(p));
    }
    let o = g.perm(a).orig;
    let Some(i) = g.player(o).gy.iter().position(|&x| x == jc) else { return Ok(()) };
    g.player_mut(o).gy.remove(i);
    g.add_land_entering(p, jc, false);
    crate::glog!(g, "    Journey to Eternity returns transformed: Atzal, Cave of Eternity");
    landfall(g, p)
}

static JOURNEY_SPEC: AuraSpec =
    AuraSpec { target: "own", host_pick: Some(journey_host), host_dies: Some(journey_dies), ..AURA };

/// Atzal: {T}: Add one mana of any color.
fn atzal_cols(g: &Game, p: PlayerId, _l: LandId) -> Colors {
    g.player(p).ident
}

/// the best creature card in p's graveyard to return
fn atzal_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    max_by(&cs, |c| creature_card_worth(g, c))
}

/// {3}{B}{G}, {T}: Return target creature card from your graveyard to the battlefield.
fn atzal_options(g: &mut Game, l: LandId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.land(l).tapped || g.land(l).owner != p || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 4, "BG", false) {
        return Ok(vec![]); // Atzal taps for the ability, so the cost comes from the other sources
    }
    let Some(c) = atzal_pick(g, p) else { return Ok(vec![]) };
    let w = creature_card_worth(g, c);
    if w < 4.0 {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.0 + 0.5 * w, format!("Atzal, Cave of Eternity: return {}", cname(g, c)), atzal_go, l.0 as i64)])
}

fn atzal_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !g.land(l).on_bf || g.land(l).tapped || g.land(l).owner != p {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    if !can_pay(g, p, 3, "BG", false) || !pay(g, p, 3, "BG", false)? {
        g.land_mut(l).tapped = false;
        return Ok(false);
    }
    let jc = g.land(l).cd;
    if !ability_window_card(g, p, jc, "return a creature card from your graveyard", None, None)? {
        return Ok(true);
    }
    let Some(c) = atzal_pick(g, p) else { return Ok(true) };
    let gy = &mut g.player_mut(p).gy;
    let Some(i) = gy.iter().position(|&x| x == c) else { return Ok(true) };
    gy.remove(i);
    crate::glog!(g, "    Atzal, Cave of Eternity returns {} to the battlefield", cname(g, c));
    enter(g, p, c, Enter::default())?;
    Ok(true)
}

// ================================================================== Twilight Diviner
/// Whenever one or more other creatures you control enter, if they entered or were cast from a graveyard, create a
/// token that's a copy of one of them (the best of those that came back this turn). This ability triggers only once
/// each turn.
fn diviner_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src || !live(g, src) || p != owner(g, src) || !g.perm(m).on_bf || !g.is_creature(m) {
        return Ok(());
    }
    let st = g.turn_stamp();
    if !entered_from_gy_now(g, m) || g.perm(src).data.get(DataKey::T12) == Some(&Val::Stamp(st)) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "create a token copy of a creature that came back", Some(5.0))? {
        return Ok(());
    }
    if g.perm(src).data.get(DataKey::T12) == Some(&Val::Stamp(st)) {
        return Ok(()); // another of the same batch resolved first
    }
    g.perm_mut(src).data.set(DataKey::T12, Val::Stamp(st));
    let back: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&x| x != src && live(g, x) && g.is_creature(x) && !g.perm(x).token && entered_from_gy_now(g, x))
        .collect();
    let Some(best) = max_by(&back, |x| pval(g, x)) else { return Ok(()) };
    let Some(cd) = g.perm(best).cd else { return Ok(()) };
    crate::glog!(g, "    {DIVINER}: a token copy of {}", cname(g, cd));
    enter_token_copy(g, p, cd)?;
    Ok(())
}

// ================================================================== combos
fn not_ready() -> Ready {
    (false, vec![], vec![])
}

/// Agent of Treachery: on the battlefield (blinked again and again it takes every permanent), or cast from hand
fn agent_piece(g: &Game, p: PlayerId) -> Option<(Vec<PermId>, Vec<CardId>)> {
    if let Some(a) = mine(g, p, AGENT) {
        return Some((vec![a], vec![]));
    }
    let c = held(g, p, AGENT)?;
    castable(g, p, c, "hand").then(|| (vec![], vec![c]))
}

/// Peregrine Drake + Deadeye Navigator: {1}{U} blinks the paired Drake, which untaps five lands: unbounded mana and
/// blinks; re-pairing Deadeye with Agent of Treachery takes every permanent
fn deadeye_ready(g: &Game, p: PlayerId) -> Ready {
    let (Some(d), Some(n)) = (mine(g, p, DRAKE), mine(g, p, DEADEYE)) else { return not_ready() };
    let Some((mut keys, casts)) = agent_piece(g, p) else { return not_ready() };
    keys.extend([d, n]);
    (true, keys, casts)
}

pub static COMBOS: [Combo; 2] = [
    Combo {
        name: "Peregrine Drake + Deadeye Navigator (+ Agent of Treachery)",
        groups: &[&[DRAKE], &[DEADEYE], &[AGENT]],
        ready: deadeye_ready,
        mana: (1, "U"),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Peregrine Drake + Archaeomancer + Ghostly Flicker (+ Agent of Treachery)",
        groups: &[&[DRAKE], &[ARCH], &[FLICKER], &[AGENT]],
        // --- casual combos (brago): Agent of Treachery cast with the loop's mana (or dug for), four lands, three windows
        ready: crate::impls::cc_brago::flicker_ready,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: crate::impls::cc_brago::FLICKER_STEPS,
    },
];

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, AGENT)?;
    c.etb = Some(agent_etb);
    c.end_step = Some(agent_end);
    let c = r.card(db, RELIC)?;
    c.leaves = Some(relic_leaves);
    c.options = Some(relic_options);
    r.card(db, PORT_MAGE)?.leaves = Some(mage_leaves);
    r.card(db, WISP)?.etb = Some(wisp_etb);
    let c = r.card(db, FLICKER)?;
    c.resolve = Some(flicker_resolve);
    c.hand_options = Some(flicker_options);
    c.prio = Some(|_, _, _| 0); // cast by its options and as protection (pool::PROTECTORS)
    aura(r, db, ACID, &ACID_SPEC)?;
    let c = r.card(db, ACID)?;
    c.upkeep = Some(acid_upkeep);
    c.leaves = Some(acid_leaves);
    c.prio = Some(acid_prio);
    r.card(db, GEARHULK)?.etb = Some(gearhulk_etb);
    r.card(db, VOCALIST)?.trigger_copies = Some(vocalist_copies);
    r.card(db, TIME_WIPE)?.resolve = Some(time_wipe_resolve);
    r.card(db, YORION)?.etb = Some(yorion_etb);
    let c = r.card(db, BURIED)?;
    c.resolve = Some(buried_resolve);
    c.hand_options = Some(buried_options);
    c.prio = Some(|_, _, _| 0);
    aura(r, db, JOURNEY, &JOURNEY_SPEC)?;
    let c = r.card(db, JOURNEY)?;
    c.land_cols = Some(atzal_cols);
    c.land_options = Some(atzal_options);
    r.card(db, DIVINER)?.etb = Some(diviner_etb);
    Ok(())
}
