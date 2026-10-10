//! Python's `cards/impl/zur.py`: card rules first written for the Zur deck (removed 2026-10-07), kept for the cards
//! other lists still run (mostly the Y'shtola deck, which also uses the protection and wipe responses here).
//!
//! - Auras that lock a permanent down while they stay attached: Arrest and Prison Sentence (can't attack or block, no
//!   activated abilities), Luminous Bonds and Bound in Silence (can't attack or block), Encrust (doesn't untap, no
//!   activated abilities). Cast, they target as they enter (engine::removal::etb_removal); put onto the battlefield
//!   by Zur, they don't target, so hexproof and ward don't stop them (protection does). Python's 'kasmina' kind
//!   (loses all abilities, base 1/1) has no Aura registered with it, so it isn't ported.
//! - Zur's attack trigger for the Y'shtola deck's 99 (`zur_fetch`: what to fetch is yshtola.fetch_value).
//! - Other cards that need code: Bastion Protector (its anthem is compiled), The Eternal Wanderer, Prayer of Binding,
//!   Rootborn Defenses, Disenchant (tags only), and Azorius Guildmage (no list runs it; the stack tests use it).
//! - Protection and wipe responses (Clever Concealment, Rootborn Defenses, Restoration Angel), used by Y'shtola's AI.

use crate::cards::{CardDb, Colors, Types};
use crate::dsl::model::Ability;
use crate::engine::combat::can_block;
use crate::engine::life::gain;
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, ability_window_card, trigger_window};
use crate::engine::values::{epow, etgh, protected_from, pval, untargetable};
use crate::engine::zones::{Enter, Tokens, Zone, die, enter, leave, make_tokens, max_by, searchable, to_zone_card};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{AURA, AuraSpec};
use crate::pysum::PySum;
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

/// `m in p.perms`
fn controls(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.perm(m).owner == p
}

/// the card name of a permanent ("" for a token)
fn card_name(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn pname(g: &Game, p: PlayerId) -> &'static str {
    g.player(p).name
}

fn remove_first(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

fn is_artifact(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT))
}

/// `g.eot_kw.setdefault(id(m), set()).add(kw)`: a keyword until end of turn
fn add_kw(g: &mut Game, m: PermId, kw: Sym) {
    let k = &mut g.perm_mut(m).eot_kw;
    if !k.contains(&kw) {
        k.push(kw);
    }
}

// ================================================================== Auras that lock a permanent down
/// zur.LOCKS, by its LOCK_KINDS key: the Aura's kind
const LOCKS: [(&str, &str); 5] = [
    ("Arrest", "arrest"),
    ("Prison Sentence", "arrest"),
    ("Luminous Bonds", "pacify"),
    ("Bound in Silence", "pacify"),
    ("Encrust", "encrust"),
];

/// zur.LOCK_KINDS: the locks a kind puts on what it enchants: pacify (can't attack or block), noact (no activated
/// abilities), frozen (doesn't untap), neuter (loses all abilities; base 1/1)
fn lock_kinds(kind: &str) -> &'static [&'static str] {
    match kind {
        "arrest" => &["pacify", "noact"],
        "pacify" => &["pacify"],
        "encrust" => &["frozen", "noact"],
        "kasmina" => &["neuter"],
        _ => &[],
    }
}

/// the lock kind of Aura `name` (the LOCK_KINDS key whose locks are LOCKS[name]); None: not a lock Aura
pub fn lock_kind(name: &str) -> Option<&'static str> {
    LOCKS.iter().find(|x| x.0 == name).map(|x| x.1)
}

/// zur.LOCKS[name]: the locks Aura `name` puts on what it enchants (none for another card)
pub fn locks_of(name: &str) -> &'static [&'static str] {
    lock_kind(name).map_or(&[], lock_kinds)
}

/// the lock Auras' AURA entry (zur.lock_aura): no bonus, goes on an opponent's permanent, and no enter hook of its
/// own (it attaches as removal: apply_lock)
static LOCK_AURA: AuraSpec = AuraSpec { target: "opp", ..AURA };

/// the lock Auras of g.auras on m, on the battlefield and not phased out
fn lock_auras_on(g: &Game, m: PermId) -> impl Iterator<Item = PermId> + '_ {
    g.auras.iter().copied().filter(move |&a| {
        let x = g.perm(a);
        x.attached == Some(m) && x.cd.is_some() && !x.phased && controls(g, x.owner, a)
    })
}

