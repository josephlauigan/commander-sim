//! Tier 0 pool deck card code: the cards of the Stella Lee, Wild Card precon (decklists/pool/t0-b2-precons/
//! stella-lee-quick-draw.md) that weren't modeled. Brief: rust/agents/archive/t0-stella.md.
//!
//! The deck is a spells deck: "the next instant or sorcery spell you cast this turn ..." effects live in
//! [`StellaState`] (a field of `Player`) and are applied by [`on_cast_next`] (called from `engine::cast::on_cast`) and
//! [`kaza_delta`] (from `cost_of`). Free casts and copies reuse the engine's `cast_card` (zone "lib": the card is
//! already out of its zone) and `copy_spell(.., cast = true)`; storm and cascade are written once here ([`storm_copies`]
//! and `legends::cascade`). Plot follows green_cards.rs: a plotted card stays in the hand marked in `impulse_long`
//! (`u32::MAX - 1 - turn`) and is cast free on a later turn by its own option; suspend is `Player::suspended`.
//!
//! AI choices (the person's choices are not implemented here): X is chosen by the card's own option (never "all spare
//! mana" blindly), free casts take the spells worth the most and skip counterspells, spells that need a target nobody
//! has, and board wipes.

use crate::engine::repl;
use crate::engine::pool::Kind;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, casts_this_turn, copy_spell, is_casts, spell_targets};
use crate::engine::life::check_state;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, has_type, pval};
use crate::engine::zones::{Tokens, bounce, discard_cards, draw, exile_perm, make_tokens, max_by, mill, min_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, TurnStamp, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

// ------------------------------------------------------------------ state
/// The Stella precon's one-shot and remembered effects, per player.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StellaState {
    /// Crackling Spellslinger: the next instant or sorcery spell cast this turn has storm
    pub storm_next: Option<TurnStamp>,
    /// Smoldering Stagecoach: the next instant spell / the next sorcery spell cast this turn has cascade
    pub cascade_instant: Option<TurnStamp>,
    pub cascade_sorcery: Option<TurnStamp>,
    /// Kaza, Roil Chaser: the next instant or sorcery spell this turn costs this much less
    pub kaza: Option<(TurnStamp, i32)>,
    /// Thunderclap Drake: when you next cast an instant or sorcery this turn, copy it for each commander cast
    pub drake_next: Option<TurnStamp>,
    /// Forger's Foundry: the instant or sorcery (mana value 3 or less) its mana was spent on
    pub foundry_mark: Option<CardId>,
    /// the cards exiled with each Forger's Foundry
    pub foundry_exiled: Vec<(PermId, Vec<CardId>)>,
    /// the cards exiled with each Arcane Bombardment
    pub bombard: Vec<(PermId, Vec<CardId>)>,
    /// Finale of Revelation (X of 10 or more): no maximum hand size for the rest of the game
    pub nomax: bool,
}

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

fn is_isc(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.instant || d.sorcery
}

/// instant and sorcery cards in p's graveyard
fn isc_gy(g: &Game, p: PlayerId) -> usize {
    g.player(p).gy.iter().filter(|&&x| is_isc(g, x)).count()
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) -> bool {
    match v.iter().position(|&x| x == c) {
        Some(i) => {
            v.remove(i);
            true
        }
        None => false,
    }
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

/// can a permanent's {T} ability be used now? (untapped, not summoning sick unless it has haste)
fn can_tap(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.tapped && !x.phased && (!x.sick || !g.is_creature(m) || crate::dsl::has_kw(g, m, "haste"))
}

/// how many times p has cast their commander from the command zone (each adds {2} of tax)
fn commander_casts(g: &Game, p: PlayerId) -> i32 {
    let pl = g.player(p);
    (pl.tax_of(pl.cmd) / 2) as i32 + pl.cmd2.map_or(0, |c| (pl.tax_of(c) / 2) as i32)
}

/// A spell the AI is happy to cast or copy for free: its worth, or None for counterspells, spells that need a
/// target nobody offers, board wipes, and spells with choices this code doesn't make.
fn free_worth(g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
    let d = g.db.get(c);
    let t = &d.tags;
    const SKIP: [Tag; 10] = [
        Tag::Ctr,
        Tag::Wipe,
        Tag::Prot,
        Tag::Rean,
        Tag::Fill,
        Tag::Yawg,
        Tag::Avarice,
        Tag::Mastery,
        Tag::Crackle,
        Tag::Xdrain,
    ];
    if !(d.instant || d.sorcery) || SKIP.iter().any(|&k| t.has(k)) {
        return None;
    }
    if [SWINE, EXPERIMENT, PROMISE, REVELATION].contains(&&*d.name) {
        return None; // X spells cast free have X = 0
    }
    if t.has(Tag::Rem) {
        let ctx = spell_targets(g, p, c, None);
        if ctx.target.is_none() && ctx.face.is_none() {
            return None;
        }
    }
    Some(card_worth(g, p, c, false).max(d.cmc as f64))
}

/// cast card c (already out of its zone) without paying its mana cost; `exile_after`: exiled instead of going to the
/// graveyard
fn free_cast(g: &mut Game, p: PlayerId, c: CardId, exile_after: bool) -> Res<bool> {
    g.last_x = 0; // X is 0
    let ctx = Ctx { exile_after, ..spell_targets(g, p, c, None) };
    crate::glog!(g, "    {} casts {} without paying its mana cost", pname(g, p), cname(g, c));
    cast_card(g, p, c, "lib", ctx)
}

/// storm: a copy for each spell cast before c this turn (the copies go on the stack above it)
pub fn storm_copies(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let n = casts_this_turn(g, p) - 1;
    if n <= 0 {
        return Ok(());
    }
    let label = format!("storm: {n} copies of {}", cname(g, c));
    if !trigger_window(g, p, None, &label, Some(5.0))? {
        return Ok(());
    }
    for _ in 0..n {
        if g.over || !g.player(p).alive {
            break;
        }
        copy_spell(g, p, c, None, false)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ engine hooks
/// engine::cast::on_cast: p cast c. The "next instant or sorcery spell you cast this turn" effects apply to it.
pub fn on_cast_next(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let d = g.db.get(c);
    let (instant, sorcery, cmc) = (d.instant, d.sorcery, d.cmc);
    if !(instant || sorcery) {
        return Ok(());
    }
    let st = g.turn_stamp();
    if g.player(p).t0s.kaza.is_some_and(|k| k.0 == st) {
        g.player_mut(p).t0s.kaza = None; // Kaza's reduction was used up by this spell
    }
    if g.player(p).t0s.storm_next == Some(st) {
        g.player_mut(p).t0s.storm_next = None;
        storm_copies(g, p, c)?; // Crackling Spellslinger
    }
    let casc =
        if instant { &mut g.player_mut(p).t0s.cascade_instant } else { &mut g.player_mut(p).t0s.cascade_sorcery };
    if *casc == Some(st) {
        *casc = None;
        if trigger_window(g, p, None, "Smoldering Stagecoach: cascade", Some(4.0))? {
            crate::impls::legends::cascade(g, p, cmc, "Smoldering Stagecoach")?;
        }
    }
    if g.player(p).t0s.drake_next == Some(st) {
        g.player_mut(p).t0s.drake_next = None;
        for _ in 0..commander_casts(g, p) {
            if g.over || !g.player(p).alive {
                break;
            }
            copy_spell(g, p, c, None, false)?; // Thunderclap Drake
        }
    }
    Ok(())
}

/// engine::mana::cost_of: Kaza's reduction for the next instant or sorcery this turn
pub fn kaza_delta(g: &Game, p: PlayerId, c: CardId) -> i32 {
    match g.player(p).t0s.kaza {
        Some((st, n)) if st == g.turn_stamp() && is_isc(g, c) => -n,
        _ => 0,
    }
}

/// engine::cast::to_gy_or_exile: a spell Forger's Foundry's mana paid for is exiled with the Foundry instead of going
/// to the graveyard (the AI always takes the offer)
pub fn foundry_exiles(g: &mut Game, p: PlayerId, c: CardId) -> bool {
    if g.player(p).t0s.foundry_mark != Some(c) {
        return false;
    }
    g.player_mut(p).t0s.foundry_mark = None;
    let Some(f) = g
        .player(p)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).on_bf && g.perm(m).cd.is_some_and(|x| &*g.db.get(x).name == FOUNDRY))
    else {
        return false;
    };
    let ex = &mut g.player_mut(p).t0s.foundry_exiled;
    match ex.iter_mut().find(|e| e.0 == f) {
        Some(e) => e.1.push(c),
        None => ex.push((f, vec![c])),
    }
    true
}

/// cardcode::aura_ward: Winged Boots gives the creature ward {4}
pub fn boots_ward(g: &Game, m: PermId) -> u32 {
    let o = owner(g, m);
    let attached = g.player(o).perms.iter().any(|&e| {
        g.perm(e).on_bf && g.perm(e).attached == Some(m) && g.perm(e).cd.is_some_and(|c| &*g.db.get(c).name == BOOTS)
    });
    if attached { 4 } else { 0 }
}

/// engine::cast::magecraft: Octavia's trigger on every instant or sorcery you cast or copy: target creature has base
/// power and toughness 8/8 until end of turn (the AI picks the creature that gains the most)
pub fn magecraft_extra(g: &mut Game, p: PlayerId, _copy: bool) -> Res {
    if g.player(p).perms.is_empty() {
        return Ok(());
    }
    for o in g.player(p).perms.clone() {
        let x = g.perm(o);
        if !x.on_bf || x.phased || x.neutered || x.cd.is_none_or(|c| &*g.db.get(c).name != OCTAVIA) {
            continue;
        }
        let now = Val::Stamp(g.turn_stamp());
        let mut best: Option<(i32, PermId)> = None;
        for &m in &g.player(p).perms {
            let y = g.perm(m);
            if !y.on_bf || y.phased || !g.is_creature(m) || y.data.get(DataKey::T0Stella) == Some(&now) {
                continue;
            }
            let gain = (8 - y.pow) + (8 - y.tgh);
            if gain > 0 && best.is_none_or(|b| gain > b.0) {
                best = Some((gain, m));
            }
        }
        let Some((_, m)) = best else { continue };
        if !trigger_window(g, p, Some(o), &format!("{} has base power and toughness 8/8", g.perm(m).name), None)? {
            continue;
        }
        if !g.perm(m).on_bf {
            continue;
        }
        let y = g.perm_mut(m);
        y.eot_pt = (y.eot_pt.0 + 8 - y.pow, y.eot_pt.1 + 8 - y.tgh);
        y.data.set(DataKey::T0Stella, now);
    }
    Ok(())
}

// ================================================================== Stella Lee, Wild Card
const STELLA: &str = "Stella Lee, Wild Card";
const BOOTS: &str = "Winged Boots";

/// Whenever you cast your second spell each turn, exile the top card of your library; you may play it until the end
/// of your next turn (it waits in your hand, marked, as Prosper's cards do). {T}: copy target instant or sorcery
/// spell you control, if you've cast three or more spells this turn: the AI uses it on its own spell as it is cast.
fn stella_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    if caster != o {
        return Ok(());
    }
    let n = casts_this_turn(g, o);
    if n == 2 && !g.player(o).library.is_empty() && trigger_window(g, o, Some(src), "exile the top card", None)? {
        if let Some(top) = g.player_mut(o).library.pop() {
            crate::glog!(g, "    Stella Lee exiles {} (playable until the end of your next turn)", cname(g, top));
            let pl = g.player_mut(o);
            pl.seen.insert(top);
            pl.hand.push(top);
            let until = pl.turns + 1;
            pl.impulse_long.push((top, until));
        }
    }
    if n >= 3 && is_isc(g, c) && can_tap(g, src) && free_worth(g, o, c).is_some() {
        let label = format!("copy {}", cname(g, c));
        if ability_window(g, o, Some(src), &label, None, None)? && can_tap(g, src) {
            g.perm_mut(src).tapped = true;
            copy_spell(g, o, c, None, false)?;
        }
    }
    Ok(())
}

