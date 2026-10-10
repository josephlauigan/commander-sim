//! The AI (Python's `ais.py` decisions, `ai/brain.py`, `ai/pool_ai.py`, `ai/pool_decks.py`, `ai/deck_plans.py`,
//! `ai/gc_prio.py`, `ai/search.py`).
//!
//! - `brain`: the heuristic AI. It lists every legal play with a utility, samples one from the softmax of the
//!   utilities (Gumbel keys), and repeats until it samples "stop". Main phases, the end-of-turn window, defender
//!   choice, attacks and the answer to an attack.
//! - `act`: what each listed play does (`hooks::Action` is data, so a copied game can find and make it again).
//! - `search`: the look-ahead. It tries the heuristic AI's top plays on copies of the game, played out to the end of
//!   the player's next turn, and keeps the best.
//! - `pool`: the outside decks' AI (a cast priority from card tags, their protection and attack rules).
//! - `plans`: per-deck configuration and plans for outside decks, and the Game Changers' priorities.
//! - `decks`: your decks' priorities, tutor targets, protection and wipe answers, and the wipe evaluation.
//! - `topdeck`: how much a player wants each card next, and scry / surveil.
//!
//! This file has the decisions the engine calls inline. Deck-specific code for your other decks (Sephiroth, Veyran,
//! Galadriel, Y'shtola, Alela, Jodah) waits for phase 6 (`PORT(phase 6)`), card code for M5 (`PORT(M5)`).

pub mod act;
pub mod brain;
pub mod decks;
pub mod plans;
pub mod pool;
pub mod search;
pub mod topdeck;

use crate::engine::values::epow;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::rng::Rng;
use crate::state::{Ctx, Game};
use crate::sym::Sym;

// ------------------------------------------------------------------ play styles
/// How a deck plays (brain.STYLE): temp is the randomness of its choices, aggression its attacking and pressure,
/// caution how much it holds up interaction and keeps blockers home.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub temp: f64,
    pub aggression: f64,
    pub caution: f64,
}

const fn st(temp: f64, aggression: f64, caution: f64) -> Style {
    Style { temp, aggression, caution }
}

/// brain.STYLE: your decks (their keys are also ais.MAIN, the decks with hand-written plans)
const MAIN_STYLES: [(&str, Style); 8] = [
    ("seph", st(1.0, 0.55, 0.60)),
    ("veyran", st(1.0, 0.40, 0.80)),
    ("sauron", st(1.0, 0.55, 0.75)),
    ("galadriel", st(1.0, 0.70, 0.55)),
    ("yshtola", st(1.0, 0.50, 0.75)),
    ("alela", st(1.0, 0.70, 0.60)),
    ("jodah", st(1.0, 0.90, 0.30)), // audit/jodah: holding back cost ~2 points
    ("najeela", st(1.0, 0.90, 0.30)),
];

/// one of your decks (Python: `p.key in STYLE`, `p.key in MAIN`, `p.key in E.IDENT`), not an outside deck
pub fn is_main(key: &str) -> bool {
    MAIN_STYLES.iter().any(|x| x.0 == key)
}

/// brain.style: your deck's style, or the outside deck's configured one
pub fn style(_g: &Game, key: &str) -> Style {
    match MAIN_STYLES.iter().find(|x| x.0 == key) {
        Some(x) => x.1,
        None => plans::config(key).style.unwrap_or(pool::DEFAULT_STYLE),
    }
}

/// brain.T: the temperature p's choices are sampled at
pub fn temp(g: &Game, p: PlayerId) -> f64 {
    (style(g, g.player(p).key).temp * g.settings.temp_scale).max(0.05)
}

pub fn sig(x: f64) -> f64 {
    1.0 / (1.0 + (-x.clamp(-30.0, 30.0)).exp())
}

