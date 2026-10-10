//! Combat (Python's `ais.py` "combat" section): who can block whom, splitting an attack between players, attack
//! triggers, blocks and combat damage, attack taxes and caps, planeswalker attacks, and the combat phase itself.
//!
//! The choices the heuristic AI makes here (which player to attack, which creatures stay home, chump blocks) are
//! `ai.rs`'s; the look-ahead's attack plan comes in M4.

use crate::ai;
use crate::cardcode;
use crate::cards::Types;
use crate::engine::hooks::{self, fire_trigger};
use crate::engine::life::{check_state, gain, lose_life, prevents_damage};
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, step_priority, trigger_window};
use crate::engine::values::*;
use crate::engine::zones::*;
use crate::flow::Res;
use crate::hooks::{Call, Event};
use crate::ids::{PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::tag::Tag;

/// does permanent m have keyword k (printed, granted until end of turn, or from a static ability)
pub fn kw(g: &Game, m: PermId, k: &str) -> bool {
    crate::dsl::has_kw(g, m, k)
}

pub fn double_strike(g: &Game, m: PermId) -> bool {
    kw(g, m, "double strike")
}

pub fn first_strike(g: &Game, m: PermId) -> bool {
    kw(g, m, "first strike") || kw(g, m, "double strike")
}

/// Fiery Emancipation: damage from pl's sources is tripled (once per copy)
pub fn fiery(g: &Game, pl: PlayerId) -> i32 {
    let n = g
        .player(pl)
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased && card_name(g, m) == Some("Fiery Emancipation"))
        .count();
    3i32.pow(n as u32)
}

fn untapped_creatures(g: &Game, q: PlayerId) -> impl Iterator<Item = PermId> + '_ {
    g.player(q).perms.iter().copied().filter(|&b| g.is_creature(b) && !g.perm(b).tapped && !g.perm(b).phased)
}

/// q's untapped creatures that could block attacker a, kill it and survive
pub fn good_blockers(g: &Game, q: PlayerId, a: PermId, p: PlayerId) -> Vec<PermId> {
    untapped_creatures(g, q)
        .filter(|&b| {
            can_block(g, b, a)
                && (epow(g, b) * fiery(g, q) >= etgh(g, a) || g.perm(b).dt)
                && !(epow(g, a) * fiery(g, p) >= etgh(g, b) || g.perm(a).dt)
        })
        .collect()
}

pub fn blockable(g: &Game, q: PlayerId, a: PermId) -> bool {
    untapped_creatures(g, q).any(|b| can_block(g, b, a))
}

/// Veyran's small creature tokens: the next spell replaces them, so they block freely
pub fn spare_tokens(g: &Game, d: PlayerId, cands: &[PermId]) -> Vec<PermId> {
    if g.player(d).key != "veyran" {
        return vec![];
    }
    cands.iter().copied().filter(|&x| g.perm(x).token && !g.perm(x).army && epow(g, x) <= 2).collect()
}

pub fn can_block(g: &Game, b: PermId, a: PermId) -> bool {
    let (xb, xa) = (g.perm(b), g.perm(a));
    let cb = xb.cd.map(|c| g.db.get(c));
    let ca = xa.cd.map(|c| g.db.get(c));
    if cb.is_some_and(|d| d.tag(Tag::Noblock)) {
        return false;
    }
    if !g.auras.is_empty() && cardcode::locked(g, b, "pacify") {
        return false; // Arrest, Luminous Bonds
    }
    if ca.is_some_and(|d| d.has_kw("unblockable_shadow")) || cb.is_some_and(|d| d.has_kw("unblockable_shadow")) {
        return false; // shadow
    }
    if g.dsl_on {
        if kw(g, b, "cant_block") || kw(g, a, "unblockable") {
            return false;
        }
        if kw(g, a, "flying")
            && !(xb.fly || kw(g, b, "flying") || kw(g, b, "reach") || cb.is_some_and(|d| d.tag(Tag::Reach)))
        {
            return false;
        }
    }
    if cardcode::evasion_blocked(g, b, a) {
        return false;
    }
    if cardcode::granted_kw(g, a, "forestwalk")
        && g.player(xb.owner).lands.iter().any(|&l| {
            let d = g.db.get(g.land(l).cd);
            &*d.name == "Forest" || d.has_subtype("forest")
        })
    {
        return false;
    }
    if ca.is_some_and(|d| d.tag(Tag::Swampwalk))
        && g.player(xb.owner).lands.iter().any(|&l| {
            matches!(&*g.db.get(g.land(l).cd).name, "Swamp" | "Watery Grave" | "Blood Crypt" | "Overgrown Tomb")
        })
    {
        return false; // Sheoldred, Whispering One: swampwalk
    }
    let reach = cb.is_some_and(|d| d.tag(Tag::Reach));
    if xa.fly && !xb.fly && !reach && !(g.player(xb.owner).elspeth_emblem && g.is_creature(b)) {
        return false;
    }
    if xa.eot_kw.contains(&"flying") && !(xb.fly || reach || xb.eot_kw.contains(&"flying")) {
        return false; // Iron Man
    }
    if cardcode::ring_unblockable(g, b, a) {
        return false; // the Ring, level 1
    }
    if protected_from(g, a, colors_of(g, b)) {
        return false; // protection: can't be blocked by that colour
    }
    true
}

/// "granted or printed haste, and Lightning Greaves / Swiftfoot Boots"
pub fn has_haste(g: &Game, m: PermId) -> bool {
    if kw(g, m, "haste") {
        return true;
    }
    g.player(g.perm(m).owner).perms.iter().any(|&e| {
        g.perm(e).attached == Some(m) && g.perm(e).cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Prot) == Some("boots"))
    })
}

