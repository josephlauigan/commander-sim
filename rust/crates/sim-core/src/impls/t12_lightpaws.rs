//! Tier 1 pool deck `light-paws-aura-voltron` (EDHREC's bracket-2 average list): the cards the ability language
//! doesn't cover (rust/agents/t12-lightpaws.md). Transcendent Envoy and Starnheim Courser are compiled (dsl_parse.rs,
//! `// --- Tiers 1/2 (lightpaws)`); Reprobation is the removal kind `coward` (data/card_overrides.toml); the rest is
//! here. The Auras are `common::aura` specs, so Light-Paws can fetch them (`cardcode::aura_known`).
//!
//! Auras:
//! - Armored Ascension: +1/+1 per Plains you control, flying.
//! - Angelic Gift: draws a card as it enters; flying.
//! - Benevolent Blessing (flash): protection from the colour chosen as it enters (Cho-Manno's Blessing's choice: the
//!   colour of the spell it answers, else the opponents' most common). Left out: the Auras-and-Equipment exception
//!   (protection here never knocks your own Auras off).
//! - Chains of Custody: exiles the best opposing nonland permanent until it leaves; ward {2}.
//! - Face of Divinity: +2/+2; first strike and lifelink while another Aura is on the creature (`grant_kw`).
//! - Flickerform: {2}{W}{W}, used in response to removal or a wipe that would take the creature: exile it and its
//!   Auras; at the next end step the creature returns, then the Auras return attached to it (`flicker_returns`).
//!   Left out: activating it at other times (a blink for value).
//! - Holy Mantle: +2/+2, protection from creatures (unblockable, no damage from creatures: rules2::prot_vs).
//! - Idolized: whenever the creature attacks alone, +X/+X until end of turn (X: your nonland permanents).
//! - Light of Promise: whenever you gain life, that many +1/+1 counters on the creature.
//! - Mantle of the Ancients (your creature): enters: every Aura card that can enchant it and every Equipment card in
//!   your graveyard returns attached to it; +1/+1 per Aura and Equipment on it.
//! - Mask of Law and Grace: protection from black and red. Shield of Duty and Reason: from green and blue.
//! - Pariah: damage that would be dealt to you is dealt to the enchanted creature instead (combat damage in
//!   engine/combat.rs, the rest in engine/life.rs); the damage adds up over the turn and kills a creature it reaches
//!   the toughness of. The AI puts it only on an indestructible creature or one with toughness 10 or more.
//! - Stoneskin (flash): +0/+10 (also cast on a creature in response to damage it would then survive).
//! - Twinblade Blessing (flash): double strike.
//!
//! Other cards:
//! - Blacksmith's Skill: held for a response: target permanent gains hexproof and indestructible until end of turn
//!   (+2/+2 for an artifact creature).
//! - Karametra's Blessing: held for a response: +2/+2 until end of turn, and hexproof and indestructible if the
//!   creature is enchanted or an enchantment creature (the compiled pump is not used on its own).
//! - Pearl-Ear, Imperial Advisor: enchantment spells cost {1} less per Aura you control (affinity); whenever you cast
//!   an Aura spell that will go on a modified permanent of yours (counters, your Auras or Equipment on it), draw.
//! - Retether: each Aura card in your graveyard that can enchant one of your creatures returns onto one.
//! - Single Combat: each player keeps their best creature or planeswalker and sacrifices the rest; nobody can cast
//!   creature or planeswalker spells until the end of the caster's next turn (a cast lock). Cast when it costs the
//!   opponents much more than you.
//! - Thran Power Suit: equipped creature gets +1/+1 per Aura and Equipment attached to it, and ward {2}.
//! - Umbra Mystic: Auras on your permanents have umbra armor (common::umbra_save).
//! - Sheltered by Ghosts (already modelled): its exile now ends when it leaves (it had no leaves hook).
//!
//! The deck's answers to removal (`protect`) and wipes (`wipe_response`) are called for every outside deck from
//! ai/decks.rs; they also cast Shardmage's Rescue (bulk_white::white_protect), which the pool path never used.