/// zur.locked: is permanent m locked down by an Aura (kind: pacify, noact, frozen, neuter)?
pub fn locked(g: &Game, m: PermId, kind: &str) -> bool {
    lock_auras_on(g, m).any(|a| locks_of(card_name(g, a)).contains(&kind))
}

/// zur.lock_factor: how much of m's value is left under the Auras locking it
pub fn lock_factor(g: &Game, m: PermId) -> f64 {
    let mut f: f64 = 1.0;
    for a in lock_auras_on(g, m) {
        let ks = locks_of(card_name(g, a));
        if ks.is_empty() {
            continue;
        }
        if ks.contains(&"neuter") {
            f = f.min(0.15);
        } else if ks.contains(&"pacify") {
            f = f.min(if ks.contains(&"noact") { 0.3 } else { 0.4 });
        } else if ks.contains(&"frozen") {
            f = f.min(0.35);
        }
    }
    f
}

/// zur.lock_attach: Aura a (already on the battlefield) enchants host and locks it. (The 'neuter' kind's ability
/// loss is not ported: no Aura has it.)
pub fn lock_attach(g: &mut Game, a: PermId, host: PermId) -> Res<bool> {
    if !g.perm(host).on_bf || !g.perm(a).on_bf {
        return Ok(false);
    }
    g.perm_mut(a).attached = Some(host);
    if !g.auras.contains(&a) {
        g.auras.push(a);
    }
    g.dsl_on = true;
    g.bf_ver += 1;
    crate::glog!(g, "    {} enchants {} ({})", card_name(g, a), g.perm(host).name, pname(g, owner(g, host)));
    if card_name(g, a) == "Prison Sentence" {
        crate::cardcode::scry(g, owner(g, a), 2, false)?;
    }
    Ok(true)
}

/// zur.apply_lock: apply_removal's lock kinds (arrest, pacify, encrust, kasmina): the entering Aura (g.rem_src)
/// enchants m
pub fn apply_lock(g: &mut Game, _actor: Option<PlayerId>, m: PermId, _kind: Sym) -> Res {
    let Some(a) = g.rem_src else { return Ok(()) };
    if g.perm(a).cd.is_none() {
        return Ok(());
    }
    lock_attach(g, a, m)?;
    Ok(())
}

/// zur.lock_host: the best opposing permanent for lock Aura `name` (targeted: cast, so hexproof and ward stop it), or
/// None
pub fn lock_host(g: &Game, p: PlayerId, name: &str, targeted: bool, exclude: &[PermId]) -> Option<PermId> {
    let kind = lock_kind(name)?;
    let ok_art = name == "Encrust";
    let col = Colors::from_letters(if name != "Encrust" && name != "Kasmina's Transmutation" { "W" } else { "U" });
    let (mut best, mut bv) = (None, 0.0);
    for q in g.opps(p) {
        for &m in &g.player(q).perms {
            if g.perm(m).phased || exclude.contains(&m) {
                continue;
            }
            if !(g.is_creature(m) || (ok_art && is_artifact(g, m))) {
                continue;
            }
            if protected_from(g, m, col) {
                continue;
            }
            if targeted && untargetable(g, m) {
                continue;
            }
            if locks_of(name).iter().any(|k| locked(g, m, k)) {
                continue;
            }
            let v = lock_worth(g, p, m, kind);
            if v > bv {
                (best, bv) = (Some(m), v);
            }
        }
    }
    if bv >= 2.0 { best } else { None }
}

/// zur.lock_worth: what locking m this way takes away from its controller
pub fn lock_worth(g: &Game, _p: PlayerId, m: PermId, kind: &str) -> f64 {
    let v = pval(g, m) * lock_factor(g, m);
    let acts = activated(g, m);
    let pw = epow(g, m).min(4) as f64;
    match kind {
        "pacify" => {
            if !g.is_creature(m) {
                return 0.0;
            }
            v * (0.45 + 0.1 * pw) * if acts { 0.5 } else { 1.0 }
        }
        "arrest" => v * (0.55 + 0.08 * pw + if acts { 0.4 } else { 0.0 }),
        "encrust" => {
            if g.is_creature(m) || acts {
                v * (0.5 + if acts { 0.5 } else { 0.0 })
            } else {
                0.0
            }
        }
        "kasmina" => {
            if !g.is_creature(m) {
                return 0.0;
            }
            let live = g
                .perm(m)
                .cd
                .is_some_and(|c| g.registry.get(c).is_some_and(|i| i.live()) || g.db.get(c).has_dsl() || acts);
            v * if live { 0.9 } else { 0.6 }
        }
        _ => v,
    }
}