/// (generic mana per attacker, max attackers or None) for p attacking d
pub fn attack_restrictions(g: &Game, p: PlayerId, d: PlayerId) -> (i32, Option<i32>) {
    let tax: i32 = hooks::hooked(g, Event::AttackTax)
        .iter()
        .map(|(src, imp)| (imp.attack_tax.unwrap())(g, *src, p, d).unwrap_or(0))
        .sum();
    let cap = hooks::hooked(g, Event::AttackCap)
        .iter()
        .filter_map(|(src, imp)| (imp.attack_cap.unwrap())(g, *src, p, d))
        .min();
    (tax, cap)
}

/// Ghostly Prison-style taxes and Crawlspace-style caps: attack with the best creatures that are allowed and worth
/// their tax
pub fn attack_limits(g: &mut Game, p: PlayerId, d: PlayerId, atk: Vec<PermId>) -> Res<Vec<PermId>> {
    let (tax, cap) = attack_restrictions(g, p, d);
    let mut atk = atk;
    atk.sort_by_key(|&m| -epow(g, m)); // stable
    let moc: Vec<PermId> = atk.iter().copied().filter(|&m| card_name(g, m) == Some("Master of Cruelties")).collect();
    if let Some(&mc) = moc.first()
        && atk.len() > 1
    {
        // Master of Cruelties attacks alone
        let others: i32 = atk.iter().filter(|&&m| m != mc).map(|&m| epow(g, m)).sum();
        let life = g.player(d).life;
        atk = if life > 1 && life > others { vec![mc] } else { atk.into_iter().filter(|&m| m != mc).collect() };
    }
    if let Some(c) = cap {
        atk.truncate(c.max(0) as usize);
    }
    if tax > 0 {
        let worth: Vec<PermId> =
            atk.iter().copied().filter(|&m| epow(g, m) >= tax || (g.perm(m).is_cmd && epow(g, m) >= 3)).collect();
        let mut k = 0;
        while k < worth.len() && can_pay(g, p, (tax * (k as i32 + 1)) as u32, "", false) {
            k += 1;
        }
        atk = worth[..k].to_vec();
        if !atk.is_empty() {
            let total = tax * atk.len() as i32;
            pay(g, p, total as u32, "", false)?;
            crate::glog!(g, "  {} pays {} to attack {} with {}", g.player(p).name, total, g.player(d).name, atk.len());
            g.player_mut(p).stat("attack_tax_paid", total as i64);
        }
    }
    Ok(atk)
}

/// The AI's attackers may go at different players, as in a real game: [(defending player, attackers)]. Everything
/// starts at d; an attacker moves when that clearly helps (a player the attack can kill outright, away from a bad
/// block, a held-back attacker somewhere safe). Python's `split_attack`.
pub fn split_attack(
    g: &Game,
    p: PlayerId,
    d: PlayerId,
    atk: &[PermId],
    held: &[PermId],
) -> Vec<(PlayerId, Vec<PermId>)> {
    let others: Vec<PlayerId> = g.opps(p).filter(|&q| q != d && g.player(q).alive && !shielded(g, q)).collect();
    if others.is_empty() || (atk.is_empty() && held.is_empty()) {
        return vec![(d, atk.to_vec())];
    }
    let hooked = !g.hooks.is_empty();
    let tax = |q: PlayerId| if hooked { attack_restrictions(g, p, q).0 } else { 0 };
    let hit = |a: PermId| epow(g, a) * if double_strike(g, a) { 2 } else { 1 } * fiery(g, p);
    // `to`: where each attacker goes, in the order Python's dict keeps (attackers first, then held ones as moved)
    let mut to: Vec<(PermId, PlayerId)> = atk.iter().map(|&a| (a, d)).collect();
    let dest = |to: &[(PermId, PlayerId)], a: PermId| to.iter().find(|x| x.0 == a).map(|x| x.1);
    let d_kill: Vec<PermId> = atk.iter().copied().filter(|&a| !blockable(g, d, a)).collect();
    let d_dies = d_kill.iter().map(|&a| hit(a)).sum::<i32>() >= g.player(d).life;
    let mut by_life = others.clone();
    by_life.sort_by_key(|&q| g.player(q).life);
    for q in by_life {
        // finish a player off
        if tax(q) > tax(d) {
            continue;
        }
        let mut free: Vec<PermId> = atk
            .iter()
            .chain(held)
            .copied()
            .filter(|&a| {
                dest(&to, a).unwrap_or(d) == d && !blockable(g, q, a) && !(d_dies && d_kill.contains(&a)) && hit(a) > 0
            })
            .collect();
        free.sort_by_key(|&a| std::cmp::Reverse(hit(a)));
        if free.iter().map(|&a| hit(a)).sum::<i32>() < g.player(q).life {
            continue;
        }
        let mut need = 0;
        for a in free {
            if need >= g.player(q).life {
                break;
            }
            match to.iter_mut().find(|x| x.0 == a) {
                Some(x) => x.1 = q,
                None => to.push((a, q)),
            }
            need += hit(a);
        }
    }
    let mut risky: Vec<PermId> =
        atk.iter().copied().filter(|&a| dest(&to, a) == Some(d) && !good_blockers(g, d, a, p).is_empty()).collect();
    risky.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
    let mut eaters: Vec<PermId> = risky.iter().flat_map(|&a| good_blockers(g, d, a, p)).collect();
    eaters.sort();
    eaters.dedup();
    let safest = |a: PermId, qs: &[PlayerId]| {
        // not blockable there first, then the biggest threat
        qs.iter().copied().max_by(|&x, &y| {
            (!blockable(g, x, a), threat(g, p, x)).partial_cmp(&(!blockable(g, y, a), threat(g, p, y))).unwrap()
        })
    };
    for &a in risky.iter().take(eaters.len()) {
        // away from a bad block (each blocker stops only one attacker)
        let safe: Vec<PlayerId> =
            others.iter().copied().filter(|&q| tax(q) <= tax(d) && good_blockers(g, q, a, p).is_empty()).collect();
        if let Some(q) = safest(a, &safe) {
            to.iter_mut().find(|x| x.0 == a).unwrap().1 = q;
        }
    }
    for &a in held {
        // held back from d: somewhere it's safe
        if dest(&to, a).is_some() {
            continue;
        }
        let safe: Vec<PlayerId> =
            others.iter().copied().filter(|&q| tax(q) == 0 && good_blockers(g, q, a, p).is_empty()).collect();
        if let Some(q) = safest(a, &safe) {
            to.push((a, q));
        }
    }
    let mut groups = vec![];
    for q in std::iter::once(d).chain(others.iter().copied()) {
        let xs: Vec<PermId> = to.iter().filter(|x| x.1 == q).map(|x| x.0).collect();
        if !xs.is_empty() {
            groups.push((q, xs));
        }
    }
    if groups.is_empty() { vec![(d, vec![])] } else { groups }
}