// ================================================================== Thunderclap Drake
const DRAKE: &str = "Thunderclap Drake";

/// {2}{U}, sacrifice: when you next cast an instant or sorcery this turn, copy it for each time you've cast your
/// commander from the command zone this game. Worth it with a good spell in hand and the mana for both.
fn drake_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let st = g.turn_stamp();
    if owner(g, src) != p || post.is_none() || g.active != Some(p) || commander_casts(g, p) < 1 {
        return Ok(vec![]);
    }
    if g.player(p).t0s.drake_next == Some(st) || !can_pay(g, p, 2, "U", false) {
        return Ok(vec![]);
    }
    let spare = total_mana(g, p, false) as i32 - 3;
    let best = g
        .player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let (gn, pips) = cost_of(g, p, c);
            is_isc(g, c) && (gn as i32 + pips.chars().count() as i32) <= spare
        })
        .filter_map(|c| free_worth(g, p, c).map(|w| (w, c)))
        .max_by(|a, b| a.0.total_cmp(&b.0));
    let Some((w, c)) = best else { return Ok(vec![]) };
    let n = commander_casts(g, p) as f64;
    let u = 1.0 + 0.04 * w * n.min(3.0) + 0.3 * n;
    if u < 2.0 {
        return Ok(vec![]);
    }
    Ok(vec![ability(u, format!("{DRAKE}: copy {} x{}", cname(g, c), n as i32), src, drake_go, 0)])
}

fn drake_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !g.perm(src).on_bf || owner(g, src) != p || !can_pay(g, p, 2, "U", false) {
        return Ok(false);
    }
    pay(g, p, 2, "U", false)?;
    let st = g.turn_stamp();
    crate::engine::zones::die(g, src, "sac")?; // sacrifice is a cost
    if ability_window(g, p, None, "Thunderclap Drake: copy your next instant or sorcery", Some(4.0), None)? {
        g.player_mut(p).t0s.drake_next = Some(st);
    }
    Ok(true)
}

// ================================================================== Kaza, Roil Chaser
const KAZA: &str = "Kaza, Roil Chaser";

/// {T}: the next instant or sorcery spell you cast this turn costs {X} less, X the Wizards you control as this
/// resolves. Used before a spell in hand it makes cheaper.
fn kaza_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || post.is_none() || g.active != Some(p) || !can_tap(g, src) {
        return Ok(vec![]);
    }
    let st = g.turn_stamp();
    if g.player(p).t0s.kaza.is_some_and(|k| k.0 == st) {
        return Ok(vec![]);
    }
    let wiz = wizards(g, p);
    let helps = g.player(p).hand.iter().any(|&c| {
        let (gn, _) = cost_of(g, p, c);
        is_isc(g, c) && gn > 0 && free_worth(g, p, c).is_some()
    });
    if !helps {
        return Ok(vec![]);
    }
    Ok(vec![ability(2.4 + 0.2 * wiz as f64, format!("{KAZA}: next spell {wiz} cheaper"), src, kaza_go, 0)])
}

fn wizards(g: &Game, p: PlayerId) -> i32 {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && !g.perm(m).phased && has_type(g, m, "wizard")).count()
        as i32
}

fn kaza_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_tap(g, src) || owner(g, src) != p {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "the next instant or sorcery costs less", None, None)? {
        let (st, n) = (g.turn_stamp(), wizards(g, p));
        g.player_mut(p).t0s.kaza = Some((st, n));
    }
    Ok(true)
}

// ================================================================== Crackling Spellslinger
/// flash; when it enters, if you cast it, the next instant or sorcery spell you cast this turn has storm
fn spellslinger_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || owner(g, src) != p || !g.last_cast_etb {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "the next instant or sorcery has storm", None)? {
        let st = g.turn_stamp();
        g.player_mut(p).t0s.storm_next = Some(st);
    }
    Ok(())
}

/// worth casting first when there is a spell in hand to storm
fn spellslinger_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let _ = c;
    let mana = total_mana(g, p, false) as i32;
    let big = g
        .player(p)
        .hand
        .iter()
        .any(|&x| is_isc(g, x) && g.db.get(x).cmc as i32 + 5 <= mana && casts_this_turn(g, p) >= 1);
    if big { 66 } else { 46 }
}

