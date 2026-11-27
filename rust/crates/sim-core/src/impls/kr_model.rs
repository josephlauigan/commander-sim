//! Wasteland and Umezawa's Jitte, the two cards of the karn_reforged deck that had no engine record (2026-11-26).
//!
//! **Wasteland** ({T}: Add {C}. {T}, Sacrifice this land: Destroy target nonbasic land.): the mana is the land's `c=C`
//! tag; the destroy is the `land_options` hook (the Ghost Quarter and Strip Mine pattern). It is an ability on the stack
//! (`ability_run_w`: the table can respond, a person's Rings of Brighthearth copy asks for a new target), the land is
//! sacrificed as the cost, and only a nonbasic land can be the target (the card's own `basic` supertype: a Leyline of the
//! Guildpact adds types, not the supertype). The AI uses it on an opponent's key nonbasic land (`lands::KEY_LANDS`, or a
//! land making two or more mana); a person gets the same option from the hook and picks any nonbasic land.
//!
//! **Umezawa's Jitte** (equip {2}; whenever equipped creature deals combat damage, put two charge counters on it;
//! remove a charge counter: +2/+2, or -1/-1 on a target creature, or gain 2 life):
//! - *Equip*: the compiled `equip_cost` static of its override, so the AI's `dsl::equip_options` and a person's Equip
//!   (`human::legal::equip_cost`) both find it.
//! - *Counters*: `DataKey::Charge` on the Jitte itself, so they stay when it moves or its creature dies, and proliferate
//!   adds to them.
//! - *The trigger*: `jitte_pair` (a creature and its blocker, both sides), `jitte_hits` (damage to a player, a planeswalker or
//!   a battle) are called from `engine::combat::resolve_blocks` where the damage is dealt. It triggers once for each damage
//!   step in which the creature dealt combat damage (a double striker: twice), once even if the damage goes to several
//!   creatures or to a creature and the player (trample), never for noncombat damage, and Cloud's doubler adds a time
//!   (`stack::extra_for`, as for the Swords).
//! - *The abilities*: `jitte_options` (the hook: the AI's kills and life gain, a person's three modes), `jitte_combat` (the
//!   AI pumping or shrinking in a fight once blocks are declared, and pushing lethal damage through). Each activation pays
//!   its counter first, then goes through `ability_run_w`.

use crate::cards::CardDb;
use crate::engine::combat::{double_strike, first_strike};
use crate::engine::life::{check_state, gain};
use crate::engine::repl;
use crate::engine::stack::ability_run_w;
use crate::engine::values::{
    colors_of, epow, etgh, indestructible, protected_from, pval, threat, untargetable, untargetable_by_its_controller,
};
use crate::engine::zones::{die, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{PermId, PlayerId};
use crate::state::{DataKey, Game, Val};

pub const WASTELAND: &str = "Wasteland";
pub const JITTE: &str = "Umezawa's Jitte";

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn human(g: &Game, p: PlayerId) -> bool {
    crate::human::is_human(g, p)
}

fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &*g.db.get(c).name)
}

fn ask(g: &Game, p: PlayerId, prompt: &str, labels: &[String]) -> Res<Option<usize>> {
    crate::human::choices::choose(g, p, "choose", prompt, labels, Some("cancel"), None)
}

// ================================================================== Wasteland
/// the argument of a person's Wasteland: no target yet, the person picks it
const ASK: u32 = u32::MAX;

fn pack(l: PermId, t: u32) -> i64 {
    ((t as i64) << 32) | l.0 as i64
}

/// every nonbasic land on the battlefield except `except`, with its controller
fn nonbasic_lands(g: &Game, except: PermId) -> Vec<(PlayerId, PermId)> {
    g.players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.lands.iter().map(move |&x| (q.id, x)))
        .filter(|&(_, x)| x != except && !g.db.get(g.perm(x).card()).basic)
        .collect()
}

fn land_label(g: &Game, p: PlayerId, q: PlayerId, x: PermId) -> String {
    format!(
        "{} ({})",
        g.db.get(g.perm(x).card()).name,
        if q == p { "yours".to_string() } else { g.player(q).name.to_string() }
    )
}