// ------------------------------------------------------------------ attack triggers
const ATTACK_TRIGGER_TAGS: [Tag; 7] =
    [Tag::Witchking, Tag::Archon, Tag::Titan, Tag::Tokatk, Tag::Suntitan, Tag::Necromancer, Tag::Kylox];

fn attack_imp(t: &crate::cards::Tags) -> f64 {
    for (k, v) in
        [(Tag::Archon, 7.0), (Tag::Kylox, 6.0), (Tag::Witchking, 5.0), (Tag::Necromancer, 4.0), (Tag::Suntitan, 4.0)]
    {
        if t.has(k) {
            return v;
        }
    }
    3.0
}

/// p attacked d with atk: attack triggers (once per copy: Isshin). Returns the new attacking creatures.
pub fn attack_triggers(g: &mut Game, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let copies = 1 + hooks::total_trigger_copies(g, p, "attack", None); // Isshin
    let mut new = vec![];
    for _ in 0..copies {
        new.extend(attack_triggers_once(g, p, atk, d)?);
        if g.over {
            break;
        }
    }
    check_state(g)?;
    Ok(new.into_iter().filter(|&x| g.perm(x).tapped && g.perm(x).on_bf && g.perm(x).owner == p).collect())
}

fn attack_triggers_once(g: &mut Game, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let mut new = vec![];
    for &m in atk {
        let Some(c) = g.perm(m).cd else { continue };
        let t = g.db.get(c).tags.clone();
        let name = g.db.get(c).name.to_string();
        if ATTACK_TRIGGER_TAGS.iter().any(|&k| t.has(k))
            && !trigger_window(g, p, Some(m), "attacks", Some(attack_imp(&t)))?
        {
            continue;
        }
        if t.has(Tag::Witchking) {
            edict(g, d, true)?;
        }
        if t.has(Tag::Archon) && g.player(d).alive {
            archon_attack(g, p, d)?;
        }
        if t.has(Tag::Titan) {
            make_tokens(g, p, Tokens::new(2, 2))?;
        }
        if let Some(n) = t.int(Tag::Tokatk) {
            let att = name.starts_with("Adeline");
            let n = if att { g.opps(p).count() as u32 } else { n.max(0) as u32 };
            new.extend(make_tokens(g, p, Tokens { attacking: att, sick: !att, lifelink: !att, ..Tokens::new(n, 1) })?);
        }
        if t.has(Tag::Suntitan) {
            sun_titan(g, p)?;
        }
        if t.has(Tag::Necromancer) {
            new.extend(cardcode::necromancer_attack(g, p, m)?); // an attacking token copy of a graveyard creature
        }
        if t.has(Tag::Kylox) {
            // sacrifice tokens, cast the instants and sorceries among the top X for free
            let toks: Vec<PermId> = g
                .player(p)
                .perms
                .iter()
                .copied()
                .filter(|&x| g.perm(x).token && g.is_creature(x) && !atk.contains(&x))
                .collect();
            let x: i32 = toks.iter().map(|&x| epow(g, x)).sum();
            for &tk in &toks {
                die(g, tk, "sac")?;
            }
            let n = (x.max(0) as usize).min(g.player(p).library.len());
            let top: Vec<_> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
            for c in top {
                let d2 = g.db.get(c);
                if d2.instant || d2.sorcery {
                    let draws = d2.tags.int(Tag::Draw);
                    crate::engine::cast::cast_copy_counted(g, p)?; // PORT(M5): a named copy goes on the stack
                    if let Some(k) = draws {
                        draw(g, p, k.max(0) as u32, false)?;
                    }
                }
                g.player_mut(p).exile.push(c);
            }
        }
        if equipped(g, m, Tag::Animist) {
            land_ramp(g, p, 1, true)?; // Sword of the Animist
        }
    }
    let rab = find(g, p, Tag::Rabble);
    if !rab.is_empty() && trigger_window(g, p, Some(rab[0]), "a Citizen per attacker", None)? {
        make_tokens(g, p, Tokens::new((rab.len() * atk.len()) as u32, 1))?; // Rabble Rousing: one Citizen per attacker
    }
    if g.dsl_on {
        g.new_attackers.clear();
        let fired =
            crate::dsl::Fired { attackers: atk.to_vec(), defender: Some(d), player: Some(p), ..Default::default() };
        crate::dsl::fire(g, "attack", fired)?;
        new.append(&mut g.new_attackers); // attacking tokens its triggers made
    }
    new.extend(cardcode::keyword_attack(g, p, atk, d)?);
    if !g.hooks.is_empty() {
        g.new_attackers.clear();
        fire_trigger(g, Event::Attack, Call::Attack { p, atk: atk.to_vec(), d })?;
        new.append(&mut g.new_attackers);
    }
    Ok(new)
}

