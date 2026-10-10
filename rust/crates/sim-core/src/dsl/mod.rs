//! The card ability language's interpreter (Python's `cards/dsl.py`): compiled abilities (`model.rs`) run by the
//! engine. Hand-tagged cards keep the engine's own code; both kinds play together in one game.
//!
//!   - `fire`: triggered abilities (enters, dies, attacks, cast, draw, upkeep ...), put on the stack as known triggers
//!   - `resolve_spell`: instants and sorceries
//!   - `pt`, `has_kw`, `protection`, `cost_delta`, `token_mult`, `counter_mult`, `no_lifegain`: static abilities
//!   - `ability_options`: activated, loyalty and equip abilities as choices for the AI
//!   - `card_value`, `value_of`: how much the AI wants to cast a card it has no hand-written priority for

pub mod model;

use crate::cards::{Colors, Types};
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::values::{colors_of, epow, etgh, has, has_type, protected_from, pval, threat, untargetable};
use crate::engine::zones::{self, Enter, Tokens, max_by, min_by};
use crate::flow::Res;
use crate::hooks::{TrigAct, Trigger};
use crate::ids::{CardId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};
use crate::tag::Tag;
use model::{Ability, Effect, Filter, Num, Sel, To};
use std::sync::Arc;

pub const MAX_DEPTH: u32 = 6;

/// What an effect knows about why it's happening (Python's ctx dict for the interpreter).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DslCtx {
    /// X (Python's default: 3)
    pub x: i32,
    /// a target chosen as the spell was cast
    pub target: Option<PermId>,
    /// the last target's power, toughness and mana value as it was hit
    pub snapshot: Option<(i32, i32, u32)>,
    pub target_owner: Option<PlayerId>,
    /// the permanent and player the trigger is about
    pub event_perm: Option<PermId>,
    pub event_player: Option<PlayerId>,
    /// the spell a cast trigger is about
    pub spell: Option<CardId>,
    /// life opponents lost to this ability so far
    pub lost: i32,
    /// an attack trigger: attacking tokens it makes join the attack
    pub attack: bool,
}

impl DslCtx {
    fn base() -> DslCtx {
        DslCtx { x: 3, ..DslCtx::default() }
    }
}

pub fn active(g: &Game) -> bool {
    g.dsl_on
}

fn abilities(g: &Game, c: CardId) -> Arc<[Ability]> {
    g.db.get(c).abilities.clone()
}

// ------------------------------------------------------------------ numbers, filters, players
pub fn num(g: &Game, p: PlayerId, n: Option<&Num>, ctx: &DslCtx, src: Option<PermId>) -> i32 {
    let name = match n {
        None => return 1,
        Some(Num::Int(k)) => return *k,
        Some(Num::Name(s)) => s.as_str(),
    };
    let pl = g.player(p);
    match name {
        "X" => ctx.x.max(1),
        "creatures_you_control" => pl.perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() as i32,
        "cards_in_hand" => pl.hand.len() as i32,
        "opponents" => g.opps(p).count() as i32,
        "spells_this_turn" => pl.spells_this_turn as i32,
        "devotion_B" => pl
            .perms
            .iter()
            .filter_map(|&m| g.perm(m).cd)
            .filter(|&c| g.db.get(c).perm)
            .map(|c| g.db.get(c).pips.chars().filter(|&x| x == 'B').count() as i32)
            .sum::<i32>()
            .min(10),
        "target_power" => ctx.snapshot.map_or(2, |s| s.0),
        "target_toughness" => ctx.snapshot.map_or(2, |s| s.1),
        "target_mv" => ctx.snapshot.map_or(2, |s| s.2 as i32),
        "last_lost" => ctx.lost,
        "event_spell_mv" => ctx.spell.map_or(2, |c| g.db.get(c).cmc as i32),
        "source_power" => src.map_or(2, |m| epow(g, m)),
        "counters_on_self" => src.map_or(0, |m| g.perm(m).plus.max(0)),
        s if s.starts_with("type_you:") || s.starts_with("type_all:") => {
            let t = &s[9..];
            let qs: Vec<PlayerId> = if s.starts_with("type_you") {
                vec![p]
            } else {
                g.players.iter().filter(|q| q.alive).map(|q| q.id).collect()
            };
            qs.iter()
                .flat_map(|&q| g.player(q).perms.iter())
                .filter(|&&m| !g.perm(m).phased && has_type(g, m, t))
                .count() as i32
        }
        s if s.starts_with("target_") => 2,
        _ => 2,
    }
}

fn type_ok(g: &Game, m: PermId, t: &str) -> Option<bool> {
    let ty = g.perm(m).cd.map(|c| g.db.get(c).types);
    let has_t = |x: Types| ty.is_some_and(|t| t.has(x));
    let cre = g.is_creature(m);
    Some(match t {
        "creature" => cre,
        "artifact" => has_t(Types::ARTIFACT),
        "enchantment" => has_t(Types::ENCHANTMENT),
        "planeswalker" => has_t(Types::PLANESWALKER),
        "land" => false,
        "nonland" | "permanent" => true,
        "cp" => cre || has_t(Types::PLANESWALKER),
        "ac" => cre || has_t(Types::ARTIFACT),
        "acp" => cre || has_t(Types::ARTIFACT) || has_t(Types::PLANESWALKER),
        "ce" => cre || has_t(Types::ENCHANTMENT),
        "ae" => has_t(Types::ARTIFACT) || has_t(Types::ENCHANTMENT),
        "nonartifact_creature" => cre && !has_t(Types::ARTIFACT),
        "noncreature" => !cre,
        _ => return None,
    })
}

/// does permanent m satisfy filter f, from the point of view of player p (the ability's controller)?
pub fn matches(g: &Game, p: PlayerId, m: PermId, f: Option<&Filter>, src: Option<PermId>) -> bool {
    let x = g.perm(m);
    if x.phased {
        return false;
    }
    let Some(f) = f else { return true };
    if let Some(t) = &f.kind
        && type_ok(g, m, t) == Some(false)
    {
        return false;
    }
    match f.controller.as_deref() {
        Some("you") if x.owner != p => return false,
        Some("opp") if x.owner == p => return false,
        _ => {}
    }
    if f.other == Some(true) && src == Some(m) {
        return false;
    }
    let cd = x.cd.map(|c| g.db.get(c));
    if let (Some(mx), Some(d)) = (f.max_mv, cd)
        && d.cmc > mx
    {
        return false;
    }
    if let Some(mn) = f.min_mv
        && cd.is_none_or(|d| d.cmc < mn)
    {
        return false;
    }
    if f.max_pow.is_some_and(|v| epow(g, m) > v) || f.min_pow.is_some_and(|v| epow(g, m) < v) {
        return false;
    }
    if f.nontoken == Some(true) && x.token {
        return false;
    }
    if let Some(kw) = &f.keyword
        && !((kw == "flying" && x.fly) || has_kw(g, m, kw))
    {
        return false; // "creatures you control with flying" (Alela)
    }
    if f.nonlegendary == Some(true) && cd.is_some_and(|d| d.tag(Tag::Leg)) {
        return false;
    }
    if f.legendary == Some(true) && !cd.is_some_and(|d| d.tag(Tag::Leg)) {
        return false;
    }
    if let Some(st) = f.subtype.as_deref().filter(|s| !s.is_empty()) {
        let ok = match st {
            "warrior" => x.warrior || has_type(g, m, "warrior"),
            "commander" => x.is_cmd, // "commander creatures you control" (Bastion Protector)
            _ => has_type(g, m, st),
        };
        if !ok {
            return false;
        }
    }
    true
}

/// does card c (in a library, hand or graveyard) satisfy filter f?
pub fn card_matches(g: &Game, c: CardId, f: Option<&Filter>) -> bool {
    let d = g.db.get(c);
    let empty = Filter::default();
    let f = f.unwrap_or(&empty);
    let ty = |t: Types| d.types.has(t);
    let ok = match f.kind.as_deref() {
        None | Some("any") => true,
        Some("permanent") => d.perm || d.land,
        Some("creature") => d.creature,
        Some("land") => d.land && (f.basic != Some(true) || zones::BASIC_LANDS.contains(&&*d.name)),
        Some("artifact") => ty(Types::ARTIFACT),
        Some("enchantment") => ty(Types::ENCHANTMENT),
        Some("instant_or_sorcery") => d.instant || d.sorcery,
        Some("noncreature") => !d.creature && !d.land,
        Some("planeswalker") => ty(Types::PLANESWALKER),
        Some("nonland") | Some("spell") => !d.land,
        Some("cp") => d.creature || ty(Types::PLANESWALKER),
        Some("ac") => d.creature || ty(Types::ARTIFACT),
        Some("acp") => d.creature || ty(Types::ARTIFACT) || ty(Types::PLANESWALKER),
        Some("ce") => d.creature || ty(Types::ENCHANTMENT),
        Some("ae") => ty(Types::ARTIFACT) || ty(Types::ENCHANTMENT),
        Some("nonartifact_creature") => d.creature && !ty(Types::ARTIFACT),
        Some(_) => true,
    };
    if !ok {
        return false;
    }
    if f.max_mv.is_some_and(|v| d.cmc > v)
        || f.min_mv.is_some_and(|v| d.cmc < v)
        || f.max_pow.is_some_and(|v| d.pow > v)
    {
        return false;
    }
    !(f.nonlegendary == Some(true) && d.tag(Tag::Leg))
}