// ================================================================== Elemental Eruption (storm)
const ERUPTION: &str = "Elemental Eruption";

fn eruption_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    storm_copies(g, p, c)
}

/// a 4/4 red Dragon Elemental with flying and prowess
fn eruption_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let spec = Tokens {
        fly: true,
        color: Some(Colors::from_letters("R")),
        types: vec!["dragon", "elemental"],
        tgh: Some(4),
        ..Tokens::new(1, 4)
    };
    for m in make_tokens(g, p, spec)? {
        g.perm_mut(m).data.set(DataKey::Prowess, Val::Bool(true));
    }
    Ok(intern("gy"))
}

/// more copies with more spells cast first
fn eruption_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    38 + 12 * casts_this_turn(g, p).min(4)
}

// ================================================================== Volcanic Torrent (cascade)
const TORRENT: &str = "Volcanic Torrent";

fn torrent_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    if !trigger_window(g, p, None, "Volcanic Torrent: cascade", Some(4.0))? {
        return Ok(());
    }
    let mv = g.db.get(c).cmc;
    crate::impls::legends::cascade(g, p, mv, "Volcanic Torrent")?;
    Ok(())
}

/// X damage to each creature and planeswalker your opponents control, X the spells you've cast this turn
fn torrent_resolve(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let x = casts_this_turn(g, p);
    crate::glog!(g, "    Volcanic Torrent deals {x} damage to each opposing creature and planeswalker");
    if x > 0 {
        let kind = format!("dmg{x}");
        let victims: Vec<PermId> = g
            .opps(p)
            .flat_map(|q| g.player(q).perms.clone())
            .filter(|&m| g.perm(m).on_bf && (g.is_creature(m) || g.perm(m).loyalty.is_some()))
            .collect();
        for m in victims {
            if !g.perm(m).on_bf {
                continue;
            }
            if g.is_creature(m) {
                if crate::engine::values::etgh(g, m) <= x {
                    apply_removal(g, Some(p), m, &kind, Some(c))?; // damage that kills; smaller damage wears off
                }
            } else if let Some(l) = g.perm(m).loyalty {
                g.perm_mut(m).loyalty = Some(l - x);
                if l - x <= 0 {
                    crate::engine::zones::die(g, m, "sba")?;
                }
            }
        }
    }
    check_state(g)?;
    Ok(intern("gy"))
}

/// a five-mana cascade is fine anytime; better against a board of small creatures
fn torrent_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let x = casts_this_turn(g, p) + 1;
    let hit = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.perm(m).on_bf && g.is_creature(m) && crate::engine::values::etgh(g, m) <= x)
        .count() as i32;
    50 + 4 * hit.min(5)
}

// ================================================================== Arcane Bombardment
const BOMBARD: &str = "Arcane Bombardment";

/// Whenever you cast your first instant or sorcery spell each turn, exile an instant or sorcery card at random from
/// your graveyard. Then copy each card exiled with this enchantment; you may cast any number of the copies free.
fn bombard_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    if caster != o || !is_isc(g, c) || is_casts(g, o) != 1 {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "exile an instant or sorcery and cast copies of what it exiled", Some(5.0))? {
        return Ok(());
    }
    let cands: Vec<CardId> = g.player(o).gy.iter().copied().filter(|&x| is_isc(g, x)).collect();
    if let Some(&x) = g.rng.choice(&cands) {
        remove_card(&mut g.player_mut(o).gy, x);
        g.player_mut(o).exile.push(x);
        crate::glog!(g, "    Arcane Bombardment exiles {} from the graveyard", cname(g, x));
        let ex = &mut g.player_mut(o).t0s.bombard;
        match ex.iter_mut().find(|e| e.0 == src) {
            Some(e) => e.1.push(x),
            None => ex.push((src, vec![x])),
        }
    }
    let pile: Vec<CardId> =
        g.player(o).t0s.bombard.iter().find(|e| e.0 == src).map(|e| e.1.clone()).unwrap_or_default();
    let mut pile: Vec<(f64, CardId)> = pile.into_iter().filter_map(|x| free_worth(g, o, x).map(|w| (w, x))).collect();
    pile.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (_, x) in pile {
        if g.over || !g.player(o).alive {
            break;
        }
        copy_spell(g, o, x, None, true)?; // a real cast of a copy
    }
    Ok(())
}

// ================================================================== Baral's Expertise
const BARAL: &str = "Baral's Expertise";

/// the artifacts and creatures worth returning: opposing ones by value
fn baral_targets(g: &Game, p: PlayerId, c: CardId) -> Vec<PermId> {
    let mut tg: Vec<PermId> = legal_targets(g, p, "bounce", "nl", false, Some(c))
        .into_iter()
        .filter(|&m| {
            let x = g.perm(m);
            x.owner != p
                && !x.token
                && (g.is_creature(m) || x.cd.is_some_and(|cd| g.db.get(cd).types.has(Types::ARTIFACT)))
        })
        .collect();
    tg.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
    tg.truncate(3);
    tg
}

/// the best spell with mana value 4 or less in hand to cast free
fn baral_pick(g: &Game, p: PlayerId, except: CardId) -> Option<CardId> {
    g.player(p)
        .hand
        .iter()
        .copied()
        .filter(|&x| x != except && !g.db.get(x).land && g.db.get(x).cmc <= 4)
        .filter(|&x| g.db.get(x).perm || free_worth(g, p, x).is_some())
        .max_by(|&a, &b| {
            (g.db.get(a).cmc as f64 * 3.0 + card_worth(g, p, a, false) * 0.01)
                .total_cmp(&(g.db.get(b).cmc as f64 * 3.0 + card_worth(g, p, b, false) * 0.01))
        })
}

/// Return up to three target artifacts and/or creatures to their owners' hands. You may cast a spell with mana value
/// 4 or less from your hand without paying its mana cost.
fn baral_resolve(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for m in baral_targets(g, p, c) {
        if g.perm(m).on_bf && pval(g, m) >= 2.0 {
            bounce(g, m)?;
        }
    }
    if let Some(x) = baral_pick(g, p, c)
        && remove_card(&mut g.player_mut(p).hand, x)
    {
        let ex = g.db.get(x).perm;
        if ex {
            g.last_x = 0;
            cast_card(g, p, x, "lib", Ctx::default())?;
        } else {
            free_cast(g, p, x, false)?;
        }
    }
    Ok(intern("gy"))
}

fn baral_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let tg = baral_targets(g, p, c);
    let worth: f64 = tg.iter().map(|&m| pval(g, m)).filter(|&v| v >= 2.0).sum();
    let free = baral_pick(g, p, c).map_or(0, |x| g.db.get(x).cmc as i32);
    if free >= 3 || worth >= 8.0 { 48 + free.min(4) * 2 + (worth as i32).min(10) } else { 0 }
}

// ================================================================== Curse of the Swine
const SWINE: &str = "Curse of the Swine";

/// the opposing creatures worth exiling (a 2/2 Boar replaces each)
fn swine_targets(g: &Game, p: PlayerId, c: CardId) -> Vec<PermId> {
    let mut tg: Vec<PermId> = legal_targets(g, p, "exile", "c", false, Some(c))
        .into_iter()
        .filter(|&m| g.perm(m).owner != p && g.is_creature(m) && pval(g, m) >= 3.0)
        .collect();
    tg.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
    tg
}

fn swine_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    let (gn, pips) = cost_of(g, p, c);
    let spare = total_mana(g, p, false) as i32 - gn as i32 - pips.chars().count() as i32;
    let tg = swine_targets(g, p, c);
    let x = (tg.len() as i32).min(spare);
    if x < 1 || !can_pay(g, p, gn + x as u32, &pips, false) {
        return Ok(vec![]);
    }
    let val: f64 = tg.iter().take(x as usize).map(|&m| pval(g, m) - 1.5).sum();
    if val < 3.0 {
        return Ok(vec![]);
    }
    Ok(vec![plan(0.9 * val, format!("{SWINE} (X = {x})"), swine_go, c.0 as i64)])
}

