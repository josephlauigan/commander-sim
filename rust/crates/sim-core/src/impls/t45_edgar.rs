//! Tier 4 pool deck `edgar-markov-mardu-vampires` (EDHREC's bracket-4 average list): the cards the ability language
//! doesn't cover (rust/agents/t45-edgar.md). Compiled under `// --- Tiers 4/5 (edgar)` in dsl_parse.rs: Edgar
//! Markov's attack trigger, Cordial Vampire, Indulgent Aristocrat, Drana, Liberator of Malakir and Malakir
//! Bloodwitch's drain ("each Vampire you control", "each attacking creature you control", "loses life equal to the
//! number of ..."); New Blood's "tap an untapped Vampire you control" is paid by engine/cast.rs `additional_cost`.
//! The rest is here:
//!
//! - Edgar Markov: eminence: whenever you cast another Vampire spell while Edgar is in the command zone or on the
//!   battlefield, a 1/1 black Vampire (engine/cast.rs `on_cast` calls `eminence`).
//! - Edgar, Charmed Groom // Edgar Markov's Coffin: other Vampires you control get +1/+1 (Groom face only); when it
//!   dies it returns transformed: the Coffin (not a creature) makes a 1/1 white-black lifelink Vampire and a bloodline
//!   counter each upkeep, and at three it transforms back. Left out: the Coffin is not an artifact for removal.
//! - Bloodletter of Aclazotz: an opponent losing life during your turn loses twice that (engine/life.rs).
//! - Bloodthirsty Conqueror: an opponent loses life: you gain that much (as Exquisite Blood).
//! - Vito, Thorn of the Dusk Rose: you gain life: the opponent it hurts most loses that much (its lifelink activation
//!   is compiled).
//! - Blade of the Bloodchief: a creature dies: a +1/+1 counter on the equipped creature, two on a Vampire.
//! - Captivating Vampire: tap five untapped Vampires (summoning-sick ones too): gain control of the best opposing
//!   creature for good; it becomes a Vampire. Used at the end of the turn before yours or after your combat.
//! - Clavileño, First of the Blessed: whenever you attack, an attacking non-Demon Vampire (the likeliest to die)
//!   becomes a Demon with "when this dies, draw a card and create a tapped 4/3 white-black flying Vampire Demon"
//!   (engine/zones.rs `die_triggers` calls `demon_dies`, so it works after Clavileño has gone).
//! - Damn: the compiled "destroy target creature" can't be regenerated (the `noregen` tag); overloaded for {2}{W}{W}
//!   (a hand option when the wipe is worth it) it destroys each creature.
//! - Emeritus of Woe // Demonic Tutor: enters prepared; your end step, prepared again if two or more creatures died
//!   this turn; prepared, {1}{B}: cast a copy of Demonic Tutor.
//! - Forerunner of the Legion: enters: a Vampire card from the library on top (after the shuffle); another Vampire of
//!   yours enters (tokens too): your best creature gets +1/+1 until end of turn.
//! - Knight of the Ebon Legion: your end step, a +1/+1 counter if a player lost 4 or more life this turn (its pump is
//!   compiled).
//! - Markov Baron: madness {2}{B} when discarded (the anthem, lifelink and convoke are compiled / tags).
//! - New Blood: gain control of the best opposing creature; it becomes a Vampire. Left out: the rest of the text
//!   change (its other words naming the replaced type).
//! - Olivia's Wrath: each non-Vampire creature gets -X/-X until end of turn, X the Vampires you control (cast when it
//!   kills more of theirs than of yours).
//! - Sorin, Imperious Bloodlord: +1 deathtouch and lifelink (and a +1/+1 counter on a Vampire); +1 sacrifice a
//!   Vampire for 3 damage and 3 life; -3 a Vampire creature card from hand onto the battlefield.
//! - Twilight Prophet: ascend (ten permanents, Treasure / Clue / Food / Blood tokens included: the blessing stays on
//!   it); your upkeep with the blessing: the top card to hand, each opponent loses its mana value and you gain that
//!   much. Left out: the blessing is kept on the Prophet, not the player (a new Prophet checks again).
//! - Vampire Socialite: enters: +1/+1 counters on your other Vampires if an opponent lost life this turn; while one
//!   has, your other Vampires enter with an additional +1/+1 counter.
//! - Voldaren Estate: {C}, or any colour for a Vampire spell (1 life); {5},{T}, {1} less per Vampire: a Blood token;
//!   a Blood's rummage. Left out: the life is paid whenever it taps while a Vampire spell is being paid for.
//!
//! Combos: a life-gain drain (Sanguine Bond, Vito, Marauding Blight-Priest) with a life-loss gain (Exquisite Blood,
//! Bloodthirsty Conqueror): an unbounded drain. Sanguine Bond + Exquisite Blood is combos.rs's own entry.

use crate::engine::repl;
use crate::cards::{CardDb, Colors};
use crate::engine::cast::cast_card;
use crate::engine::control::{Duration, gain_control};
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::{apply_removal, apply_wipe};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library, tutor};
use crate::engine::values::{epow, etgh, has_type, indestructible, pval, threat, untargetable};
use crate::engine::zones::{Enter, Tokens, die, draw, enter, make_tokens, max_by, min_by, searchable};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready};
use crate::impls::common::{WalkerAb, walker};
use crate::impls::lands::{can_pay_without, pay_without};
use crate::impls::partials::{at_once, eot_kw, remove_card};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};