/// Archon of Cruelty's attack trigger
pub fn archon_attack(g: &mut Game, p: PlayerId, d: PlayerId) -> Res {
    // HUMAN(phase 9): the person picks the opponent (hc.target_opponent)
    edict(g, d, false)?;
    let n = g.player(d).hand.len() as u64;
    if n > 0 {
        let i = g.rng.below(n) as usize;
        discard_index(g, d, i)?;
    }
    lose_life(g, d, 3, Some(p), "drain", None)?;
    gain(g, p, 3)?;
    draw(g, p, 1, false)
}

// ------------------------------------------------------------------ blocks and damage
/// p's attack on d: blocks, then combat damage. Returns the attackers that connected. Python's `resolve_combat`.
pub fn resolve_combat(g: &mut Game, p: PlayerId, atk: &[PermId], d: PlayerId, unbl: &[PermId]) -> Res<Vec<PermId>> {
    let mut tot = 0;
    let r = resolve_combat_inner(g, p, atk, d, unbl, &mut tot);
    crate::glog!(
        g,
        "  {} attacks {} with {} creature(s): {} damage (life now {})",
        g.player(p).name,
        g.player(d).name,
        atk.len(),
        tot,
        g.player(d).life
    );
    r
}

/// how many of d's creatures block, from the AI's rules: kill and survive, trade, chump (Python's inline blocks)
fn ai_blocks(
    g: &Game,
    p: PlayerId,
    atk: &[PermId],
    d: PlayerId,
    unbl: &[PermId],
    rng: &mut crate::rng::Rng,
) -> Vec<(PermId, PermId)> {
    let blockers: Vec<PermId> = untapped_creatures(g, d).collect();
    let mut incoming: i32 = atk.iter().map(|&m| epow(g, m) * if double_strike(g, m) { 2 } else { 1 }).sum();
    let (fd, fp) = (fiery(g, d), fiery(g, p));
    let mut assign = vec![];
    let mut used: Vec<PermId> = vec![];
    let mut order = atk.to_vec();
    order.sort_by_key(|&m| -epow(g, m)); // stable
    for a in order {
        if unbl.contains(&a) || !g.perm(a).on_bf {
            continue;
        }
        let cands: Vec<PermId> =
            blockers.iter().copied().filter(|b| !used.contains(b) && can_block(g, *b, a)).collect();
        if cands.is_empty() {
            continue;
        }
        if kw(g, a, "menace") && cands.len() < 2 {
            continue; // menace: two blockers or none
        }
        let (ap, at) = (epow(g, a), etgh(g, a));
        let good: Vec<PermId> = cands
            .iter()
            .copied()
            .filter(|&b| (epow(g, b) * fd >= at || g.perm(b).dt) && !(ap * fp >= etgh(g, b) || g.perm(a).dt))
            .collect();
        let b = if let Some(b) = min_by(&good, |x| pval(g, x)) {
            b
        } else {
            let trade: Vec<PermId> = cands
                .iter()
                .copied()
                .filter(|&b| epow(g, b) * fd >= at || g.perm(b).dt || card_name(g, b) == Some("Defiant Vanguard"))
                .collect();
            let spare = spare_tokens(g, d, &cands);
            if !trade.is_empty() && pval(g, a) >= trade.iter().map(|&x| pval(g, x)).fold(f64::INFINITY, f64::min) {
                min_by(&trade, |x| pval(g, x)).unwrap()
            } else if shielded(g, d) {
                continue; // the damage is prevented anyway: no chump blocks
            } else if !spare.is_empty() && ap >= 2 && (ap * fp >= 3 || incoming * fp >= g.player(d).life / 5) {
                min_by(&spare, |x| pval(g, x)).unwrap() // chump with a spare token
            } else if ap >= 2 && rng.random() < ai::chump_prob(g, d, incoming) {
                let mut cs = cands.clone();
                if g.player(d).key == "yshtola" && incoming < g.player(d).life {
                    cs.retain(|&x| !g.perm(x).is_cmd); // Y'shtola is the engine: she chumps only to live
                    if cs.is_empty() {
                        continue;
                    }
                }
                min_by(&cs, |x| pval(g, x)).unwrap()
            } else {
                continue;
            }
        };
        assign.push((a, b));
        used.push(b);
        incoming -= ap;
        if kw(g, a, "menace") {
            // the second blocker is spent; only the first fights
            if let Some(second) = min_by(&cands.iter().copied().filter(|&x| x != b).collect::<Vec<_>>(), |x| pval(g, x))
            {
                used.push(second);
            }
        }
    }
    assign
}

