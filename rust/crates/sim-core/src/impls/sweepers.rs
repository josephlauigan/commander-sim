//! One-sided board control for the Sauron deck's sweeper tests (2026-10-12): Crippling Fear, Widespread Brutality,
//! Chandra's Ignition, Inferno Titan and Ral and the Implicit Maze. Massacre Wurm is the `wurm` tag (engine/zones.rs).
//!
//! The three sorcery sweepers are `wipe` kinds ("fear", "brutal", "ignite"): [`Sweep`] is the one rule for which
//! permanents each hits, read by the AI's wipe valuation (ai/decks.rs `wipe_eval`, `WipeRule`) and by the resolution
//! (engine/removal.rs `apply_wipe_inner`), so what the AI expects is what happens. Damage that does not kill a
//! creature is not tracked (as for every damage effect in the engine), so a creature is hit when the damage is at
//! least its toughness.

use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::discard_worst;
use crate::engine::life::{check_state, lose_life};
use crate::engine::removal::apply_removal;
use crate::engine::stack::trigger_window;
use crate::engine::values::{army_of, epow, etgh, has_type, indestructible, no_damage, pval, subtypes, untargetable};
use crate::engine::zones::{Tokens, amass, die, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

/// the `wipe=` kinds this module owns
pub const KINDS: [&str; 4] = ["fear", "brutal", "ignite", "dust"];

pub fn is_kind(kind: &str) -> bool {
    KINDS.contains(&kind)
}

/// which permanents one of the three sweepers hits, planned from the board (`resolving`: after Widespread Brutality's
/// amass, so the Army's real power is used; before it, the power it will have)
pub struct Sweep {
    kind: Sym,
    ty: Sym,
    src: Option<PermId>,
    power: i32,
}

/// m is of creature type t (a Army made by amass counts as an Orc: it is made as an "Orc Army")
fn of_type(g: &Game, m: PermId, t: &str) -> bool {
    (g.perm(m).army && t == "orc") || has_type(g, m, t)
}

fn live_creatures(g: &Game) -> Vec<(PlayerId, PermId)> {
    g.players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().map(move |&m| (q.id, m)))
        .filter(|&(_, m)| g.perm(m).on_bf && !g.perm(m).phased && g.is_creature(m))
        .collect()
}

/// Crippling Fear's creature type: the one that leaves the caster the most (opposing value lost minus 1.2 times its own
/// value lost). Orc is tried first, so a tie keeps the Army safe.
pub fn fear_type(g: &Game, caster: PlayerId) -> Sym {
    let cs = live_creatures(g);
    let mut cands: Vec<Sym> = vec!["orc"];
    for &(_, m) in &cs {
        for t in subtypes(g, m) {
            if !cands.contains(&t) {
                cands.push(t);
            }
        }
    }
    let score = |t: Sym| -> f64 {
        cs.iter()
            .filter(|&&(_, m)| !of_type(g, m, t) && etgh(g, m) <= 3)
            .map(|&(q, m)| if q == caster { -1.2 * pval(g, m) } else { pval(g, m) })
            .sum()
    };
    let mut best = (cands[0], score(cands[0]));
    for &t in &cands[1..] {
        let s = score(t);
        if s > best.1 {
            best = (t, s);
        }
    }
    best.0
}

impl Sweep {
    pub fn new(g: &Game, caster: PlayerId, kind: &str, resolving: bool) -> Sweep {
        let kind = intern(kind);
        let mut s = Sweep { kind, ty: "", src: None, power: 0 };
        match kind {
            "fear" => s.ty = fear_type(g, caster),
            "brutal" => {
                let bonus = if crate::engine::values::has(g, caster, Tag::Mauhur) { 1 } else { 0 };
                s.power = match army_of(g, caster) {
                    Some(a) if resolving => epow(g, a),
                    Some(a) => epow(g, a) + 2 + bonus,
                    None => 2 + bonus,
                };
            }
            "ignite" => {
                let own: Vec<PermId> = g
                    .player(caster)
                    .perms
                    .iter()
                    .copied()
                    .filter(|&m| g.perm(m).on_bf && !g.perm(m).phased && g.is_creature(m))
                    .collect();
                s.src = max_by(&own, |m| epow(g, m) as f64);
                s.power = s.src.map_or(0, |m| epow(g, m).max(0));
            }
            _ => {}
        }
        s
    }