use crate::ai::decks::{WipeRule, pay_card, wipe_loss};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::add_cast_lock;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::{epow, etgh, indestructible, no_sac, protected_from, pval};
use crate::engine::zones::{Enter, die, draw, enter, leave, max_by};
use crate::flow::Res;
use crate::hooks::{Event, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{AURA, AuraSpec, aura, aura_spec, auras_on, oring_exile, oring_return, own_host};
use crate::state::{Ctx, DataKey, Game, LockEnd, LockKind, Val};
use crate::sym::Sym;

const PARIAH: &str = "Pariah";
const FLICKERFORM: &str = "Flickerform";
const CHAINS: &str = "Chains of Custody";
const SUIT: &str = "Thran Power Suit";
const MYSTIC: &str = "Umbra Mystic";
const SKILL: &str = "Blacksmith's Skill";
const KARAMETRA: &str = "Karametra's Blessing";
const BENEVOLENT: &str = "Benevolent Blessing";
const STONESKIN: &str = "Stoneskin";
const RESCUE: &str = "Shardmage's Rescue";

/// The deck's bookkeeping (a field of `Game`): Flickerform's exiles, each (the creature's owner, its card, whether it
/// was a commander, the Auras exiled with it as (owner, card)), returned at the next end step.
#[derive(Debug, Clone, Default)]
pub struct T12 {
    pub flicker: Vec<(PlayerId, CardId, Vec<(PlayerId, CardId)>)>,
}

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

fn name_is(g: &Game, m: PermId, n: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == n)
}

fn in_hand(g: &Game, p: PlayerId, n: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == n)
}

/// the live permanents named n that p controls
fn named(g: &Game, p: PlayerId, n: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && !g.perm(m).neutered && name_is(g, m, n)).collect()
}

fn is_equipment(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).has_subtype("equipment"))
}

/// the Equipment attached to m (anyone's)
fn equipment_on(g: &Game, m: PermId) -> Vec<PermId> {
    g.players
        .iter()
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&e| live(g, e) && g.perm(e).attached == Some(m) && is_equipment(g, e))
        .collect()
}

/// the Auras and Equipment attached to m (Mantle of the Ancients, Thran Power Suit)
fn attachments(g: &Game, m: PermId) -> i32 {
    (auras_on(g, m).len() + equipment_on(g, m).len()) as i32
}

/// p's Aura permanents (attached or not): Pearl-Ear's affinity
fn auras_controlled(g: &Game, p: PlayerId) -> i32 {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| live(g, m) && g.perm(m).cd.is_some_and(|c| g.db.get(c).has_subtype("aura")))
        .count() as i32
}

/// worth a card to save: the commander, or a permanent worth 4
fn worth(g: &Game, m: PermId) -> bool {
    g.perm(m).is_cmd || pval(g, m) >= 4.0
}

/// the first colour (W U B R G) of a spell's mana cost
fn spell_color(g: &Game, spell: Option<CardId>) -> Option<char> {
    let s = spell?;
    ['W', 'U', 'B', 'R', 'G'].into_iter().find(|&ch| g.db.get(s).pips.contains(ch))
}

/// removal that destroys or deals damage: indestructible answers it
fn destroyish(kind: &str) -> bool {
    kind == "destroy" || kind.starts_with("dmg")
}

fn eot_pump(g: &mut Game, m: PermId, dp: i32, dt: i32) {
    let x = g.perm_mut(m);
    x.eot_pt = (x.eot_pt.0 + dp, x.eot_pt.1 + dt);
}

// ================================================================== the Auras
/// Armored Ascension: +1/+1 per Plains you control
fn ascension(g: &Game, p: PlayerId, _a: PermId, _m: PermId) -> (i32, i32) {
    let k = crate::impls::bulk_white::plains(g, p) as i32;
    (k, k)
}