/// brain.gumbel_order: an ordering sampled from the softmax of the utilities (Plackett-Luce, via Gumbel keys).
/// Returns indices into `utils`.
pub fn gumbel_order(rng: &mut Rng, utils: &[f64], temp: f64) -> Vec<usize> {
    let mut keyed: Vec<(f64, usize)> = utils
        .iter()
        .enumerate()
        .map(|(i, &u)| {
            let mut r = rng.random();
            if r == 0.0 {
                r = 1e-12;
            }
            (u / temp - (-r.ln()).ln(), i)
        })
        .collect();
    keyed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    keyed.into_iter().map(|x| x.1).collect()
}

/// brain.sample: one item from the softmax of the utilities
pub fn sample<T: Copy>(rng: &mut Rng, items: &[(f64, T)], temp: f64) -> Option<T> {
    let us: Vec<f64> = items.iter().map(|x| x.0).collect();
    gumbel_order(rng, &us, temp).first().map(|&i| items[i].1)
}

/// brain.explain: a trace line with the distribution the AI sampled from
pub fn explain(g: &mut Game, p: PlayerId, items: &[(f64, String)], chosen: &str, what: &str) {
    if g.log.is_none() || items.len() < 2 {
        return;
    }
    let t = temp(g, p);
    let mx = items.iter().map(|x| x.0).fold(f64::NEG_INFINITY, f64::max);
    let mut ws: Vec<(f64, &str)> = items.iter().map(|(u, l)| (((u - mx) / t).exp(), l.as_str())).collect();
    let tot: f64 = ws.iter().map(|x| x.0).psum();
    ws.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let dist: Vec<String> = ws.iter().take(4).map(|(w, l)| format!("{l} {:.0}%", 100.0 * w / tot)).collect();
    let name = g.player(p).name;
    crate::glog!(g, "      [{name} {what}: {} -> {chosen}]", dist.join(", "));
}

// ------------------------------------------------------------------ memory and outcomes
/// brain.note_damage: remember who hurt p (the defender choice hits back at a grudge, which fades by 0.7 a hit)
pub fn note_damage(g: &mut Game, p: PlayerId, src: Option<PlayerId>, n: i32) {
    let Some(s) = src.filter(|&s| s != p) else { return };
    let key: Sym = g.player(s).key;
    let gr = g.player_mut(p).grudge.entry(key).or_insert(0.0);
    *gr = *gr * 0.7 + n as f64;
}

/// ais.win: p wins, every opponent loses. A win through life or damage (a drain, burn or combat loop: through_life,
/// the default for 'combo') is survived by an opponent whose life can't change (Teferi's Protection, cast now if
/// they hold it); alternate wins (Thassa's Oracle, Revel in Riches, mill) are not.
pub fn win(g: &mut Game, p: PlayerId, how: &'static str, through_life: Option<bool>) -> Res {
    let through_life = through_life.unwrap_or(how == "combo");
    let mut left = vec![];
    for q in g.opps(p).collect::<Vec<_>>() {
        if through_life && crate::engine::zones::last_chance(g, q)? {
            crate::glog!(g, "    {} survives: its life total can't change", g.player(q).name);
            left.push(q);
            continue;
        }
        let pl = g.player_mut(q);
        pl.last_src = Some(p);
        pl.last_kind = how;
        crate::engine::life::eliminate(g, q);
    }
    if !left.is_empty() && !g.goldfish {
        let names: Vec<&str> = left.iter().map(|&q| g.player(q).name).collect();
        crate::glog!(g, "*** {} goes off, but {} survive", g.player(p).name, names.join(", "));
        return crate::engine::life::check_state(g);
    }
    crate::glog!(g, "*** {} wins by {}", g.player(p).name, how);
    g.over = true;
    g.winner = Some(p);
    g.wintype = Some(how);
    Ok(())
}