const EDGAR: &str = "Edgar Markov";
const GROOM: &str = "Edgar, Charmed Groom // Edgar Markov's Coffin";
const BLOODLETTER: &str = "Bloodletter of Aclazotz";
const CONQUEROR: &str = "Bloodthirsty Conqueror";
const VITO: &str = "Vito, Thorn of the Dusk Rose";
const BLADE: &str = "Blade of the Bloodchief";
const CAPTIVATING: &str = "Captivating Vampire";
const CLAVILENO: &str = "Clavileño, First of the Blessed";
const DAMN: &str = "Damn";
const EMERITUS: &str = "Emeritus of Woe // Demonic Tutor";
const FORERUNNER: &str = "Forerunner of the Legion";
const KNIGHT: &str = "Knight of the Ebon Legion";
const BARON: &str = "Markov Baron";
const NEW_BLOOD: &str = "New Blood";
const WRATH: &str = "Olivia's Wrath";
const SORIN: &str = "Sorin, Imperious Bloodlord";
const PROPHET: &str = "Twilight Prophet";
const SOCIALITE: &str = "Vampire Socialite";
const ESTATE: &str = "Voldaren Estate";
const BOND: &str = "Sanguine Bond";
const BLOOD: &str = "Exquisite Blood";
const PRIEST: &str = "Marauding Blight-Priest";

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn named(g: &Game, m: PermId, name: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == name)
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// p's live permanent with this name
fn mine(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && named(g, m, name))
}

fn held(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| cname(g, c) == name)
}

fn vampire(g: &Game, m: PermId) -> bool {
    g.is_creature(m) && has_type(g, m, "vampire")
}

fn vampire_card(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.has_subtype("vampire") || d.has_kw("changeling")
}

/// p's live Vampires
fn vampires(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && vampire(g, m)).collect()
}

/// did an opponent of p lose life this turn?
fn opp_lost(g: &Game, p: PlayerId) -> bool {
    let st = g.turn_stamp();
    g.opps(p).any(|q| g.player(q).lost_turn.is_some_and(|x| x.0 == st && x.1 > 0))
}

/// the opposing creature p would most like to take (a hexproof or shrouded one can't be targeted)
fn steal_pick(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && !untargetable(g, m))
        .collect();
    max_by(&cs, |m| pval(g, m))
}

/// p takes m for good; it becomes a Vampire in addition to its other types
fn take_as_vampire(g: &mut Game, p: PlayerId, m: PermId, by: &str) {
    if gain_control(g, p, m, Duration::Permanent) {
        let x = g.perm_mut(m);
        if !x.ttypes.contains(&"vampire") {
            x.ttypes.push("vampire");
        }
        crate::glog!(g, "    {} takes {} ({by}); it is a Vampire now", g.player(p).name, g.perm(m).name);
    }
}

fn vampire_token(g: &mut Game, p: PlayerId, colors: &str, lifelink: bool) -> Res<Vec<PermId>> {
    let spec =
        Tokens { color: Some(Colors::from_letters(colors)), types: vec!["vampire"], lifelink, ..Tokens::new(1, 1) };
    make_tokens(g, p, spec)
}

// ------------------------------------------------------------------ Edgar Markov: eminence
/// "Whenever you cast another Vampire spell, if Edgar is in the command zone or on the battlefield, create a 1/1 black
/// Vampire creature token." Called by engine/cast.rs `on_cast` for every spell cast.
pub fn eminence(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    if g.over || !g.player(p).alive || cname(g, c) == EDGAR || !vampire_card(g, c) {
        return Ok(());
    }
    let pl = g.player(p);
    let in_zone = pl.commanders().any(|x| cname(g, x) == EDGAR && pl.cmd_zone_of(x));
    let src = mine(g, p, EDGAR);
    if !in_zone && src.is_none() {
        return Ok(());
    }
    if trigger_window(g, p, src, "Edgar Markov: create a 1/1 Vampire", Some(2.0))? {
        vampire_token(g, p, "B", false)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Edgar, Charmed Groom // Edgar Markov's Coffin
fn coffin(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Flipped)
}

/// other Vampires you control get +1/+1 (one per Groom, its Coffin face doesn't)
fn groom_pt(g: &Game, m: PermId) -> (i32, i32) {
    if !vampire(g, m) {
        return (0, 0);
    }
    let o = owner(g, m);
    let k = g.player(o).perms.iter().filter(|&&x| x != m && live(g, x) && named(g, x, GROOM) && !coffin(g, x)).count();
    (k as i32, k as i32)
}

fn groom_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

/// as it enters: the Groom returning from its own death trigger enters transformed (the Coffin, not a creature)
fn groom_as_enters(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    let cd = g.perm(m).cd;
    let back = (0..g.perms.len())
        .map(|i| PermId(i as u32))
        .find(|&x| x != m && g.perm(x).cd == cd && !g.perm(x).on_bf && g.perm(x).data.truthy(DataKey::Flip));
    if let Some(x) = back {
        g.perm_mut(x).data.set(DataKey::Flip, Val::Bool(false));
        let y = g.perm_mut(m);
        y.data.set(DataKey::Flipped, Val::Bool(true));
        y.data.set(DataKey::NotCreature, Val::Bool(true));
        crate::glog!(g, "    it returns transformed: Edgar Markov's Coffin");
    }
    Ok(())
}

/// "When Edgar dies, return it to the battlefield transformed under its owner's control."
fn groom_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src || coffin(g, m) || g.perm(m).token {
        return Ok(());
    }
    let (Some(cd), o) = (g.perm(m).cd, g.perm(m).orig) else { return Ok(()) };
    if !g.player(o).gy.contains(&cd) || !trigger_window(g, o, Some(m), "Edgar returns transformed", Some(3.0))? {
        return Ok(());
    }
    if !remove_card(&mut g.player_mut(o).gy, cd) {
        return Ok(()); // exiled or otherwise gone
    }
    g.perm_mut(m).data.set(DataKey::Flip, Val::Bool(true)); // read by `groom_as_enters`
    enter(g, o, cd, Enter::default())?;
    Ok(())
}