/// the land the AI destroys: the key nonbasic land (or one making two or more mana) of the most threatening opponent
/// that destruction can take
fn wasteland_target(g: &Game, p: PlayerId, wl: PermId) -> Option<(PlayerId, PermId)> {
    let key: Vec<(PlayerId, PermId)> = nonbasic_lands(g, wl)
        .into_iter()
        .filter(|&(q, x)| {
            q != p
                && !indestructible(g, x)
                && !crate::dsl::lands_have_kw(g, q, "indestructible")
                && (crate::impls::lands::KEY_LANDS.contains(&&*g.db.get(g.perm(x).card()).name)
                    || g.db.get(g.perm(x).card()).tags.int(crate::tag::Tag::Amt).unwrap_or(1) >= 2)
        })
        .collect();
    max_by(&key, |(q, _)| threat(g, p, q))
}

/// {T}, sacrifice: destroy target nonbasic land. The AI: an opponent's key land, with three other lands to play on
/// without; a person: any nonbasic land, chosen as the ability is used.
fn wasteland_options(g: &mut Game, l: PermId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(l).tapped || !g.player(p).lands.contains(&l) {
        return Ok(vec![]);
    }
    if human(g, p) {
        if nonbasic_lands(g, l).is_empty() {
            return Ok(vec![]);
        }
        return Ok(vec![Opt {
            utility: 2.5,
            label: "Wasteland: {T}, sacrifice: destroy target nonbasic land".into(),
            act: Some(Action::Plan { f: wasteland_go, arg: pack(l, ASK) }),
        }]);
    }
    if g.player(p).lands.len() < 4 {
        return Ok(vec![]);
    }
    let Some((_, x)) = wasteland_target(g, p, l) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 3.0,
        label: format!("Wasteland -> {}", g.db.get(g.perm(x).card()).name),
        act: Some(Action::Plan { f: wasteland_go, arg: pack(l, x.0) }),
    }])
}

fn wasteland_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, t) = (PermId(arg as u32), (arg >> 32) as u32);
    if !g.player(p).lands.contains(&l) || g.perm(l).tapped || g.perm(l).cd.is_none() {
        return Ok(false);
    }
    let x = if t == ASK {
        let cands = nonbasic_lands(g, l);
        let labels: Vec<String> = cands.iter().map(|&(q, x)| land_label(g, p, q, x)).collect();
        let Some(k) = ask(g, p, "Wasteland: destroy which nonbasic land?", &labels)? else { return Ok(false) };
        cands[k].1
    } else {
        PermId(t)
    };
    let q = owner(g, x);
    if !g.player(q).lands.contains(&x) || g.db.get(g.perm(x).card()).basic {
        return Ok(false); // gone, or not a nonbasic land
    }
    let cd = g.perm(l).real_card();
    // the cost: {T} (it was untapped) and sacrifice it
    crate::impls::lands::sac_land(g, p, l)?;
    let what = format!("destroy {}", g.db.get(g.perm(x).card()).name);
    crate::glog!(g, "  {} sacrifices Wasteland: {what} ({})", g.player(p).name, g.player(q).name);
    ability_run_w(g, p, None, Some(cd), &what, Some(3.0), None, 0.0, |g, is_copy| {
        let (mut q, mut x) = (q, x);
        if is_copy && human(g, p) {
            // "you may choose a new target for the copy" (a person's Rings of Brighthearth)
            let cands = nonbasic_lands(g, l);
            let labels: Vec<String> = cands.iter().map(|&(o, y)| land_label(g, p, o, y)).collect();
            let prompt = "The copy of Wasteland's ability: destroy which nonbasic land?";
            if let Some(k) =
                crate::human::choices::choose(g, p, "target", prompt, &labels, Some("keep the target"), None)?
            {
                (q, x) = cands[k];
            }
        }
        if !g.player(q).lands.contains(&x) || g.db.get(g.perm(x).card()).basic {
            return Ok(()); // the target is gone or is not a nonbasic land: the ability does nothing
        }
        crate::impls::fixes::destroy_land(g, q, x)
    })?;
    Ok(true)
}