/// ais.regrow (Eternal Witness, Mnemonic Wall): the first card of this list in the graveyard back to hand
pub fn regrow(g: &mut Game, p: PlayerId, only_is: bool) -> Res {
    const ORDER: [&str; 17] = [
        "Reanimate",
        "Animate Dead",
        "Demonic Tutor",
        "Necromancy",
        "Entomb",
        "Buried Alive",
        "Dread Return",
        "Diabolic Tutor",
        "Unburial Rites",
        "Persist",
        "Counterspell",
        "Swan Song",
        "Dovin's Veto",
        "Heroic Intervention",
        "Toxic Deluge",
        "Swords to Plowshares",
        "Assassin's Trophy",
    ];
    for n in ORDER {
        let Some(c) = g.db.id(n) else { continue };
        let d = g.db.get(c);
        if only_is && !(d.instant || d.sorcery) {
            continue;
        }
        let key = crate::engine::zones::KEYSPELL.iter().any(|&t| d.tag(t));
        let pl = g.player_mut(p);
        if let Some(i) = pl.gy.iter().position(|&x| x == c) {
            pl.gy.remove(i);
            pl.hand.push(c);
            if key {
                pl.stat("key_regrown", 1);
            }
            return Ok(());
        }
    }
    Ok(())
}

/// ais.necro_floor: the life to keep: the biggest board one opponent could swing at you plus 6, never under 10
pub fn necro_floor(g: &Game, p: PlayerId) -> i32 {
    let biggest = g
        .opps(p)
        .map(|q| {
            g.player(q).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).map(|&m| epow(g, m)).sum()
        })
        .max()
        .unwrap_or(0);
    10.max(biggest + 6)
}

// ------------------------------------------------------------------ counterspells
/// pool_ai.counter_threshold: the importance a spell needs before q counters it, lowered for decks built around
/// counterspells
pub fn counter_threshold(g: &Game, q: PlayerId, base: f64) -> f64 {
    let n = g.player(q).deck_names.iter().filter(|&&c| g.db.get(c).tag(crate::tag::Tag::Ctr)).count() as f64;
    base - (0.3 * n).min(2.5)
}

/// brain.wants_counter: the probability q spends a counter on a spell of importance val (to q)
pub fn wants_counter(val: f64, thr: f64, ncounters: usize) -> f64 {
    let scarcity = 0.65 + 0.35 * (ncounters as f64 / 2.0).min(1.0);
    sig((val - thr + 0.5) / 0.8) * scarcity
}

/// search.choose_counter, when the look-ahead decides: q may counter p's spell c (the only item on the stack, cast
/// in p's main phase). None: the look-ahead doesn't decide this one (the probability roll does).
pub fn search_counter(g: &mut Game, q: PlayerId, p: PlayerId, c: CardId, generic: bool) -> Option<bool> {
    use crate::state::Step;
    if generic
        && g.stack.len() == 1
        && search::enabled(g, q)
        && g.active == Some(p)
        && matches!(g.step, Step::Main1 | Step::Main2)
        && crate::engine::stack::pick_counter(g, q, c).is_some()
    {
        return Some(search::choose_counter(g, q, p, c));
    }
    None
}

// ------------------------------------------------------------------ choices the engine asks for
/// ais.flute_pick: the card name Disruptor Flute names, with how much it hurts an opponent
pub fn flute_pick(g: &Game, p: PlayerId) -> Option<(CardId, f64)> {
    decks::flute_pick(g, p)
}

/// ais.tutor_pick: the card a tutor of this kind fetches
pub fn tutor_pick(g: &Game, p: PlayerId, kind: &str) -> Option<CardId> {
    decks::tutor_pick(g, p, kind)
}

/// ais.tutor_value
pub fn tutor_value(g: &Game, p: PlayerId, c: CardId) -> f64 {
    decks::tutor_value(g, p, c)
}

/// ais.gy_worth (what a card in q's graveyard is worth to q)
pub fn gy_worth(g: &Game, q: PlayerId, c: CardId) -> f64 {
    decks::gy_worth(g, q, c)
}

/// ais.deck_prio, the deck's cast priority for a card (0-90)
pub fn deck_prio(g: &Game, p: PlayerId, c: CardId) -> f64 {
    decks::deck_prio(g, p, c) as f64
}