/// the Coffin, your upkeep: a 1/1 white-black lifelink Vampire and a bloodline counter; at three, transform back
fn coffin_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !live(g, src) || !coffin(g, src) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "Edgar Markov's Coffin: a 1/1 lifelink Vampire", None)? || !coffin(g, src) {
        return Ok(());
    }
    vampire_token(g, p, "WB", true)?;
    let n = g.perm(src).data.int(DataKey::Bloodline) + 1;
    if n >= 3 {
        let x = g.perm_mut(src);
        x.data.set(DataKey::Bloodline, Val::Int(0));
        x.data.set(DataKey::Flipped, Val::Bool(false));
        x.data.set(DataKey::NotCreature, Val::Bool(false));
        g.selfpt = true;
        crate::glog!(g, "    Edgar Markov's Coffin transforms into Edgar, Charmed Groom");
    } else {
        g.perm_mut(src).data.set(DataKey::Bloodline, Val::Int(n));
    }
    Ok(())
}

// ------------------------------------------------------------------ Bloodletter of Aclazotz
/// "If an opponent would lose life during your turn, they lose twice that much life instead." The life player p
/// would lose (engine/life.rs `lose_life`), doubled once per Bloodletter the active player controls.
pub fn bloodletter(g: &Game, p: PlayerId, n: i32) -> i32 {
    let Some(a) = g.active else { return n };
    if a == p || !g.player(a).alive {
        return n;
    }
    let k = g.player(a).perms.iter().filter(|&&m| live(g, m) && named(g, m, BLOODLETTER)).count().min(4);
    n.saturating_mul(1 << k)
}

// ------------------------------------------------------------------ Bloodthirsty Conqueror, Vito
/// "Whenever an opponent loses life, you gain that much life." (Exquisite Blood's loop guard: three deep)
fn conqueror(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    let o = owner(g, src);
    if p != o
        && live(g, src)
        && g.bond_depth < 3
        && trigger_window(g, o, Some(src), &format!("gain {n} life"), Some(4.0))?
    {
        g.bond_depth += 1;
        let r = gain(g, o, n);
        g.bond_depth -= 1;
        r?;
    }
    Ok(())
}

/// "Whenever you gain life, target opponent loses that much life." (one it kills, else the most threatening)
fn vito(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    if p == owner(g, src)
        && live(g, src)
        && g.opps(p).next().is_some()
        && g.bond_depth < 3
        && trigger_window(g, p, Some(src), &format!("target opponent loses {n} life"), Some(5.0))?
    {
        if g.opps(p).next().is_none() {
            return Ok(());
        }
        g.bond_depth += 1;
        let q = crate::impls::t4::bond_target(g, p, n)?;
        let r = lose_life(g, q, n, Some(p), "drain", None);
        g.bond_depth -= 1;
        r?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Blade of the Bloodchief
fn blade_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let held_by = g.perm(src).attached.filter(|&t| t != m && live(g, t) && g.is_creature(t));
    if !g.is_creature(m) || m == src || held_by.is_none() {
        return Ok(());
    }
    if !trigger_window(g, owner(g, src), Some(src), "a +1/+1 counter on the equipped creature", None)? {
        return Ok(());
    }
    if let Some(t) = g.perm(src).attached.filter(|&t| live(g, t) && g.is_creature(t)) {
        let k = if vampire(g, t) { 2 } else { 1 };
        repl::add_counters(g, t, k);
    }
    Ok(())
}

// ------------------------------------------------------------------ Captivating Vampire
/// p's untapped Vampires, the five it taps first: the least valuable (an attacker is worth more on its own turn)
fn tap_five(g: &Game, p: PlayerId) -> Option<Vec<PermId>> {
    let mut vs: Vec<PermId> = vampires(g, p).into_iter().filter(|&m| !g.perm(m).tapped).collect();
    if vs.len() < 5 {
        return None;
    }
    vs.sort_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)));
    vs.truncate(5);
    Some(vs)
}