// ================================================================== Umezawa's Jitte
const MODE_PUMP: i64 = 0;
const MODE_SHRINK: i64 = 1;
const MODE_LIFE: i64 = 2;

/// the charge counters on a Jitte
pub fn counters(g: &Game, jitte: PermId) -> i64 {
    g.perm(jitte).data.int(DataKey::Charge).max(0)
}

fn set_counters(g: &mut Game, jitte: PermId, n: i64) {
    g.perm_mut(jitte).data.set(DataKey::Charge, Val::Int(n));
}

/// the Jittes (any player's) attached to creature `c`
fn jittes_on(g: &Game, c: PermId) -> Vec<PermId> {
    g.players
        .iter()
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&e| {
            let x = g.perm(e);
            x.on_bf && !x.phased && !x.neutered && x.attached == Some(c) && name_of(g, e) == JITTE
        })
        .collect()
}

/// "Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte": `steps` damage steps
/// of creature `c` (each one trigger, and one more for each doubler: Cloud)
pub fn jitte_hits(g: &mut Game, c: PermId, steps: u8) -> Res {
    for e in jittes_on(g, c) {
        for _ in 0..steps {
            let times = 1 + crate::engine::stack::extra_for(g, e) as i64;
            let n = counters(g, e) + 2 * times;
            set_counters(g, e, n);
            crate::glog!(g, "    Umezawa's Jitte gets {} charge counters ({n})", 2 * times);
        }
    }
    Ok(())
}

/// In how many of the two combat damage steps (first strike, regular) each creature of a blocked pair deals combat
/// damage to the other: (a, b). `*_hits`: its damage is more than 0 and not prevented; `*_kills`: it destroys the other
/// (lethal damage or deathtouch, not indestructible); `*_fs`, `*_ds`: first strike (a double striker has it too) and
/// double strike. A creature that dies in the first step deals nothing in the second; a creature whose foe died in the
/// first step has nothing left to damage.
pub fn pair_steps(
    (a_hits, a_kills, a_fs, a_ds): (bool, bool, bool, bool),
    (b_hits, b_kills, b_fs, b_ds): (bool, bool, bool, bool),
) -> (u8, u8) {
    let a_alive = !(b_fs && b_hits && b_kills);
    let b_alive = !(a_fs && a_hits && a_kills);
    let first = |hits: bool, fs: bool| u8::from(hits && fs);
    let second = |hits: bool, fs: bool, ds: bool| u8::from(hits && (!fs || ds) && a_alive && b_alive);
    (first(a_hits, a_fs) + second(a_hits, a_fs, a_ds), first(b_hits, b_fs) + second(b_hits, b_fs, b_ds))
}

/// Attacker `a` and its blocker `b` deal combat damage to each other: Umezawa's Jitte on either
/// triggers once for each damage step in which its creature dealt damage. Called before either creature dies, so a
/// creature that dies in the damage still counts. Returns how many damage steps `a` dealt damage to `b` in (the caller
/// counts `a`'s trample damage to the player only when this is 0: one damage step is one trigger).
pub fn jitte_pair(g: &mut Game, a: PermId, b: PermId) -> Res<u8> {
    if jittes_on(g, a).is_empty() && jittes_on(g, b).is_empty() {
        return Ok(0);
    }
    let t0 = crate::impls::t0_giada::combat_proof;
    let fogged = g.fog == Some(g.turn_stamp()) && !g.unpreventable();
    let (pa, pb) = (epow(g, a), epow(g, b));
    let a_hits = pa > 0
        && !fogged
        && !repl::source_prevented(g, a)
        && !repl::dovin(g, a)
        && !repl::no_damage(g, b)
        && !t0(g, b)
        && !protected_from(g, b, colors_of(g, a))
        && !crate::cardcode::prot_vs(g, b, a);
    let b_hits = pb > 0
        && !fogged
        && !repl::source_prevented(g, b)
        && !repl::no_damage(g, a)
        && !t0(g, a)
        && !protected_from(g, a, colors_of(g, b))
        && !crate::cardcode::prot_vs(g, a, b);
    let a_kills = a_hits && (pa >= etgh(g, b) || g.perm(a).dt) && !indestructible(g, b);
    let b_kills = b_hits && (pb >= etgh(g, a) || g.perm(b).dt) && !indestructible(g, a);
    let (na, nb) = pair_steps(
        (a_hits, a_kills, first_strike(g, a), double_strike(g, a)),
        (b_hits, b_kills, first_strike(g, b), double_strike(g, b)),
    );
    jitte_hits(g, a, na)?;
    jitte_hits(g, b, nb)?;
    Ok(na)
}