/// zur.activated: does m have activated abilities worth stopping (mana abilities included)?
pub fn activated(g: &Game, m: PermId) -> bool {
    let Some(c) = g.perm(m).cd else { return false };
    let d = g.db.get(c);
    if d.tag(Tag::Rock) || d.tag(Tag::Dork) || d.tag(Tag::Clamp) {
        return true;
    }
    if g.registry.get(c).is_some_and(|i| i.options.is_some()) {
        return true;
    }
    if matches!(&*d.name, "Lightning Greaves" | "Swiftfoot Boots" | "Whispersilk Cloak") {
        return true;
    }
    d.abilities.iter().any(|a| matches!(a, Ability::Activated { .. } | Ability::Loyalty { .. }))
}

/// zur.untargetable_by_you: shroud (Lightning Greaves) stops even your own spells and abilities
pub fn untargetable_by_you(g: &Game, m: PermId) -> bool {
    let o = owner(g, m);
    g.player(o)
        .perms
        .iter()
        .any(|&e| g.perm(e).attached == Some(m) && card_name(g, e) == "Lightning Greaves" && controls(g, o, e))
        || crate::dsl::has_kw(g, m, "shroud")
}

// ================================================================== Azorius Guildmage
/// {2}{W}: tap target creature (before your attack: the creature that would block your best attacker). Its {2}{U}
/// counter is partials.answer_ability.
fn guildmage(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post != Some(false) || g.active != Some(p) || !can_pay(g, p, 2, "W", false) {
        return Ok(vec![]);
    }
    let Some(t) = guildmage_target(g, p) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 1.0 + 0.5 * pval(g, t),
        label: format!("Azorius Guildmage: tap {}", g.perm(t).name),
        act: Some(Action::Ability { src, f: guildmage_go, arg: t.0 as i64 }),
    }])
}

fn guildmage_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !controls(g, p, src) || !g.perm(t).on_bf || g.perm(t).tapped || !can_pay(g, p, 2, "W", false) {
        return Ok(false);
    }
    pay(g, p, 2, "W", false)?;
    crate::glog!(g, "  {} activates Azorius Guildmage: tap {}", pname(g, p), g.perm(t).name);
    let name = format!("tap {}", g.perm(t).name);
    if ability_window(g, p, Some(src), &name, None, Some(t))? {
        g.perm_mut(t).tapped = true;
    }
    Ok(true)
}

/// zur.guildmage_target: an untapped opposing creature that could block your best attacker (your commander first)
/// and win the fight (also Galadriel's tappers)
pub fn guildmage_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let mine: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.tapped && !x.phased && (!x.sick || x.is_cmd)
        })
        .collect();
    let a = max_by(&mine, |m| (g.perm(m).is_cmd as i32 * 5 + epow(g, m)) as f64)?;
    let blockers: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&b| {
            let x = g.perm(b);
            g.is_creature(b)
                && !x.tapped
                && !x.phased
                && !untargetable(g, b)
                && can_block(g, b, a)
                && epow(g, b) >= etgh(g, a)
        })
        .collect();
    max_by(&blockers, |b| epow(g, b) as f64)
}

// ================================================================== The Eternal Wanderer
/// the ability used this turn (`src.data.get('act') == turn_stamp(g)`)
fn wanderer_used(g: &Game, src: PermId) -> bool {
    matches!(g.perm(src).data.get(DataKey::Act), Some(Val::Stamp(s)) if *s == g.turn_stamp())
}