fn captivating_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let when = post.is_none() || (post == Some(true) && g.active == Some(p));
    if owner(g, src) != p || !live(g, src) || !when {
        return Ok(vec![]);
    }
    let (Some(five), Some(t)) = (tap_five(g, p), steal_pick(g, p)) else { return Ok(vec![]) };
    let cost: f64 = five.iter().map(|&m| 0.1 * pval(g, m)).sum();
    let u = 1.0 + pval(g, t) - cost;
    if u < 2.0 {
        return Ok(vec![]);
    }
    let label = format!("Captivating Vampire: take {}", g.perm(t).name);
    Ok(vec![Opt { utility: u, label, act: Some(Action::Ability { src, f: captivating_go, arg: t.0 as i64 }) }])
}

fn captivating_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    let Some(five) = tap_five(g, p) else { return Ok(false) };
    if !live(g, t) || owner(g, t) == p {
        return Ok(false);
    }
    for m in five {
        g.perm_mut(m).tapped = true;
    }
    let label = format!("gain control of {}", g.perm(t).name);
    if ability_window(g, p, Some(src), &label, Some(5.0), Some(t))? && live(g, t) && owner(g, t) != p {
        take_as_vampire(g, p, t, CAPTIVATING);
    }
    Ok(true)
}

// ------------------------------------------------------------------ Clavileño, First of the Blessed
/// whenever you attack: the attacking non-Demon Vampire likeliest to die becomes a Demon with the death trigger
fn clavileno_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let ok = |g: &Game, m: PermId| live(g, m) && vampire(g, m) && !has_type(g, m, "demon");
    if p != owner(g, src) || !atk.iter().any(|&m| ok(g, m)) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "an attacking Vampire becomes a Demon", None)? {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> = atk.iter().copied().filter(|&m| ok(g, m)).collect();
    // the one most likely to die in this fight: the lowest toughness, a token before a card
    let Some(t) = min_by(&cands, |m| etgh(g, m) as f64 + if g.perm(m).token { 0.0 } else { 0.5 }) else {
        return Ok(vec![]);
    };
    let x = g.perm_mut(t);
    x.ttypes.push("demon");
    x.data.set(DataKey::DemonGift, Val::Bool(true));
    crate::glog!(g, "    {} becomes a Demon (Clavileño)", g.perm(t).name);
    Ok(vec![])
}

/// "When this creature dies, draw a card and create a tapped 4/3 white and black Vampire Demon creature token with
/// flying." (engine/zones.rs `die_triggers`, for a creature Clavileño marked)
pub fn demon_dies(g: &mut Game, m: PermId) -> Res {
    let p = owner(g, m);
    if !g.player(p).alive || !trigger_window(g, p, None, "draw a card and a 4/3 Vampire Demon", Some(2.0))? {
        return Ok(());
    }
    draw(g, p, 1, false)?;
    let spec = Tokens {
        color: Some(Colors::from_letters("WB")),
        types: vec!["vampire", "demon"],
        fly: true,
        tgh: Some(3),
        ..Tokens::new(1, 4)
    };
    for t in make_tokens(g, p, spec)? {
        g.perm_mut(t).tapped = true;
    }
    Ok(())
}

// ------------------------------------------------------------------ Damn
/// Damn resolves: overloaded, each creature is destroyed (no regeneration); else the compiled "destroy target
/// creature" (the `noregen` tag stops regeneration)
fn damn_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if ctx.overload {
        crate::glog!(g, "    Damn (overloaded) destroys each creature");
        let prev = std::mem::replace(&mut g.noregen, true);
        let r = apply_wipe(g, p, intern("destroy"), Some(c), ctx);
        g.noregen = prev;
        r?;
    } else {
        crate::dsl::resolve_spell(g, p, c, ctx)?;
    }
    Ok("gy")
}

/// the overload in hand: {2}{W}{W} when wiping every creature leaves p well ahead
fn damn_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "WW", false) {
        return Ok(vec![]);
    }
    let (ol, ml, _) = crate::ai::decks::wipe_eval(g, p, "destroy");
    let u = (ol - 1.2 * ml) / 2.2 - 2.5;
    if ol <= 0.0 || u < 0.5 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: u,
        label: "Damn (overloaded)".into(),
        act: Some(Action::Plan { f: damn_go, arg: c.0 as i64 }),
    }])
}

fn damn_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "WW", false) {
        return Ok(false);
    }
    g.pay_for = Some(c);
    let paid = pay(g, p, 2, "WW", false);
    g.pay_for = None;
    if !paid? {
        return Ok(false);
    }
    cast_card(g, p, c, "hand", Ctx { overload: true, ..Ctx::default() })?;
    g.player_mut(p).stat("wipes_cast", 1);
    Ok(true)
}

// ------------------------------------------------------------------ Emeritus of Woe // Demonic Tutor
fn emeritus_as_enters(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    g.perm_mut(m).data.set(DataKey::Prepared, Val::Bool(true)); // "This creature enters prepared."
    Ok(())
}

/// your end step: prepared again if two or more creatures died this turn
fn emeritus_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let died = if g.died_count.0 == Some(g.turn_stamp()) { g.died_count.1 } else { 0 };
    if p != owner(g, src) || !live(g, src) || died < 2 || g.perm(src).data.truthy(DataKey::Prepared) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "Emeritus of Woe becomes prepared", None)? && live(g, src) {
        g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(true));
    }
    Ok(())
}