/// Angelic Gift: draw a card as it enters
fn gift_etb(g: &mut Game, p: PlayerId, a: PermId, _host: PermId) -> Res {
    if trigger_window(g, p, Some(a), "draw a card", None)? {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// Chains of Custody: exile the best opposing nonland permanent until this Aura leaves
fn chains_etb(g: &mut Game, p: PlayerId, a: PermId, _host: PermId) -> Res {
    if !trigger_window(g, p, Some(a), "exile a nonland permanent", Some(5.0))? || !live(g, a) {
        return Ok(());
    }
    oring_exile(g, a, p, |g, m| g.perm(m).cd.is_some() || g.perm(m).token, false, false)
}

/// the O-Ring Auras left: what they exiled returns (an effect ending, not a trigger)
fn oring_aura_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
}

/// Mantle of the Ancients: +1/+1 per Aura and Equipment on the creature
fn mantle_bonus(g: &Game, _p: PlayerId, _a: PermId, m: PermId) -> (i32, i32) {
    let k = attachments(g, m);
    (k, k)
}

/// Mantle of the Ancients enters: each Aura card that can enchant the creature and each Equipment card in your
/// graveyard returns to the battlefield attached to it
fn mantle_etb(g: &mut Game, p: PlayerId, a: PermId, host: PermId) -> Res {
    let fits = |g: &Game, c: CardId| {
        let d = g.db.get(c);
        d.has_subtype("equipment")
            || (d.has_subtype("aura") && crate::cardcode::aura_own_fits(g, p, c, host) && aura_spec(g, c).is_some())
    };
    if !g.player(p).gy.iter().any(|&c| fits(g, c))
        || !trigger_window(g, p, Some(a), "return Auras and Equipment", None)?
    {
        return Ok(());
    }
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| fits(g, c)).collect();
    for c in cs {
        if !live(g, host) {
            break;
        }
        let gy = &mut g.player_mut(p).gy;
        let Some(i) = gy.iter().position(|&x| x == c) else { continue };
        gy.remove(i);
        crate::glog!(g, "    Mantle of the Ancients returns {}", g.db.get(c).name);
        if g.db.get(c).has_subtype("aura") {
            g.attach_to = Some(host);
            let r = enter(g, p, c, Enter::default());
            g.attach_to = None;
            r?;
        } else {
            let e = enter(g, p, c, Enter::default())?;
            crate::impls::cloud::attach(g, e, host);
        }
    }
    Ok(())
}

/// Face of Divinity: first strike and lifelink while another Aura is attached to the creature
fn divinity_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    (kw == "first strike" || kw == "lifelink") && g.perm(src).attached == Some(m) && auras_on(g, m).len() >= 2
}

/// Idolized: whenever the creature attacks alone, +X/+X until end of turn (X: your nonland permanents)
fn idolized_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let host = g.perm(src).attached;
    if p != owner(g, src) || atk.len() != 1 || host != Some(atk[0]) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "+X/+X (attacks alone)", None)? || !live(g, atk[0]) {
        return Ok(vec![]);
    }
    let x = g.player(p).perms.iter().filter(|&&m| live(g, m)).count() as i32;
    eot_pump(g, atk[0], x, x);
    crate::glog!(g, "    Idolized: {} gets +{x}/+{x}", g.perm(atk[0]).name);
    Ok(vec![])
}

/// Light of Promise: whenever you gain life, that many +1/+1 counters on the creature
fn promise_gain(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    let Some(host) = g.perm(src).attached else { return Ok(()) };
    if p != owner(g, src) || n <= 0 || !live(g, host) || !g.is_creature(host) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), &format!("{n} +1/+1 counters"), None)? && live(g, host) {
        crate::cardcode::put_plus(g, host, n);
    }
    Ok(())
}

/// Pariah goes only on a creature that survives the damage: indestructible, or toughness 10 or more
fn pariah_ok(g: &Game, _p: PlayerId, m: PermId) -> bool {
    indestructible(g, m) || etgh(g, m) >= 10
}

/// cast Pariah only with a creature for it
fn pariah_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let ok = g.player(p).perms.iter().any(|&m| live(g, m) && g.is_creature(m) && pariah_ok(g, p, m));
    if ok { 55 } else { 0 }
}