/// is creature `t` a legal target of Jitte `src`'s -1/-1 (a creature on the battlefield that its controller's
/// hexproof, shroud or protection does not stop)
fn targetable_by(g: &Game, src: PermId, t: PermId) -> bool {
    if !live(g, t) || !g.is_creature(t) {
        return false;
    }
    if owner(g, t) == owner(g, src) {
        return !untargetable_by_its_controller(g, t, colors_of(g, src));
    }
    !(untargetable(g, t) || protected_from(g, t, colors_of(g, src)))
}

/// One activation of Jitte `src` for `p`: the counter is the cost, then the ability goes on the stack. `target` is
/// the creature of the -1/-1 mode. False if it can't be activated.
fn use_jitte(g: &mut Game, p: PlayerId, src: PermId, mode: i64, target: Option<PermId>) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).neutered || counters(g, src) <= 0 || name_of(g, src) != JITTE
    {
        return Ok(false);
    }
    if mode == MODE_SHRINK && !target.is_some_and(|t| targetable_by(g, src, t)) {
        return Ok(false);
    }
    let left = counters(g, src) - 1;
    set_counters(g, src, left);
    let (what, imp) = match mode {
        MODE_PUMP => ("equipped creature gets +2/+2 until end of turn", 3.0),
        MODE_SHRINK => ("target creature gets -1/-1 until end of turn", 3.0),
        _ => ("you gain 2 life", 2.0),
    };
    crate::glog!(g, "  {} removes a charge counter from Umezawa's Jitte ({left} left): {what}", g.player(p).name);
    let tp = if mode == MODE_SHRINK { target } else { None };
    ability_run_w(g, p, Some(src), None, what, Some(imp), tp, 0.0, |g, is_copy| {
        match mode {
            MODE_PUMP => {
                if let Some(c) = g.perm(src).attached.filter(|&c| live(g, c) && g.is_creature(c)) {
                    let x = g.perm_mut(c);
                    x.eot_pt = (x.eot_pt.0 + 2, x.eot_pt.1 + 2);
                    crate::glog!(g, "    {} gets +2/+2 until end of turn", g.perm(c).name);
                }
            }
            MODE_SHRINK => {
                let mut t = target;
                if is_copy && human(g, p) {
                    // "you may choose a new target for the copy" (a person's Rings of Brighthearth)
                    let cands: Vec<PermId> = crate::human::cards::creatures(g, p, false)
                        .into_iter()
                        .filter(|&m| targetable_by(g, src, m))
                        .collect();
                    let prompt = "The copy of Umezawa's Jitte's ability: -1/-1 on which creature?";
                    if let Some(m) =
                        crate::human::cards::pick_perm(g, p, "target", prompt, &cands, Some("keep the target"))?
                    {
                        t = Some(m);
                    }
                }
                if let Some(t) = t.filter(|&t| targetable_by(g, src, t)) {
                    let foreign = owner(g, t) != p;
                    if !foreign || crate::engine::ward::pay_ward(g, p, t)? {
                        let x = g.perm_mut(t);
                        x.eot_pt = (x.eot_pt.0 - 1, x.eot_pt.1 - 1);
                        crate::glog!(g, "    {} gets -1/-1 until end of turn", g.perm(t).name);
                        if etgh(g, t) <= 0 {
                            die(g, t, "sba")?;
                        }
                    }
                }
            }
            _ => gain(g, p, 2)?,
        }
        check_state(g)
    })?;
    Ok(true)
}