fn emeritus_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let ok = post.is_some() && g.active == Some(p) && owner(g, src) == p && live(g, src);
    if !ok || !g.perm(src).data.truthy(DataKey::Prepared) || !can_pay(g, p, 1, "B", false) {
        return Ok(vec![]);
    }
    if g.player(p).library.is_empty() {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 3.0,
        label: "Demonic Tutor (Emeritus of Woe, prepared copy)".into(),
        act: Some(Action::Ability { src, f: emeritus_go, arg: 0 }),
    }])
}

fn emeritus_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !g.perm(src).data.truthy(DataKey::Prepared) || !can_pay(g, p, 1, "B", false) || !pay(g, p, 1, "B", false)? {
        return Ok(false);
    }
    g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(false));
    crate::impls::mine::cast_copy_with(g, p, None, Some("Demonic Tutor"), false, 5.0, &mut |g| tutor(g, p, "any"))?;
    Ok(true)
}

// ------------------------------------------------------------------ Forerunner of the Legion
/// enters: you may search your library for a Vampire card, then shuffle and put it on top
fn forerunner_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src {
        return forerunner_other(g, src, p, &[m]);
    }
    let lib: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| vampire_card(g, c)).collect();
    if lib.is_empty() || !trigger_window(g, p, Some(src), "a Vampire card on top of your library", None)? {
        return Ok(());
    }
    let lib: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| vampire_card(g, c)).collect();
    let Some(c) = max_by(&lib, |c| card_worth(g, p, c, false)) else { return Ok(()) };
    remove_card(&mut g.player_mut(p).library, c);
    shuffle_library(g, p);
    g.player_mut(p).library.push(c); // the end of the list is the top
    crate::glog!(g, "    Forerunner of the Legion puts a Vampire card on top");
    Ok(())
}

/// another Vampire you control enters: target creature gets +1/+1 until end of turn (your best attacker), once per
/// Vampire
fn forerunner_other(g: &mut Game, src: Src, p: PlayerId, ms: &[PermId]) -> Res {
    let o = owner(g, src);
    if p != o || !live(g, src) {
        return Ok(());
    }
    let k = ms.iter().filter(|&&m| m != src && g.perm(m).on_bf && vampire(g, m)).count();
    for _ in 0..k {
        if !trigger_window(g, o, Some(src), "target creature gets +1/+1", None)? {
            continue;
        }
        let cs: Vec<PermId> = g.player(o).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
        if let Some(t) = max_by(&cs, |m| epow(g, m) as f64 + if g.perm(m).sick { 0.0 } else { 2.0 }) {
            let x = g.perm_mut(t);
            x.eot_pt = (x.eot_pt.0 + 1, x.eot_pt.1 + 1);
        }
    }
    Ok(())
}

fn forerunner_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    forerunner_other(g, src, p, toks)
}

// ------------------------------------------------------------------ Knight of the Ebon Legion
/// your end step: a +1/+1 counter if a player lost 4 or more life this turn
fn knight_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let st = g.turn_stamp();
    let lost = g.players.iter().any(|q| q.lost_turn.is_some_and(|x| x.0 == st && x.1 >= 4));
    if p == owner(g, src) && live(g, src) && lost && trigger_window(g, p, Some(src), "a +1/+1 counter", None)? {
        repl::add_counters(g, src, 1);
    }
    Ok(())
}

// ------------------------------------------------------------------ Markov Baron: madness
/// discarded Markov Baron (kefka::discarded, every discard path): cast it for {2}{B}, or it stays in the graveyard
pub fn madness(g: &mut Game, q: PlayerId, cards: &[CardId]) -> Res {
    for &c in cards {
        if g.over || !g.player(q).alive || cname(g, c) != BARON || !g.player(q).gy.contains(&c) {
            continue;
        }
        if !can_pay(g, q, 2, "B", false) || !pay(g, q, 2, "B", false)? {
            continue;
        }
        crate::glog!(g, "    {} casts Markov Baron for its madness cost", g.player(q).name);
        cast_card(g, q, c, "escape", Ctx::default())?;
    }
    Ok(())
}

// ------------------------------------------------------------------ New Blood
fn new_blood_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let untapped = vampires(g, p).iter().any(|&m| !g.perm(m).tapped);
    match steal_pick(g, p) {
        Some(t) if untapped && pval(g, t) >= 3.0 => 60,
        _ => 0,
    }
}

fn new_blood_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = ctx.target.filter(|&t| live(g, t) && g.is_creature(t) && owner(g, t) != p).or_else(|| steal_pick(g, p));
    if let Some(t) = t {
        take_as_vampire(g, p, t, NEW_BLOOD);
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Olivia's Wrath
/// what Olivia's Wrath would do now: (the worth it kills of the opponents', of p's own)
fn wrath_eval(g: &Game, p: PlayerId) -> (f64, f64) {
    let x = vampires(g, p).len() as i32;
    let (mut theirs, mut ours) = (0.0, 0.0);
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in &q.perms {
            if !live(g, m) || !g.is_creature(m) || vampire(g, m) || etgh(g, m) > x {
                continue;
            }
            if q.id == p {
                ours += pval(g, m);
            } else {
                theirs += 1.0 + pval(g, m);
            }
        }
    }
    (theirs, ours)
}