static ASCENSION: AuraSpec = AuraSpec { kws: &["flying"], bonus: Some(ascension), ..AURA };
static GIFT: AuraSpec = AuraSpec { kws: &["flying"], on_etb: Some(gift_etb), ..AURA };
static BENEVOLENT_SPEC: AuraSpec = AuraSpec { on_etb: Some(crate::impls::bulk_white::chomanno_etb), ..AURA };
static CHAINS_SPEC: AuraSpec = AuraSpec { on_etb: Some(chains_etb), ..AURA };
static DIVINITY: AuraSpec = AuraSpec { pow: 2, tgh: 2, ..AURA };
static HOLY: AuraSpec = AuraSpec { pow: 2, tgh: 2, kws: &["unblockable"], ..AURA };
static MANTLE: AuraSpec = AuraSpec { bonus: Some(mantle_bonus), on_etb: Some(mantle_etb), ..AURA };
static MASK: AuraSpec = AuraSpec { prot: "BR", ..AURA };
static SHIELD: AuraSpec = AuraSpec { prot: "GU", ..AURA };
static PARIAH_SPEC: AuraSpec = AuraSpec { host_ok: Some(pariah_ok), ..AURA };
static STONESKIN_SPEC: AuraSpec = AuraSpec { tgh: 10, ..AURA };
static TWINBLADE: AuraSpec = AuraSpec { kws: &["double strike"], ..AURA };

// ================================================================== Pariah
/// the live Pariah p controls on a creature, and that creature
fn pariah_of(g: &Game, p: PlayerId) -> Option<(PermId, PermId)> {
    named(g, p, PARIAH).into_iter().find_map(|a| {
        let h = g.perm(a).attached?;
        (live(g, h) && g.is_creature(h)).then_some((a, h))
    })
}

/// engine/combat.rs and engine/life.rs: dmg damage would be dealt to p: with a Pariah out it is dealt to the enchanted
/// creature instead (added to the damage it has taken this turn; `pariah_finish` kills it). True if redirected.
pub fn pariah_take(g: &mut Game, p: PlayerId, dmg: i32) -> bool {
    if g.auras.is_empty() || dmg <= 0 {
        return false;
    }
    let Some((a, h)) = pariah_of(g, p) else { return false };
    let st = g.turn_stamp();
    let before = match g.perm(a).data.get(DataKey::T12Pariah) {
        Some(Val::List(v)) if v.first() == Some(&Val::Stamp(st)) => match v.get(1) {
            Some(Val::Int(n)) => *n,
            _ => 0,
        },
        _ => 0,
    };
    g.perm_mut(a).data.set(DataKey::T12Pariah, Val::List(vec![Val::Stamp(st), Val::Int(before + dmg as i64)]));
    crate::glog!(g, "    Pariah: {dmg} damage to {} is dealt to {} instead", g.player(p).name, g.perm(h).name);
    g.mark_damaged(h); // "dealt damage this turn"
    true
}

/// the creatures under a Pariah that have taken damage equal to their toughness this turn die (after combat damage,
/// or at once for other damage); one that survives being destroyed (umbra armor, regeneration) loses that damage
pub fn pariah_finish(g: &mut Game) -> Res {
    if g.auras.is_empty() {
        return Ok(());
    }
    let st = g.turn_stamp();
    for a in g.auras.clone() {
        if !live(g, a) || !name_is(g, a, PARIAH) {
            continue;
        }
        let Some(h) = g.perm(a).attached.filter(|&h| live(g, h)) else { continue };
        let n = match g.perm(a).data.get(DataKey::T12Pariah) {
            Some(Val::List(v)) if v.first() == Some(&Val::Stamp(st)) => match v.get(1) {
                Some(Val::Int(n)) => *n as i32,
                _ => 0,
            },
            _ => 0,
        };
        if n <= 0 || n < etgh(g, h) || indestructible(g, h) {
            continue;
        }
        die(g, h, "destroy")?;
        if live(g, h) && g.perm(a).on_bf {
            g.perm_mut(a).data.remove(DataKey::T12Pariah);
        }
    }
    Ok(())
}