/// the players a 'who' names
pub fn players(g: &Game, p: PlayerId, w: Option<&str>, ctx: &DslCtx) -> Vec<PlayerId> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    match w.unwrap_or("you") {
        "you" => vec![p],
        "each_opponent" => opps,
        "each_player" => g.players.iter().filter(|q| q.alive).map(|q| q.id).collect(),
        "target_opponent" | "target_player" => {
            let ok: Vec<PlayerId> =
                opps.into_iter().filter(|&q| !crate::engine::values::player_hexproof(g, q)).collect();
            max_by(&ok, |q| threat(g, p, q)).into_iter().collect() // Shalai, Voice of Plenty
        }
        "target_controller" => ctx.target_owner.filter(|&t| g.player(t).alive).into_iter().collect(),
        "event_player" | "defending_player" => ctx.event_player.filter(|&t| g.player(t).alive).into_iter().collect(),
        _ => vec![p],
    }
}

fn src_colors(g: &Game, src: Option<PermId>, spell: Option<CardId>) -> Colors {
    match (spell, src) {
        (Some(s), _) => Colors::from_letters(&g.db.get(s).pips),
        (None, Some(m)) => colors_of(g, m),
        _ => Colors::NONE,
    }
}

/// pick the most sensible legal target (or reuse the one chosen as the spell was cast)
pub fn choose_target(
    g: &Game,
    p: PlayerId,
    sel: &Sel,
    harmful: bool,
    ctx: &DslCtx,
    src: Option<PermId>,
    spell: Option<CardId>,
) -> Option<PermId> {
    let f = sel.filter.as_ref();
    if let Some(pre) = ctx.target
        && g.perm(pre).on_bf
        && matches(g, p, pre, f, src)
    {
        return Some(pre);
    }
    // HUMAN(phase 9): a person picks their own targets (_ask_target)
    let cols = src_colors(g, src, spell);
    let cands: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| {
            matches(g, p, m, f, src) && !(g.perm(m).owner != p && (untargetable(g, m) || protected_from(g, m, cols)))
        })
        .collect();
    if harmful {
        let opp: Vec<PermId> = cands.iter().copied().filter(|&m| g.perm(m).owner != p).collect();
        if !opp.is_empty() {
            return max_by(&opp, |m| pval(g, m));
        }
        if f.and_then(|f| f.controller.as_deref()) != Some("you") {
            return None;
        }
        return min_by(&cands, |m| pval(g, m)); // a cost on your own: the least valuable
    }
    let mine: Vec<PermId> = cands.iter().copied().filter(|&m| g.perm(m).owner == p).collect();
    if !mine.is_empty() { max_by(&mine, |m| pval(g, m)) } else { max_by(&cands, |m| pval(g, m)) }
}

pub fn select(
    g: &Game,
    p: PlayerId,
    sel: &Sel,
    harmful: bool,
    ctx: &DslCtx,
    src: Option<PermId>,
    spell: Option<CardId>,
) -> Vec<PermId> {
    match sel.sel.as_deref() {
        Some("self") => src.filter(|&m| g.perm(m).on_bf).into_iter().collect(),
        Some("all") => g
            .players
            .iter()
            .filter(|q| q.alive)
            .flat_map(|q| q.perms.iter().copied())
            .filter(|&m| matches(g, p, m, sel.filter.as_ref(), src))
            .collect(),
        Some("event") => ctx.event_perm.into_iter().collect(),
        _ => choose_target(g, p, sel, harmful, ctx, src, spell).into_iter().collect(),
    }
}

fn snapshot(g: &Game, m: PermId, ctx: &mut DslCtx) {
    let x = g.perm(m);
    ctx.snapshot = Some((epow(g, m), x.tgh + x.plus, x.cd.map_or(0, |c| g.db.get(c).cmc)));
    ctx.target_owner = Some(x.owner);
}

// ------------------------------------------------------------------ effects
pub fn execute(
    g: &mut Game,
    p: PlayerId,
    effects: &[Effect],
    src: Option<PermId>,
    ctx: &mut DslCtx,
    spell: Option<CardId>,
    depth: u32,
) -> Res {
    for e in effects {
        if g.over || !g.player(p).alive {
            return Ok(());
        }
        run(g, p, e, src, ctx, spell, depth)?;
    }
    check_state(g)
}