fn swine_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, p, c);
    let spare = total_mana(g, p, false) as i32 - gn as i32 - pips.chars().count() as i32;
    let x = (swine_targets(g, p, c).len() as i32).min(spare);
    if x < 1 || !can_pay(g, p, gn + x as u32, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn + x as u32, &pips, false)?;
    cast_card(g, p, c, "hand", Ctx { x, ..Ctx::default() })?;
    Ok(true)
}

/// Exile X target creatures. For each creature exiled this way, its controller creates a 2/2 green Boar.
fn swine_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let tg = swine_targets(g, p, c);
    for m in tg.into_iter().take(ctx.x.max(0) as usize) {
        let q = g.perm(m).owner;
        apply_removal(g, Some(p), m, "exile", Some(c))?;
        if !g.perm(m).on_bf {
            let spec = Tokens {
                color: Some(Colors::from_letters("G")),
                types: vec!["boar"],
                tgh: Some(2),
                ..Tokens::new(1, 2)
            };
            make_tokens(g, q, spec)?;
        }
    }
    Ok(intern("gy"))
}

// ================================================================== Epic Experiment
const EXPERIMENT: &str = "Epic Experiment";

fn experiment_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || post == Some(true) || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    let x = (total_mana(g, p, false) as i32 - 2).min(7);
    let (gn, pips) = cost_of(g, p, c);
    if x < 3 || !can_pay(g, p, gn + x as u32, &pips, false) {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.5 + 0.5 * x as f64, format!("{EXPERIMENT} (X = {x})"), experiment_go, c.0 as i64)])
}

fn experiment_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (gn, pips) = cost_of(g, p, c);
    let x = (total_mana(g, p, false) as i32 - gn as i32 - pips.chars().count() as i32).min(7);
    if !g.player(p).hand.contains(&c) || x < 1 || !can_pay(g, p, gn + x as u32, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn + x as u32, &pips, false)?;
    cast_card(g, p, c, "hand", Ctx { x, ..Ctx::default() })?;
    Ok(true)
}

/// Exile the top X cards of your library. You may cast instant and sorcery spells with mana value X or less from
/// among them free. Then put all cards exiled this way that weren't cast into your graveyard.
fn experiment_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let mut pile = vec![];
    for _ in 0..ctx.x.max(0) {
        match g.player_mut(p).library.pop() {
            Some(x) => pile.push(x),
            None => break,
        }
    }
    crate::glog!(g, "    Epic Experiment exiles {} cards", pile.len());
    let mut cast: Vec<(f64, CardId)> = pile
        .iter()
        .copied()
        .filter(|&x| g.db.get(x).cmc as i32 <= ctx.x)
        .filter_map(|x| free_worth(g, p, x).map(|w| (w, x)))
        .collect();
    cast.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (_, x) in cast {
        if g.over || !g.player(p).alive {
            break;
        }
        remove_card(&mut pile, x);
        free_cast(g, p, x, false)?;
    }
    g.player_mut(p).gy.extend(pile);
    Ok(intern("gy"))
}

// ================================================================== Finale of Promise
const PROMISE: &str = "Finale of Promise";

/// the best instant and sorcery in the graveyard with mana value `x` or less
fn promise_picks(g: &Game, p: PlayerId, x: u32) -> Vec<CardId> {
    let mut out = vec![];
    for instant in [true, false] {
        let best = g
            .player(p)
            .gy
            .iter()
            .copied()
            .filter(|&c| {
                let d = g.db.get(c);
                d.cmc <= x && if instant { d.instant } else { d.sorcery }
            })
            .filter_map(|c| free_worth(g, p, c).map(|w| (w, c)))
            .max_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((_, c)) = best {
            out.push(c);
        }
    }
    out
}

/// Sword of Once and Future: cast the best instant or sorcery card with mana value `x` or less from p's graveyard
/// without paying its mana cost; exiled instead of going to the graveyard. False if none is worth casting
pub fn free_from_gy(g: &mut Game, p: PlayerId, x: u32) -> Res<bool> {
    let best = promise_picks(g, p, x)
        .into_iter()
        .filter_map(|c| free_worth(g, p, c).map(|w| (w, c)))
        .max_by(|a, b| a.0.total_cmp(&b.0));
    let Some((_, c)) = best else { return Ok(false) };
    if !remove_card(&mut g.player_mut(p).gy, c) {
        return Ok(false);
    }
    free_cast(g, p, c, true)
}

fn promise_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    let (gn, pips) = cost_of(g, p, c);
    let spare = total_mana(g, p, false) as i32 - gn as i32 - pips.chars().count() as i32;
    if spare < 1 || !can_pay(g, p, gn + 1, &pips, false) {
        return Ok(vec![]);
    }
    // the smallest X that reaches the best cards it can afford
    let top = promise_picks(g, p, spare as u32);
    let x = top.iter().map(|&t| g.db.get(t).cmc).max().unwrap_or(0).max(1);
    let val: f64 = top.iter().map(|&t| free_worth(g, p, t).unwrap_or(0.0)).sum();
    if top.is_empty() || val < 5.0 {
        return Ok(vec![]);
    }
    Ok(vec![plan(0.6 * val.min(14.0), format!("{PROMISE} (X = {x})"), promise_go, c.0 as i64)])
}

fn promise_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, p, c);
    let spare = total_mana(g, p, false) as i32 - gn as i32 - pips.chars().count() as i32;
    let top = promise_picks(g, p, spare.max(0) as u32);
    let x = top.iter().map(|&t| g.db.get(t).cmc).max().unwrap_or(0).max(1) as i32;
    if top.is_empty() || spare < x || !can_pay(g, p, gn + x as u32, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn + x as u32, &pips, false)?;
    cast_card(g, p, c, "hand", Ctx { x, ..Ctx::default() })?;
    Ok(true)
}

/// Cast up to one instant and up to one sorcery from your graveyard, each with mana value X or less, free; a spell
/// cast this way is exiled instead of going to the graveyard. If X is 10 or more, copy each of them twice.
fn promise_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    for x in promise_picks(g, p, ctx.x.max(0) as u32) {
        if g.over || !g.player(p).alive || !remove_card(&mut g.player_mut(p).gy, x) {
            continue;
        }
        free_cast(g, p, x, true)?;
        if ctx.x >= 10 {
            for _ in 0..2 {
                copy_spell(g, p, x, None, false)?;
            }
        }
    }
    Ok(intern("gy"))
}

// ================================================================== Finale of Revelation
const REVELATION: &str = "Finale of Revelation";

fn revelation_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    let (gn, pips) = cost_of(g, p, c);
    let x = total_mana(g, p, false) as i32 - gn as i32 - pips.chars().count() as i32;
    if x < 3 || !can_pay(g, p, gn + x as u32, &pips, false) {
        return Ok(vec![]);
    }
    let hand = g.player(p).hand.len() as i32;
    if x < 10 && (hand > 3 || (post == Some(false) && x < 5)) {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.0 + 0.45 * x as f64, format!("{REVELATION} (X = {x})"), revelation_go, c.0 as i64)])
}

fn revelation_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (gn, pips) = cost_of(g, p, c);
    let x = total_mana(g, p, false) as i32 - gn as i32 - pips.chars().count() as i32;
    if !g.player(p).hand.contains(&c) || x < 1 || !can_pay(g, p, gn + x as u32, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn + x as u32, &pips, false)?;
    cast_card(g, p, c, "hand", Ctx { x, ..Ctx::default() })?;
    Ok(true)
}

/// Draw X cards. If X is 10 or more, instead shuffle your graveyard into your library, draw X cards, untap up to five
/// lands, and you have no maximum hand size for the rest of the game. Exile Finale of Revelation.
fn revelation_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x.max(0);
    if x >= 10 {
        let gy = std::mem::take(&mut g.player_mut(p).gy);
        g.player_mut(p).library.extend(gy);
        shuffle_library(g, p);
        draw(g, p, x as u32, false)?;
        let tapped: Vec<_> = g.player(p).lands.iter().copied().filter(|&l| g.perm(l).tapped).take(5).collect();
        for l in tapped {
            g.perm_mut(l).tapped = false;
        }
        g.player_mut(p).t0s.nomax = true;
    } else {
        draw(g, p, x as u32, false)?;
    }
    Ok(intern("exile"))
}

// ================================================================== Rousing Refrain (suspend)
const REFRAIN: &str = "Rousing Refrain";