// ================================================================== Flickerform
/// exile m and every Aura on it; the creature comes back at the next end step, then the Auras attached to it
fn flicker_exile(g: &mut Game, m: PermId) -> Res {
    let mut back = vec![];
    for a in auras_on(g, m) {
        let x = g.perm(a);
        let (o, cd, tok) = (x.orig, x.phys.or(x.cd), x.token);
        leave(g, a)?;
        if let Some(cd) = cd.filter(|_| !tok) {
            g.player_mut(o).exile.push(cd);
            back.push((o, cd));
        }
    }
    let x = g.perm(m);
    let (o, cd, tok) = (x.orig, x.phys.or(x.cd), x.token);
    crate::glog!(g, "    Flickerform exiles {} and {} Aura(s) until the end step", g.perm(m).name, back.len());
    leave(g, m)?;
    if let Some(cd) = cd.filter(|_| !tok) {
        g.player_mut(o).exile.push(cd); // not to the command zone: it returns at the end step
        g.t12lp.flicker.push((o, cd, back));
    }
    Ok(())
}

/// cardcode::delayed_end_step: Flickerform's creatures return, then the Auras exiled with each, attached to it
pub fn flicker_returns(g: &mut Game) -> Res {
    if g.t12lp.flicker.is_empty() {
        return Ok(());
    }
    for (o, cd, auras) in std::mem::take(&mut g.t12lp.flicker) {
        if !g.player(o).alive {
            continue;
        }
        let Some(i) = g.player(o).exile.iter().position(|&x| x == cd) else { continue };
        g.player_mut(o).exile.remove(i);
        let n = enter(g, o, cd, Enter::default())?;
        g.perm_mut(n).is_cmd = cd == g.player(o).cmd;
        crate::glog!(g, "    {} returns to the battlefield (Flickerform)", g.db.get(cd).name);
        for (ao, acd) in auras {
            let ex = &mut g.player_mut(ao).exile;
            let Some(j) = ex.iter().position(|&x| x == acd) else { continue };
            ex.remove(j);
            if !live(g, n) {
                g.player_mut(ao).exile.push(acd);
                continue;
            }
            g.attach_to = Some(n);
            let r = enter(g, ao, acd, Enter::default());
            g.attach_to = None;
            r?;
        }
    }
    Ok(())
}

/// p's Flickerform on m, if p can pay {2}{W}{W}
fn flickerform_on(g: &Game, p: PlayerId, m: PermId) -> Option<PermId> {
    if !can_pay(g, p, 2, "WW", false) {
        return None;
    }
    auras_on(g, m).into_iter().find(|&a| owner(g, a) == p && !g.perm(a).neutered && name_is(g, a, FLICKERFORM))
}

/// activate Flickerform f on m: true once m is exiled
fn flicker_go(g: &mut Game, p: PlayerId, f: PermId, m: PermId) -> Res<bool> {
    if !pay(g, p, 2, "WW", false)? {
        return Ok(false);
    }
    if !ability_window(g, p, Some(f), "exile enchanted creature and its Auras", None, None)? || !live(g, m) {
        return Ok(false);
    }
    flicker_exile(g, m)?;
    g.player_mut(p).stat("protection_used", 1);
    Ok(true)
}

// ================================================================== answers to removal and wipes
/// cast a held instant (Blacksmith's Skill, Karametra's Blessing) from p's hand; true if it resolved
fn cast_held(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    if !crate::engine::cast::castable(g, p, c, "hand") {
        return Ok(false);
    }
    pay_card(g, p, c, 0)
}