/// PORT(M5): marchesa.card_etb_value (what a creature card does for p as it enters)
pub fn card_etb_value(_g: &Game, _p: PlayerId, _c: CardId) -> f64 {
    0.0
}

/// CI.combo_imp, a combo piece's importance (9: completes a combo, 7: one short)
pub fn combo_imp(g: &Game, p: PlayerId, c: CardId) -> f64 {
    crate::cardcode::combo_imp(g, p, c)
}

/// CI.spell_importance, how much opponents want to counter a pool deck's spell (0-9)
pub fn spell_importance(g: &Game, p: PlayerId, c: CardId) -> f64 {
    pool::spell_importance(g, p, c)
}

/// the cards an outside deck keeps when discarding: combo pieces and its wish list (engine.discard_worst)
pub fn discard_keep(g: &Game, p: PlayerId) -> Vec<CardId> {
    if is_main(g.player(p).key) {
        return vec![];
    }
    let mut keep = crate::cardcode::combo_pieces(g);
    for c in plans::wish_list(g, p) {
        if !keep.contains(&c) {
            keep.push(c);
        }
    }
    keep
}

/// ais.protect_response: owner protects m from removal (Heroic Intervention, Ephemerate ...): true if it did
pub fn protect_response(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    decks::protect_response(g, owner, m, kind, actor, spell)
}

/// ais.wipe_modes (Farewell, Austere Command's modes)
pub fn wipe_modes(g: &Game, p: PlayerId, kind: Sym) -> Vec<Sym> {
    decks::wipe_modes(g, p, kind)
}

/// ais.wipe_response, q's answer to a wipe: 'all' (Teferi's Protection ...), 'indes', or none
pub fn wipe_response(g: &mut Game, q: PlayerId, kind: Sym, caster: PlayerId) -> Res<Option<Sym>> {
    decks::wipe_response(g, q, kind, caster)
}

/// ais.seph_fill_resolve: a spell that fills the graveyard resolves (Entomb, Buried Alive, Unmarked Grave, Grisly
/// Salvage; Deadly Dispute's Treasure). HUMAN(phase 9): a person picks (hc.fill).
pub fn seph_fill_resolve(g: &mut Game, p: PlayerId, kind: Sym, _ctx: &Ctx) -> Res {
    use crate::engine::zones::{add_treasure, agent_for, agent_take};
    use crate::tag::Tag;
    let mut bombs: Vec<CardId> =
        g.player(p).library.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).bomb >= 5).collect();
    let val: Vec<f64> = bombs.iter().map(|&c| -seph_bval(g, p, c)).collect();
    let mut keyed: Vec<(f64, CardId)> = val.into_iter().zip(bombs.iter().copied()).collect();
    keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)); // stable, as Python's sorted
    bombs = keyed.into_iter().map(|x| x.1).collect();
    let searches = matches!(kind, "entomb" | "buried" | "unmarked");
    let take_from = |g: &mut Game, c: CardId| {
        let lib = &mut g.player_mut(p).library;
        if let Some(i) = lib.iter().position(|&x| x == c) {
            lib.remove(i);
        }
    };
    if searches && let Some(a) = agent_for(g, p) {
        // Opposition Agent takes what they search for
        let take: Vec<CardId> = if kind == "buried" {
            bombs.iter().take(3).copied().collect()
        } else {
            bombs.iter().copied().filter(|&c| !g.db.get(c).tag(Tag::Leg) || kind == "entomb").take(1).collect()
        };
        for c in take {
            take_from(g, c);
            agent_take(g, a, p, c);
        }
        crate::engine::tutors::shuffle_library(g, p);
        return Ok(());
    }
    match kind {
        "entomb" => {
            if let Some(&c) = bombs.first() {
                take_from(g, c);
                g.player_mut(p).gy.push(c);
            }
        }
        "buried" => {
            for &c in bombs.iter().take(3) {
                take_from(g, c);
                g.player_mut(p).gy.push(c);
            }
        }
        "unmarked" => {
            if let Some(&c) = bombs.iter().find(|&&c| !g.db.get(c).tag(Tag::Leg)) {
                take_from(g, c);
                g.player_mut(p).gy.push(c);
            }
        }
        "grisly" => {
            let n = g.player(p).library.len().min(5);
            let mut top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
            if let Some(i) = top.iter().position(|&c| g.db.get(c).land)
                && g.player(p).lands.len() < 7
            {
                let l = top.remove(i);
                g.player_mut(p).hand.push(l);
            }
            for c in top {
                g.player_mut(p).gy.push(c);
                if crate::engine::zones::KEYSPELL.iter().any(|&t| g.db.get(c).tag(t)) {
                    g.player_mut(p).stat("key_milled", 1);
                }
            }
        }
        "dispute" => add_treasure(g, p, 1)?,
        _ => {}
    }
    if searches {
        crate::engine::tutors::shuffle_library(g, p);
    }
    Ok(())
}