fn wrath_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let (theirs, ours) = wrath_eval(g, p);
    if theirs - 1.2 * ours >= 6.0 { 66 } else { 0 }
}

/// each non-Vampire creature gets -X/-X until end of turn, X the Vampires you control as it resolves
fn wrath_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let x = vampires(g, p).len() as i32;
    crate::glog!(g, "    Olivia's Wrath: each non-Vampire creature gets -{x}/-{x}");
    if x == 0 {
        return Ok("gy");
    }
    g.batch += 1; // they die together
    let prev = g.cur_batch.replace(g.batch);
    let victims: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && !vampire(g, m))
        .collect();
    let mut r = Ok(());
    for v in victims {
        if !g.perm(v).on_bf {
            continue;
        }
        let y = g.perm_mut(v);
        y.eot_pt = (y.eot_pt.0 - x, y.eot_pt.1 - x);
        if etgh(g, v) <= 0 {
            r = die(g, v, intern("sba"));
            if r.is_err() {
                break;
            }
        }
    }
    g.cur_batch = prev;
    r.map(|_| "gy")
}

// ------------------------------------------------------------------ Sorin, Imperious Bloodlord
/// +1's target: the creature of p's that will use deathtouch and lifelink best (a Vampire, the biggest attacker)
fn sorin_plus_pick(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
    max_by(&cs, |m| epow(g, m) as f64 + if vampire(g, m) { 2.0 } else { 0.0 })
}

fn sorin_plus_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = sorin_plus_pick(g, p)?;
    Some(1.0 + if vampire(g, t) { 0.8 } else { 0.0 })
}

/// +1: target creature you control gains deathtouch and lifelink until end of turn; a Vampire gets a +1/+1 counter
fn sorin_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let Some(t) = sorin_plus_pick(g, p) else { return Ok(()) };
    eot_kw(g, t, "deathtouch");
    eot_kw(g, t, "lifelink");
    if vampire(g, t) {
        repl::add_counters(g, t, 1);
    }
    Ok(())
}

/// the Vampire Sorin's second +1 sacrifices: the least valuable (a token first), never the commander
fn sorin_fodder(g: &Game, p: PlayerId) -> Option<PermId> {
    let vs: Vec<PermId> = vampires(g, p).into_iter().filter(|&m| !g.perm(m).is_cmd).collect();
    min_by(&vs, |m| pval(g, m) + if g.perm(m).token { 0.0 } else { 1.0 })
}

/// the 3 damage: an opponent it kills, else the best opposing creature it kills, else the most threatening opponent
enum Hit {
    Face(PlayerId),
    Creature(PermId),
}

fn sorin_hit(g: &Game, p: PlayerId) -> Option<(Hit, f64)> {
    if let Some(q) = g.opps(p).find(|&q| g.player(q).life <= 3 && !g.player(q).life_locked) {
        return Some((Hit::Face(q), 9.0));
    }
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && etgh(g, m) <= 3 && !indestructible(g, m) && !untargetable(g, m))
        .collect();
    if let Some(m) = max_by(&cs, |m| pval(g, m)) {
        return Some((Hit::Creature(m), pval(g, m)));
    }
    let os: Vec<PlayerId> = g.opps(p).collect();
    max_by(&os, |q| threat(g, p, q)).map(|q| (Hit::Face(q), 0.8))
}

fn sorin_sac_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let f = sorin_fodder(g, p)?;
    let (_, v) = sorin_hit(g, p)?;
    let u = 0.6 + v - pval(g, f);
    (u > 0.8).then_some(u)
}

/// +1: you may sacrifice a Vampire; when you do, 3 damage to any target and you gain 3 life
fn sorin_sac(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let Some(f) = sorin_fodder(g, p) else { return Ok(()) };
    if sorin_sac_v(g, p, src).is_none() {
        return Ok(());
    }
    die(g, f, intern("sac"))?;
    if !trigger_window(g, p, Some(src), "3 damage to any target and gain 3 life", None)? {
        return Ok(());
    }
    match sorin_hit(g, p) {
        Some((Hit::Face(q), _)) => lose_life(g, q, 3, Some(p), "burn", Some(true))?,
        Some((Hit::Creature(m), _)) => apply_removal(g, Some(p), m, "dmg3", None)?,
        None => {}
    }
    gain(g, p, 3)
}

/// −3's card: the best Vampire creature card in hand
fn sorin_minus_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let hs: Vec<CardId> =
        g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).creature && vampire_card(g, c)).collect();
    max_by(&hs, |c| g.db.get(c).cmc as f64 + 0.3 * g.db.get(c).bomb as f64)
}

fn sorin_minus_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let c = sorin_minus_pick(g, p)?;
    let mv = g.db.get(c).cmc as f64;
    (mv >= 4.0).then_some(0.5 + 0.6 * mv)
}

/// −3: you may put a Vampire creature card from your hand onto the battlefield
fn sorin_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let Some(c) = sorin_minus_pick(g, p) else { return Ok(()) };
    remove_card(&mut g.player_mut(p).hand, c);
    enter(g, p, c, Enter::default())?;
    Ok(())
}