/// ai/decks.rs: targeted removal of kind `kind` is aimed at owner's permanent m. In order: Blacksmith's Skill and
/// Karametra's Blessing (hexproof and indestructible: the removal loses its target), Benevolent Blessing (protection
/// from the spell's colour), Stoneskin (damage it then survives), Shardmage's Rescue (bulk_white), Flickerform (the
/// creature leaves until the end step). True when the removal is stopped.
pub fn protect(
    g: &mut Game,
    p: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if actor.is_none() || actor == Some(p) || !live(g, m) || owner(g, m) != p || !worth(g, m) {
        return Ok(false);
    }
    if matches!(kind, "edict" | "sacrifice" | "wipe") {
        return Ok(false);
    }
    let creature = g.is_creature(m);
    if let Some(c) = in_hand(g, p, SKILL)
        && cast_held(g, p, c)?
    {
        crate::impls::partials::eot_kw(g, m, "hexproof");
        crate::impls::partials::eot_kw(g, m, "indestructible");
        if creature && g.perm(m).cd.is_some_and(|d| g.db.get(d).types.has(Types::ARTIFACT)) {
            eot_pump(g, m, 2, 2);
        }
        crate::glog!(g, "    Blacksmith's Skill: {} gains hexproof and indestructible", g.perm(m).name);
        return Ok(true);
    }
    let enchanted =
        !auras_on(g, m).is_empty() || g.perm(m).cd.is_some_and(|d| g.db.get(d).types.has(Types::ENCHANTMENT));
    if creature
        && enchanted
        && let Some(c) = in_hand(g, p, KARAMETRA)
        && cast_held(g, p, c)?
    {
        eot_pump(g, m, 2, 2);
        crate::impls::partials::eot_kw(g, m, "hexproof");
        crate::impls::partials::eot_kw(g, m, "indestructible");
        crate::glog!(g, "    Karametra's Blessing: {} gets +2/+2, hexproof and indestructible", g.perm(m).name);
        return Ok(true);
    }
    if creature
        && let Some(ch) = spell_color(g, spell)
        && let Some(c) = in_hand(g, p, BENEVOLENT)
        && can_pay(g, p, 1, "W", false)
        && crate::impls::bulk_white::cast_aura_on(g, p, c, m, 1, "W", Some(ch))?
    {
        return Ok(protected_from(g, m, Colors::from_letters(&ch.to_string())));
    }
    if creature
        && let Some(n) = kind.strip_prefix("dmg").and_then(|s| s.parse::<i32>().ok())
        && etgh(g, m) <= n
        && etgh(g, m) + 10 > n
        && !indestructible(g, m)
        && let Some(c) = in_hand(g, p, STONESKIN)
        && can_pay(g, p, 2, "W", false)
        && crate::impls::bulk_white::cast_aura_on(g, p, c, m, 2, "W", None)?
    {
        return Ok(etgh(g, m) > n);
    }
    if creature
        && in_hand(g, p, RESCUE).is_some()
        && crate::impls::bulk_white::white_protect(g, p, m, kind, actor, spell)?
    {
        return Ok(true);
    }
    if destroyish(kind) && indestructible(g, m) {
        return Ok(false); // nothing to spend: it survives anyway
    }
    if let Some(f) = flickerform_on(g, p, m) {
        return flicker_go(g, p, f, m);
    }
    Ok(false)
}

/// ai/decks.rs: a wipe is coming: Flickerform saves the creature it is on (exiled until the end step) when the wipe
/// would take it and it is worth it
pub fn wipe_response(g: &mut Game, q: PlayerId, kind: Sym, caster: PlayerId) -> Res {
    if q == caster || g.auras.is_empty() || wipe_loss(g, q, kind, caster) < 4.0 {
        return Ok(());
    }
    let rule = WipeRule::new(g, caster, kind);
    for a in named(g, q, FLICKERFORM) {
        let Some(m) = g.perm(a).attached.filter(|&m| live(g, m) && owner(g, m) == q) else { continue };
        if !worth(g, m) || !rule.hits(g, m, q) || flickerform_on(g, q, m) != Some(a) {
            continue;
        }
        flicker_go(g, q, a, m)?;
    }
    Ok(())
}

// ================================================================== static abilities read by the engine
/// cardcode::aura_ward: ward {2} from Chains of Custody and Thran Power Suit
pub fn ward(g: &Game, m: PermId) -> u32 {
    let chains = if g.auras.is_empty() { 0 } else { auras_on(g, m).iter().filter(|&&a| name_is(g, a, CHAINS)).count() };
    let suits = g
        .player(owner(g, m))
        .perms
        .iter()
        .filter(|&&e| live(g, e) && g.perm(e).attached == Some(m) && name_is(g, e, SUIT))
        .count();
    2 * (chains + suits) as u32
}