/// ais.seph_rean_resolve: a reanimation spell resolves: its target (chosen now if it was cast by a generic path)
/// enters under p's control
pub fn seph_rean_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res {
    use crate::tag::Tag;
    let kind = g.db.get(c).tags.str(Tag::Rean).unwrap_or("animate").to_string();
    let (cd, src) = match (ctx.rean_target, ctx.rean_src) {
        (Some(cd), Some(src)) => (cd, src),
        _ => {
            let tg = decks::rean_targets(g, p, &kind);
            let Some(&(_, cd, src)) = tg.first() else { return Ok(()) };
            (cd, src)
        }
    };
    let Some(i) = g.player(src).gy.iter().position(|&x| x == cd) else { return Ok(()) };
    g.player_mut(src).gy.remove(i);
    if kind == "reanimate" {
        let mv = g.db.get(cd).cmc as i32;
        crate::engine::life::lose_life(g, p, mv, Some(p), "life", None)?;
    }
    let was_removed = g.player(p).removed_bombs.contains(&cd);
    let m =
        crate::engine::zones::enter(g, p, cd, crate::engine::zones::Enter { orig: Some(src), ..Default::default() })?;
    if kind == "persist" {
        crate::engine::zones::minus_counter(g, m, 1);
    }
    if kind == "evil" {
        g.perm_mut(m).plus += 2;
    }
    g.player_mut(p).stat("rean_resolved", 1);
    note_bomb(g, p, cd, was_removed);
    Ok(())
}

// ------------------------------------------------------------------ combat
/// brain.chump_prob: how likely d chump-blocks, by how much of its life is coming at it
pub fn chump_prob(g: &Game, d: PlayerId, incoming: i32) -> f64 {
    let frac = incoming as f64 / g.player(d).life.max(1) as f64;
    sig((frac - 0.35) / 0.08)
}

/// brain.choose_defender (threat, lethal, grudge, blockers, taxes, play style)
pub fn choose_defender(g: &mut Game, p: PlayerId) -> PlayerId {
    brain::choose_defender(g, p)
}

/// brain.filter_attackers (keep ground creatures home when the table threatens a lot of damage)
pub fn filter_attackers(g: &mut Game, p: PlayerId, atk: Vec<PermId>) -> Vec<PermId> {
    brain::filter_attackers(g, p, atk)
}

/// pool_ai.attack_filter (an outside deck's own attack rules)
pub fn attack_filter(g: &mut Game, p: PlayerId, atk: Vec<PermId>, d: PlayerId) -> Vec<PermId> {
    if is_main(g.player(p).key) {
        return atk;
    }
    pool::attack_filter(g, p, atk, d)
}

/// the attack plan at the first combat of p's turn: (defender, 'filtered' | 'all' | 'none'). A look-ahead copy
/// playing out one plan has it forced; otherwise the look-ahead chooses it, when it's on.
pub fn choose_attack(g: &mut Game, p: PlayerId) -> Res<Option<(PlayerId, Sym)>> {
    if let Some(plan) = g.forced_attack.take() {
        return Ok(Some(plan));
    }
    if !search::enabled(g, p) {
        return Ok(None);
    }
    Ok(search::choose_attack(g, p))
}