/// Add {R} for each card in target opponent's hand (kept until the end of the turn, as
/// steps and phases end). Exile it with three time counters; the suspend upkeep casts it free when the last is removed.
fn refrain_resolve(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = g.opps(p).map(|q| g.player(q).hand.len()).max().unwrap_or(0) as u32;
    crate::glog!(g, "    Rousing Refrain adds {n} red mana");
    g.player_mut(p).mana.add_kept(Kind::R, n);
    let pl = g.player_mut(p);
    pl.exile.push(c);
    pl.suspended.push((c, 3));
    Ok(intern("handled"))
}

/// Suspend 3 {1}{R}: during your main phase with the mana spare, when opponents hold cards
fn refrain_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "R", false) {
        return Ok(vec![]);
    }
    let theirs = g.opps(p).map(|q| g.player(q).hand.len()).max().unwrap_or(0);
    if theirs < 3 || g.player(p).suspended.iter().any(|e| e.0 == c) {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.3 + 0.1 * theirs as f64, "suspend Rousing Refrain".into(), refrain_go, c.0 as i64)])
}

fn refrain_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "R", false) {
        return Ok(false);
    }
    pay(g, p, 1, "R", false)?;
    let pl = g.player_mut(p);
    remove_card(&mut pl.hand, c);
    pl.exile.push(c);
    pl.suspended.push((c, 3));
    crate::glog!(g, "  {} suspends Rousing Refrain", pname(g, p));
    Ok(true)
}

// ================================================================== Mizzix's Mastery (casual combos)
const MASTERY: &str = "Mizzix's Mastery";

/// your permanents that pay off each instant or sorcery cast (Guttersnipe, Electrostatic Field, Niv-Mizzet, token
/// makers, Archmage Emeritus, Storm-Kiln Artist); Veyran doubles them
fn spell_payoffs(g: &Game, p: PlayerId) -> f64 {
    const PAY: [Tag; 5] = [Tag::Ping, Tag::Spelltok, Tag::Spelldraw, Tag::Kiln, Tag::Mystic];
    const NAMES: [&str; 2] = ["Electrostatic Field", "Niv-Mizzet, Parun"];
    let mut n = 0.0;
    let mut veyran = false;
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        let Some(c) = x.cd.filter(|_| x.on_bf && !x.phased) else { continue };
        let d = g.db.get(c);
        veyran |= &*d.name == "Veyran, Voice of Duality";
        if PAY.iter().any(|&k| d.tag(k)) || NAMES.contains(&&*d.name) {
            n += 1.0;
        }
    }
    if veyran { 2.0 * n } else { n }
}

/// instant and sorcery cards in p's graveyard the Mastery copies are worth casting
fn mastery_pool(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).gy.iter().copied().filter(|&x| free_worth(g, p, x).is_some()).collect()
}

/// The pool AI never cast Mizzix's Mastery (its tag holds it out of the normal priorities, and only Veyran's brain
/// offered it): overloaded ({5}{R}{R}{R}) with four or more spells to copy, or for {3}{R} on the best one. Your own
/// decks' brains (Veyran) make this choice themselves.
fn mastery_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || crate::ai::is_main(g.player(p).key) || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    let pool = mastery_pool(g, p);
    if pool.is_empty() {
        return Ok(vec![]);
    }
    let pay = spell_payoffs(g, p);
    let mut o = vec![];
    let k = pool.len() as f64;
    let best = pool.iter().filter_map(|&x| free_worth(g, p, x)).fold(0.0, f64::max);
    let one = 1.0 + 0.4 * best + 0.5 * pay;
    if pool.len() >= 4 && can_pay(g, p, 5, "RRR", false) {
        // every copy is worth at least the one copy of the best of them
        let u = (3.0 + 0.5 * k + 1.5 * pay).max(one + 0.5 * (k - 1.0));
        o.push(plan(u, format!("{MASTERY} overload ({k} spells)"), mastery_go, c.0 as i64 | 1 << 32));
    }
    if best >= 4.0 && can_pay(g, p, 3, "R", false) {
        o.push(plan(one, format!("{MASTERY} (one spell)"), mastery_go, c.0 as i64));
    }
    Ok(o)
}

fn mastery_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff) as u16);
    let overload = arg >> 32 != 0;
    let (gn, pips) = if overload { (5, "RRR") } else { (3, "R") };
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, gn, pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, pips, false)?;
    cast_card(g, p, c, "hand", Ctx { overload, ..Ctx::default() })?;
    Ok(true)
}

// ================================================================== Galvanic Iteration (casual combos)
/// The pool decks' priority (it was 0: held forever; your decks' own priorities decide for them): with another
/// instant or sorcery worth copying castable this turn after its {U}{R}, as Veyran's priority does.
fn galvanic_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let rest = total_mana(g, p, false) as i64 - 2;
    let any = g.player(p).hand.iter().any(|&x| {
        let d = g.db.get(x);
        x != c
            && (d.instant || d.sorcery)
            && (d.cmc as i64) <= rest
            && (d.tag(Tag::Draw) || d.tag(Tag::Burn) || d.tag(Tag::Rem) || [ERUPTION, TORRENT].contains(&&*d.name))
    });
    if any { 47 } else { 0 }
}

// ================================================================== Bloodthirsty Adversary
const ADVERSARY: &str = "Bloodthirsty Adversary";

/// haste. When it enters, you may pay {2}{R} any number of times. When you pay it one or more times, put that many
/// +1/+1 counters on it, then exile up to that many instant and/or sorcery cards with mana value 3 or less from your
/// graveyard and copy them; you may cast any number of the copies free.
fn adversary_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || owner(g, src) != p {
        return Ok(());
    }
    let mut cands: Vec<(f64, CardId)> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&x| g.db.get(x).cmc <= 3)
        .filter_map(|x| free_worth(g, p, x).map(|w| (w, x)))
        .collect();
    cands.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut k = 0;
    while (k as usize) < cands.len() && can_pay(g, p, 2 * (k + 1), &"R".repeat(k as usize + 1), false) {
        k += 1;
    }
    if k == 0 {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), &format!("pay {{2}}{{R}} x{k}: counters and copies"), None)? {
        return Ok(());
    }
    pay(g, p, 2 * k, &"R".repeat(k as usize), false)?;
    repl::put_plus(g, m, k as i32);
    for (_, x) in cands.into_iter().take(k as usize) {
        if g.over || !g.player(p).alive || !remove_card(&mut g.player_mut(p).gy, x) {
            continue;
        }
        g.player_mut(p).exile.push(x);
        copy_spell(g, p, x, None, true)?;
    }
    Ok(())
}

// ================================================================== Cursed Mirror
const MIRROR: &str = "Cursed Mirror";

/// {T}: add {R}. As it enters, you may have it become a copy of any creature on the battlefield until end of turn,
/// except it has haste. The AI copies the best creature (its stats and flying/deathtouch/lifelink/vigilance; the
/// copy's own triggers and enters effects are not copied).
fn mirror_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || owner(g, src) != p {
        return Ok(());
    }
    let all: Vec<PermId> = g
        .players
        .iter()
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&x| x != src && g.perm(x).on_bf && !g.perm(x).phased && g.is_creature(x))
        .collect();
    let human = crate::human::is_human(g, p);
    let t = if human {
        // "you may have it become a copy of any creature": the person picks the creature, or declines
        let labels: Vec<String> = all.iter().map(|&x| g.perm(x).name.to_string()).collect();
        if labels.is_empty() {
            return Ok(());
        }
        let prompt = "Cursed Mirror: have it become a copy of a creature until end of turn?";
        match crate::human::choices::choose(g, p, "target", prompt, &labels, Some("no"), None)? {
            Some(k) => all[k],
            None => return Ok(()),
        }
    } else {
        let Some(t) = max_by(&all, |x| (epow(g, x) + crate::engine::values::etgh(g, x)) as f64) else { return Ok(()) };
        t
    };
    let (tp, tt) = (epow(g, t), crate::engine::values::etgh(g, t));
    if (tp < 3 && !human) || !trigger_window(g, p, Some(src), &format!("becomes a copy of {}", g.perm(t).name), None)? {
        return Ok(());
    }
    let (fly, dt, vig, life) = (g.perm(t).fly, g.perm(t).dt, g.perm(t).vig, g.perm(t).lifelink);
    let now = Val::Stamp(g.turn_stamp());
    let x = g.perm_mut(src);
    x.data.set(DataKey::Anim, Val::Bool(true));
    x.eot_pt = (tp, tt);
    (x.fly, x.dt, x.vig, x.lifelink) = (x.fly || fly, x.dt || dt, x.vig || vig, x.lifelink || life);
    x.sick = false; // haste
    x.data.set(DataKey::T0Stella, now);
    crate::glog!(g, "    Cursed Mirror becomes a copy of {} until end of turn", g.perm(t).name);
    Ok(())
}