    /// does it hit permanent m? (the caller skips phased-out permanents)
    pub fn hits(&self, g: &Game, m: PermId) -> bool {
        if self.kind == "dust" {
            return crate::impls::karn_cards::dust_hits(g, m); // All Is Dust: every coloured permanent, not only creatures
        }
        if !g.is_creature(m) {
            return false;
        }
        match self.kind {
            "fear" => !of_type(g, m, self.ty) && etgh(g, m) <= 3,
            "brutal" => !g.perm(m).army && etgh(g, m) <= self.power && !no_damage(g, m) && !indestructible(g, m),
            "ignite" => {
                self.src.is_some_and(|s| s != m)
                    && self.power > 0
                    && etgh(g, m) <= self.power
                    && !no_damage(g, m)
                    && !indestructible(g, m)
            }
            _ => false,
        }
    }
}

/// ais.wipe_eval for these kinds: (what the opponents lose, what the caster loses, no single victim)
pub fn eval(g: &Game, p: PlayerId, kind: &str) -> (f64, f64, Option<PlayerId>) {
    if kind == "dust" {
        return crate::impls::karn_cards::dust_eval(g, p);
    }
    let rule = Sweep::new(g, p, kind, false);
    let (mut ol, mut ml) = (0.0, 0.0);
    for (q, m) in live_creatures(g) {
        if !rule.hits(g, m) {
            continue;
        }
        if q == p {
            ml += pval(g, m);
        } else {
            ol += pval(g, m);
        }
    }
    if kind == "brutal" && ol > 0.0 {
        ol += 2.0; // and the Army grows by two
    }
    if kind == "ignite" && rule.power > 0 {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        if ol > 0.0 {
            ol += 0.25 * rule.power as f64 * opps.len() as f64; // the damage to each opponent, a bonus on a real sweep
        }
        let dead: Vec<&PlayerId> = opps.iter().filter(|&&q| g.player(q).life <= rule.power).collect();
        ol += 30.0 * dead.len() as f64;
        if !opps.is_empty() && dead.len() == opps.len() {
            ol += 100.0; // it wins the game
        }
    }
    (ol, ml, None)
}

/// resolve one of the three sweepers for p (called from `apply_wipe_inner`; `respond` lets each victim answer)
pub fn resolve(g: &mut Game, p: PlayerId, kind: Sym) -> Res {
    if kind == "dust" {
        return crate::impls::karn_cards::dust_resolve(g, p);
    }
    if kind == "brutal" {
        amass(g, p, 2)?; // "Amass Zombies 2": a Zombie Army, here the Orc Army; then the Army deals damage
    }
    let rule = Sweep::new(g, p, kind, true);
    if kind == "fear" {
        crate::glog!(g, "    Crippling Fear names {}", rule.ty);
    }
    if kind == "ignite" && rule.src.is_none() {
        return Ok(()); // no creature of yours to deal the damage
    }
    let victims: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    let mut doomed: Vec<PermId> = vec![];
    for q in victims {
        if g.over {
            return Ok(());
        }
        let prot = crate::ai::wipe_response(g, q, kind, p)?;
        if prot == Some("all") {
            continue;
        }
        for m in g.player(q).perms.clone() {
            if !g.perm(m).on_bf || g.perm(m).phased || !rule.hits(g, m) {
                continue;
            }
            if prot == Some("indes") && kind != "fear" {
                continue;
            }
            doomed.push(m);
        }
    }
    for m in doomed {
        if g.perm(m).on_bf {
            die(g, m, if kind == "fear" { "sba" } else { "destroy" })?;
        }
    }
    if kind == "ignite" && rule.power > 0 {
        // "and each opponent"
        for q in g.opps(p).collect::<Vec<_>>() {
            lose_life(g, q, rule.power, Some(p), "burn", None)?;
        }
    }
    check_state(g)
}