/// +1: exile up to one target artifact or creature until its owner's next end step. 0: a 2/2 double-strike Samurai.
/// -4: each player keeps one creature and sacrifices the rest. (Only one creature can attack it: not modeled.)
fn wanderer(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post.is_none() || g.perm(src).phased || g.perm(src).loyalty.is_none() {
        return Ok(vec![]);
    }
    if wanderer_used(g, src) {
        return Ok(vec![]);
    }
    let mut out = vec![];
    let t = wanderer_target(g, p);
    let opt = |utility: f64, label: String, arg: i64| Opt {
        utility,
        label,
        act: Some(Action::Ability { src, f: wanderer_go, arg }),
    };
    if let Some(t) = t {
        out.push(opt(
            1.5 + 0.6 * pval(g, t),
            format!("The Eternal Wanderer +1 (exile {})", g.perm(t).name),
            WANDERER_PLUS + t.0 as i64,
        ));
    }
    out.push(opt(1.2, "The Eternal Wanderer 0 (Samurai)".into(), WANDERER_ZERO));
    if g.perm(src).loyalty.unwrap_or(0) >= 4 {
        let gain_v = ult_value(g, p);
        if gain_v >= 6.0 {
            out.push(opt(gain_v / 2.0, "The Eternal Wanderer -4".into(), WANDERER_ULT));
        }
    }
    Ok(out)
}

/// the Wanderer's ability arguments: the 0, the -4, and the +1 with its target's id added (no target: just the +1)
const WANDERER_ZERO: i64 = -1;
const WANDERER_ULT: i64 = -2;
const WANDERER_PLUS: i64 = 1 << 40;

fn wanderer_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    if !controls(g, p, src) || wanderer_used(g, src) {
        return Ok(false);
    }
    let st = g.turn_stamp();
    g.perm_mut(src).data.set(DataKey::Act, Val::Stamp(st));
    if arg >= WANDERER_PLUS {
        let t = PermId((arg - WANDERER_PLUS) as u32);
        let x = g.perm_mut(src);
        x.loyalty = Some(x.loyalty.unwrap_or(0) + 1);
        crate::glog!(g, "  {} uses The Eternal Wanderer +1", pname(g, p));
        if ability_window(g, p, Some(src), "+1", None, Some(t))? && g.perm(t).on_bf {
            wanderer_exile(g, p, t)?;
        }
    } else if arg == WANDERER_ZERO {
        crate::glog!(g, "  {} uses The Eternal Wanderer 0", pname(g, p));
        if !ability_window(g, p, Some(src), "0", None, None)? {
            return Ok(true);
        }
        let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["samurai"], ..Tokens::new(1, 2) };
        for x in make_tokens(g, p, spec)? {
            g.perm_mut(x).data.set(DataKey::Kws, Val::List(vec![Val::Str("double strike")]));
        }
        crate::glog!(g, "  {} uses The Eternal Wanderer 0: a 2/2 double-strike Samurai", pname(g, p));
    } else {
        let x = g.perm_mut(src);
        x.loyalty = Some(x.loyalty.unwrap_or(0) - 4);
        crate::glog!(g, "  {} uses The Eternal Wanderer -4", pname(g, p));
        let dead = g.perm(src).loyalty.unwrap_or(0) <= 0;
        if dead {
            leave(g, src)?;
            to_zone_card(g, src, Zone::Gy);
        }
        let ok = if dead {
            let cd = g.perm(src).cd.unwrap();
            ability_window_card(g, p, cd, "-4", Some(8.0), None)?
        } else {
            ability_window(g, p, Some(src), "-4", Some(8.0), None)?
        };
        if ok {
            wanderer_ult(g, p)?;
        }
    }
    Ok(true)
}

/// zur.wanderer_target: the best opposing nontoken artifact or creature to exile (pval 2.5 or more)
fn wanderer_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cands: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| {
            let x = g.perm(m);
            !x.phased && !untargetable(g, m) && !x.token && (g.is_creature(m) || is_artifact(g, m))
        })
        .collect();
    max_by(&cands, |m| pval(g, m)).filter(|&b| pval(g, b) >= 2.5)
}

/// zur.wanderer_exile: exile m until its owner's next end step
pub fn wanderer_exile(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let x = g.perm(m);
    let (o, cd) = (x.orig, x.phys.or(x.cd));
    crate::glog!(
        g,
        "  {} uses The Eternal Wanderer +1: exiles {} until {}'s next end step",
        pname(g, p),
        g.perm(m).name,
        pname(g, o)
    );
    apply_removal(g, Some(p), m, "exile", None)?;
    if let Some(cd) = cd
        && !g.perm(m).on_bf
        && !g.perm(m).token
        && g.player(o).exile.contains(&cd)
        && cd != g.player(o).cmd
    {
        g.zur_due.push((o, cd));
    }
    Ok(())
}