/// common::umbra_save: Umbra Mystic: the Auras on m have umbra armor (m's controller controls a Mystic)
pub fn mystic_umbra(g: &Game, m: PermId) -> bool {
    !named(g, owner(g, m), MYSTIC).is_empty()
}

/// Thran Power Suit: +1/+1 per Aura and Equipment on the creature wearing it (common.CREATURE_PT)
fn suit_pt(g: &Game, m: PermId) -> (i32, i32) {
    let suits = equipment_on(g, m).into_iter().filter(|&e| name_is(g, e, SUIT) && !g.perm(e).neutered).count() as i32;
    if suits == 0 {
        return (0, 0);
    }
    let k = suits * attachments(g, m);
    (k, k)
}

/// a permanent with a size rule entered: turn the rule on (runs at once)
fn selfpt_on(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

// ================================================================== Pearl-Ear, Imperial Advisor
/// Enchantment spells you cast have affinity for Auras
fn pearl_ear_cost(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    if caster == owner(g, src) && g.db.get(c).types.has(Types::ENCHANTMENT) { -auras_controlled(g, caster) } else { 0 }
}

/// a modified permanent: counters, your Auras or Equipment on it
fn modified(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).plus != 0
        || auras_on(g, m).iter().any(|&a| owner(g, a) == p)
        || equipment_on(g, m).iter().any(|&e| owner(g, e) == p)
}

/// Whenever you cast an Aura spell that targets a modified permanent you control, draw a card (its target: the
/// creature it is being cast onto, else the one the AI will put it on)
fn pearl_ear_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster != owner(g, src) || !g.db.get(c).has_subtype("aura") {
        return Ok(());
    }
    let Some(spec) = aura_spec(g, c) else { return Ok(()) };
    let host = g.attach_to.filter(|&h| live(g, h)).or_else(|| own_host(g, caster, spec, None));
    let Some(h) = host.filter(|&h| owner(g, h) == caster && spec.target != "opp") else { return Ok(()) };
    if modified(g, caster, h) && trigger_window(g, caster, Some(src), "draw a card", None)? {
        draw(g, caster, 1, false)?;
    }
    Ok(())
}

// ================================================================== Retether
/// the Aura cards in p's graveyard that can enchant one of p's creatures
fn retether_cards(g: &Game, p: PlayerId) -> Vec<CardId> {
    let cs: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
    g.player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| {
            g.db.get(c).has_subtype("aura")
                && aura_spec(g, c).is_some()
                && cs.iter().any(|&m| crate::cardcode::aura_own_fits(g, p, c, m))
        })
        .collect()
}

/// Return each Aura card from your graveyard to the battlefield (each picks its creature; one that can't enchant a
/// creature stays in the graveyard)
fn retether_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for c in retether_cards(g, p) {
        let gy = &mut g.player_mut(p).gy;
        let Some(i) = gy.iter().position(|&x| x == c) else { continue };
        gy.remove(i);
        crate::glog!(g, "    Retether returns {}", g.db.get(c).name);
        enter(g, p, c, Enter::default())?;
    }
    Ok("done")
}

fn retether_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if retether_cards(g, p).len() >= 2 { 60 } else { 0 }
}

// ================================================================== Single Combat
/// q's creatures and planeswalkers
fn fighters(g: &Game, q: PlayerId) -> Vec<PermId> {
    g.player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            live(g, m) && (g.is_creature(m) || g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER)))
        })
        .collect()
}

/// the one q keeps (the best; the commander first for its caster)
fn keeper(g: &Game, q: PlayerId) -> Option<PermId> {
    max_by(&fighters(g, q), |m| pval(g, m) + if g.perm(m).is_cmd { 5.0 } else { 0.0 } + 0.1 * epow(g, m) as f64)
}