// ------------------------------------------------------------------ Inferno Titan
/// Inferno Titan: "Whenever this creature enters or attacks, it deals 3 damage divided as you choose among one, two,
/// or three targets." The best opponent targets it can kill with its 3 points (creatures worth killing, planeswalkers),
/// a player when that is lethal; what is left goes to a planeswalker or to the opponent with the least life.
fn titan_damage(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if opps.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "3 damage divided", Some(5.0))? {
        return Ok(());
    }
    if crate::human::is_human(g, p) {
        return crate::human::choices::divide_damage(g, p, 3, "Inferno Titan", "burn");
    }
    let mut left = 3;
    // lethal to a player: all of it at that player
    if let Some(q) = opps.iter().copied().filter(|&q| g.player(q).life <= 3).min_by_key(|&q| g.player(q).life) {
        crate::glog!(g, "    Inferno Titan deals 3 damage to {}", g.player(q).name);
        lose_life(g, q, 3, Some(p), "burn", None)?;
        return check_state(g);
    }
    // (cost to kill, value) per target
    let mut cands: Vec<(PermId, i32, f64)> = vec![];
    for &q in &opps {
        for &m in &g.player(q).perms {
            let x = g.perm(m);
            if !x.on_bf || x.phased || untargetable(g, m) || no_damage(g, m) || indestructible(g, m) {
                continue;
            }
            let walker = x.cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER));
            let cost = if g.is_creature(m) {
                etgh(g, m).max(1)
            } else if walker {
                x.loyalty.unwrap_or(0).max(1)
            } else {
                continue;
            };
            if cost <= 3 && pval(g, m) >= 1.0 {
                cands.push((m, cost, pval(g, m)));
            }
        }
    }
    cands.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    cands.truncate(8);
    // the best subset of at most three targets whose kill costs fit in 3
    let n = cands.len();
    let mut best: (f64, Vec<usize>) = (0.0, vec![]);
    for mask in 1u32..(1 << n) {
        if mask.count_ones() > 3 {
            continue;
        }
        let (mut cost, mut val) = (0, 0.0);
        let mut idx = vec![];
        for (i, c) in cands.iter().enumerate() {
            if mask >> i & 1 == 1 {
                cost += c.1;
                val += c.2;
                idx.push(i);
            }
        }
        if cost <= 3 && val > best.0 {
            best = (val, idx);
        }
    }
    for i in best.1 {
        let (m, cost, _) = cands[i];
        crate::glog!(g, "    Inferno Titan deals {cost} damage to {}", g.perm(m).name);
        apply_removal(g, Some(p), m, &format!("dmg{cost}"), None)?;
        left -= cost;
    }
    if left > 0 {
        // a planeswalker it cannot kill loses the rest of the loyalty, else the opponent with the least life
        let walker = opps
            .iter()
            .flat_map(|&q| g.player(q).perms.clone())
            .filter(|&m| {
                let x = g.perm(m);
                x.on_bf && !x.phased && x.loyalty.is_some() && !untargetable(g, m)
            })
            .max_by_key(|&m| g.perm(m).loyalty.unwrap_or(0));
        if let Some(w) = walker {
            let l = g.perm(w).loyalty.unwrap_or(0) - left;
            g.perm_mut(w).loyalty = Some(l);
            if l <= 0 {
                apply_removal(g, Some(p), w, "dmg6", None)?;
            }
        } else if let Some(q) = opps.iter().copied().min_by_key(|&q| g.player(q).life) {
            lose_life(g, q, left, Some(p), "burn", None)?;
        }
    }
    check_state(g)
}

fn titan_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && g.perm(src).owner == p {
        titan_damage(g, p, src)?;
    }
    Ok(())
}