/// zur.keep_one: the creature q keeps from the -4 (me: my commander first)
pub fn keep_one(g: &Game, q: PlayerId, me: PlayerId) -> Option<PermId> {
    let cr: Vec<PermId> =
        g.player(q).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
    if q == me {
        max_by(&cr, |m| g.perm(m).is_cmd as i32 as f64 * 5.0 + pval(g, m))
    } else {
        max_by(&cr, |m| pval(g, m))
    }
}

/// zur.ult_value: what -4 gains: opponents' creatures lost minus yours
fn ult_value(g: &Game, p: PlayerId) -> f64 {
    let mut v = 0.0;
    for q in g.players.iter().filter(|q| q.alive).map(|q| q.id) {
        let k = keep_one(g, q, p);
        let lost = g
            .player(q)
            .perms
            .iter()
            .filter(|&&m| g.is_creature(m) && !g.perm(m).phased && Some(m) != k)
            .map(|&m| pval(g, m))
            .psum();
        v += if q != p { lost } else { -1.5 * lost };
    }
    v
}

/// zur.wanderer_ult: each player keeps one creature and sacrifices the rest
pub fn wanderer_ult(g: &mut Game, p: PlayerId) -> Res {
    for q in g.players.iter().map(|q| q.id).collect::<Vec<_>>() {
        if !g.player(q).alive {
            continue;
        }
        let k = keep_one(g, q, p);
        let rest: Vec<PermId> = g
            .player(q)
            .perms
            .iter()
            .copied()
            .filter(|&m| g.is_creature(m) && !g.perm(m).phased && Some(m) != k)
            .collect();
        for m in rest {
            die(g, m, "sac")?;
        }
    }
    Ok(())
}

/// zur.zur_end_step: at the beginning of p's end step, The Eternal Wanderer's exiles owned by p come home
pub fn zur_end_step(g: &mut Game, p: PlayerId) -> Res {
    if g.zur_due.is_empty() {
        return Ok(());
    }
    let mut keep = vec![];
    for (o, cd) in std::mem::take(&mut g.zur_due) {
        if o != p {
            keep.push((o, cd));
            continue;
        }
        if g.player(o).exile.contains(&cd) && g.player(o).alive {
            remove_first(&mut g.player_mut(o).exile, cd);
            enter(g, o, cd, Enter::default())?;
            crate::glog!(g, "    {} returns to the battlefield (The Eternal Wanderer)", g.db.get(cd).name);
        }
    }
    g.zur_due = keep; // (as Python reassigns it: an entry added while these returned is dropped)
    Ok(())
}

// ================================================================== spells and the rest
/// Prayer of Binding: flash; exiles the best opposing nonland permanent until it leaves the battlefield; you gain 2
fn prayer(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "exile a permanent; gain 2 life", Some(5.0))? {
        return Ok(());
    }
    // HUMAN(phase 9): the person picks the permanent (hc.choose over legal.describe_target)
    crate::impls::common::oring_exile(g, src, o, |g, x| !g.perm(x).token || g.is_creature(x), false, false)?;
    gain(g, o, 2)
}

fn prayer_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    crate::impls::common::oring_return(g, src)
}

/// zur.populate: a copy of p's best creature token
pub fn populate(g: &mut Game, p: PlayerId) -> Res {
    let toks: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).token && g.is_creature(m) && !g.perm(m).phased)
        .collect();
    let mut best: Option<(PermId, (i32, bool))> = None;
    for &m in &toks {
        let k = (epow(g, m), g.perm(m).fly);
        if best.is_none_or(|b| k > b.1) {
            best = Some((m, k));
        }
    }
    let Some((t, _)) = best else { return Ok(()) };
    let x = g.perm(t).clone();
    let spec = Tokens {
        tgh: Some(x.tgh),
        fly: x.fly,
        color: Some(x.colors),
        types: x.ttypes.clone(),
        ..Tokens::new(1, x.pow)
    };
    for n in make_tokens(g, p, spec)? {
        let y = g.perm_mut(n);
        y.lifelink = x.lifelink;
        y.dt = x.dt;
        if !x.data.is_empty() {
            y.data = x.data.clone();
        }
    }
    Ok(())
}