/// the hook's argument: a person (-1 - mode: asks for the target), or the AI's mode, number of activations and target
fn arg_of(mode: i64, uses: i64, target: Option<PermId>) -> i64 {
    mode | (uses << 4) | (target.map_or(0, |t| t.0 as i64 + 1) << 12)
}

fn jitte_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    if arg < 0 {
        let mode = -1 - arg;
        let target = if mode == MODE_SHRINK {
            let cands: Vec<PermId> =
                crate::human::cards::creatures(g, p, false).into_iter().filter(|&m| targetable_by(g, src, m)).collect();
            if cands.is_empty() {
                return Ok(false);
            }
            let prompt = "Umezawa's Jitte: -1/-1 on which creature?";
            let Some(t) = crate::human::cards::pick_perm(g, p, "target", prompt, &cands, Some("cancel"))? else {
                return Ok(false);
            };
            Some(t)
        } else {
            None
        };
        return use_jitte(g, p, src, mode, target);
    }
    let (mode, uses, t) = (arg & 15, (arg >> 4) & 255, arg >> 12);
    let target = (t > 0).then(|| PermId((t - 1) as u32));
    let mut did = false;
    for _ in 0..uses.max(1) {
        if mode == MODE_SHRINK && !target.is_some_and(|t| live(g, t) && etgh(g, t) > 0) {
            break; // it is gone (or dead already): keep the rest of the counters
        }
        if !use_jitte(g, p, src, mode, target)? {
            break;
        }
        did = true;
    }
    Ok(did)
}

fn jitte_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let k = counters(g, src);
    if k <= 0 || owner(g, src) != p || !live(g, src) || g.perm(src).neutered {
        return Ok(vec![]);
    }
    let opt = |u: f64, label: &str, arg: i64| Opt {
        utility: u,
        label: label.into(),
        act: Some(Action::Ability { src, f: jitte_go, arg }),
    };
    if human(g, p) {
        let mut v = vec![];
        if g.perm(src).attached.is_some_and(|c| live(g, c) && g.is_creature(c)) {
            v.push(opt(1.0, "Umezawa's Jitte: remove a charge counter: equipped creature gets +2/+2", -1 - MODE_PUMP));
        }
        if crate::human::cards::creatures(g, p, false).into_iter().any(|m| targetable_by(g, src, m)) {
            v.push(opt(1.0, "Umezawa's Jitte: remove a charge counter: target creature gets -1/-1", -1 - MODE_SHRINK));
        }
        v.push(opt(1.0, "Umezawa's Jitte: remove a charge counter: you gain 2 life", -1 - MODE_LIFE));
        return Ok(v);
    }
    let mut out = vec![];
    // -1/-1: kill an opposing creature the counters can take (each point of toughness is one counter); a 1-toughness
    // creature is worth one, a bigger one only if it is worth a lot
    let mut best: Option<(f64, PermId, i64)> = None;
    for m in g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).collect::<Vec<_>>() {
        if !targetable_by(g, src, m) || etgh(g, m) < 1 || etgh(g, m) as i64 > k {
            continue;
        }
        let (t, v) = (etgh(g, m) as i64, pval(g, m));
        if v < if t == 1 { 2.5 } else { 4.0 } || !crate::engine::ward::ward_affordable(g, p, m, 0, "") {
            continue;
        }
        let u = 1.0 + v / 2.5 - 0.5 * t as f64;
        if u > 0.5 && best.is_none_or(|b| u > b.0) {
            best = Some((u, m, t));
        }
    }
    if let Some((u, m, t)) = best {
        out.push(opt(u, "Umezawa's Jitte -1/-1", arg_of(MODE_SHRINK, t, Some(m))));
    }
    // 2 life when it is low
    if g.player(p).life <= 10 {
        out.push(opt(
            1.5 + 0.2 * (11 - g.player(p).life) as f64,
            "Umezawa's Jitte: gain 2",
            arg_of(MODE_LIFE, 1, None),
        ));
    }
    Ok(out)
}