static SORIN_ABS: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "deathtouch and lifelink", val: sorin_plus_v, eff: sorin_plus },
    WalkerAb { delta: 1, label: "sacrifice a Vampire: 3 damage", val: sorin_sac_v, eff: sorin_sac },
    WalkerAb { delta: -3, label: "a Vampire from hand", val: sorin_minus_v, eff: sorin_minus },
];

// ------------------------------------------------------------------ Twilight Prophet
/// p's permanents: the battlefield's, the lands and the Treasure / Clue / Food / Blood tokens
fn permanents(g: &Game, p: PlayerId) -> u32 {
    let pl = g.player(p);
    let perms = pl.perms.iter().filter(|&&m| live(g, m)).count() as u32;
    perms + pl.lands.len() as u32 + pl.treasures + pl.clues + pl.foods + pl.bloods
}

/// ascend: the city's blessing once you control ten permanents (kept on the Prophet)
fn prophet_sba(g: &mut Game, src: Src) -> Res {
    if live(g, src) && !g.perm(src).data.truthy(DataKey::Blessing) && permanents(g, owner(g, src)) >= 10 {
        g.perm_mut(src).data.set(DataKey::Blessing, Val::Bool(true));
        crate::glog!(g, "    {} gets the city's blessing (Twilight Prophet)", g.player(owner(g, src)).name);
    }
    Ok(())
}

/// your upkeep with the blessing: the top card to hand; each opponent loses its mana value, you gain that much
fn prophet_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let blessed = g.perm(src).data.truthy(DataKey::Blessing) || permanents(g, p) >= 10;
    if p != owner(g, src) || !live(g, src) || !blessed || g.player(p).library.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "reveal the top card: to hand, drain its mana value", None)? {
        return Ok(());
    }
    g.perm_mut(src).data.set(DataKey::Blessing, Val::Bool(true));
    let Some(c) = g.player_mut(p).library.pop() else { return Ok(()) };
    g.player_mut(p).hand.push(c);
    let x = g.db.get(c).cmc as i32;
    crate::glog!(g, "    Twilight Prophet reveals {} (mana value {x})", cname(g, c));
    if x > 0 {
        for q in g.opps(p).collect::<Vec<_>>() {
            lose_life(g, q, x, Some(p), "drain", None)?;
        }
        gain(g, p, x)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Vampire Socialite
/// enters: if an opponent lost life this turn, a +1/+1 counter on each other Vampire you control; another Vampire of
/// yours entering while an opponent has lost life this turn gets its additional counter
fn socialite_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if p != o || !live(g, src) || !opp_lost(g, o) {
        return Ok(());
    }
    if m == src {
        for v in vampires(g, o).into_iter().filter(|&v| v != src) {
            repl::add_counters(g, v, 1);
        }
        crate::glog!(g, "    Vampire Socialite: +1/+1 counters on the other Vampires");
    } else if vampire(g, m) && g.perm(m).on_bf {
        repl::add_counters(g, m, 1); // enters with an additional +1/+1 counter
    }
    Ok(())
}