/// zur.rootborn: Rootborn Defenses resolves: populate, then indestructible until end of turn
pub fn rootborn(g: &mut Game, p: PlayerId) -> Res {
    populate(g, p)?;
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) {
            add_kw(g, m, "indestructible");
        }
    }
    crate::glog!(g, "  {} casts Rootborn Defenses: creatures gain indestructible", pname(g, p));
    Ok(())
}

// ================================================================== Zur's attack trigger (the Y'shtola deck's 99)
/// zur.zur_fetch: Zur attacks: search for an enchantment card with mana value 3 or less and put it onto the
/// battlefield (t5's Zur attack calls it for the Y'shtola deck)
pub fn zur_fetch(g: &mut Game, src: PermId, p: PlayerId) -> Res {
    let cs: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&c| g.db.get(c).types.has(Types::ENCHANTMENT) && g.db.get(c).cmc <= 3)
        .collect();
    if cs.is_empty() {
        return Ok(());
    }
    // HUMAN(phase 9): the person searches (hc.search)
    let mut best: Option<(f64, CardId)> = None;
    for &c in &cs {
        let v = fetch_value(g, p, c, src);
        if best.is_none_or(|b| v > b.0) {
            best = Some((v, c));
        }
    }
    let (v, c) = best.unwrap();
    if v <= 0.0 {
        return Ok(());
    }
    remove_first(&mut g.player_mut(p).library, c);
    crate::engine::tutors::shuffle_library(g, p);
    put_enchantment(g, p, c, src)?;
    crate::glog!(g, "    Zur fetches {}", g.db.get(c).name);
    Ok(())
}

/// zur.put_enchantment: an enchantment put onto the battlefield (not cast): an Aura goes where it's needed, without
/// targeting
pub fn put_enchantment(g: &mut Game, p: PlayerId, c: CardId, _zur: PermId) -> Res {
    let name = g.db.get(c).name.clone();
    if lock_kind(&name).is_some() {
        // HUMAN(phase 9): human_lock_host (the person picks the permanent)
        let host = lock_host(g, p, &name, false, &[]);
        g.aura_put = true;
        let r = enter(g, p, c, Enter::default());
        g.aura_put = false;
        let a = r?;
        let ok = match host {
            Some(h) => lock_attach(g, a, h)?,
            None => false,
        };
        if !ok {
            leave(g, a)?;
            g.player_mut(p).gy.push(c);
        }
        return Ok(());
    }
    if g.db.get(c).has_subtype("aura") && crate::impls::common::aura_spec(g, c).is_some() {
        g.attach_to = None; // the AI's host pick, or the person chooses
        g.aura_put = true;
        let r = enter(g, p, c, Enter::default());
        g.attach_to = None;
        g.aura_put = false;
        r?;
        return Ok(());
    }
    enter(g, p, c, Enter::default())?;
    Ok(())
}

/// zur.fetch_value: how much an enchantment from Zur is worth now (Y'shtola's values: CI.yshtola_fetch_value).
/// PORT(phase 6): yshtola.fetch_value (the Y'shtola port's); until it's merged, 0: Zur fetches nothing.
fn fetch_value(_g: &Game, _p: PlayerId, _c: CardId, _zur: PermId) -> f64 {
    0.0
}