// ------------------------------------------------------------------ the AI in combat
/// what the fight of `mine` and `theirs` is worth to mine's side after `x` pumps (+2/+2 on mine) and `y` shrinks
/// (-1/-1 on theirs): the creature it kills less the creature it loses
fn fight_value(g: &Game, mine: PermId, theirs: PermId, x: i32, y: i32) -> f64 {
    let (mp, mt) = (epow(g, mine) + 2 * x, etgh(g, mine) + 2 * x);
    let (tp, tt) = (epow(g, theirs) - y, etgh(g, theirs) - y);
    if tt <= 0 {
        return pval(g, theirs); // it dies to the -1/-1s before damage
    }
    let (mfs, tfs) = (first_strike(g, mine), first_strike(g, theirs));
    let mut i_kill = (mp >= tt || (g.perm(mine).dt && mp > 0)) && !indestructible(g, theirs);
    let mut they_kill = (tp >= mt || (g.perm(theirs).dt && tp > 0)) && !indestructible(g, mine);
    if mfs && !tfs && i_kill {
        they_kill = false;
    } else if tfs && !mfs && they_kill {
        i_kill = false;
    }
    (if i_kill { pval(g, theirs) } else { 0.0 }) - if they_kill { pval(g, mine) } else { 0.0 }
}

/// Once blocks are declared, p's and the defender d's AI uses the counters of a Jitte on a creature in a fight to win it
/// (pump its creature, shrink the other), and p's pushes lethal damage through with the counters of a Jitte on an
/// unblocked attacker. A counter is kept for later unless it changes the fight by more than 0.7.
pub fn jitte_combat(g: &mut Game, p: PlayerId, atk: &[PermId], d: PlayerId, assign: &[(PermId, PermId)]) -> Res {
    for &(a, b) in assign {
        for (mine, theirs) in [(a, b), (b, a)] {
            for j in jittes_on(g, mine) {
                let jo = owner(g, j);
                if human(g, jo) || !live(g, mine) || !live(g, theirs) {
                    continue;
                }
                let k = counters(g, j) as i32;
                let shrink_ok =
                    targetable_by(g, j, theirs) && crate::engine::ward::ward_affordable(g, jo, theirs, 0, "");
                let base = fight_value(g, mine, theirs, 0, 0);
                let mut best = (base, 0, 0);
                for x in 0..=k {
                    for y in 0..=(if shrink_ok { k - x } else { 0 }) {
                        let v = fight_value(g, mine, theirs, x, y) - 0.7 * (x + y) as f64;
                        if v > best.0 + 0.01 {
                            best = (v, x, y);
                        }
                    }
                }
                let (_, x, y) = best;
                for _ in 0..y {
                    if live(g, theirs) && etgh(g, theirs) > 0 {
                        use_jitte(g, jo, j, MODE_SHRINK, Some(theirs))?;
                    }
                }
                for _ in 0..x {
                    if live(g, mine) && g.perm(j).attached == Some(mine) {
                        use_jitte(g, jo, j, MODE_PUMP, None)?;
                    }
                }
            }
        }
    }
    // lethal: the unblocked attackers plus the pumps of a Jitte on one of them reach the defender's life
    if human(g, p) || !g.player(d).alive {
        return Ok(());
    }
    let blocked: Vec<PermId> = assign.iter().map(|x| x.0).collect();
    let free: Vec<PermId> = atk.iter().copied().filter(|&a| live(g, a) && !blocked.contains(&a)).collect();
    let dmg = |g: &Game, a: PermId| epow(g, a).max(0) * if double_strike(g, a) { 2 } else { 1 };
    let total: i32 = free.iter().map(|&a| dmg(g, a)).sum();
    let life = g.player(d).life;
    if total >= life {
        return Ok(());
    }
    for &a in &free {
        for j in jittes_on(g, a) {
            let per = 2 * if double_strike(g, a) { 2 } else { 1 };
            let need = (life - total + per - 1) / per;
            if owner(g, j) == p && counters(g, j) >= need as i64 {
                for _ in 0..need {
                    use_jitte(g, p, j, MODE_PUMP, None)?;
                }
                return Ok(());
            }
        }
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, WASTELAND)?.land_options = Some(wasteland_options);
    r.card(db, JITTE)?.options = Some(jitte_options);
    Ok(())
}