fn resolve_combat_inner(
    g: &mut Game,
    p: PlayerId,
    atk: &[PermId],
    d: PlayerId,
    unbl: &[PermId],
    tot: &mut i32,
) -> Res<Vec<PermId>> {
    step_priority(g, "attackers", Some(d), atk)?; // the declare attackers step's priority
    let mut atk: Vec<PermId> = atk.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).owner == p).collect();
    if g.over || !g.player(d).alive {
        return Ok(vec![]);
    }
    // HUMAN(phase 9): a person declares their own blockers (play/combat.human_blocks)
    let mut rng = std::mem::replace(&mut g.rng, crate::rng::Rng::from_u64(0));
    let mut assign = ai_blocks(g, p, &atk, d, unbl, &mut rng);
    g.rng = rng;
    cardcode::blocks_hooks(g, p, &atk, d, &mut assign)?; // card code that changes blocks
    for &(a, b) in &assign.clone() {
        // flanking: a blocker without flanking gets -1/-1
        if g.perm(b).on_bf && kw(g, a, "flanking") && !kw(g, b, "flanking") {
            let x = g.perm_mut(b);
            x.eot_pt = (x.eot_pt.0 - 1, x.eot_pt.1 - 1);
            if etgh(g, b) <= 0 {
                die(g, b, "sba")?;
            }
        }
    }
    g.blocking = assign.iter().map(|x| x.1).collect();
    let ringblk: Vec<PermId> =
        assign.iter().filter(|&&(a, b)| cardcode::ring_blocked(g, p, a, b)).map(|x| x.1).collect();
    let to_walker = walker_attacks(g, p, &atk, d, &assign);
    cardcode::defend_hooks(g, p, &mut atk, d, &mut assign)?; // Aetherize, Yawgmoth, Kor Haven, ninjutsu ...
    let blocker_of = |assign: &[(PermId, PermId)], a: PermId| assign.iter().find(|x| x.0 == a).map(|x| x.1);
    if !shielded(g, d)
        && !g.player(d).life_locked
        && g.db.id(TEFERIS_PROTECTION).is_some_and(|c| g.player(d).hand.contains(&c))
        && g.humans.get(d).is_none()
    {
        let unbl_dmg: Vec<(PermId, i32)> = atk
            .iter()
            .copied()
            .filter(|&a| {
                g.perm(a).on_bf
                    && blocker_of(&assign, a).is_none_or(|b| !g.perm(b).on_bf || g.perm(b).owner != d)
                    && !to_walker.iter().any(|x| x.0 == a)
            })
            .map(|a| (a, epow(g, a) * if double_strike(g, a) { 2 } else { 1 }))
            .collect();
        let lethal = unbl_dmg.iter().map(|x| x.1).sum::<i32>() >= g.player(d).life
            || unbl_dmg.iter().any(|&(a, x)| g.perm(a).is_cmd && g.player(d).cmd_dmg[p.index()] + x >= 21);
        if lethal {
            last_chance(g, d)?; // lethal coming: Teferi's Protection
        }
    }
    let mut conn = vec![];
    g.resolving += 1; // combat damage is dealt at once: its triggers wait
    let r = (|| -> Res {
        for &a in &atk {
            if !g.perm(a).on_bf || !g.player(d).alive {
                continue;
            }
            let mut ap = epow(g, a) * if double_strike(g, a) { 2 } else { 1 };
            if cardcode::damage_prevented(g, a) || cardcode::dovin_blocked(g, a) {
                ap = 0; // Old Fat Spider's chapter II, Dovin's lock
            }
            let tr =
                g.player(p).trample || has(g, p, Tag::Uprising) || card_tag(g, a, Tag::Trample) || kw(g, a, "trample");
            let mut dmg = match blocker_of(&assign, a) {
                None => ap,
                Some(b) if !(g.perm(b).on_bf && g.perm(b).owner == d) => {
                    if tr { ap } else { 0 } // its blocker is gone: still blocked (trample: all to the player)
                }
                Some(b) => {
                    let bt = etgh(g, b);
                    let mut a_dies = (epow(g, b) * fiery(g, d) >= etgh(g, a) || g.perm(b).dt)
                        && !protected_from(g, a, colors_of(g, b))
                        && !no_damage(g, a);
                    if cardcode::damage_prevented(g, b) {
                        a_dies = false;
                    }
                    let mut b_dies = (ap * fiery(g, p) >= bt || g.perm(a).dt)
                        && !protected_from(g, b, colors_of(g, a))
                        && !no_damage(g, b);
                    if cardcode::prot_vs(g, a, b) {
                        a_dies = false; // protection from creatures / Demons and Dragons
                    }
                    if cardcode::prot_vs(g, b, a) {
                        b_dies = false;
                    }
                    let (fa, fb) = (first_strike(g, a), first_strike(g, b));
                    if fa && !fb && b_dies {
                        a_dies = false; // first strike kills the blocker first
                    } else if fb && !fa && a_dies {
                        b_dies = false;
                    }
                    let dmg = if tr { (ap - bt).max(0) } else { 0 };
                    if ap > 0 && cardcode::kaldra_exile(g, a, b)? {
                        b_dies = false; // Sword of Kaldra: exiles what it damages
                    }
                    if epow(g, b) > 0 && g.perm(a).on_bf && cardcode::kaldra_exile(g, b, a)? {
                        a_dies = false;
                    }
                    if b_dies {
                        die(g, b, "destroy")?;
                    }
                    if a_dies {
                        die(g, a, "destroy")?;
                    }
                    dmg
                }
            };
            if dmg > 0 && g.fog == Some(g.turn_stamp()) {
                dmg = 0; // Spore Frog and other fogs
            }
            if dmg > 0
                && let Some(&(_, w)) = to_walker.iter().find(|x| x.0 == a)
                && g.perm(w).on_bf
                && g.perm(w).owner == d
            {
                // this attacker went after a planeswalker
                let l = g.perm(w).loyalty.unwrap_or(0) - dmg;
                g.perm_mut(w).loyalty = Some(l);
                *tot += dmg;
                crate::glog!(g, "    {} deals {} to {} (loyalty {})", g.perm(a).name, dmg, g.perm(w).name, l);
                if l <= 0 {
                    leave(g, w)?;
                    to_zone_card(g, w, Zone::Gy);
                    let name = g.perm(w).name;
                    *g.player_mut(d).lost_names.entry(name).or_insert(0) += 1;
                }
                dmg = 0;
            }
            if dmg > 0 && prevents_damage(g, d, Some(p)) {
                // protection / Glacial Chasm: no damage, lifelink, commander damage or triggers
                g.player_mut(d).stat("dmg_prevented", dmg as i64);
                crate::glog!(
                    g,
                    "    {} combat damage from {} to {} is prevented",
                    dmg,
                    g.perm(a).name,
                    g.player(d).name
                );
                dmg = 0;
            }
            if dmg > 0 && card_name(g, a) == Some("Szadek, Lord of Secrets") {
                // instead: +1/+1 counters, and they mill that many
                g.perm_mut(a).plus += dmg;
                mill(g, d, dmg as usize)?;
                conn.push(a);
                crate::glog!(g, "    Szadek: {} +1/+1 counters, {} mills {}", dmg, g.player(d).name, dmg);
                dmg = 0;
            }
            if dmg > 0 {
                lose_life(g, d, dmg, Some(p), "combat", None)?;
                conn.push(a);
                *tot += dmg;
                if g.dsl_on {
                    let fired = crate::dsl::Fired { attacker: Some(a), defender: Some(d), ..Default::default() };
                    crate::dsl::fire(g, "combat_damage", fired)?;
                }
                if !g.hooks.is_empty() {
                    fire_trigger(g, Event::CombatDamage, Call::CombatDamage { p, a, d, dmg })?;
                }
                cardcode::combat_damage_cards(g, p, a, d, dmg)?; // Insight, emblems
                if g.monarch == Some(d) {
                    cardcode::become_monarch(g, p)?;
                }
                if card_tag(g, a, Tag::Hellkite) {
                    // Hellkite Tyrant steals their artifacts
                    let arts: Vec<PermId> = g
                        .player(d)
                        .perms
                        .iter()
                        .copied()
                        .filter(|&x| {
                            !g.is_creature(x) && g.perm(x).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT))
                        })
                        .collect();
                    for x in arts {
                        g.player_mut(d).perms.retain(|&y| y != x);
                        let px = g.perm_mut(x);
                        px.owner = p;
                        px.attached = None;
                        g.player_mut(p).perms.push(x);
                        g.bf_ver += 1;
                    }
                }
                if g.perm(a).lifelink || kw(g, a, "lifelink") {
                    gain(g, p, dmg)?;
                }
                if g.perm(a).is_cmd {
                    g.player_mut(d).cmd_dmg[p.index()] += dmg;
                }
                if g.perm(a).army && has(g, p, Tag::Sauron) {
                    cardcode::ring_tempt(g, p)?; // Sauron: an Army's combat damage tempts
                }
                cardcode::ring_damage(g, p, a, d)?; // the Ring, level 4: each opponent loses 3
                if equipped(g, a, Tag::Sword) {
                    let n = g.player(d).hand.len() as u64;
                    if n > 0 {
                        let i = g.rng.below(n) as usize;
                        discard_index(g, d, i)?;
                    }
                    for l in g.player(p).lands.clone() {
                        g.land_mut(l).tapped = false;
                    }
                }
            }
        }
        for b in ringblk {
            if g.perm(b).on_bf {
                die(g, b, "sac")?; // the Ring, level 3: blockers are sacrificed
            }
        }
        cardcode::vanguard_blocks(g, &assign)?; // Defiant Vanguard: at end of combat
        g.blocking.clear();
        if !conn.is_empty() && has(g, p, Tag::Facebreaker) {
            let n = find(g, p, Tag::Facebreaker).len() as i32;
            add_treasure(g, p, n)?; // Professional Face-Breaker
        }
        check_state(g)
    })();
    g.resolving -= 1;
    r?;
    if g.resolving == 0 && !g.trig_queue.is_empty() && !g.over {
        crate::engine::stack::flush_triggers(g)?;
        check_state(g)?;
    }
    Ok(conn)
}