fn socialite_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    let o = owner(g, src);
    if p != o || !live(g, src) || !opp_lost(g, o) {
        return Ok(());
    }
    for &t in toks {
        if g.perm(t).on_bf && vampire(g, t) {
            repl::add_counters(g, t, 1);
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ Voldaren Estate
fn paying_vampire(g: &Game) -> bool {
    g.pay_for.is_some_and(|c| vampire_card(g, c))
}

/// {C}, or (paying 1 life) any colour, spent only on a Vampire spell
fn estate_cols(g: &Game, p: PlayerId, _l: PermId) -> Colors {
    if paying_vampire(g) { g.player(p).ident } else { Colors::NONE }
}

fn estate_tap(g: &mut Game, p: PlayerId, _l: PermId, _n: u32) -> Res {
    if paying_vampire(g) {
        lose_life(g, p, 1, Some(p), "life", Some(false))?;
    }
    Ok(())
}

fn estate_cost(g: &Game, p: PlayerId) -> u32 {
    5u32.saturating_sub(vampires(g, p).len() as u32)
}

/// {5},{T}, {1} less per Vampire: a Blood token (at the end of the turn before yours, with the mana spare); a Blood's
/// rummage
fn estate_options(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = vec![];
    let ok = g.perm(l).on_bf && g.player(p).lands.contains(&l) && !g.perm(l).tapped;
    let cost = estate_cost(g, p);
    if ok && post.is_none() && cost <= 3 && can_pay_without(g, p, l, cost, "") {
        let label = format!("Voldaren Estate: a Blood token ({{{cost}}})");
        out.push(Opt { utility: 1.0, label, act: Some(Action::Plan { f: estate_go, arg: l.0 as i64 }) });
    }
    let bloodseeker = g.player(p).perms.iter().any(|&m| live(g, m) && named(g, m, "Restless Bloodseeker"));
    if !bloodseeker && g.player(p).bloods > 0 && can_pay(g, p, 1, "", false) && g.player(p).hand.len() > 1 {
        let hand = g.player(p).hand.clone();
        let worst = min_by(&hand, |c| card_worth(g, p, c, false));
        if worst.is_some_and(|c| card_worth(g, p, c, false) < 15.0) && !g.player(p).library.is_empty() {
            out.push(Opt {
                utility: 1.1,
                label: "rummage with a Blood".into(),
                act: Some(Action::Plan { f: crate::impls::sheoldred::blood_rummage, arg: 0 }),
            });
        }
    }
    Ok(out)
}

fn estate_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    let cost = estate_cost(g, p);
    if !g.perm(l).on_bf || g.perm(l).tapped || !pay_without(g, p, l, cost, "")? {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    crate::impls::sheoldred::make_blood(g, p, 1)?;
    crate::glog!(g, "  {} makes a Blood token (Voldaren Estate)", g.player(p).name);
    Ok(true)
}

// ------------------------------------------------------------------ combos
const GAIN_DRAINS: [&str; 3] = [BOND, VITO, PRIEST];
const LOSS_GAINS: [&str; 2] = [BLOOD, CONQUEROR];

/// a life-gain drain and a life-loss gain: any life change loops (the plain Sanguine Bond + Exquisite Blood pair is
/// combos.rs's own entry). Both out, or one out and the other in hand to cast.
fn drain_loop_ready(g: &Game, p: PlayerId) -> Ready {
    for a in GAIN_DRAINS {
        for b in LOSS_GAINS {
            if a == BOND && b == BLOOD {
                continue;
            }
            match (mine(g, p, a), mine(g, p, b)) {
                (Some(x), Some(y)) => return (true, vec![x, y], vec![]),
                (Some(x), None) => {
                    if let Some(c) = held(g, p, b) {
                        return (true, vec![x], vec![c]);
                    }
                }
                (None, Some(y)) => {
                    if let Some(c) = held(g, p, a) {
                        return (true, vec![y], vec![c]);
                    }
                }
                (None, None) => {}
            }
        }
    }
    (false, vec![], vec![])
}

pub static COMBOS: [Combo; 1] = [Combo {
    name: "Vito / Sanguine Bond / Marauding Blight-Priest + Exquisite Blood / Bloodthirsty Conqueror",
    groups: &[&GAIN_DRAINS, &LOSS_GAINS],
    ready: drain_loop_ready,
    mana: (0, ""),
    finish: None,
    xcost: &[],
    steps: &[],
}];

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let has = |n: &str| db.id(n).is_some();
    if has(GROOM) {
        let c = r.card(db, GROOM)?;
        c.etb = Some(groom_etb);
        at_once(c, Event::Etb, true);
        c.as_enters = Some(groom_as_enters);
        c.self_dies = Some(groom_dies);
        c.upkeep = Some(coffin_upkeep);
        r.creature_pt.push(groom_pt);
    }
    if has(CONQUEROR) {
        r.card(db, CONQUEROR)?.lose_life = Some(conqueror);
    }
    if has(VITO) {
        r.card(db, VITO)?.gain_life = Some(vito);
    }
    if has(BLADE) {
        r.card(db, BLADE)?.dies = Some(blade_dies);
    }
    if has(CAPTIVATING) {
        r.card(db, CAPTIVATING)?.options = Some(captivating_options);
    }
    if has(CLAVILENO) {
        r.card(db, CLAVILENO)?.attack = Some(clavileno_attack);
    }
    if has(DAMN) {
        let c = r.card(db, DAMN)?;
        c.resolve = Some(damn_resolve);
        c.hand_options = Some(damn_options);
    }
    if has(EMERITUS) {
        let c = r.card(db, EMERITUS)?;
        c.as_enters = Some(emeritus_as_enters);
        c.end_step = Some(emeritus_end);
        c.options = Some(emeritus_options);
    }
    if has(FORERUNNER) {
        let c = r.card(db, FORERUNNER)?;
        c.etb = Some(forerunner_etb);
        c.tokens_enter = Some(forerunner_tokens);
    }
    if has(KNIGHT) {
        r.card(db, KNIGHT)?.end_step = Some(knight_end);
    }
    if has(NEW_BLOOD) {
        let c = r.card(db, NEW_BLOOD)?;
        c.resolve = Some(new_blood_resolve);
        c.prio = Some(new_blood_prio);
    }
    if has(WRATH) {
        let c = r.card(db, WRATH)?;
        c.resolve = Some(wrath_resolve);
        c.prio = Some(wrath_prio);
    }
    if has(SORIN) {
        walker(r, db, SORIN, &SORIN_ABS)?;
    }
    if has(PROPHET) {
        let c = r.card(db, PROPHET)?;
        c.sba = Some(prophet_sba);
        c.upkeep = Some(prophet_upkeep);
    }
    if has(SOCIALITE) {
        let c = r.card(db, SOCIALITE)?;
        c.etb = Some(socialite_etb);
        at_once(c, Event::Etb, true); // the other Vampires' counter is a replacement as they enter
        c.tokens_enter = Some(socialite_tokens);
    }
    if has(ESTATE) {
        let c = r.card(db, ESTATE)?;
        c.land_cols = Some(estate_cols);
        c.on_tap_land = Some(estate_tap);
        c.land_options = Some(estate_options);
    }
    if has(BLOODLETTER) {
        r.card(db, BLOODLETTER)?.pval = Some(5.0); // the doubling is engine/life.rs's, read by name
    }
    Ok(())
}