/// at the end step the Mirror is an artifact again
fn mirror_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let x = g.perm_mut(src);
    if x.data.get(DataKey::T0Stella).is_some() {
        x.data.remove(DataKey::T0Stella);
        x.data.set(DataKey::Anim, Val::Bool(false));
        x.fly = false;
        x.dt = false;
        x.vig = false;
        x.lifelink = false;
    }
    Ok(())
}

// ================================================================== Forger's Foundry
const FOUNDRY: &str = "Forger's Foundry";

/// {T}: add {U}; when that mana is spent on an instant or sorcery spell with mana value 3 or less, it may be exiled
/// with the Foundry as it resolves (run when the Foundry is tapped for mana while paying for a spell)
fn foundry_tapped(g: &mut Game, src: Src, p: PlayerId, m: PermId, _n: u32) -> Res {
    if m != src || owner(g, src) != p {
        return Ok(());
    }
    if let Some(c) = g.pay_for
        && is_isc(g, c)
        && g.db.get(c).cmc <= 3
    {
        g.player_mut(p).t0s.foundry_mark = Some(c);
    }
    Ok(())
}

/// {3}{U}{U}, {T}: cast any number of the spells exiled with the Foundry free. Activate only as a sorcery.
fn foundry_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || post.is_none() || g.active != Some(p) || !can_tap(g, src) || !can_pay(g, p, 3, "UU", false)
    {
        return Ok(vec![]);
    }
    let pile = foundry_pile(g, p, src);
    let worth: f64 = pile.iter().filter_map(|&x| free_worth(g, p, x)).sum();
    if worth < 4.0 {
        return Ok(vec![]);
    }
    Ok(vec![ability(
        0.3 * worth.min(16.0),
        format!("{FOUNDRY}: cast {} exiled spells", pile.len()),
        src,
        foundry_go,
        0,
    )])
}

fn foundry_pile(g: &Game, p: PlayerId, src: PermId) -> Vec<CardId> {
    g.player(p)
        .t0s
        .foundry_exiled
        .iter()
        .find(|e| e.0 == src)
        .map(|e| e.1.iter().copied().filter(|x| g.player(p).exile.contains(x)).collect())
        .unwrap_or_default()
}

fn foundry_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_tap(g, src) || owner(g, src) != p || !can_pay(g, p, 3, "UU", false) {
        return Ok(false);
    }
    pay(g, p, 3, "UU", false)?;
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "cast the exiled spells free", None, None)? {
        return Ok(true);
    }
    let mut pile: Vec<(f64, CardId)> =
        foundry_pile(g, p, src).into_iter().filter_map(|x| free_worth(g, p, x).map(|w| (w, x))).collect();
    pile.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (_, x) in pile {
        if g.over || !g.player(p).alive || !g.player_mut(p).exile.take(x) {
            continue;
        }
        if let Some(e) = g.player_mut(p).t0s.foundry_exiled.iter_mut().find(|e| e.0 == src) {
            remove_card(&mut e.1, x);
        }
        free_cast(g, p, x, true)?; // cast from exile: exiled again unless it is a copy
    }
    Ok(true)
}

// ================================================================== Leyline Dowser
const DOWSER: &str = "Leyline Dowser";

/// {1}, {T}: mill a card; you may put an instant or sorcery card milled this way into your hand. Tap an untapped
/// legendary creature you control: untap the Dowser.
fn dowser_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || !can_tap(g, src) || !can_pay(g, p, 1, "", false) || g.player(p).library.len() < 12 {
        return Ok(vec![]);
    }
    // a hole in the mana: at the end of an opponent's turn, or after combat on your own with the mana to spare
    let spare = total_mana(g, p, false);
    let ok = post.is_none() && g.active != Some(p) || post == Some(true) && spare >= 2;
    if !ok {
        return Ok(vec![]);
    }
    let arg = (post.is_none() || post == Some(true)) as i64;
    Ok(vec![ability(1.1 + 0.1 * isc_gy(g, p).min(5) as f64, "Leyline Dowser: mill".into(), src, dowser_go, arg)])
}

fn dowser_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let mut done = false;
    for _ in 0..3 {
        if !can_tap(g, src) || !can_pay(g, p, 1, "", false) || g.player(p).library.len() < 8 {
            break;
        }
        pay(g, p, 1, "", false)?;
        g.perm_mut(src).tapped = true;
        done = true;
        if ability_window(g, p, Some(src), "mill a card", None, None)? {
            let top = g.player(p).library.last().copied();
            mill(g, p, 1)?;
            if let Some(c) = top
                && is_isc(g, c)
                && g.player(p).gy.last() == Some(&c)
            {
                g.player_mut(p).gy.pop();
                g.player_mut(p).hand.push(c);
                crate::glog!(g, "    Leyline Dowser puts {} into the hand", cname(g, c));
            }
        }
        // untap by tapping a legendary creature that has nothing better to do
        if arg == 0 || !can_pay(g, p, 1, "", false) {
            break;
        }
        let legs: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                let x = g.perm(m);
                x.on_bf && !x.tapped && !x.phased && g.is_creature(m) && crate::impls::legends::legendary(g, m)
            })
            .collect();
        let Some(l) = min_by(&legs, |m| pval(g, m)) else { break };
        g.perm_mut(l).tapped = true;
        g.perm_mut(src).tapped = false;
    }
    Ok(done)
}

// ================================================================== Midnight Clock
const CLOCK: &str = "Midnight Clock";

fn hours(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::T0Stella)
}

/// put hour counters on the Clock; the twelfth shuffles your hand and graveyard into your library, draws seven and
/// exiles the Clock
fn add_hours(g: &mut Game, m: PermId, n: i64) -> Res {
    for _ in 0..n {
        if !g.perm(m).on_bf {
            return Ok(());
        }
        let h = hours(g, m) + 1;
        g.perm_mut(m).data.set(DataKey::T0Stella, Val::Int(h));
        if h == 12 {
            let o = owner(g, m);
            if trigger_window(g, o, Some(m), "shuffle your hand and graveyard into your library, draw seven", None)? {
                let pl = g.player_mut(o);
                let (hand, gy) = (std::mem::take(&mut pl.hand), std::mem::take(&mut pl.gy));
                pl.library.extend(hand);
                pl.library.extend(gy);
                shuffle_library(g, o);
                draw(g, o, 7, false)?;
                exile_perm(g, m)?;
            }
            return Ok(());
        }
    }
    Ok(())
}

/// At the beginning of each upkeep, put an hour counter on it.
fn clock_upkeep(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "an hour counter", None)? {
        add_hours(g, src, 1)?;
    }
    Ok(())
}

/// {2}{U}: an hour counter. The AI winds it when the wheel helps (a nearly empty hand) and the mana is spare.
fn clock_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p
        || !can_pay(g, p, 2, "U", false)
        || !(post.is_none() && g.active != Some(p) || post == Some(true))
    {
        return Ok(vec![]);
    }
    let (h, hand) = (hours(g, src), g.player(p).hand.len());
    let wheel = h + 1 == 12;
    if (wheel && hand > 2) || (!wheel && (hand > 2 || h < 3)) {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0 + 0.2 * h as f64, format!("Midnight Clock: hour {}", h + 1), src, clock_go, 0)])
}

fn clock_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !g.perm(src).on_bf || !can_pay(g, p, 2, "U", false) {
        return Ok(false);
    }
    pay(g, p, 2, "U", false)?;
    if ability_window(g, p, Some(src), "an hour counter", None, None)? {
        add_hours(g, src, 1)?;
    }
    Ok(true)
}