/// Pool games: send unblocked attackers at d's most valuable planeswalker when that's worth more than the face
/// damage (enough to kill it, or it's near its ultimate)
pub fn walker_attacks(
    g: &Game,
    p: PlayerId,
    atk: &[PermId],
    d: PlayerId,
    assign: &[(PermId, PermId)],
) -> Vec<(PermId, PermId)> {
    let ws: Vec<PermId> = g
        .player(d)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            !g.perm(m).phased
                && g.perm(m).loyalty.is_some_and(|l| l != 0)
                && g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER))
        })
        .collect();
    if ws.is_empty() {
        return vec![];
    }
    let mut free: Vec<PermId> = atk
        .iter()
        .copied()
        .filter(|&a| !assign.iter().any(|x| x.0 == a) && g.perm(a).on_bf && g.perm(a).owner == p)
        .collect();
    free.sort_by_key(|&a| epow(g, a)); // stable
    if free.is_empty() {
        return vec![];
    }
    if g.player(d).life as f64 <= free.iter().map(|&a| epow(g, a)).sum::<i32>() as f64 * 1.2 {
        return vec![]; // lethal on the player instead
    }
    let mut out: Vec<(PermId, PermId)> = vec![];
    let mut order = ws;
    // the most dangerous walkers first (closest to an ultimate, then value)
    order.sort_by(|&a, &b| {
        (cardcode::ult_pressure(g, b), pval(g, b)).partial_cmp(&(cardcode::ult_pressure(g, a), pval(g, a))).unwrap()
    });
    for w in order {
        if pval(g, w) < 4.0 && cardcode::ult_pressure(g, w) < 0.6 {
            continue;
        }
        let mut need = g.perm(w).loyalty.unwrap_or(0);
        let wanderer = card_name(g, w) == Some("The Eternal Wanderer"); // no more than one creature can attack it
        let mut cap = if wanderer { 1 } else { 99 };
        let mut avail: Vec<PermId> = free.iter().copied().filter(|a| !out.iter().any(|x| x.0 == *a)).collect();
        if wanderer {
            avail.reverse();
        }
        for a in avail {
            if need <= 0 || cap <= 0 {
                break;
            }
            cap -= 1;
            out.push((a, w));
            need -= epow(g, a) * if double_strike(g, a) { 2 } else { 1 };
        }
    }
    out
}