/// what q would sacrifice to Single Combat
fn combat_loss(g: &Game, q: PlayerId) -> f64 {
    let k = keeper(g, q);
    fighters(g, q).into_iter().filter(|&m| Some(m) != k && !no_sac(g, m)).map(|m| pval(g, m)).sum()
}

fn single_combat_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let theirs: f64 = g.opps(p).map(|q| combat_loss(g, q)).sum();
    if theirs >= 8.0 && theirs - 1.5 * combat_loss(g, p) >= 8.0 { 55 } else { 0 }
}

/// Each player chooses a creature or planeswalker they control, then sacrifices the rest. Players can't cast creature
/// or planeswalker spells until the end of your next turn.
fn single_combat_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let order: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    let keep: Vec<(PlayerId, Option<PermId>)> = order.iter().map(|&q| (q, keeper(g, q))).collect();
    for (q, k) in keep {
        for m in fighters(g, q) {
            if Some(m) != k && live(g, m) && !no_sac(g, m) {
                die(g, m, "sac")?;
            }
        }
    }
    let t = g.player(p).turns;
    for q in order {
        add_cast_lock(g, q, LockKind::CreatureWalker, LockEnd::ThroughNextTurn(p, t));
    }
    crate::engine::life::check_state(g)?;
    Ok("done")
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let has = |n: &str| db.id(n).is_some();
    for (name, spec) in [
        ("Armored Ascension", &ASCENSION),
        ("Angelic Gift", &GIFT),
        (BENEVOLENT, &BENEVOLENT_SPEC),
        (CHAINS, &CHAINS_SPEC),
        ("Face of Divinity", &DIVINITY),
        (FLICKERFORM, &AURA),
        ("Holy Mantle", &HOLY),
        ("Idolized", &AURA),
        ("Light of Promise", &AURA),
        ("Mantle of the Ancients", &MANTLE),
        ("Mask of Law and Grace", &MASK),
        ("Shield of Duty and Reason", &SHIELD),
        (PARIAH, &PARIAH_SPEC),
        (STONESKIN, &STONESKIN_SPEC),
        ("Twinblade Blessing", &TWINBLADE),
    ] {
        if has(name) {
            aura(r, db, name, spec)?;
        }
    }
    if has(CHAINS) {
        let c = r.card(db, CHAINS)?;
        c.leaves = Some(oring_aura_leaves);
        *c = c.at_once(Event::Leaves);
    }
    // Sheltered by Ghosts (common.rs): its exile ends when it leaves
    if has("Sheltered by Ghosts") {
        let c = r.card(db, "Sheltered by Ghosts")?;
        c.leaves = Some(oring_aura_leaves);
        *c = c.at_once(Event::Leaves);
    }
    if has("Face of Divinity") {
        r.card(db, "Face of Divinity")?.grant_kw = Some(divinity_kw);
    }
    if has("Idolized") {
        r.card(db, "Idolized")?.attack = Some(idolized_attack);
    }
    if has("Light of Promise") {
        r.card(db, "Light of Promise")?.gain_life = Some(promise_gain);
    }
    if has(PARIAH) {
        r.card(db, PARIAH)?.prio = Some(pariah_prio);
    }
    for name in [SKILL, KARAMETRA] {
        if has(name) {
            r.card(db, name)?.prio = Some(|_, _, _| 0); // held for a response (`protect`)
        }
    }
    if has(SUIT) {
        let c = r.card(db, SUIT)?;
        c.etb = Some(selfpt_on);
        *c = c.at_once(Event::Etb);
        r.creature_pt.push(suit_pt);
    }
    if has("Pearl-Ear, Imperial Advisor") {
        let c = r.card(db, "Pearl-Ear, Imperial Advisor")?;
        c.cost = Some(pearl_ear_cost);
        c.cast = Some(pearl_ear_cast);
    }
    if has("Retether") {
        let c = r.card(db, "Retether")?;
        c.resolve = Some(retether_resolve);
        c.prio = Some(retether_prio);
    }
    if has("Single Combat") {
        let c = r.card(db, "Single Combat")?;
        c.resolve = Some(single_combat_resolve);
        c.prio = Some(single_combat_prio);
    }
    Ok(())
}