fn titan_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p && atk.contains(&src) && g.perm(src).on_bf {
        titan_damage(g, p, src)?;
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ Ral and the Implicit Maze
// (the Saga code that was Zada's, from commit e4d8039^ impls/zada_cards.rs)
const ROMAN: [&str; 4] = ["", "I", "II", "III"];

fn exile_top(g: &mut Game, p: PlayerId) -> Option<CardId> {
    let c = g.player_mut(p).library.pop()?;
    g.player_mut(p).seen.insert(c);
    Some(c)
}

/// a card exiled "you may play it until the end of your next turn"
fn playable_until_next_turn(g: &mut Game, p: PlayerId, c: CardId) {
    let pl = g.player_mut(p);
    pl.hand.push(c);
    let until = pl.turns + 1;
    pl.impulse_long.push((c, until));
}

/// I 2 damage to each creature and planeswalker your opponents control, II you may discard a card: if you do, exile
/// the top two (playable until the end of your next turn), III a Spellgorger Weird
fn ral_run(g: &mut Game, p: PlayerId, n: i64) -> Res {
    match n {
        1 => {
            for q in g.opps(p).collect::<Vec<_>>() {
                for m in g.player(q).perms.clone() {
                    if !g.perm(m).on_bf || g.perm(m).phased {
                        continue;
                    }
                    let walker = g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER));
                    // a creature is hit when 2 damage kills it (other damage is not tracked); a planeswalker
                    // loses 2 loyalty
                    if g.is_creature(m) {
                        if etgh(g, m) <= 2 {
                            apply_removal(g, Some(p), m, "dmg2", None)?;
                        }
                    } else if walker {
                        let l = g.perm(m).loyalty.unwrap_or(0);
                        if l <= 2 {
                            apply_removal(g, Some(p), m, "dmg6", None)?;
                        } else {
                            g.perm_mut(m).loyalty = Some(l - 2);
                        }
                    }
                }
            }
        }
        2 => {
            if crate::human::is_human(g, p) {
                // "you may discard a card. If you do, exile the top two cards of your library"
                if !g.player(p).hand.is_empty()
                    && crate::human::choices::yes_no(
                        g,
                        p,
                        "Ral and the Implicit Maze: discard a card to exile the top two cards of your library?",
                    )?
                {
                    crate::human::choices::discard(g, p, 1, "Ral and the Implicit Maze: discard a card")?;
                    for _ in 0..2 {
                        if let Some(c) = exile_top(g, p) {
                            crate::glog!(g, "    Ral and the Implicit Maze exiles {}", g.db.get(c).name);
                            playable_until_next_turn(g, p, c);
                        }
                    }
                }
            } else if !g.player(p).hand.is_empty() && g.player(p).library.len() >= 2 {
                discard_worst(g, p, 1)?;
                for _ in 0..2 {
                    if let Some(c) = exile_top(g, p) {
                        crate::glog!(g, "    Ral and the Implicit Maze exiles {}", g.db.get(c).name);
                        playable_until_next_turn(g, p, c);
                    }
                }
            }
        }
        _ => {
            let spec = Tokens { color: Some(Colors::from_letters("R")), types: vec!["weird"], ..Tokens::new(1, 2) };
            for t in make_tokens(g, p, Tokens { tgh: Some(2), ..spec })? {
                g.perm_mut(t).data.set(DataKey::WeirdToken, Val::Bool(true));
            }
        }
    }
    Ok(())
}

const RAL_TEXT: [&str; 3] =
    ["2 damage to each opposing creature and planeswalker", "discard, exile two", "Spellgorger Weird"];

fn ral_chapter(g: &mut Game, src: PermId, p: PlayerId, n: i64) -> Res {
    let name = format!("chapter {}: {}", ROMAN[n as usize], RAL_TEXT[n as usize - 1]);
    if trigger_window(g, p, Some(src), &name, None)? && g.perm(src).on_bf {
        ral_run(g, p, n)?;
    }
    if n >= 3 && g.perm(src).on_bf {
        die(g, src, "sac")?; // the final chapter is gone: sacrifice the Saga
    }
    Ok(())
}

fn ral_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(1));
    ral_chapter(g, src, p, 1)
}

fn ral_main(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || !g.perm(src).on_bf || g.perm(src).data.get(DataKey::Lore).is_none() {
        return Ok(());
    }
    let n = g.perm(src).data.int(DataKey::Lore) + 1;
    if n > 3 {
        return Ok(());
    }
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(n));
    ral_chapter(g, src, p, n)
}

/// cast it for chapter I when it kills something of theirs (the creatures it would kill are small: hold it otherwise)
fn ral_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let hurt = g.opps(p).any(|q| g.player(q).perms.iter().any(|&m| g.is_creature(m) && etgh(g, m) <= 2));
    if hurt { 64 } else { 50 }
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Inferno Titan")?;
    c.etb = Some(titan_etb);
    c.attack = Some(titan_attack);
    let c = r.card(db, "Ral and the Implicit Maze")?;
    c.etb = Some(ral_etb);
    c.main1 = Some(ral_main);
    c.prio = Some(ral_prio);
    Ok(())
}