/// brain.attack_response (instant removal on a dangerous attacker)
pub fn attack_response(g: &mut Game, d: PlayerId, p: PlayerId, atk: &[PermId]) -> Res<bool> {
    brain::attack_response(g, d, p, atk)
}

// ------------------------------------------------------------------ turns
/// brain.end_of_turn_window (instant-speed draw, flash creatures and removal before your turn)
pub fn end_of_turn_window(g: &mut Game, p: PlayerId) -> Res {
    brain::end_of_turn_window(g, p)
}

/// brain.main, the AI's main phase (with the look-ahead's choices)
pub fn main(g: &mut Game, p: PlayerId, post: bool) -> Res {
    brain::main(g, p, post)
}

/// ais.combo_interrupted: p goes for a combo (`which`: 'veyran', 'sauron') whose key permanents are `key`; an opponent
/// stops it with instant removal on a key piece, or (Veyran's spell loop) a counterspell. Seph's Tidebinder first.
pub fn combo_interrupted(g: &mut Game, p: PlayerId, which: &str, key: &[PermId]) -> Res<bool> {
    use crate::engine::cast::cast_card;
    use crate::engine::mana::{can_pay, pay};
    use crate::engine::removal::legal_targets;
    use crate::engine::stack::{cast_counter, free_counter, pick_counter};
    use crate::tag::Tag;
    let mut uncounterable = false;
    if which == "veyran" {
        // Mistrise Village: {U},{T}: the next spell can't be countered
        let ml =
            g.player(p).lands.iter().copied().find(|&l| g.db.get(g.land(l).cd).tag(Tag::Mistrise) && !g.land(l).tapped);
        if let Some(l) = ml {
            g.land_mut(l).tapped = true;
            if can_pay(g, p, 0, "U", false) {
                pay(g, p, 0, "U", false)?;
                uncounterable = true;
                crate::glog!(g, "    Mistrise Village: the loop spell can't be countered");
            } else {
                g.land_mut(l).tapped = false;
            }
        }
    }
    if tide_response(g, p, &format!("combo_{which}"), 10.0, None)? {
        if which == "veyran" {
            for m in crate::engine::values::find(g, p, Tag::Vkitten) {
                g.perm_mut(m).neutered = true;
            }
        }
        return Ok(true);
    }
    let qs: Vec<PlayerId> = g.after(p).collect();
    for q in qs {
        if !g.player(q).alive || q == p || g.over || crate::engine::values::silenced(g, q) {
            continue;
        }
        if g.rng.random() > 0.95 {
            continue;
        }
        for c in g.player(q).hand.clone() {
            let d = g.db.get(c);
            let Some(rem) = d.tags.str(Tag::Rem) else { continue };
            if !d.instant {
                continue;
            }
            let tgt = d.tags.str(Tag::Tgt).unwrap_or("c");
            let tg: Vec<PermId> = legal_targets(g, q, rem, tgt, d.tag(Tag::Mv4), Some(c))
                .into_iter()
                .filter(|m| key.contains(m))
                .collect();
            let (generic, pips) = (d.generic, d.pips.to_string());
            if tg.is_empty() || !can_pay(g, q, generic, &pips, false) {
                continue;
            }
            pay(g, q, generic, &pips, false)?;
            cast_card(g, q, c, "hand", Ctx { target: Some(tg[0]), ..Ctx::default() })?;
            let x = g.perm(tg[0]);
            if !x.on_bf || x.owner != p || x.phased {
                g.player_mut(q).stat("combo_stops_removal", 1);
                return Ok(true);
            }
            break;
        }
        if which == "veyran" && !uncounterable {
            let ctrs: Vec<CardId> = g
                .player(q)
                .hand
                .iter()
                .copied()
                .filter(|&c| matches!(g.db.get(c).tags.str(Tag::Ctr), Some("any" | "nc" | "ise")))
                .collect();
            for ctr in ctrs {
                let d = g.db.get(ctr);
                if !free_counter(g, q, ctr) && !can_pay(g, q, d.generic, &d.pips.to_string(), false) {
                    continue;
                }
                if !cast_counter(g, q, ctr, None)? {
                    continue;
                }
                if let Some(back) = pick_counter(g, p, ctr)
                    && cast_counter(g, p, back, Some(ctr))?
                {
                    break;
                }
                g.player_mut(q).stat("combo_stops_counter", 1);
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// ais.tide_response: Sephiroth's Tishana's Tidebinder counters an ability `actor` activates or triggers. No card has
/// the `tide` tag now (the Tidebinder left Sephiroth's list), so it never fires; PORT(phase 6) if the card comes back
/// (it needs `tide` in rust/tools/gen_tags.py's CODE_ONLY).
pub fn tide_response(
    _g: &mut Game,
    _actor: PlayerId,
    _what: &str,
    _value: f64,
    _victim: Option<PlayerId>,
) -> Res<bool> {
    Ok(false)
}

/// PORT(phase 6): ais.seph_bval (Sephiroth's reanimation target value). Until then: the card's bomb rating, or a
/// creature's power.
pub fn seph_bval(g: &Game, _p: PlayerId, c: CardId) -> f64 {
    let d = g.db.get(c);
    if d.bomb != 0 {
        d.bomb as f64
    } else if d.creature {
        d.pow as f64
    } else {
        0.0
    }
}

/// ais.note_bomb: a bomb landed (the reports, and Sephiroth's plan turn)
pub fn note_bomb(g: &mut Game, p: PlayerId, c: CardId, was_removed: bool) {
    let d = g.db.get(c);
    if d.bomb >= 6 || d.pow >= 6 {
        let name = d.name.to_string();
        crate::glog!(
            g,
            "    {name} hits the battlefield{}",
            if was_removed { " (recovered after removal)" } else { "" }
        );
        let pl = g.player_mut(p);
        pl.stat("bombs_landed", 1);
        if pl.first_bomb.is_none() {
            pl.first_bomb = Some(pl.turns);
        }
        if was_removed {
            pl.stat("bomb_recovered", 1);
        }
    }
}

/// PORT(phase 6): ais.seph_dredge (Sephiroth dredges instead of drawing)
pub fn seph_dredge(_g: &mut Game, _p: PlayerId) -> Res<bool> {
    Ok(false)
}

/// PORT(phase 6): ais.engine_payoff (Veyran's engine is online)
pub fn engine_payoff(_g: &Game, _p: PlayerId) -> bool {
    false
}

/// PORT(phase 6): ais.end_step's Sephiroth milestones (reports)
pub fn seph_end_milestones(_g: &mut Game, _p: PlayerId) {}

/// the ability language's search choice (dsl.py's search): for any card, ais.tutor_pick; for a card type, your
/// deck's named list or an outside deck's wish list. None: the interpreter takes the card it values most.
pub fn dsl_search_pick(g: &Game, p: PlayerId, f: &crate::dsl::model::Filter, cands: &[CardId]) -> Option<CardId> {
    let ty = f.kind.as_deref();
    if ty.is_none() || ty == Some("any") {
        let nm = decks::tutor_pick(g, p, "any")?;
        return cands.iter().copied().find(|&c| c == nm);
    }
    let kind = match ty {
        Some("enchantment") => Some("ench"),
        Some("creature") => Some("cre"),
        Some("artifact") => Some("art"),
        Some("ae") => Some("ae"),
        _ => None,
    };
    let key = g.player(p).key;
    if let Some(k) = kind
        && is_main(key)
        && let Some(nm) = decks::tutor_pick_named(g, p, k)
        && cands.contains(&nm)
    {
        return Some(nm);
    }
    if is_main(key) {
        return None;
    }
    let wish = plans::wish_list(g, p);
    wish.into_iter().find(|w| cands.contains(w))
}