/// The Ozolith: at the beginning of combat on your turn, move its counters onto a creature (the Army first)
pub fn ozolith_move(g: &mut Game, p: PlayerId) {
    let k = g.player(p).ozolith_counters;
    if k == 0 || !has(g, p, Tag::Ozolith) {
        return;
    }
    let cr: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
    if cr.is_empty() {
        return;
    }
    let tgt = army_of(g, p)
        .unwrap_or_else(|| max_by(&cr, |m| if g.perm(m).noatk { 0.0 } else { 1000.0 } + epow(g, m) as f64).unwrap());
    g.perm_mut(tgt).plus += k;
    g.player_mut(p).ozolith_counters = 0;
    g.player_mut(p).stat("ozolith_moves", 1);
    crate::glog!(g, "  The Ozolith moves {} counters onto {}", k, g.perm(tgt).name);
}

/// p's combat phase (and extra combats). Python's `combat`.
pub fn combat(g: &mut Game, p: PlayerId) -> Res {
    ozolith_move(g, p);
    g.player_mut(p).haste_all = false;
    let mut ncomb = 0;
    while ncomb < 4 && !g.over && g.player(p).alive {
        ncomb += 1;
        g.player_mut(p).combat_no = ncomb;
        if g.opps(p).next().is_none() {
            return Ok(());
        }
        step_priority(g, "combat", None, &[])?;
        if g.over || !g.player(p).alive {
            return Ok(());
        }
        if !g.hooks.is_empty() && ncomb == 1 {
            fire_trigger(g, Event::Crew, Call::Player { p })?;
        }
        // HUMAN(phase 9): a person declares their attackers (play/combat.human_attack)
        let haste_all = g.player(p).haste_all;
        let mut atk: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                let x = g.perm(m);
                g.is_creature(m)
                    && !x.tapped
                    && !x.phased
                    && (!x.sick || haste_all || has_haste(g, m))
                    && (!x.noatk || epow(g, m) >= 3) // pumped mana dorks attack
                    && epow(g, m) > 0
                    && !crate::impls::mine::ring_loot_decks(g, p, m)
            })
            .collect();
        if !g.auras.is_empty() {
            atk.retain(|&m| !cardcode::locked(g, m, "pacify")); // Arrest
        }
        if chasm(g, p) {
            atk.clear(); // Glacial Chasm: creatures you control can't attack
        }
        if atk.is_empty() {
            break;
        }
        let plan = if ncomb == 1 { ai::choose_attack(g, p)? } else { None }; // the look-ahead's plan (M4)
        if plan.is_some_and(|x| x.1 == "none") {
            break;
        }
        let all_atk = atk.clone();
        let mut d = match plan {
            Some((i, _)) => i,
            None => ai::choose_defender(g, p),
        };
        let mut groups = if plan.is_some_and(|x| x.1 == "all") {
            vec![(d, all_atk)] // everything at one player
        } else {
            if ncomb == 1 {
                atk = ai::filter_attackers(g, p, atk);
            }
            let pre = atk.clone();
            atk = ai::attack_filter(g, p, atk, d); // outside decks' own filter
            let held: Vec<PermId> = pre.into_iter().filter(|m| !atk.contains(m)).collect();
            split_attack(g, p, d, &atk, &held)
        };
        let declared: Vec<PermId> = groups.iter().flat_map(|x| x.1.iter().copied()).collect();
        let mut kept = vec![];
        for (gd, xs) in groups.clone() {
            // each defending player's taxes and restrictions
            let cand0: Vec<PermId> = declared
                .iter()
                .copied()
                .filter(|m| xs.contains(m) || !groups.iter().any(|(q, ys)| *q != gd && ys.contains(m)))
                .collect();
            let mut xs = xs;
            if !g.hooks.is_empty() {
                xs = attack_limits(g, p, gd, xs)?;
                if attack_restrictions(g, p, gd).0 == 0 && gd == d {
                    xs = cardcode::forced_attackers(g, p, xs, &cand0)?;
                }
                xs = cardcode::annex_life(g, p, gd, xs)?;
            }
            if !xs.is_empty() {
                kept.push((gd, xs));
            }
        }
        groups = kept;
        if !groups.iter().any(|x| x.0 == d)
            && let Some(first) = groups.first()
        {
            d = first.0;
        }
        let mut atk: Vec<PermId> = groups.iter().flat_map(|x| x.1.iter().copied()).collect();
        if !g.hooks.is_empty() && !atk.is_empty() {
            // beginning of combat (Helm of the Host)
            let new = cardcode::combat_start(g, p)?;
            for m in new {
                if !atk.contains(&m) {
                    atk.push(m);
                    if let Some(gr) = groups.iter_mut().find(|x| x.0 == d) {
                        gr.1.push(m);
                    }
                }
            }
        }
        if atk.is_empty() {
            break;
        }
        let mut unbl = vec![];
        let army = army_of(g, p);
        let st = g.turn_stamp();
        for &m in &atk {
            let x = g.perm(m);
            if x.army && equipped(g, m, Tag::Cloak) {
                unbl.push(m);
            }
            if x.data.get(DataKey::Unbl) == Some(&Val::Stamp(st)) {
                unbl.push(m); // Rogue's Passage, activated by a person
            }
            if g.player(p).unbl_all == Some(st) {
                unbl.push(m); // Venser, the Sojourner's -1
            }
        }
        if let Some(a) = army
            && atk.contains(&a)
            && !unbl.contains(&a)
            && epow(g, a) >= 5
        {
            let cands: Vec<_> = g
                .player(p)
                .lands
                .iter()
                .copied()
                .filter(|&l| g.db.get(g.land(l).cd).tag(Tag::Passage) && !g.land(l).tapped)
                .collect();
            let mut passage = None;
            for l in cands {
                let name = g.db.get(g.land(l).cd).name.to_string();
                if !blocked(g, p, &name) {
                    passage = Some(l);
                    break;
                }
            }
            if let Some(l) = passage {
                g.land_mut(l).tapped = true;
                if can_pay(g, p, 4, "", false) {
                    pay(g, p, 4, "", false)?;
                    unbl.push(a);
                } else {
                    g.land_mut(l).tapped = false;
                }
            }
        }
        for &m in &atk {
            if !(g.perm(m).vig || kw(g, m, "vigilance")) {
                g.perm_mut(m).tapped = true;
            }
        }
        let new = attack_triggers(g, p, &atk, d)?; // once for the whole attack, naming the main defender
        atk.extend(&new);
        if let Some(gr) = groups.iter_mut().find(|x| x.0 == d) {
            gr.1.extend(&new);
        }
        cardcode::ring_attack(g, p, &atk)?; // the Ring, level 2: loot
        cardcode::hand_attack(g, p, &atk, d)?;
        if g.over || !groups.iter().any(|x| g.player(x.0).alive) {
            continue;
        }
        g.in_combat = atk.clone(); // attacking creatures (Divine Verdict's targets)
        let mut conn = vec![];
        let r = (|| -> Res {
            for (gd, xs) in &groups {
                // each defending player blocks, then takes damage, in turn
                if g.over || !g.player(p).alive || !g.player(*gd).alive {
                    continue;
                }
                conn.extend(resolve_combat(g, p, xs, *gd, &unbl)?);
            }
            Ok(())
        })();
        g.in_combat.clear();
        r?;
        if g.over || !g.player(p).alive {
            return Ok(());
        }
        if g.player(p).key == "sauron"
            && has(g, p, Tag::Assault)
            && let Some(a) = army.filter(|&a| g.perm(a).on_bf)
            && !blocked(g, p, "Aggravated Assault")
        {
            if conn.contains(&a) && equipped(g, a, Tag::Sword) && unbl.contains(&a) && can_pay(g, p, 3, "RR", false) {
                pay(g, p, 3, "RR", false)?;
                g.player_mut(p).stat("combo_attempt", 1);
                let turns = g.player(p).turns;
                g.player_mut(p).milestone.entry("combo").or_insert(turns);
                crate::glog!(g, "  Sauron goes for Sword + Aggravated Assault infinite combats");
                if ai::combo_interrupted(g, p, "sauron", &[a])? {
                    g.player_mut(p).stat("combo_stopped", 1);
                    crate::glog!(g, "    ...the combo is stopped");
                    break;
                }
                return ai::win(g, p, "combo", None);
            }
            let again = conn.contains(&a) && equipped(g, a, Tag::Sword); // the Sword untapped the lands: again
            if (ncomb == 1 || again) && ncomb < 12 && can_pay(g, p, 3, "RR", false) {
                pay(g, p, 3, "RR", false)?;
                let asl = find(g, p, Tag::Assault).first().copied();
                if asl.is_some() && !ability_window(g, p, asl, "untap, an additional combat", Some(7.0), None)? {
                    break;
                }
                untap_creatures(g, p);
                continue;
            }
        }
        if g.player(p).extra_combats > 0 && ncomb < 4 {
            // extra combat phases granted by card abilities
            g.player_mut(p).extra_combats -= 1;
            untap_creatures(g, p);
            continue;
        }
        break;
    }
    let pl = g.player_mut(p);
    pl.haste_all = false;
    pl.trample = false;
    Ok(())
}

fn untap_creatures(g: &mut Game, p: PlayerId) {
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) {
            g.perm_mut(m).tapped = false;
        }
    }
}

/// an activated ability of `name` is shut off by Disruptor Flute (counts it for the reports)
pub fn blocked(g: &mut Game, p: PlayerId, name: &str) -> bool {
    if stopped(g, name) {
        g.player_mut(p).stat("flute_blocked", 1);
        return true;
    }
    false
}