fn sel_or_empty(s: &Option<Sel>) -> Sel {
    s.clone().unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
fn run(
    g: &mut Game,
    p: PlayerId,
    e: &Effect,
    src: Option<PermId>,
    ctx: &mut DslCtx,
    spell: Option<CardId>,
    depth: u32,
) -> Res {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let kind: Sym = if spell.is_some() { "burn" } else { "triggers" };
    let n = |g: &Game, ctx: &DslCtx| num(g, p, e.n.as_ref(), ctx, src);
    match e.kind.as_str() {
        "scry" | "surveil" => {
            let k = match &e.n {
                Some(Num::Int(k)) => *k,
                _ => 1,
            };
            crate::cardcode::scry(g, p, k, e.kind == "surveil")?;
        }
        "draw" => {
            for q in players(g, p, Some(e.who.as_deref().unwrap_or("you")), ctx) {
                // "you may draw": the AI doesn't deck itself (HUMAN(phase 9): a person chooses)
                if e.optional == Some(true) && g.player(q).library.len() <= 12 {
                    continue;
                }
                let k = n(g, ctx);
                zones::draw(g, q, k.max(0) as u32, false)?;
            }
        }
        "damage" => {
            let dmg = n(g, ctx) + has(g, p, Tag::Thor) as i32;
            let to = match &e.to {
                Some(To::Sel(s)) => (**s).clone(),
                _ => Sel::default(),
            };
            match to.sel.as_deref() {
                Some("any_target") => {
                    // HUMAN(phase 9): the person's target (hc.deal_damage)
                    let cols = src_colors(g, src, spell);
                    let tg: Vec<PermId> = opps
                        .iter()
                        .flat_map(|&q| g.player(q).perms.iter().copied())
                        .filter(|&m| {
                            g.is_creature(m) && !untargetable(g, m) && etgh(g, m) <= dmg && !protected_from(g, m, cols)
                        })
                        .collect();
                    let best = max_by(&tg, |m| pval(g, m));
                    let lethal: Vec<PlayerId> = opps.iter().copied().filter(|&q| g.player(q).life <= dmg).collect();
                    if let Some(&q) = lethal.first() {
                        lose_life(g, q, dmg, Some(p), kind, None)?;
                    } else if let Some(b) = best.filter(|&b| pval(g, b) >= 3.0) {
                        snapshot(g, b, ctx);
                        apply_removal(g, Some(p), b, &format!("dmg{dmg}"), spell)?;
                    } else if let Some(q) = min_by(&opps, |q| g.player(q).life as f64) {
                        lose_life(g, q, dmg, Some(p), kind, None)?;
                    }
                }
                Some("player") => {
                    for q in players(g, p, Some(to.who.as_deref().unwrap_or("each_opponent")), ctx) {
                        if q != p {
                            lose_life(g, q, dmg, Some(p), kind, None)?;
                        }
                    }
                }
                _ => {
                    for m in select(g, p, &to, true, ctx, src, spell) {
                        if to.sel.as_deref() == Some("target") {
                            snapshot(g, m, ctx);
                        }
                        if etgh(g, m) <= dmg {
                            if to.sel.as_deref() == Some("target") {
                                apply_removal(g, Some(p), m, &format!("dmg{dmg}"), spell)?;
                            } else {
                                zones::die(g, m, "destroy")?;
                            }
                        }
                    }
                }
            }
        }
        "destroy" | "exile" | "bounce" | "tuck" => {
            let sel = sel_or_empty(&e.what);
            let ms = select(g, p, &sel, true, ctx, src, spell);
            if sel.sel.as_deref() == Some("all") {
                let wipe_kind: Sym = match e.kind.as_str() {
                    "exile" => "exile",
                    "bounce" => "evac",
                    _ => "destroy",
                };
                let mut hit: Vec<PlayerId> = ms.iter().map(|&m| g.perm(m).owner).collect();
                hit.dedup();
                let mut prot = vec![];
                for q in hit {
                    prot.push((q, crate::ai::wipe_response(g, q, wipe_kind, p)?));
                }
                g.batch += 1; // they die at the same time
                let prev = g.destroyer.replace(p); // Karmic Justice: who destroyed them
                let r = (|| -> Res {
                    for m in ms {
                        let x = g.perm(m);
                        if !x.on_bf || x.phased {
                            continue;
                        }
                        let pr = prot.iter().find(|y| y.0 == x.owner).and_then(|y| y.1);
                        if pr == Some("indes") && e.kind == "destroy" {
                            continue;
                        }
                        match e.kind.as_str() {
                            "destroy" => zones::die(g, m, "destroy")?,
                            "exile" => zones::exile_perm(g, m)?,
                            "bounce" => {
                                if g.perm(m).token {
                                    zones::leave(g, m)?
                                } else {
                                    zones::bounce(g, m)?
                                }
                            }
                            _ => zones::tuck(g, m)?,
                        }
                    }
                    Ok(())
                })();
                g.destroyer = prev;
                r?;
            } else {
                for m in ms {
                    snapshot(g, m, ctx);
                    apply_removal(g, Some(p), m, &e.kind, spell)?;
                }
            }
        }
        "token" => {
            let k = n(g, ctx);
            let pw = num(g, p, e.pow.as_ref().or(Some(&Num::Int(1))), ctx, src);
            let tg = match &e.tgh {
                Some(t) => num(g, p, Some(t), ctx, src),
                None => pw,
            };
            let kws = &e.keywords;
            let attacking = e.attacking == Some(true);
            let spec = Tokens {
                tgh: Some(tg),
                fly: kws.iter().any(|k| k == "flying"),
                dt: kws.iter().any(|k| k == "deathtouch"),
                lifelink: kws.iter().any(|k| k == "lifelink"),
                warrior: e.warrior == Some(true),
                attacking,
                sick: !(kws.iter().any(|k| k == "haste") || attacking),
                types: e.types.iter().flatten().map(|t| intern(t)).collect(),
                ..Tokens::new(k.max(0) as u32, pw)
            };
            let made = zones::make_tokens(g, p, spec)?;
            if attacking && ctx.attack {
                g.new_attackers.extend(made);
            }
        }
        "treasure" => {
            let k = n(g, ctx);
            zones::add_treasure(g, p, k)?;
        }
        "clue" => {
            let k = n(g, ctx);
            g.player_mut(p).clues += k.max(0) as u32;
        }
        "gain_life" => {
            let mut k = n(g, ctx) * e.mult.unwrap_or(1);
            if e.per_opponent == Some(true) {
                k *= opps.len().max(1) as i32;
            }
            for q in players(g, p, Some(e.who.as_deref().unwrap_or("you")), ctx) {
                gain(g, q, k)?;
            }
        }
        "lose_life" => {
            let k = n(g, ctx);
            for q in players(g, p, Some(e.who.as_deref().unwrap_or("each_opponent")), ctx) {
                let before = g.player(q).life;
                lose_life(g, q, k, Some(if q != p { p } else { q }), "drain", None)?;
                if q != p {
                    ctx.lost += before - g.player(q).life;
                }
            }
        }
        "counters" => {
            let k = n(g, ctx) * counter_mult(g, p);
            let sel = e.what.clone().unwrap_or(Sel { sel: Some("self".into()), ..Sel::default() });
            for m in select(g, p, &sel, false, ctx, src, spell) {
                g.perm_mut(m).plus += k;
            }
        }
        "pump" => {
            let dp = num(g, p, e.pow.as_ref(), ctx, src);
            let dt = num(g, p, e.tgh.as_ref(), ctx, src);
            for m in select(g, p, &sel_or_empty(&e.what), dp < 0, ctx, src, spell) {
                let x = g.perm_mut(m);
                x.eot_pt = (x.eot_pt.0 + dp, x.eot_pt.1 + dt);
            }
            if dt < 0 {
                for i in 0..g.players.len() {
                    for m in g.players[i].perms.clone() {
                        if g.is_creature(m) && etgh(g, m) <= 0 {
                            zones::die(g, m, "destroy")?;
                        }
                    }
                }
            }
        }
        "set_pt" => {
            let t = total_mana(g, p, false) as i32;
            let pl = g.player_mut(p);
            pl.pump = pl.pump.max(t);
        }
        "grant" => {
            let kw = intern(e.keyword.as_deref().unwrap_or(""));
            for m in select(g, p, &sel_or_empty(&e.what), false, ctx, src, spell) {
                let x = g.perm_mut(m);
                if !x.eot_kw.contains(&kw) {
                    x.eot_kw.push(kw);
                }
            }
        }
        "search" => search(g, p, e)?,
        "reanimate" => {
            let f = e.filter.clone().unwrap_or(Filter { kind: Some("creature".into()), ..Filter::default() });
            let mut pool: Vec<(CardId, PlayerId)> =
                g.player(p).gy.iter().copied().filter(|&c| card_matches(g, c, Some(&f))).map(|c| (c, p)).collect();
            if e.from.as_deref() == Some("any") {
                for q in g.opps(p).collect::<Vec<_>>() {
                    pool.extend(
                        g.player(q).gy.iter().copied().filter(|&c| card_matches(g, c, Some(&f))).map(|c| (c, q)),
                    );
                }
            }
            if let Some(&(c, q)) = pool.iter().max_by(|a, b| {
                let (x, y) = (g.db.get(a.0), g.db.get(b.0));
                (x.bomb, x.pow, x.cmc).cmp(&(y.bomb, y.pow, y.cmc))
            }) {
                let gy = &mut g.player_mut(q).gy;
                let i = gy.iter().position(|&x| x == c).unwrap();
                gy.remove(i);
                zones::enter(g, p, c, Enter { orig: Some(q), ..Enter::default() })?;
                crate::glog!(g, "    {} returns to the battlefield", g.db.get(c).name);
            }
        }
        "regrow" => {
            let cs: Vec<CardId> =
                g.player(p).gy.iter().copied().filter(|&c| card_matches(g, c, e.filter.as_ref())).collect();
            if let Some(&c) = cs.iter().max_by_key(|&&c| (g.db.get(c).bomb, g.db.get(c).cmc)) {
                let pl = g.player_mut(p);
                let i = pl.gy.iter().position(|&x| x == c).unwrap();
                pl.gy.remove(i);
                pl.hand.push(c);
            }
        }
        "discard" => {
            for q in players(g, p, Some(e.who.as_deref().unwrap_or("each_opponent")), ctx) {
                let k = n(g, ctx);
                if q == p && e.random != Some(true) {
                    crate::engine::cast::discard_worst(g, q, k.max(0) as u32)?;
                } else {
                    for _ in 0..k {
                        let h = g.player(q).hand.len() as u64;
                        if h > 0 {
                            let i = g.rng.below(h) as usize;
                            zones::discard_index(g, q, i)?;
                        }
                    }
                }
            }
        }
        "mill" => {
            for q in players(g, p, Some(e.who.as_deref().unwrap_or("you")), ctx) {
                let k = n(g, ctx);
                zones::mill(g, q, k.max(0) as usize)?;
            }
        }
        "sacrifice" => {
            let mut f = e.filter.clone().unwrap_or(Filter { kind: Some("creature".into()), ..Filter::default() });
            f.controller = None;
            for q in players(g, p, Some(e.who.as_deref().unwrap_or("each_opponent")), ctx) {
                for _ in 0..num(g, p, Some(e.n.as_ref().unwrap_or(&Num::Int(1))), ctx, src) {
                    let cs: Vec<PermId> = g
                        .player(q)
                        .perms
                        .iter()
                        .copied()
                        .filter(|&m| matches(g, q, m, Some(&f), None) && !crate::engine::values::no_sac(g, m))
                        .collect();
                    if let Some(m) = min_by(&cs, |m| pval(g, m)) {
                        zones::die(g, m, "sac")?;
                    }
                }
            }
        }
        "add_mana" => {
            let k = n(g, ctx);
            g.player_mut(p).floating.any += k.max(0) as u32;
        }
        "extra_combat" => g.player_mut(p).extra_combats += 1,
        "proliferate" => {
            for m in g.player(p).perms.clone() {
                if g.perm(m).plus > 0 {
                    g.perm_mut(m).plus += 1;
                }
            }
        }
        "double_counters" => {
            for m in g.player(p).perms.clone() {
                if g.perm(m).plus > 0 {
                    g.perm_mut(m).plus *= 2;
                }
            }
        }
        "amass" => {
            let k = n(g, ctx);
            zones::amass(g, p, k)?;
        }
        "untap" => {
            let sel = sel_or_empty(&e.what);
            if sel.filter.as_ref().and_then(|f| f.kind.as_deref()) == Some("land") {
                for l in g.player(p).lands.clone() {
                    g.land_mut(l).tapped = false;
                }
            } else {
                for m in select(g, p, &sel, false, ctx, src, spell) {
                    g.perm_mut(m).tapped = false;
                }
            }
        }
        "tap" => {
            for m in select(g, p, &sel_or_empty(&e.what), true, ctx, src, spell) {
                g.perm_mut(m).tapped = true;
            }
        }
        "phase_out" => {
            for m in select(g, p, &sel_or_empty(&e.what), false, ctx, src, spell) {
                g.perm_mut(m).phased = true;
            }
        }
        "blink" => {
            let sel = sel_or_empty(&e.what);
            let ms: Vec<PermId> = if sel.upto == Some(true) || sel.sel.as_deref() == Some("target") {
                // only blink a nontoken creature that gets something from entering again
                // (HUMAN(phase 9): a person picks any of theirs, or none)
                let cmd = g.player(p).cmd;
                let cands: Vec<PermId> = g
                    .player(p)
                    .perms
                    .iter()
                    .copied()
                    .filter(|&m| {
                        let x = g.perm(m);
                        g.is_creature(m)
                            && x.cd.is_some_and(|c| c != cmd && etb_value(g, c) > 0)
                            && !x.phased
                            && matches(g, p, m, sel.filter.as_ref(), src)
                    })
                    .collect();
                max_by(&cands, |m| etb_value(g, g.perm(m).cd.unwrap()) as f64).into_iter().collect()
            } else {
                select(g, p, &sel, false, ctx, src, spell)
            };
            for m in ms {
                let x = g.perm(m);
                if let Some(cd) = x.cd
                    && x.owner == p
                    && !x.token
                {
                    zones::leave(g, m)?;
                    zones::enter(g, p, cd, Enter::default())?;
                    crate::glog!(g, "    {} is blinked", g.db.get(cd).name);
                }
            }
        }
        "opp_ramp" => {
            if let Some(q) = ctx.target_owner.filter(|&q| g.player(q).alive) {
                zones::land_ramp(g, q, 1, true)?;
            }
        }
        "opp_token" => {
            if let Some(q) = ctx.target_owner.filter(|&q| g.player(q).alive) {
                let pw = num(g, p, e.pow.as_ref().or(Some(&Num::Int(3))), ctx, src);
                zones::make_tokens(g, q, Tokens::new(1, pw))?;
            }
        }
        "opp_treasure" => {
            if let Some(q) = ctx.target_owner.filter(|&q| g.player(q).alive) {
                let k = n(g, ctx);
                zones::add_treasure(g, q, k)?;
            }
        }
        "exile_graveyard" => {
            for q in players(g, p, Some(e.who.as_deref().unwrap_or("each_player")), ctx) {
                let gy = std::mem::take(&mut g.player_mut(q).gy);
                g.player_mut(q).exile.extend(gy);
            }
        }
        "wheel" => {
            let qs = players(g, p, Some(e.who.as_deref().unwrap_or("each_player")), ctx);
            let k = qs.iter().map(|&q| g.player(q).hand.len()).max().unwrap_or(0);
            for &q in &qs {
                let hand = g.player(q).hand.clone();
                zones::discard_cards(g, q, &hand)?;
            }
            for &q in &qs {
                zones::draw(g, q, k as u32, false)?;
            }
        }
        "modal" => {
            let k = (e.choose.unwrap_or(1) as usize).min(e.modes.len());
            // HUMAN(phase 9): a person picks the modes (hc.pick_modes)
            let mut order: Vec<(usize, f64)> =
                e.modes.iter().enumerate().map(|(i, ms)| (i, value_of(g, p, ms, spell))).collect();
            order.sort_by(|a, b| b.1.total_cmp(&a.1)); // stable, best first
            for &(i, _) in order.iter().take(k) {
                execute(g, p, &e.modes[i], src, ctx, spell, depth + 1)?;
            }
        }
        "counter_spell" => {} // counterspells are cast through the engine's response window
        "oracle" => {
            // Thassa's Oracle
            let x: usize = g
                .player(p)
                .perms
                .iter()
                .filter(|&&m| !g.perm(m).phased)
                .filter_map(|&m| g.perm(m).cd)
                .filter(|&c| g.db.get(c).perm)
                .map(|c| g.db.get(c).pips.chars().filter(|&ch| ch == 'U').count())
                .sum();
            if x >= g.player(p).library.len() {
                return crate::ai::win(g, p, "combo", Some(false)); // an alternate win
            }
            let k = x.min(g.player(p).library.len());
            let mut top: Vec<CardId> = (0..k).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
            if let Some(best) = max_by(&top, |c| crate::engine::tutors::card_worth(g, p, c, false)) {
                let i = top.iter().position(|&c| c == best).unwrap();
                top.remove(i);
                g.rng.shuffle(&mut top);
                let pl = g.player_mut(p);
                pl.library.splice(0..0, top);
                pl.library.push(best);
            }
        }
        "narset_dig" => {
            // Narset -2: the top four, take a noncreature nonland card, the rest on the bottom
            let k = g.player(p).library.len().min(4);
            let mut top: Vec<CardId> = (0..k).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
            let ok: Vec<CardId> = top.iter().copied().filter(|&c| !g.db.get(c).creature && !g.db.get(c).land).collect();
            if let Some(c) = max_by(&ok, |c| crate::engine::tutors::card_worth(g, p, c, false)) {
                let i = top.iter().position(|&x| x == c).unwrap();
                top.remove(i);
                g.player_mut(p).hand.push(c);
                g.player_mut(p).seen.insert(c);
            }
            g.rng.shuffle(&mut top);
            g.player_mut(p).library.splice(0..0, top);
        }
        "put_land" => {
            // put a land card from your hand onto the battlefield
            let ls: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
            if let Some(c) = max_by(&ls, |c| {
                let t = &g.db.get(c).tags;
                t.str(Tag::C).map_or(0, |s| s.chars().count()) as f64 * 10.0 + t.has(Tag::F) as i32 as f64
            }) {
                let pl = g.player_mut(p);
                let i = pl.hand.iter().position(|&x| x == c).unwrap();
                pl.hand.remove(i);
                let tapped = crate::engine::turn::land_enters_tapped(g, p, c);
                let l = g.add_land(p, c, tapped);
                zones::landfall(g, p)?;
                if g.db.get(c).tag(Tag::F) && g.player(p).lands.last() == Some(&l) {
                    crate::engine::turn::crack_fetch(g, p, l)?;
                }
            }
        }
        "ring_protection" => {
            g.player_mut(p).ring_prot = true; // The One Ring: protection from everything until your next turn
            crate::glog!(g, "    {} gains protection from everything until their next turn", g.player(p).name);
        }
        _ => {}
    }
    Ok(())
}

/// search effects: a land to the battlefield or hand, or a card matching the filter
fn search(g: &mut Game, p: PlayerId, e: &Effect) -> Res {
    let f = e.filter.clone().unwrap_or_default();
    let to = match &e.to {
        Some(To::Zone(z)) => z.as_str(),
        _ => "hand",
    };
    let k = match &e.n {
        Some(Num::Int(k)) => *k,
        _ => 1,
    };
    let tapped = e.tapped == Some(true);
    for _ in 0..k {
        if f.kind.as_deref() == Some("land") && to == "battlefield" {
            zones::land_ramp(g, p, 1, tapped)?;
            continue;
        }
        if f.kind.as_deref() == Some("land") {
            zones::land_to_hand(g, p)?;
            continue;
        }
        let cands: Vec<CardId> =
            zones::searchable(g, p).into_iter().filter(|&c| card_matches(g, c, Some(&f))).collect();
        if cands.is_empty() {
            return Ok(());
        }
        // HUMAN(phase 9): a person searches (hc.search)
        let pick = crate::ai::dsl_search_pick(g, p, &f, &cands);
        let c =
            pick.unwrap_or_else(|| max_by(&cands, |c| card_value(g, p, c) * 1000.0 + g.db.get(c).cmc as f64).unwrap());
        let lib = &mut g.player_mut(p).library;
        if let Some(i) = lib.iter().position(|&x| x == c) {
            lib.remove(i);
        }
        if let Some(a) = zones::agent_for(g, p) {
            zones::agent_take(g, a, p, c); // Opposition Agent
            continue;
        }
        match to {
            "battlefield" => {
                if g.db.get(c).land {
                    g.add_land(p, c, tapped);
                } else {
                    zones::enter(g, p, c, Enter::default())?;
                }
            }
            "graveyard" => g.player_mut(p).gy.push(c),
            "top" => {
                crate::engine::tutors::shuffle_library(g, p);
                g.player_mut(p).library.push(c);
            }
            _ => {
                g.player_mut(p).hand.push(c);
                g.player_mut(p).seen.insert(c);
            }
        }
        g.player_mut(p).stat("tutored", 1);
        crate::glog!(g, "    {} searches for {}", g.player(p).name, g.db.get(c).name); // PORT(phase 9): log_secret
    }
    Ok(())
}

/// an instant or sorcery with compiled abilities resolves
pub fn resolve_spell(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res {
    let mut dctx = DslCtx { x: ctx.x, target: ctx.target, ..DslCtx::default() };
    if dctx.x == 0 {
        dctx.x = 3; // Python's ctx.get('x', 3) when no X was chosen
    }
    for a in abilities(g, c).iter() {
        if let Ability::Spell { effects } = a {
            execute(g, p, effects, None, &mut dctx, Some(c), 0)?;
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ triggers
/// What happened, for the ability language's triggers (Python's keyword arguments to `fire`).
#[derive(Debug, Clone, Default)]
pub struct Fired {
    pub perm: Option<PermId>,
    pub owner: Option<PlayerId>,
    pub dying: Option<PermId>,
    pub was_cast: bool,
    pub player: Option<PlayerId>,
    pub caster: Option<PlayerId>,
    pub spell: Option<CardId>,
    pub attackers: Vec<PermId>,
    pub defender: Option<PlayerId>,
    pub attacker: Option<PermId>,
}

/// An event happened: every compiled triggered ability that cares puts a trigger on the queue. Python's `dsl.fire`.
pub fn fire(g: &mut Game, event: &str, kw: Fired) -> Res {
    if !active(g) || g.over || g.dsl_depth > MAX_DEPTH {
        return Ok(());
    }
    g.dsl_depth += 1;
    let r = fire_inner(g, event, &kw);
    g.dsl_depth -= 1;
    r
}

fn fire_inner(g: &mut Game, event: &str, kw: &Fired) -> Res {
    for i in 0..g.players.len() {
        let q = PlayerId(i as u8);
        if !g.player(q).alive {
            continue;
        }
        let mut srcs = g.player(q).perms.clone();
        if let Some(x) = kw.dying
            && g.perm(x).owner == q
            && !srcs.contains(&x)
        {
            srcs.push(x); // "when ~ dies"
        }
        for src in srcs {
            let x = g.perm(src);
            let Some(cd) = x.cd else { continue };
            if !g.db.get(cd).has_dsl() || (x.phased && Some(src) != kw.dying) {
                continue;
            }
            for (idx, a) in abilities(g, cd).iter().enumerate() {
                let Ability::Triggered { event: ev, source, .. } = a else { continue };
                if ev != event {
                    continue;
                }
                if Some(src) == kw.dying && !g.perm(src).on_bf && source.as_deref() != Some("self") {
                    continue;
                }
                let reps = trigger_copies(g, q, src, event, kw);
                let ctxs = trigger_matches(g, q, src, a, kw);
                let mut entries = vec![];
                for ctx in ctxs {
                    for _ in 0..reps {
                        entries.push(Trigger {
                            controller: q,
                            src: Some(src),
                            act: TrigAct::Dsl { p: q, src, card: cd, idx: idx as u16, ctx: Box::new(ctx.clone()) },
                            name: Some("trigger"),
                            known: true,
                            imp: None,
                            cast_etb: false,
                        });
                    }
                }
                crate::engine::stack::queue_triggers(g, entries)?; // on the stack (or once the spell resolving is done)
                if g.over {
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

/// a compiled trigger resolves: its effects
pub fn run_trigger(g: &mut Game, p: PlayerId, src: PermId, card: CardId, idx: u16, ctx: &DslCtx) -> Res {
    let ab = abilities(g, card);
    let effects = match &ab[idx as usize] {
        Ability::Triggered { effects, .. } => effects,
        _ => return Ok(()),
    };
    let mut ctx = ctx.clone();
    execute(g, p, effects, Some(src), &mut ctx, None, 0)
}

/// Veyran: your instant/sorcery casts make your permanents' triggers fire an extra time. Harmonic Prodigy: triggers
/// of your Shamans and other Wizards fire an extra time. Teysa and Panharmonicon through card code.
fn trigger_copies(g: &Game, q: PlayerId, src: PermId, event: &str, kw: &Fired) -> i32 {
    let mut n = 1;
    if event == "cast"
        && kw.caster == Some(q)
        && kw.spell.is_some_and(|c| g.db.get(c).instant || g.db.get(c).sorcery)
        && has(g, q, Tag::Veyran)
    {
        n += 1;
    }
    let t = g.perm(src).cd.map(|c| &g.db.get(c).tags);
    if t.is_some_and(|t| (t.has(Tag::Shaman) || t.has(Tag::Wizard)) && !t.has(Tag::Prodigy)) && has(g, q, Tag::Prodigy)
    {
        n += 1;
    }
    if event == "dies" {
        n += crate::engine::hooks::total_trigger_copies(g, q, "dies", kw.perm);
    }
    if event == "etb" {
        n += crate::engine::hooks::total_trigger_copies(g, q, "etb", kw.perm);
    }
    n
}

/// one ctx per time the ability triggers
fn trigger_matches(g: &mut Game, q: PlayerId, src: PermId, a: &Ability, kw: &Fired) -> Vec<DslCtx> {
    let Ability::Triggered { event, source, who, spell, nth, other, if_cast, min_pow, subtype, .. } = a else {
        return vec![];
    };
    let (s, w) = (source.as_deref(), who.as_deref());
    let base = DslCtx::base();
    let mut out = vec![];
    match event.as_str() {
        "etb" | "dies" => {
            let Some(m) = kw.perm else { return out };
            let owner = kw.owner.unwrap_or(g.perm(m).owner);
            if s == Some("self") {
                if if_cast == &Some(true) && !kw.was_cast {
                    return out; // "when ~ enters, if you cast it"
                }
                if m == src {
                    out.push(DslCtx { event_perm: Some(m), ..base });
                }
                return out;
            }
            if !g.is_creature(m) || (other == &Some(true) && m == src) || min_pow.is_some_and(|v| epow(g, m) < v) {
                return out;
            }
            match s {
                Some("you_creature") if owner == q => out.push(DslCtx { event_perm: Some(m), ..base }),
                Some("opp_creature") if owner != q => {
                    out.push(DslCtx { event_perm: Some(m), event_player: Some(owner), ..base })
                }
                Some("any_creature") => out.push(DslCtx { event_perm: Some(m), ..base }),
                _ => {}
            }
        }
        "attack" => {
            let ctx = DslCtx { event_player: kw.defender, attack: true, ..base };
            match s {
                Some("self") if kw.attackers.contains(&src) => out.push(ctx),
                Some("you_any") if kw.player == Some(q) && !kw.attackers.is_empty() => out.push(ctx),
                Some("you_each") if kw.player == Some(q) => {
                    for &m in &kw.attackers {
                        if subtype.as_deref() == Some("warrior") && !g.perm(m).warrior {
                            continue;
                        }
                        out.push(DslCtx { event_perm: Some(m), ..ctx.clone() });
                    }
                }
                Some("equipped") if kw.attackers.iter().any(|&m| g.perm(src).attached == Some(m)) => out.push(ctx),
                _ => {}
            }
        }
        "combat_damage" => {
            let m = kw.attacker;
            let ctx = DslCtx { event_player: kw.defender, event_perm: m, ..base };
            match s {
                Some("self") if m == Some(src) => out.push(ctx),
                Some("you_creature") if m.is_some_and(|m| g.perm(m).owner == q) => {
                    let key = intern(&format!("cd{}{:p}{}", src.0, a as *const Ability, g.round));
                    if crate::engine::values::once_per_turn(g, q, key) {
                        out.push(ctx);
                    }
                }
                Some("equipped") if m.is_some() && g.perm(src).attached == m => out.push(ctx),
                _ => {}
            }
        }
        "cast" => {
            let (Some(caster), Some(c)) = (kw.caster, kw.spell) else { return out };
            if (w == Some("you") && caster != q) || (w == Some("opponent") && caster == q) {
                return out;
            }
            let d = g.db.get(c);
            let ok = match spell.as_deref().unwrap_or("any") {
                "instant_or_sorcery" => d.instant || d.sorcery,
                "noncreature" => !d.creature,
                "creature" => d.creature,
                "artifact" => d.types.has(Types::ARTIFACT),
                "enchantment" => d.types.has(Types::ENCHANTMENT),
                "legendary" => d.tag(Tag::Leg),
                _ => true,
            };
            if !ok || nth.is_some_and(|k| g.player(caster).spells_this_turn != k) {
                return out;
            }
            out.push(DslCtx { spell: Some(c), event_player: Some(caster), ..base });
        }
        "draw" | "upkeep" | "end_step" => {
            let pl = kw.player;
            match w {
                Some("you") if pl == Some(q) => out.push(base),
                Some("opponent") if pl.is_some() && pl != Some(q) => out.push(DslCtx { event_player: pl, ..base }),
                Some("any") if event != "draw" => out.push(DslCtx { event_player: pl, ..base }),
                _ => {}
            }
        }
        "landfall" if kw.player == Some(q) => out.push(base),
        _ => {}
    }
    out
}

// ------------------------------------------------------------------ static abilities
/// (controller, source, ability) for every static or replacement ability of this kind on the battlefield
fn statics<'a>(g: &'a Game, kind: &'a str) -> impl Iterator<Item = (PlayerId, PermId, Arc<[Ability]>, usize)> + 'a {
    g.players.iter().filter(|q| q.alive).flat_map(move |q| {
        q.perms
            .iter()
            .filter_map(move |&src| {
                let x = g.perm(src);
                let cd = x.cd?;
                if x.phased || x.owner != q.id {
                    return None;
                }
                let ab = &g.db.get(cd).abilities;
                if ab.is_empty() {
                    return None;
                }
                let idx: Vec<usize> =
                    ab.iter().enumerate().filter(|(_, a)| a.static_kind() == Some(kind)).map(|(i, _)| i).collect();
                if idx.is_empty() { None } else { Some((q.id, src, ab.clone(), idx)) }
            })
            .flat_map(|(q, src, ab, idx)| idx.into_iter().map(move |i| (q, src, ab.clone(), i)))
    })
}

/// Power and toughness changes beyond printed stats and counters: until-end-of-turn pumps, Auras (card code), then
/// the ability language's anthems, equipment bonuses and self-scaling creatures. Python's `dsl.pt`.
pub fn pt(g: &Game, m: PermId) -> (i32, i32) {
    let x = g.perm(m);
    let (mut dp, mut dt) = x.eot_pt;
    if !g.auras.is_empty() || g.selfpt || g.player(x.owner).elspeth_emblem {
        let (a, b) = crate::cardcode::attached_bonus(g, m);
        dp += a;
        dt += b;
    }
    if !active(g) {
        return (dp, dt);
    }
    let cre = g.is_creature(m);
    for (q, src, ab, i) in statics(g, "anthem") {
        if let Ability::Static { pow, tgh, filter, .. } = &ab[i]
            && cre
            && matches(g, q, m, filter.as_ref(), Some(src))
        {
            dp += pow.unwrap_or(0);
            dt += tgh.unwrap_or(0);
        }
    }
    for (_, src, ab, i) in statics(g, "equip_bonus") {
        if let Ability::Static { pow, tgh, .. } = &ab[i]
            && g.perm(src).attached == Some(m)
        {
            dp += pow.unwrap_or(0);
            dt += tgh.unwrap_or(0);
        }
    }
    for (q, src, _, _) in statics(g, "bma_anthem") {
        if g.perm(src).plus >= 7 && cre && x.owner == q {
            dp += 5; // Beastmaster Ascension with 7+ quest counters
            dt += 5;
        }
    }
    if let Some(cd) = x.cd {
        for a in g.db.get(cd).abilities.iter() {
            if let Ability::Static { kind, per, pow, base0, .. } = a
                && kind == "self_scaling"
            {
                let k = num(g, x.owner, per.as_ref(), &DslCtx::default(), Some(m));
                dp += pow.unwrap_or(1) * k;
                if *base0 == Some(true) {
                    dt += k;
                }
            }
        }
    }
    (dp, dt)
}

/// Does m have keyword kw (lower case)? Python's `dsl.has_kw`.
pub fn has_kw(g: &Game, m: PermId, kw: &str) -> bool {
    let x = g.perm(m);
    if x.eot_kw.contains(&kw) {
        return true;
    }
    if kw == "flying" && g.is_creature(m) && g.player(x.owner).elspeth_emblem {
        return true; // Elspeth's emblem
    }
    match x.cd {
        None => {
            if let Some(crate::state::Val::List(v)) = x.data.get(crate::state::DataKey::Kws)
                && v.iter().any(|k| matches!(k, crate::state::Val::Str(s) if *s == kw))
            {
                return true; // a token made with keywords (Samurai)
            }
        }
        Some(c) => {
            if !x.neutered && g.db.get(c).has_kw(kw) {
                return true;
            }
        }
    }
    if !g.auras.is_empty() && crate::cardcode::attached_kw(g, m, kw) {
        return true;
    }
    if !g.hooks.is_empty() && crate::cardcode::granted_kw(g, m, kw) {
        return true;
    }
    if !active(g) {
        return false;
    }
    if let Some(cd) = x.cd {
        for a in g.db.get(cd).abilities.iter() {
            if let Ability::Static { kind, keyword, .. } = a {
                match kind.as_str() {
                    "unblockable" if kw == "unblockable" => return true,
                    "cant_block" if kw == "cant_block" => return true,
                    "self_keyword" if keyword.as_deref() == Some(kw) => return true,
                    _ => {}
                }
            }
        }
    }
    let cre = g.is_creature(m);
    for (q, src, ab, i) in statics(g, "keyword") {
        if let Ability::Static { keyword, filter, .. } = &ab[i]
            && keyword.as_deref() == Some(kw)
            && cre
            && matches(g, q, m, filter.as_ref(), Some(src))
        {
            return true;
        }
    }
    for (_, src, ab, i) in statics(g, "equip_keyword") {
        if let Ability::Static { keyword, .. } = &ab[i]
            && keyword.as_deref() == Some(kw)
            && g.perm(src).attached == Some(m)
        {
            return true;
        }
    }
    false
}

const ETB_TAGS: [Tag; 19] = [
    Tag::Titan,
    Tag::Archon,
    Tag::Gray,
    Tag::Wurm,
    Tag::Rsd,
    Tag::Witness,
    Tag::Wall,
    Tag::Atraxa,
    Tag::Heir,
    Tag::Suntitan,
    Tag::Tokbig,
    Tag::Bowmasters,
    Tag::Skate,
    Tag::Mycoloth,
    Tag::Recruit,
    Tag::Prepare,
    Tag::Draw,
    Tag::Tok,
    Tag::Tut,
];

/// how much entering again is worth for card cd (blink choices)
pub fn etb_value(g: &Game, cd: CardId) -> i32 {
    let d = g.db.get(cd);
    let mut v = 2 * ETB_TAGS.iter().filter(|&&k| d.tag(k)).count() as i32;
    if d.tags.has(Tag::Rem) && d.tag(Tag::Etb) {
        v += 3;
    }
    for a in d.abilities.iter() {
        if let Ability::Triggered { event, source, .. } = a
            && event == "etb"
            && source.as_deref() == Some("self")
        {
            v += 3;
        }
    }
    v
}

/// colours m has protection from through auto-modeled equipment
pub fn protection(g: &Game, m: PermId) -> Colors {
    if !active(g) {
        return Colors::NONE;
    }
    let mut cols = Colors::NONE;
    for (_, src, ab, i) in statics(g, "equip_protection") {
        if let Ability::Static { colors, .. } = &ab[i]
            && g.perm(src).attached == Some(m)
        {
            cols = cols.union(Colors::from_letters(colors.as_deref().unwrap_or("")));
        }
    }
    cols
}

/// cost changes from static abilities (generic mana, + or -)
pub fn cost_delta(g: &Game, p: PlayerId, c: CardId) -> i32 {
    if !active(g) {
        return 0;
    }
    let d = g.db.get(c);
    let mut delta = 0;
    for (q, _, ab, i) in statics(g, "cost") {
        let Ability::Static { who, spell, amount, .. } = &ab[i] else { continue };
        let mine = who.as_deref() == Some("you") && q == p;
        let theirs = who.as_deref() == Some("opponents") && q != p;
        if !(mine || theirs) {
            continue;
        }
        let ok = match spell.as_deref().unwrap_or("any") {
            "instant_or_sorcery" => d.instant || d.sorcery,
            "noncreature" => !d.creature,
            "creature" => d.creature,
            "artifact" => d.types.has(Types::ARTIFACT),
            "enchantment" => d.types.has(Types::ENCHANTMENT),
            "legendary" => d.tag(Tag::Leg),
            _ => true,
        };
        if ok {
            delta += amount.unwrap_or(0);
        }
    }
    delta
}

/// token doublers
pub fn token_mult(g: &Game, p: PlayerId) -> u32 {
    if !active(g) {
        return 1;
    }
    let mut k = 1;
    for (q, _, ab, i) in statics(g, "tokens") {
        if let Ability::Replacement { multiplier, .. } = &ab[i]
            && q == p
        {
            k *= multiplier.unwrap_or(2).max(1) as u32;
        }
    }
    k
}

/// counter doublers
pub fn counter_mult(g: &Game, p: PlayerId) -> i32 {
    let mut k = 1;
    for (q, _, ab, i) in statics(g, "counters") {
        if let Ability::Replacement { multiplier, .. } = &ab[i]
            && q == p
        {
            k *= multiplier.unwrap_or(2);
        }
    }
    k
}

/// "players can't gain life" statics of other players
pub fn no_lifegain(g: &Game, p: PlayerId) -> bool {
    active(g) && statics(g, "no_lifegain").any(|(q, ..)| q != p)
}

// ------------------------------------------------------------------ the AI's view: values and ability use
fn eff_value(kind: &str) -> f64 {
    match kind {
        "narset_dig" | "draw" => 1.3,
        "treasure" => 0.8,
        "clue" => 0.6,
        "gain_life" => 0.15,
        "amass" => 0.7,
        "proliferate" => 1.0,
        "extra_combat" => 3.0,
        "add_mana" => 0.5,
        "mill" => 0.1,
        _ => 0.3,
    }
}

/// how much these effects are worth to p now
pub fn value_of(g: &Game, p: PlayerId, effects: &[Effect], spell: Option<CardId>) -> f64 {
    let mut v = 0.0;
    for e in effects {
        let n = match &e.n {
            Some(Num::Int(k)) => *k as f64,
            None => 1.0,
            _ => 2.0,
        };
        let d = e.kind.as_str();
        match d {
            "destroy" | "exile" | "bounce" | "tuck" | "damage" => {
                let sel = e.what.clone().or_else(|| match &e.to {
                    Some(To::Sel(s)) => Some((**s).clone()),
                    _ => None,
                });
                let sel = sel.unwrap_or_default();
                match sel.sel.as_deref() {
                    Some("all") => {
                        let ms = select(g, p, &sel, true, &DslCtx::default(), None, spell);
                        v +=
                            ms.iter().map(|&m| pval(g, m) * if g.perm(m).owner != p { 1.0 } else { -1.2 }).psum() / 2.0;
                    }
                    Some("player") | Some("any_target") if d == "damage" => {
                        v += 0.6
                            * n
                            * if sel.who.as_deref() == Some("each_opponent") { g.opps(p).count() as f64 } else { 1.0 };
                    }
                    _ => match choose_target(g, p, &sel, true, &DslCtx::default(), None, spell) {
                        None => v -= 1.0,
                        Some(t) if g.perm(t).owner == p => v -= pval(g, t) + 0.5, // it would hit your own permanent
                        Some(t) => v += pval(g, t) - 1.5,
                    },
                }
            }
            "token" => {
                let pw = match &e.pow {
                    Some(Num::Int(k)) => *k as f64,
                    _ => 1.0,
                };
                v += 0.7 * n * (pw + 0.5 * e.keywords.len() as f64);
            }
            "counters" => v += 0.4 * n,
            "pump" => v += 0.3,
            "lose_life" => {
                v += 0.5
                    * n
                    * match e.who.as_deref() {
                        Some("each_opponent") => g.opps(p).count() as f64,
                        Some("you") => -0.3,
                        _ => 1.0,
                    }
            }
            "search" | "reanimate" | "regrow" => v += 3.0,
            "sacrifice" => v += if e.who.as_deref() != Some("you") { 2.0 } else { -1.0 },
            "discard" => v += if e.who.as_deref() != Some("you") { 0.8 } else { -0.5 },
            "modal" => {
                let mut ms: Vec<f64> = e.modes.iter().map(|m| value_of(g, p, m, spell)).collect();
                ms.sort_by(|a, b| b.total_cmp(a));
                v += ms.iter().take(e.choose.unwrap_or(1) as usize).copied().psum();
            }
            _ => v += eff_value(d) * if matches!(d, "draw" | "treasure" | "clue" | "add_mana") { n } else { 1.0 },
        }
    }
    v
}

/// 0-9 estimate of how good casting c is now (used when no hand-written priority exists)
pub fn card_value(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let d = g.db.get(c);
    let mut v = 0.0;
    if d.creature {
        let kws = [Tag::Fly, Tag::Dt, Tag::Lifelink, Tag::Trample, Tag::Haste].iter().filter(|&&k| d.tag(k)).count();
        v += 1.0 + 0.45 * d.pow as f64 + 0.3 * kws as f64;
    }
    for a in d.abilities.iter() {
        v += match a {
            Ability::Spell { effects } => value_of(g, p, effects, Some(c)),
            Ability::Triggered { event, source, effects, .. } => {
                let per = value_of(g, p, effects, None);
                per * if event == "etb" && source.as_deref() == Some("self") { 1.0 } else { 2.2 }
            }
            Ability::Activated { effects, .. } => 0.8 * value_of(g, p, effects, None),
            Ability::Loyalty { effects, .. } => 0.6 * value_of(g, p, effects, None),
            Ability::Static { kind, .. } if kind != "note" => 2.0,
            Ability::Replacement { .. } => 2.0,
            _ => 0.0,
        };
    }
    v.clamp(0.5, 9.0)
}

fn pay_ability_cost(g: &mut Game, p: PlayerId, src: PermId, cost: &model::Cost, dry: bool) -> Res<bool> {
    let (gn, pips) = cost.mana.as_deref().map(crate::engine::cast::parse_cost).unwrap_or((0, String::new()));
    let x = g.perm(src);
    if cost.tap == Some(true) && (x.tapped || (g.is_creature(src) && x.sick)) {
        return Ok(false);
    }
    if cost.life.is_some_and(|l| g.player(p).life <= l + 5) {
        return Ok(false);
    }
    if cost.discard.is_some_and(|k| (g.player(p).hand.len() as u32) < k) {
        return Ok(false);
    }
    let mut fod = None;
    if let Some(sac) = cost.sac.as_deref().filter(|&s| s != "self") {
        let f = Filter { kind: type_ok(g, src, sac).map(|_| sac.to_string()), ..Filter::default() };
        let fods: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| m != src && matches(g, p, m, Some(&f), None) && (g.perm(m).token || pval(g, m) < 3.0))
            .collect();
        let Some(f0) = min_by(&fods, |m| pval(g, m)) else { return Ok(false) };
        fod = Some(f0);
    }
    if cost.other.is_some() || cost.tap_other.is_some() || cost.energy.is_some() {
        return Ok(false);
    }
    if !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    if dry {
        return Ok(true);
    }
    pay(g, p, gn, &pips, false)?;
    if cost.tap == Some(true) {
        g.perm_mut(src).tapped = true;
    }
    if let Some(l) = cost.life {
        lose_life(g, p, l, Some(p), "other", None)?;
    }
    if let Some(k) = cost.discard {
        crate::engine::cast::discard_worst(g, p, k)?;
    }
    if let Some(f) = fod {
        zones::die(g, f, "sac")?;
    }
    if cost.sac.as_deref() == Some("self") {
        zones::die(g, src, "sac")?;
    }
    Ok(true)
}

/// (round, player): activated abilities count per round per player, loyalty once per round (Python's (g.round, p.key))
fn round_stamp(g: &Game, p: PlayerId) -> crate::state::TurnStamp {
    crate::state::TurnStamp { round: g.round, active: Some(p) }
}

/// the activated, loyalty and equip abilities of p's compiled permanents, as choices for the AI
pub fn ability_options(g: &mut Game, p: PlayerId, sorcery_ok: bool) -> Res<Vec<crate::hooks::Opt>> {
    use crate::hooks::{Action, Opt};
    let mut out = vec![];
    if !active(g) {
        return Ok(out);
    }
    if sorcery_ok {
        out.extend(equip_options(g, p));
    }
    let stamp = round_stamp(g, p);
    for src in g.player(p).perms.clone() {
        let x = g.perm(src);
        let Some(cd) = x.cd else { continue };
        if x.phased || !g.db.get(cd).has_dsl() || crate::engine::values::stopped(g, &g.db.get(cd).name) {
            continue; // (Disruptor Flute)
        }
        if crate::cardcode::ability_locked(g, src, p) {
            continue;
        }
        let name = g.db.get(cd).name.to_string();
        for (i, a) in abilities(g, cd).iter().enumerate() {
            match a {
                Ability::Activated { cost, effects, sorcery, once, ai_min } => {
                    if *sorcery == Some(true) && !sorcery_ok {
                        continue;
                    }
                    if effects.iter().all(|e| e.kind == "add_mana") && cost.sac.is_some() {
                        continue; // sacrificing for mana nobody is waiting to spend: never proactive
                    }
                    let uses = g.player(p).act_uses.get(&(src, i as u16)).copied();
                    let tap_only =
                        cost.mana.is_none() && cost.sac.is_none() && cost.life.is_none() && cost.discard.is_none();
                    let cap = if tap_only { 8 } else { 3 }; // tap-only abilities (untap engines) can recur
                    if let Some((st, k)) = uses
                        && st == stamp
                        && (*once == Some(true) || k >= cap)
                    {
                        continue;
                    }
                    if !pay_ability_cost(g, p, src, cost, true)? {
                        continue;
                    }
                    let mut cost_v: f64 = 0.25
                        * cost
                            .mana
                            .as_deref()
                            .unwrap_or("")
                            .chars()
                            .map(|ch| ch.to_digit(10).unwrap_or(1) as f64)
                            .psum();
                    if cost.sac.as_deref() == Some("self") {
                        cost_v += 1.5;
                    }
                    let mut u = value_of(g, p, effects, None) - cost_v;
                    if let Some(mn) = ai_min {
                        u = u.max(*mn); // card data can insist the AI uses it
                    }
                    out.push(Opt {
                        utility: u,
                        label: format!("{name} ability"),
                        act: Some(Action::Dsl { src, idx: i as u16 }),
                    });
                }
                Ability::Loyalty { loyalty, effects } if sorcery_ok => {
                    if g.perm(src).loyalty_used == Some(stamp) {
                        continue;
                    }
                    let loy = match g.perm(src).loyalty {
                        Some(l) => l,
                        None => {
                            let l = g.db.get(cd).start_loyalty.unwrap_or(3);
                            g.perm_mut(src).loyalty = Some(l);
                            l
                        }
                    };
                    if loy + loyalty < 0 {
                        continue;
                    }
                    let u = value_of(g, p, effects, None) + 0.3 * *loyalty as f64;
                    out.push(Opt {
                        utility: u,
                        label: format!("{name} {loyalty:+}"),
                        act: Some(Action::Loyalty { src, idx: i as u16 }),
                    });
                }
                _ => {}
            }
        }
    }
    Ok(out)
}

/// p activates src's compiled ability idx (an option from ability_options); false if it can't now
pub fn activate(g: &mut Game, p: PlayerId, src: PermId, idx: u16) -> Res<bool> {
    let Some(cd) = g.perm(src).cd else { return Ok(false) };
    let ab = abilities(g, cd);
    let Ability::Activated { cost, effects, .. } = &ab[idx as usize] else { return Ok(false) };
    if !g.perm(src).on_bf || g.perm(src).owner != p || !pay_ability_cost(g, p, src, cost, false)? {
        return Ok(false);
    }
    let stamp = round_stamp(g, p);
    let e = g.player_mut(p).act_uses.entry((src, idx)).or_insert((stamp, 0));
    *e = if e.0 == stamp { (stamp, e.1 + 1) } else { (stamp, 1) };
    crate::glog!(g, "  {} activates {}", g.player(p).name, g.perm(src).name);
    if !effects.iter().all(|e| e.kind == "add_mana")
        && !crate::engine::stack::ability_window(g, p, Some(src), "ability", None, None)?
    {
        return Ok(true); // countered (mana abilities skip the stack)
    }
    execute(g, p, effects, Some(src), &mut DslCtx::base(), None, 0)?;
    Ok(true)
}

/// p uses planeswalker src's loyalty ability idx
pub fn use_loyalty(g: &mut Game, p: PlayerId, src: PermId, idx: u16) -> Res<bool> {
    let Some(cd) = g.perm(src).cd else { return Ok(false) };
    let ab = abilities(g, cd);
    let Ability::Loyalty { loyalty, effects } = &ab[idx as usize] else { return Ok(false) };
    let stamp = round_stamp(g, p);
    if g.perm(src).loyalty_used == Some(stamp) || !g.perm(src).on_bf {
        return Ok(false);
    }
    let x = g.perm_mut(src);
    x.loyalty_used = Some(stamp);
    x.loyalty = Some(x.loyalty.unwrap_or(0) + loyalty);
    crate::glog!(g, "  {} uses {} ({:+})", g.player(p).name, g.perm(src).name, loyalty);
    if !crate::engine::stack::ability_window(g, p, Some(src), &format!("{loyalty:+}"), None, None)? {
        return Ok(true);
    }
    execute(g, p, effects, Some(src), &mut DslCtx::base(), None, 0)?;
    if g.perm(src).loyalty.unwrap_or(0) <= 0 && g.perm(src).on_bf {
        zones::leave(g, src)?;
        g.player_mut(p).gy.push(cd);
    }
    Ok(true)
}

/// attach unattached auto-modeled equipment ("Equip {N}") to the creature that will use it best
pub fn equip_options(g: &Game, p: PlayerId) -> Vec<crate::hooks::Opt> {
    use crate::hooks::{Action, Opt};
    let mut out = vec![];
    for &src in &g.player(p).perms {
        let x = g.perm(src);
        let Some(cd) = x.cd else { continue };
        let d = g.db.get(cd);
        if !d.has_dsl() || d.tags.str(Tag::Prot) == Some("boots") {
            continue;
        }
        let Some(n) = d.abilities.iter().find_map(|a| match a {
            Ability::Static { kind, mana, .. } if kind == "equip_cost" => Some(mana.unwrap_or(2)),
            _ => None,
        }) else {
            continue;
        };
        if x.attached.is_some_and(|t| g.perm(t).on_bf && g.perm(t).owner == p && !g.perm(t).phased) {
            continue;
        }
        if crate::engine::values::stopped(g, &d.name) {
            continue; // Disruptor Flute stops equip
        }
        let cands: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| g.is_creature(m) && !g.perm(m).phased && !g.perm(m).noatk && !untargetable(g, m))
            .collect();
        if cands.is_empty() || !can_pay(g, p, n, "", false) {
            continue;
        }
        let score = |m: PermId| {
            let y = g.perm(m);
            (if y.army { 3.0 } else { 0.0 })
                + (if y.fly || has_kw(g, m, "flying") { 2.0 } else { 0.0 })
                + 0.3 * epow(g, m) as f64
                - if y.sick { 1.0 } else { 0.0 }
        };
        let best = max_by(&cands, score).unwrap();
        let bonus: i32 = d
            .abilities
            .iter()
            .filter_map(|a| match a {
                Ability::Static { kind, pow, .. } if kind == "equip_bonus" => *pow,
                _ => None,
            })
            .sum();
        let trig = d
            .abilities
            .iter()
            .filter(|a| matches!(a, Ability::Triggered { source: Some(s), .. } if s == "equipped"))
            .count();
        let u = 2.5 + 0.5 * bonus as f64 + 2.0 * trig as f64 - 0.3 * n as f64;
        out.push(Opt {
            utility: u,
            label: format!("equip {}", d.name),
            act: Some(Action::Equip { src, target: best, n }),
        });
    }
    out
}

/// equip src to target for {n}
pub fn equip(g: &mut Game, p: PlayerId, src: PermId, target: PermId, n: u32) -> Res<bool> {
    if !can_pay(g, p, n, "", false) || !g.perm(target).on_bf || g.perm(target).owner != p {
        return Ok(false);
    }
    pay(g, p, n, "", false)?;
    crate::glog!(g, "  {} equips {} to {}", g.player(p).name, g.perm(src).name, g.perm(target).name);
    let name = format!("equip to {}", g.perm(target).name);
    if crate::engine::stack::ability_window(g, p, Some(src), &name, None, Some(target))?
        && g.perm(target).on_bf
        && g.perm(src).on_bf
    {
        g.perm_mut(src).attached = Some(target);
    }
    Ok(true)
}