// ================================================================== the AI: protection and wipe responses (Y'shtola's)
/// zur.zur_protect: removal at a key creature: phase it out with its Auras (Clever Concealment), make it
/// indestructible (Rootborn Defenses), or blink it with Restoration Angel (not a creature with Auras on it, which
/// would lose them)
pub fn zur_protect(
    g: &mut Game,
    o: PlayerId,
    m: PermId,
    kind: &str,
    _actor: Option<PlayerId>,
    _spell: Option<CardId>,
) -> Res<bool> {
    use crate::ai::decks::pay_card;
    if pval(g, m) < 4.0 || !g.is_creature(m) {
        return Ok(false);
    }
    let auras = if g.auras.is_empty() { vec![] } else { crate::impls::common::auras_on(g, m) };
    for c in g.player(o).hand.clone() {
        let d = g.db.get(c);
        let tp = d.tags.str(Tag::Prot);
        let (gen_, pips, convoke) = (d.generic, d.pips.to_string(), d.tag(Tag::Convoke));
        if tp == Some("phase") && can_pay(g, o, gen_, &pips, convoke) {
            // Clever Concealment
            if !pay_card(g, o, c, 0)? {
                return Ok(false);
            }
            g.perm_mut(m).phased = true;
            for &a in &auras {
                g.perm_mut(a).phased = true;
            }
            crate::glog!(g, "    {} casts {}: {} phases out", pname(g, o), g.db.get(c).name, g.perm(m).name);
            return Ok(true);
        }
        if tp == Some("indes") && (kind == "destroy" || kind.starts_with("dmg")) && can_pay(g, o, gen_, &pips, false) {
            // Rootborn Defenses
            if !pay_card(g, o, c, 0)? {
                return Ok(false);
            }
            rootborn(g, o)?;
            return Ok(true);
        }
    }
    if !auras.is_empty() || (!matches!(kind, "destroy" | "exile" | "bounce" | "tuck") && !kind.starts_with("dmg")) {
        return Ok(false);
    }
    for c in g.player(o).hand.clone() {
        let d = g.db.get(c);
        let (gen_, pips) = (d.generic, d.pips.to_string());
        if &*d.name == "Restoration Angel" && can_pay(g, o, gen_, &pips, false) {
            if !pay_card(g, o, c, 0)? {
                return Ok(false);
            }
            remove_first(&mut g.player_mut(o).gy, c);
            g.resto_target = Some(m); // its enters trigger blinks the creature under attack
            let r = enter(g, o, c, Enter::default());
            g.resto_target = None;
            r?;
            let x = g.perm(m);
            let what = x.cd.map_or(x.name, |cd| &*g.db.get(cd).name).to_string();
            crate::glog!(g, "    {} casts {}: blinks {}", pname(g, o), g.db.get(c).name, what);
            return Ok(true);
        }
    }
    Ok(false)
}

/// zur.zur_wipe_response: a wipe: phase out your creatures and the Auras on them (Clever Concealment), or make them
/// indestructible (Rootborn Defenses). Some('all'), Some('indes') or None.
pub fn zur_wipe_response(g: &mut Game, q: PlayerId, kind: &str) -> Res<Option<Sym>> {
    use crate::ai::decks::pay_card;
    let all = matches!(kind, "rift" | "rebuke");
    let loss = g.player(q).perms.iter().filter(|&&m| g.is_creature(m) || all).map(|&m| pval(g, m)).psum();
    if loss < 6.0 {
        return Ok(None);
    }
    for c in g.player(q).hand.clone() {
        let d = g.db.get(c);
        let (gen_, pips, convoke) = (d.generic, d.pips.to_string(), d.tag(Tag::Convoke));
        if d.tags.str(Tag::Prot) == Some("phase") && can_pay(g, q, gen_, &pips, convoke) {
            if !pay_card(g, q, c, 0)? {
                return Ok(None);
            }
            let mine: Vec<PermId> = g.player(q).perms.iter().copied().filter(|&m| g.is_creature(m)).collect();
            for m in g.player(q).perms.clone() {
                if g.is_creature(m) || g.perm(m).attached.is_some_and(|h| mine.contains(&h)) {
                    g.perm_mut(m).phased = true;
                }
            }
            crate::glog!(g, "    {} casts {}: their creatures phase out", pname(g, q), g.db.get(c).name);
            return Ok(Some("all"));
        }
    }
    if matches!(kind, "destroy" | "dmg13" | "austere" | "nib") {
        for c in g.player(q).hand.clone() {
            let d = g.db.get(c);
            let (gen_, pips) = (d.generic, d.pips.to_string());
            if d.tags.str(Tag::Prot) == Some("indes") && can_pay(g, q, gen_, &pips, false) {
                if !pay_card(g, q, c, 0)? {
                    return Ok(None);
                }
                rootborn(g, q)?;
                return Ok(Some("indes"));
            }
        }
    }
    Ok(None)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    for (name, _) in LOCKS {
        r.card(db, name)?.aura = Some(&LOCK_AURA);
    }
    r.card(db, "Azorius Guildmage")?.options = Some(guildmage);
    r.card(db, "The Eternal Wanderer")?.options = Some(wanderer);
    let c = r.card(db, "Prayer of Binding")?;
    c.etb = Some(prayer);
    c.leaves = Some(prayer_leaves);
    *c = c.at_once(Event::Leaves);
    Ok(())
}