// ================================================================== Octavia, Living Thesis
const OCTAVIA: &str = "Octavia, Living Thesis";

/// costs {8} less with eight or more instant and/or sorcery cards in your graveyard
fn octavia_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if isc_gy(g, p) >= 8 { -8 } else { 0 }
}

// ================================================================== Pteramander
const PTERAMANDER: &str = "Pteramander";

/// {7}{U}: adapt 4 (four +1/+1 counters if it has none); costs {1} less for each instant and sorcery in your
/// graveyard
fn pteramander_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || post.is_none() || g.perm(src).plus != 0 || !g.perm(src).on_bf {
        return Ok(vec![]);
    }
    let gn = 7u32.saturating_sub(isc_gy(g, p) as u32);
    if !can_pay(g, p, gn, "U", false) {
        return Ok(vec![]);
    }
    // at the end of an opponent's turn, or after combat on your own
    if !(post == Some(true) || g.active != Some(p)) {
        return Ok(vec![]);
    }
    Ok(vec![ability(2.6, "Pteramander: adapt 4".into(), src, pteramander_go, 0)])
}

fn pteramander_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let gn = 7u32.saturating_sub(isc_gy(g, p) as u32);
    if !g.perm(src).on_bf || g.perm(src).plus != 0 || !can_pay(g, p, gn, "U", false) {
        return Ok(false);
    }
    pay(g, p, gn, "U", false)?;
    if ability_window(g, p, Some(src), "adapt 4", None, None)? && g.perm(src).on_bf && g.perm(src).plus == 0 {
        repl::put_plus(g, src, 4);
    }
    Ok(true)
}

// ================================================================== plot: Lock and Load, Pyretic Charge
const LOCK: &str = "Lock and Load";
const CHARGE: &str = "Pyretic Charge";

/// the turn a card was plotted (`impulse_long` marks, `u32::MAX - 1 - turn`, as green_cards.rs keeps plotted cards)
fn plotted_turn(g: &Game, p: PlayerId, c: CardId) -> Option<u32> {
    g.player(p)
        .impulse_long
        .iter()
        .find(|x| x.0 == c && x.1 < u32::MAX && x.1 > u32::MAX / 2)
        .map(|x| u32::MAX - 1 - x.1)
}

fn plot_cost(name: &str) -> (u32, &'static str) {
    if name == LOCK { (3, "U") } else { (3, "R") }
}

/// what a card is worth right now when cast free from plot
fn plot_value(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let name = cname(g, c);
    if name == LOCK {
        let n = crate::impls::rules2::casts_isc(g, p) as f64;
        if n >= 2.0 { 2.0 + 0.9 * n } else { 0.0 }
    } else {
        let hand = g.player(p).hand.iter().filter(|&&x| x != c && !g.db.get(x).land).count();
        if hand <= 1 { 3.0 } else { 0.0 }
    }
}

/// plot (after combat, mana spare) and the free cast on a later turn; the plotted card's normal priority is 0
fn plot_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(post) = post else { return Ok(vec![]) };
    if g.active != Some(p) {
        return Ok(vec![]);
    }
    let turns = g.player(p).turns;
    let name = cname(g, c).to_string();
    let (gn, pips) = plot_cost(&name);
    Ok(match plotted_turn(g, p, c) {
        Some(t) if turns > t => {
            let u = plot_value(g, p, c);
            if u <= 0.0 { vec![] } else { vec![plan(u, format!("{name} (plotted, free)"), plot_free, c.0 as i64)] }
        }
        Some(_) => vec![],
        None if post && can_pay(g, p, gn, pips, false) => {
            vec![plan(1.35, format!("plot {name}"), plot_go, c.0 as i64)]
        }
        None => vec![],
    })
}

fn plot_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (gn, pips) = plot_cost(cname(g, c));
    if !g.player(p).hand.contains(&c) || plotted_turn(g, p, c).is_some() || !can_pay(g, p, gn, pips, false) {
        return Ok(false);
    }
    if !pay(g, p, gn, pips, false)? {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    let mark = u32::MAX - 1 - pl.turns;
    pl.impulse_long.push((c, mark));
    crate::glog!(g, "  {} plots {}", pname(g, p), cname(g, c));
    Ok(true)
}

fn plot_free(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || plotted_turn(g, p, c).is_none_or(|t| g.player(p).turns <= t) {
        return Ok(false);
    }
    g.player_mut(p).impulse_long.retain(|x| x.0 != c);
    crate::glog!(g, "  {} casts the plotted {} for free", pname(g, p), cname(g, c));
    g.last_x = 0;
    cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

/// a plotted card waits for its free cast
fn plot_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    if plotted_turn(g, p, c).is_some() {
        return 0;
    }
    if cname(g, c) == LOCK {
        let n = crate::impls::rules2::casts_isc(g, p);
        if n >= 2 { 40 + 8 * n.min(4) } else { 0 }
    } else {
        let hand = g.player(p).hand.iter().filter(|&&x| x != c && !g.db.get(x).land).count();
        if hand <= 1 { 52 } else { 0 }
    }
}

/// Draw a card, then draw a card for each other instant and sorcery spell you've cast this turn.
fn lock_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let others = (crate::impls::rules2::casts_isc(g, p) - 1).max(0) as u32;
    draw(g, p, 1 + others, false)?;
    Ok(intern("gy"))
}

/// Discard your hand, then draw four cards. For each card discarded this way, creatures you control get +1/+0 until
/// end of turn.
fn charge_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let hand = g.player(p).hand.clone();
    let n = hand.len() as i32;
    discard_cards(g, p, &hand)?;
    draw(g, p, 4, false)?;
    for m in g.player(p).perms.clone() {
        if g.perm(m).on_bf && g.is_creature(m) {
            g.perm_mut(m).eot_pt.0 += n;
        }
    }
    Ok(intern("gy"))
}

// ================================================================== Radical Idea (jump-start)
const RADICAL: &str = "Radical Idea";

/// Jump-start: cast it from the graveyard by discarding a card in addition to its cost; it is exiled afterwards
fn radical_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() && g.active == Some(p) {
        return Ok(vec![]);
    }
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 1, "U", false) || discard_choice(g, p).is_none() {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.7, "Radical Idea (jump-start)".into(), radical_go, c.0 as i64)])
}

/// the card to discard: a spare land when the hand is land-heavy, else the card worth the least
fn discard_choice(g: &Game, p: PlayerId) -> Option<CardId> {
    let pl = g.player(p);
    let lands = pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
    let cands: Vec<CardId> = pl.hand.iter().copied().filter(|&x| !(g.db.get(x).land && lands <= 2)).collect();
    min_by(&cands, |x| card_worth(g, p, x, false) + if g.db.get(x).land { -5.0 } else { 0.0 })
}

fn radical_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let Some(d) = discard_choice(g, p) else { return Ok(false) };
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 1, "U", false) {
        return Ok(false);
    }
    pay(g, p, 1, "U", false)?;
    discard_cards(g, p, &[d])?;
    cast_card(g, p, c, "gy", Ctx::default())?; // from the graveyard: exiled after it resolves
    Ok(true)
}

// ================================================================== Shark Typhoon
const TYPHOON: &str = "Shark Typhoon";

fn shark(g: &mut Game, p: PlayerId, x: i32) -> Res {
    if x <= 0 {
        return Ok(());
    }
    let spec = Tokens {
        fly: true,
        color: Some(Colors::from_letters("U")),
        types: vec!["shark"],
        tgh: Some(x),
        ..Tokens::new(1, x)
    };
    make_tokens(g, p, spec)?;
    Ok(())
}

/// Whenever you cast a noncreature spell, create an X/X blue Shark creature token with flying, X that spell's mana
/// value (X is 0 on the stack)
fn typhoon_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    let d = g.db.get(c);
    if caster != o || d.creature || d.land || d.cmc == 0 {
        return Ok(());
    }
    let x = d.cmc as i32;
    if trigger_window(g, o, Some(src), &format!("a {x}/{x} flying Shark"), None)? {
        shark(g, o, x)?;
    }
    Ok(())
}

/// Cycling {X}{1}{U}: discard it, draw a card, and when you cycle it, create an X/X Shark with flying. At the end of an
/// opponent's turn, or after combat, with the mana to spare.
fn typhoon_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !g.player(p).hand.contains(&c) || !(post.is_none() && g.active != Some(p) || post == Some(true)) {
        return Ok(vec![]);
    }
    let x = total_mana(g, p, false) as i32 - 2;
    if x < 3 {
        return Ok(vec![]);
    }
    Ok(vec![plan(0.8 + 0.5 * x.min(8) as f64, format!("cycle Shark Typhoon (X = {x})"), typhoon_cycle, c.0 as i64)])
}

fn typhoon_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let x = total_mana(g, p, false) as i32 - 2;
    if !g.player(p).hand.contains(&c) || x < 1 || !can_pay(g, p, 1 + x as u32, "U", false) {
        return Ok(false);
    }
    pay(g, p, 1 + x as u32, "U", false)?;
    crate::glog!(g, "  {} cycles Shark Typhoon (X = {x})", pname(g, p));
    discard_cards(g, p, &[c])?;
    if trigger_window(g, p, None, &format!("cycle: a {x}/{x} flying Shark"), Some(4.0))? {
        shark(g, p, x)?;
    }
    draw(g, p, 1, false)?;
    Ok(true)
}

// ================================================================== Smoldering Stagecoach
const STAGECOACH: &str = "Smoldering Stagecoach";

fn as_selfpt(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true;
    Ok(())
}

/// power equal to the instant and sorcery cards in your graveyard (toughness 5 is in the printed card)
fn stagecoach_pt(g: &Game, p: PlayerId, m: PermId) -> (i32, i32) {
    if g.perm(m).data.truthy(DataKey::Anim) { (isc_gy(g, p) as i32, 0) } else { (0, 0) }
}

/// Crew 2: before the first combat, tap creatures (summoning-sick ones first) to make it a creature this turn when it
/// can attack and its power is worth it
fn stagecoach_crew(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if owner(g, src) != p || g.perm(src).tapped || g.perm(src).sick || isc_gy(g, p) < 2 {
        return Ok(());
    }
    let mut cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && g.is_creature(m) && !g.perm(m).tapped && !g.perm(m).phased)
        .collect();
    cands.sort_by(|&a, &b| {
        (g.perm(b).sick, -pval(g, b)).partial_cmp(&(g.perm(a).sick, -pval(g, a))).unwrap_or(std::cmp::Ordering::Equal)
    });
    let (mut power, mut pick) = (0, vec![]);
    for m in cands {
        if power >= 2 {
            break;
        }
        // an attacker or a better creature than the Stagecoach stays home to attack
        if g.perm(m).sick || epow(g, m) <= 2 || g.perm(m).token {
            pick.push(m);
            power += epow(g, m);
        }
    }
    if power < 2 || !crate::impls::lands::open_attack(g, p, isc_gy(g, p) as i32, false) {
        return Ok(());
    }
    for m in pick {
        g.perm_mut(m).tapped = true;
    }
    let now = Val::Stamp(g.turn_stamp());
    let x = g.perm_mut(src);
    x.data.set(DataKey::Anim, Val::Bool(true));
    (x.pow, x.tgh) = (0, 5);
    x.data.set(DataKey::Crewed, now);
    crate::glog!(g, "  {} crews Smoldering Stagecoach", pname(g, p));
    Ok(())
}

/// Whenever this Vehicle attacks, the next instant spell and the next sorcery spell you cast this turn each have
/// cascade.
fn stagecoach_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) == p
        && atk.contains(&src)
        && trigger_window(g, p, Some(src), "the next instant and sorcery have cascade", None)?
    {
        let st = g.turn_stamp();
        let t = &mut g.player_mut(p).t0s;
        t.cascade_instant = Some(st);
        t.cascade_sorcery = Some(st);
    }
    Ok(vec![])
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // Stella Lee, Wild Card
    r.card(db, STELLA)?.cast = Some(stella_cast);
    // Thunderclap Drake (its cost reduction is the compiled static)
    r.card(db, DRAKE)?.options = Some(drake_options);
    // Kaza
    r.card(db, KAZA)?.options = Some(kaza_options);
    // Crackling Spellslinger
    let c = r.card(db, "Crackling Spellslinger")?;
    c.etb = Some(spellslinger_etb);
    c.prio = Some(spellslinger_prio);
    // Elemental Eruption
    let c = r.card(db, ERUPTION)?;
    c.self_cast = Some(eruption_cast);
    c.resolve = Some(eruption_resolve);
    c.prio = Some(eruption_prio);
    // Volcanic Torrent
    let c = r.card(db, TORRENT)?;
    c.self_cast = Some(torrent_cast);
    c.resolve = Some(torrent_resolve);
    c.prio = Some(torrent_prio);
    // Arcane Bombardment
    r.card(db, BOMBARD)?.cast = Some(bombard_cast);
    // Baral's Expertise
    let c = r.card(db, BARAL)?;
    c.resolve = Some(baral_resolve);
    c.prio = Some(baral_prio);
    // X spells: their own options choose X
    let c = r.card(db, SWINE)?;
    c.resolve = Some(swine_resolve);
    c.prio = Some(zero_prio);
    c.hand_options = Some(swine_options);
    let c = r.card(db, EXPERIMENT)?;
    c.resolve = Some(experiment_resolve);
    c.prio = Some(zero_prio);
    c.hand_options = Some(experiment_options);
    let c = r.card(db, PROMISE)?;
    c.resolve = Some(promise_resolve);
    c.prio = Some(zero_prio);
    c.hand_options = Some(promise_options);
    let c = r.card(db, REVELATION)?;
    c.resolve = Some(revelation_resolve);
    c.prio = Some(zero_prio);
    c.hand_options = Some(revelation_options);
    // Rousing Refrain
    let c = r.card(db, REFRAIN)?;
    c.resolve = Some(refrain_resolve);
    c.prio = Some(zero_prio);
    c.hand_options = Some(refrain_options);
    // Bloodthirsty Adversary
    r.card(db, ADVERSARY)?.etb = Some(adversary_etb);
    // Cursed Mirror
    let c = r.card(db, MIRROR)?;
    c.etb = Some(mirror_etb);
    c.end_step = Some(mirror_end);
    // Forger's Foundry
    let c = r.card(db, FOUNDRY)?;
    c.mana_tapped = Some(foundry_tapped);
    c.options = Some(foundry_options);
    // Leyline Dowser
    r.card(db, DOWSER)?.options = Some(dowser_options);
    // Midnight Clock
    let c = r.card(db, CLOCK)?;
    c.upkeep = Some(clock_upkeep);
    c.options = Some(clock_options);
    // Octavia (its magecraft is engine::cast::magecraft's `magecraft_extra`; ward 8 is in the card data)
    r.card(db, OCTAVIA)?.self_cost = Some(octavia_cost);
    // Pteramander
    r.card(db, PTERAMANDER)?.options = Some(pteramander_options);
    // plot
    let c = r.card(db, LOCK)?;
    c.resolve = Some(lock_resolve);
    c.prio = Some(plot_prio);
    c.hand_options = Some(plot_options);
    let c = r.card(db, CHARGE)?;
    c.resolve = Some(charge_resolve);
    c.prio = Some(plot_prio);
    c.hand_options = Some(plot_options);
    // Radical Idea
    r.card(db, RADICAL)?.gy_options = Some(radical_options);
    // Shark Typhoon
    let c = r.card(db, TYPHOON)?;
    c.cast = Some(typhoon_cast);
    c.hand_options = Some(typhoon_options);
    // Smoldering Stagecoach
    let c = r.card(db, STAGECOACH)?;
    c.self_pt = Some(stagecoach_pt);
    c.as_enters = Some(as_selfpt);
    c.crew = Some(stagecoach_crew);
    c.attack = Some(stagecoach_attack);
    // Mizzix's Mastery: the pool decks' option (casual combos)
    r.card(db, MASTERY)?.hand_options = Some(mastery_options);
    r.card(db, "Galvanic Iteration")?.prio = Some(galvanic_prio);
    Ok(())
}
