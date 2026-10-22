//! Threats, removal and wipes of JD's Karn, Legacy Reforged deck (2026-10-20). The audit verdicts are in
//! data/card_overrides.toml; each function's doc comment says how the card is modeled and what is left out.
//!
//! - **Wipes** (Nevinyrral's Disk, Oblivion Stone, Ratchet Bomb, All Is Dust): one rule per wipe, [`Wipe::hits`], read
//!   by both the AI's valuation ([`eval`]) and the resolution ([`run`]), so what the AI expects is what happens. All
//!   Is Dust is the `dust` kind of `sweepers.rs` (tag `wipe=dust`), so the usual wipe options cast it.
//! - **Static**: Darksteel Forge (indestructible artifacts, `grant_kw`), Platinum Angel (`t0_giada::cant_lose`),
//!   Ugin's cost reduction (`cost`), Liberator's flash (`engine::cast::grants_flash`), Blightsteel Colossus (infect
//!   in `engine/combat.rs`, shuffle-back in `engine::zones::to_zone_card`).
//! - **Creatures and artifacts**: Duplicant, Kuldotha Forgemaster, Mishra's Self-Replicator, Steel Overseer, Steel
//!   Hellkite, Triplicate Titan, Cityscape Leveler, Spine of Ish Sah, Portal to Phyrexia, Unstable Obelisk and Ugin.

use super::common::{WalkerAb, best_opp_nonland, make_powerstone, walker};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::life::check_state;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::shuffle_library;
use crate::engine::values::{card_name, colors_of, epow, indestructible, ptypes, pval, untargetable};
use crate::engine::zones::{Enter, Tokens, die, enter, enter_token_copy, exile_perm, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn name_of(g: &Game, m: PermId) -> &str {
    card_name(g, m).unwrap_or("token")
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg }) }
}

/// an artifact permanent (a token made as an artifact creature has "artifact" among its types)
pub fn is_art(g: &Game, m: PermId) -> bool {
    ptypes(g, m).has(Types::ARTIFACT) || g.perm(m).ttypes.contains(&"artifact")
}

/// every live permanent, with its controller (lands are not permanents here: they are `Land`s)
fn all_perms(g: &Game) -> Vec<(PlayerId, PermId)> {
    g.players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().map(move |&m| (q.id, m)))
        .filter(|&(_, m)| live(g, m))
        .collect()
}

fn mv(g: &Game, m: PermId) -> i32 {
    g.perm(m).cd.map_or(0, |c| g.db.get(c).cmc as i32)
}

/// what losing m is worth: its value to removal, with a floor (a mana rock or a token still counts for something)
fn worth(g: &Game, m: PermId) -> f64 {
    let floor = if g.perm(m).token { 0.3 } else { 0.6 + 0.2 * mv(g, m).min(6) as f64 };
    pval(g, m).max(floor)
}

fn fate(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Fate)
}

// ================================================================== the wipes
/// Which permanents a wipe hits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Wipe {
    /// Nevinyrral's Disk: destroy all artifacts, creatures and enchantments
    Disk,
    /// Oblivion Stone: destroy each nonland permanent without a fate counter
    Stone,
    /// Ratchet Bomb: destroy each nonland permanent with this mana value
    Bomb(i32),
    /// All Is Dust: each player sacrifices every coloured permanent
    Dust,
}

impl Wipe {
    pub fn label(self) -> &'static str {
        match self {
            Wipe::Disk => "Nevinyrral's Disk",
            Wipe::Stone => "Oblivion Stone",
            Wipe::Bomb(_) => "Ratchet Bomb",
            Wipe::Dust => "All Is Dust",
        }
    }

    /// does it hit m? (m is a live nonland permanent)
    pub fn hits(self, g: &Game, m: PermId) -> bool {
        match self {
            Wipe::Disk => is_art(g, m) || g.is_creature(m) || ptypes(g, m).has(Types::ENCHANTMENT),
            Wipe::Stone => fate(g, m) == 0,
            Wipe::Bomb(n) => mv(g, m) == n,
            Wipe::Dust => !colors_of(g, m).is_empty(),
        }
    }

    /// destroy (indestructible survives) or sacrifice (nothing survives)
    fn sacrifices(self) -> bool {
        self == Wipe::Dust
    }

    /// does it really remove m?
    pub fn removes(self, g: &Game, m: PermId) -> bool {
        self.hits(g, m) && (self.sacrifices() || !indestructible(g, m))
    }
}

/// what the opponents lose and what p loses to this wipe, `skip` (the wipe's own source, gone first) aside
pub fn eval(g: &Game, p: PlayerId, w: Wipe, skip: Option<PermId>) -> (f64, f64) {
    let (mut ol, mut ml) = (0.0, 0.0);
    for (q, m) in all_perms(g) {
        if Some(m) == skip || !w.removes(g, m) {
            continue;
        }
        let v = worth(g, m);
        if q == p {
            ml += if g.perm(m).is_cmd { v + 2.0 } else { v };
        } else {
            ol += v;
        }
    }
    (ol, ml)
}

/// the utility of a wipe with this swing (on brain.wipe_options' scale)
fn wipe_util(ol: f64, ml: f64) -> f64 {
    (ol - 1.2 * ml) / 2.2 - 2.0
}

/// resolve the wipe for p: every permanent it hits, all at once (Python's `apply_wipe` framing)
pub fn run(g: &mut Game, p: PlayerId, w: Wipe, skip: Option<PermId>) -> Res {
    crate::glog!(g, "    {} resolves", w.label());
    g.batch += 1; // creatures destroyed together die simultaneously
    let prev_batch = g.cur_batch.replace(g.batch);
    let prev = g.destroyer.replace(p);
    g.resolving += 1;
    let r = run_inner(g, p, w, skip);
    g.cur_batch = prev_batch;
    g.destroyer = prev;
    g.resolving -= 1;
    r?;
    if g.resolving == 0 && !g.trig_queue.is_empty() {
        crate::engine::stack::flush_triggers(g)?;
    }
    Ok(())
}

fn run_inner(g: &mut Game, _p: PlayerId, w: Wipe, skip: Option<PermId>) -> Res {
    let doomed: Vec<PermId> =
        all_perms(g).into_iter().map(|x| x.1).filter(|&m| Some(m) != skip && w.hits(g, m)).collect();
    for m in doomed {
        if g.over {
            break;
        }
        if live(g, m) {
            let nm = name_of(g, m).to_string();
            die(g, m, if w.sacrifices() { "sac" } else { "destroy" })?;
            if !g.perm(m).on_bf {
                crate::glog!(g, "      {} ({}) is removed by {}", nm, g.player(g.perm(m).owner).name, w.label());
            }
        }
    }
    if w == Wipe::Stone {
        for (_, m) in all_perms(g) {
            g.perm_mut(m).data.remove(DataKey::Fate); // remove all fate counters from all permanents
        }
    }
    check_state(g)
}

/// All Is Dust, as `sweepers.rs` calls it (the `dust` kind): what the opponents lose, what the caster loses
pub fn dust_eval(g: &Game, p: PlayerId) -> (f64, f64, Option<PlayerId>) {
    let (ol, ml) = eval(g, p, Wipe::Dust, None);
    (ol, ml, None)
}

pub fn dust_resolve(g: &mut Game, p: PlayerId) -> Res {
    run(g, p, Wipe::Dust, None)
}

pub fn dust_hits(g: &Game, m: PermId) -> bool {
    Wipe::Dust.hits(g, m)
}

// ------------------------------------------------------------------ Nevinyrral's Disk
/// it enters tapped
fn disk_enters(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    g.perm_mut(m).tapped = true;
    Ok(())
}

fn disk_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || g.perm(src).tapped || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 1, "", false) {
        return Ok(vec![]);
    }
    let (ol, ml) = eval(g, p, Wipe::Disk, None);
    let u = wipe_util(ol, ml);
    if ol < 4.0 || u < 0.8 {
        return Ok(vec![]);
    }
    Ok(vec![ability(u.min(9.0), "Nevinyrral's Disk", src, disk_go, 0)])
}

/// {1}, {T}: destroy all artifacts, creatures and enchantments
pub(crate) fn disk_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped || !pay(g, p, 1, "", false)? {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "destroy all artifacts, creatures and enchantments", Some(8.0), None)? {
        run(g, p, Wipe::Disk, None)?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ Oblivion Stone
/// the best permanent of p's to put a fate counter on (the one that is worth most, not already marked)
fn fate_target(g: &Game, p: PlayerId, src: PermId) -> Option<PermId> {
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && live(g, m) && fate(g, m) == 0 && worth(g, m) >= 1.5)
        .collect();
    max_by(&cands, |m| worth(g, m))
}

/// the swing the Stone's wipe would have now (its own sacrifice aside)
fn stone_swing(g: &Game, p: PlayerId, src: PermId) -> (f64, f64) {
    eval(g, p, Wipe::Stone, Some(src))
}

fn stone_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || g.perm(src).tapped || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    let mut out = vec![];
    let (ol, ml) = stone_swing(g, p, src);
    let u = wipe_util(ol, ml);
    if can_pay(g, p, 5, "", false) && ol >= 5.0 && u >= 0.8 {
        out.push(ability(u.min(9.0), "Oblivion Stone (destroy)", src, stone_wipe, 0));
    }
    // a fate counter on our best permanent, when the wipe would be good with it (this turn's mana is spare)
    if can_pay(g, p, 4, "", false)
        && let Some(t) = fate_target(g, p, src)
    {
        let (ol2, ml2) = {
            let held = fate(g, t);
            let (ol, _) = (ol, ml);
            let _ = held;
            let ml2 = ml - if indestructible(g, t) { 0.0 } else { worth(g, t) };
            (ol, ml2)
        };
        let u2 = wipe_util(ol2, ml2);
        if ol2 >= 5.0 && u2 > u + 0.3 && u2 >= 0.8 {
            out.push(ability(0.6 + 0.4 * u2.min(5.0), "Oblivion Stone (fate counter)", src, stone_mark, t.0 as i64));
        }
    }
    Ok(out)
}

/// {4}, {T}: put a fate counter on target permanent
pub(crate) fn stone_mark(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped || !live(g, t) || !pay(g, p, 4, "", false)? {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "fate counter", None, Some(t))? && live(g, t) {
        let n = fate(g, t) + 1;
        g.perm_mut(t).data.set(DataKey::Fate, Val::Int(n));
        crate::glog!(g, "    a fate counter on {} (Oblivion Stone)", name_of(g, t));
    }
    Ok(true)
}

/// {5}, {T}, sacrifice: destroy each nonland permanent without a fate counter, then remove all fate counters
pub(crate) fn stone_wipe(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped || !pay(g, p, 5, "", false)? {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    die(g, src, "sac")?;
    if ability_window(g, p, Some(src), "destroy each nonland permanent without a fate counter", Some(8.0), None)? {
        run(g, p, Wipe::Stone, Some(src))?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ Ratchet Bomb
pub(crate) fn charges(g: &Game, m: PermId) -> i32 {
    g.perm(m).data.int(DataKey::Charge) as i32
}

/// the best wipe score the Bomb can reach with n counters, n from its count up to four more
fn bomb_plan(g: &Game, p: PlayerId, src: PermId) -> (i32, f64, f64) {
    let c = charges(g, src);
    let mut best = (c, 0.0, 0.0);
    let mut best_u = f64::NEG_INFINITY;
    for n in c..=(c + 4).min(9) {
        let (ol, ml) = eval(g, p, Wipe::Bomb(n), Some(src));
        let u = wipe_util(ol, ml) - 0.4 * (n - c) as f64; // each turn of waiting costs
        if ol >= 3.0 && u > best_u {
            best_u = u;
            best = (n, ol, ml);
        }
    }
    best
}

fn bomb_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || g.perm(src).tapped || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    let c = charges(g, src);
    let (n, ol, ml) = bomb_plan(g, p, src);
    let u = wipe_util(ol, ml);
    let mut out = vec![];
    if n == c && ol >= 3.0 && u >= 0.8 {
        out.push(ability(u.min(9.0), &format!("Ratchet Bomb (destroy mana value {c})"), src, bomb_blow, 0));
    } else if n > c || c < 3 {
        // wind it up: it is a free tap, and a bomb is only useful with the right count
        out.push(ability(0.7, "Ratchet Bomb (charge counter)", src, bomb_tick, 0));
    }
    Ok(out)
}

/// {T}: put a charge counter on it
pub(crate) fn bomb_tick(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "charge counter", None, None)? && live(g, src) {
        let n = charges(g, src) + 1;
        g.perm_mut(src).data.set(DataKey::Charge, Val::Int(n as i64));
        crate::glog!(g, "    Ratchet Bomb has {n} charge counter(s)");
    }
    Ok(true)
}

/// {T}, sacrifice: destroy each nonland permanent with mana value equal to the charge counters
pub(crate) fn bomb_blow(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    let n = charges(g, src);
    g.perm_mut(src).tapped = true;
    die(g, src, "sac")?;
    if ability_window(g, p, Some(src), &format!("destroy each nonland permanent with mana value {n}"), Some(7.0), None)?
    {
        run(g, p, Wipe::Bomb(n), Some(src))?;
    }
    Ok(true)
}

// ================================================================== static effects
/// **Darksteel Forge** (Full): artifacts you control have indestructible (`indestructible` reads `grant_kw`).
fn forge_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "indestructible" && owner(g, src) == g.perm(m).owner && is_art(g, m)
}

/// is p's Platinum Angel (or another source) keeping p in the game: `t0_giada::cant_lose` asks this
pub fn angel_holds(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| live(g, m) && !g.perm(m).neutered && &*g.perm(m).name == "Platinum Angel")
}

/// **Ugin, the Ineffable**: colourless spells you cast cost {2} less
fn ugin_cost(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    if caster == owner(g, src) && !d.land && d.colors.is_empty() && d.phyrexian.is_empty() { -2 } else { 0 }
}

/// **Liberator, Urza's Battlethopter**: you may cast colourless spells and artifact spells as though they had flash
/// (`engine::cast::grants_flash` asks this)
pub fn liberator_flash(g: &Game, p: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    !d.land
        && (d.types.has(Types::ARTIFACT) || d.colors.is_empty())
        && g.player(p).perms.iter().any(|&m| live(g, m) && !g.perm(m).neutered && &*g.perm(m).name == LIBERATOR)
}

const LIBERATOR: &str = "Liberator, Urza's Battlethopter";

/// Liberator: whenever you cast a spell, if the mana spent is greater than its power, +1/+1 counter on it
fn liberator_cast(g: &mut Game, src: Src, p: PlayerId, c: CardId) -> Res {
    let spent = g.cur_cast.as_ref().filter(|x| x.0 == c).map_or(0, |x| x.1.mana_spent) as i32; // the mana the payment spent
    if p != owner(g, src) || !live(g, src) || spent <= epow(g, src) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "+1/+1 counter", Some(2.0))? && live(g, src) {
        crate::cardcode::put_plus(g, src, 1);
        crate::glog!(g, "    Liberator grows (+1/+1 counter)");
    }
    Ok(())
}

/// **Blightsteel Colossus**'s infect: combat damage to a player is that many poison counters instead of life loss.
/// Called by `engine/combat.rs` for a creature with infect; true if it was applied.
pub fn infect_hit(g: &mut Game, a: PermId, d: PlayerId, dmg: i32) -> Res {
    g.player_mut(d).poison += dmg.max(0) as u32;
    crate::glog!(
        g,
        "    {} deals {} to {}: {} poison counters (infect)",
        name_of(g, a),
        dmg,
        g.player(d).name,
        g.player(d).poison
    );
    check_state(g)
}

/// a card that is shuffled into its owner's library instead of going to a graveyard (Blightsteel Colossus)
pub fn shuffles_back(g: &Game, cd: CardId) -> bool {
    &*g.db.get(cd).name == "Blightsteel Colossus"
}

// ================================================================== Steel Overseer
fn art_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m) && is_art(g, m)).collect()
}

fn overseer_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || owner(g, src) != p || g.active != Some(p) || !live(g, src) {
        return Ok(vec![]);
    }
    let x = g.perm(src);
    if x.tapped || x.sick || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    let n = art_creatures(g, p).len();
    Ok(vec![ability(1.2 + 0.4 * n as f64, "Steel Overseer", src, overseer_go, 0)])
}

/// {T}: put a +1/+1 counter on each artifact creature you control
pub(crate) fn overseer_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let x = g.perm(src);
    if !live(g, src) || owner(g, src) != p || x.tapped || x.sick {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "+1/+1 counter on each artifact creature", None, None)? {
        let ms = art_creatures(g, p);
        for &m in &ms {
            crate::cardcode::put_plus(g, m, 1);
        }
        crate::glog!(g, "    Steel Overseer: +1/+1 counter on {} artifact creature(s)", ms.len());
        g.bf_ver += 1;
    }
    Ok(true)
}

// ================================================================== Triplicate Titan
/// When it dies: three 3/3 colourless Golem artifact creature tokens, one with flying, one with vigilance and one with
/// trample.
fn titan_dies(g: &mut Game, src: Src, _m: PermId, _cause: Sym) -> Res {
    let p = g.perm(src).orig;
    if !trigger_window(g, p, Some(src), "three 3/3 Golems", Some(5.0))? {
        return Ok(());
    }
    for kind in 0..3 {
        let mut spec = Tokens::new(1, 3);
        spec.tgh = Some(3);
        spec.color = Some(Colors::NONE);
        spec.types = vec!["golem", "artifact"];
        spec.fly = kind == 0;
        if kind == 2 {
            spec.data.set(DataKey::Kws, Val::List(vec![Val::Str("trample")]));
        }
        for m in make_tokens(g, p, spec)? {
            if kind == 1 {
                g.perm_mut(m).vig = true;
            }
        }
    }
    Ok(())
}

// ================================================================== Duplicant
/// Imprint: when it enters, you may exile target nontoken creature; it has that card's power, toughness and creature
/// types. The AI exiles the opposing creature that is worth most (worth 3 or more).
fn duplicant_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let p = owner(g, src);
    if m != src {
        return Ok(());
    }
    let pick = |g: &Game| {
        super::common::best_opp_creature(g, p, |x| !g.perm(x).token && g.perm(x).cd.is_some())
            .filter(|&t| pval(g, t) >= 3.0)
    };
    if pick(g).is_none() || !trigger_window(g, p, Some(src), "exile a nontoken creature", Some(6.0))? {
        return Ok(());
    }
    let Some(t) = pick(g) else { return Ok(()) };
    let cd = g.perm(t).cd.unwrap();
    let (pw, th) = {
        let d = g.db.get(cd);
        (d.pow, d.tgh)
    };
    let types: Vec<Sym> = g.db.get(cd).subtypes.iter().map(|s| crate::sym::intern(s)).collect();
    crate::glog!(g, "    Duplicant exiles {}", name_of(g, t));
    exile_perm(g, t)?;
    if live(g, src) && g.player(g.perm(t).orig).exile.contains(&cd) {
        let x = g.perm_mut(src);
        x.pow = pw;
        x.tgh = th;
        for ty in types {
            if !x.ttypes.contains(&ty) {
                x.ttypes.push(ty);
            }
        }
        x.data.set(DataKey::Imprint, Val::Card(cd));
        g.bf_ver += 1;
    }
    Ok(())
}

// ================================================================== Kuldotha Forgemaster
/// the artifact card in p's library the Forgemaster would fetch (the most powerful), with its value
fn forge_pick(g: &Game, p: PlayerId) -> Option<(CardId, f64)> {
    let lib: Vec<CardId> =
        g.player(p).library.iter().copied().filter(|&c| g.db.get(c).types.has(Types::ARTIFACT)).collect();
    let v = |c: CardId| {
        let d = g.db.get(c);
        (d.bomb as f64).max(0.6 * d.cmc as f64)
    };
    let c = max_by(&lib, v)?;
    Some((c, v(c)))
}

/// what sacrificing m costs the Forgemaster: a token or a cheap rock is nearly free, a creature costs its size
fn fodder_cost(g: &Game, m: PermId) -> f64 {
    if g.perm(m).token {
        return 0.2;
    }
    0.4 + 0.25 * mv(g, m) as f64 + if g.is_creature(m) { 0.3 * epow(g, m).max(0) as f64 } else { 0.0 }
}

/// the three artifacts to sacrifice: the cheapest ones that are not the Forgemaster or the commander
fn forge_fodder(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    let mut v: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && live(g, m) && is_art(g, m) && !g.perm(m).is_cmd && !g.perm(m).phased)
        .collect();
    v.sort_by(|&a, &b| fodder_cost(g, a).total_cmp(&fodder_cost(g, b)));
    v.truncate(3);
    v
}

fn forgemaster_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || owner(g, src) != p || g.active != Some(p) || !live(g, src) {
        return Ok(vec![]);
    }
    let x = g.perm(src);
    if x.tapped || x.sick || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    let fodder = forge_fodder(g, p, src);
    let Some((_, v)) = forge_pick(g, p) else { return Ok(vec![]) };
    if fodder.len() < 3 {
        return Ok(vec![]);
    }
    let cost: f64 = fodder.iter().map(|&m| fodder_cost(g, m)).sum();
    let swing = v - 0.8 * cost;
    if swing < 2.0 {
        return Ok(vec![]);
    }
    Ok(vec![ability((swing / 2.0).clamp(1.0, 8.0), "Kuldotha Forgemaster", src, forgemaster_go, 0)])
}

/// {T}, sacrifice three artifacts: search your library for an artifact card, put it onto the battlefield, then shuffle
fn forgemaster_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let x = g.perm(src);
    if !live(g, src) || owner(g, src) != p || x.tapped || x.sick {
        return Ok(false);
    }
    let fodder = forge_fodder(g, p, src);
    let Some((c, _)) = forge_pick(g, p) else { return Ok(false) };
    if fodder.len() < 3 {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    for &m in &fodder {
        die(g, m, "sac")?;
    }
    if ability_window(g, p, Some(src), "search for an artifact", Some(6.0), None)? {
        let pl = g.player_mut(p);
        if let Some(i) = pl.library.iter().position(|&x| x == c) {
            pl.library.remove(i);
            crate::glog!(g, "    Kuldotha Forgemaster fetches {}", g.db.get(c).name);
            enter(g, p, c, Enter::default())?;
        }
        shuffle_library(g, p);
    }
    Ok(true)
}

// ================================================================== Mishra's Self-Replicator
const REPLICATOR: &str = "Mishra's Self-Replicator";

fn historic(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.types.has(Types::ARTIFACT) || d.tag(Tag::Leg) || d.has_subtype("saga")
}

/// Whenever you cast a historic spell, you may pay {1}: create a token that's a copy of this creature. Every copy
/// triggers for the spell (the AI pays whenever it can).
fn replicator_cast(g: &mut Game, src: Src, p: PlayerId, c: CardId) -> Res {
    if p != owner(g, src) || !live(g, src) || !historic(g, c) || !can_pay(g, p, 1, "", false) {
        return Ok(());
    }
    if g.perm(src).token && g.player(p).perms.iter().filter(|&&m| g.perm(m).token).count() >= 30 {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "pay {1}: copy Mishra's Self-Replicator", Some(3.0))? || !live(g, src) {
        return Ok(());
    }
    if !pay(g, p, 1, "", false)? {
        return Ok(());
    }
    let cd = g.db.id(REPLICATOR).expect("Mishra's Self-Replicator");
    if enter_token_copy(g, p, cd)?.is_some() {
        crate::glog!(g, "    {} pays {{1}}: a copy of Mishra's Self-Replicator", g.player(p).name);
    }
    Ok(())
}

// ================================================================== Portal to Phyrexia
/// When it enters, each opponent sacrifices three creatures of their choice (their three least valuable).
fn portal_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let p = owner(g, src);
    if m != src || !trigger_window(g, p, Some(src), "each opponent sacrifices three creatures", Some(8.0))? {
        return Ok(());
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        let mut cs: Vec<PermId> =
            g.player(q).perms.iter().copied().filter(|&x| live(g, x) && g.is_creature(x)).collect();
        cs.sort_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)));
        for x in cs.into_iter().take(3) {
            crate::glog!(g, "    {} sacrifices {} (Portal to Phyrexia)", g.player(q).name, name_of(g, x));
            die(g, x, "sac")?;
        }
    }
    Ok(())
}

/// the creature card to reanimate: the best in any graveyard, with the graveyard's owner
fn portal_pick(g: &Game, p: PlayerId) -> Option<(CardId, PlayerId)> {
    let mut cands: Vec<(CardId, PlayerId)> = vec![];
    for q in g.players.iter().filter(|q| q.alive || q.id == p) {
        for &c in &q.gy {
            if g.db.get(c).creature {
                cands.push((c, q.id));
            }
        }
    }
    let v = |&(c, _): &(CardId, PlayerId)| {
        let d = g.db.get(c);
        (d.bomb as f64).max(1.0 + 0.5 * d.pow as f64).max(0.4 * d.cmc as f64)
    };
    max_by(&cands, |x| v(&x))
}

/// At the beginning of your upkeep, put target creature card from a graveyard onto the battlefield under your
/// control. It's a Phyrexian in addition to its other types.
fn portal_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !live(g, src) || portal_pick(g, p).is_none() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "reanimate a creature from a graveyard", Some(6.0))? {
        return Ok(());
    }
    let Some((c, o)) = portal_pick(g, p) else { return Ok(()) };
    let pl = g.player_mut(o);
    let Some(i) = pl.gy.iter().position(|&x| x == c) else { return Ok(()) };
    pl.gy.remove(i);
    let how = Enter { orig: (o != p).then_some(o), ..Enter::default() };
    let m = enter(g, p, c, how)?;
    let ty: Sym = "phyrexian";
    if !g.perm(m).ttypes.contains(&ty) {
        g.perm_mut(m).ttypes.push(ty);
    }
    crate::glog!(
        g,
        "    Portal to Phyrexia returns {} to the battlefield under {}'s control",
        name_of(g, m),
        g.player(p).name
    );
    Ok(())
}

// ================================================================== Spine of Ish Sah
/// When it enters, destroy target permanent: the opposing permanent worth most (3 or more).
fn spine_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let p = owner(g, src);
    if m != src {
        return Ok(());
    }
    let pick = |g: &Game| best_opp_nonland(g, p, |x| !indestructible(g, x)).filter(|&t| pval(g, t) >= 2.0);
    if pick(g).is_none() || !trigger_window(g, p, Some(src), "destroy target permanent", Some(6.0))? {
        return Ok(());
    }
    if let Some(t) = pick(g) {
        crate::engine::removal::apply_removal(g, Some(p), t, "destroy", g.perm(src).cd)?;
    }
    Ok(())
}

/// When it is put into a graveyard from the battlefield, return it to its owner's hand.
fn spine_dies(g: &mut Game, src: Src, _m: PermId, _cause: Sym) -> Res {
    let o = g.perm(src).orig;
    let Some(cd) = g.perm(src).cd else { return Ok(()) };
    if g.perm(src).token || !g.player(o).gy.contains(&cd) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "return to its owner's hand", Some(3.0))? {
        return Ok(());
    }
    let pl = g.player_mut(o);
    if let Some(i) = pl.gy.iter().position(|&x| x == cd) {
        pl.gy.remove(i);
        pl.hand.push(cd);
        crate::glog!(g, "    Spine of Ish Sah returns to {}'s hand", g.player(o).name);
    }
    Ok(())
}

// ================================================================== Unstable Obelisk
/// the permanent {7}, {T}, sacrifice destroys: the opposing permanent worth most (4 or more)
fn obelisk_target(g: &Game, p: PlayerId) -> Option<PermId> {
    best_opp_nonland(g, p, |x| !indestructible(g, x)).filter(|&t| pval(g, t) >= 4.0)
}

fn obelisk_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || g.perm(src).tapped || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    let Some(t) = obelisk_target(g, p) else { return Ok(vec![]) };
    if !can_pay(g, p, 7, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![ability(0.5 + 0.35 * pval(g, t).min(9.0), "Unstable Obelisk", src, obelisk_go, 0)])
}

/// {7}, {T}, sacrifice: destroy target permanent
fn obelisk_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    let Some(t) = obelisk_target(g, p) else { return Ok(false) };
    if !pay(g, p, 7, "", false)? {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    die(g, src, "sac")?;
    if ability_window(g, p, Some(src), "destroy target permanent", Some(6.0), Some(t))?
        && live(g, t)
        && !untargetable(g, t)
    {
        crate::engine::removal::apply_removal(g, Some(p), t, "destroy", None)?;
    }
    Ok(true)
}

// ================================================================== Steel Hellkite
const HELLKITE: &str = "Steel Hellkite";

/// the players this Hellkite dealt combat damage to this turn
pub(crate) fn hit_players(g: &Game, m: PermId) -> Vec<PlayerId> {
    match g.perm(m).data.get(DataKey::Hit) {
        Some(Val::List(v)) => {
            let st = g.turn_stamp();
            let mut out = vec![];
            for x in v.chunks(2) {
                if let [Val::Stamp(s), Val::Player(q)] = x
                    && *s == st
                {
                    out.push(*q);
                }
            }
            out
        }
        _ => vec![],
    }
}

/// combat damage dealt by the Hellkite to a player: remember it for the turn
fn hellkite_hit(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if a != src {
        return Ok(());
    }
    let st = g.turn_stamp();
    let mut v: Vec<Val> = match g.perm(src).data.get(DataKey::Hit) {
        Some(Val::List(v)) => v.chunks(2).filter(|c| c.first() == Some(&Val::Stamp(st))).flatten().cloned().collect(),
        _ => vec![],
    };
    v.push(Val::Stamp(st));
    v.push(Val::Player(d));
    g.perm_mut(src).data.set(DataKey::Hit, Val::List(v));
    Ok(())
}

/// how good {X} is for the Hellkite now: the best X, its opponents' loss and our own
fn hellkite_plan(g: &Game, p: PlayerId, src: PermId) -> Option<(i32, f64, f64)> {
    let hit = hit_players(g, src);
    if hit.is_empty() {
        return None;
    }
    let mut best: Option<(i32, f64, f64)> = None;
    for x in 0..=9 {
        let (mut ol, mut ml) = (0.0, 0.0);
        for (q, m) in all_perms(g) {
            if !hit.contains(&q) || mv(g, m) != x || indestructible(g, m) {
                continue;
            }
            if q == p {
                ml += worth(g, m);
            } else {
                ol += worth(g, m);
            }
        }
        if ol > 0.0 && (ol - 1.2 * ml) > best.map_or(0.0, |b| b.1 - 1.2 * b.2) {
            best = Some((x, ol, ml));
        }
    }
    best
}

fn hellkite_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !live(g, src) || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    if g.perm(src).data.get(DataKey::Used) == Some(&Val::Stamp(g.turn_stamp())) {
        return Ok(vec![]);
    }
    let Some((x, ol, ml)) = hellkite_plan(g, p, src) else { return Ok(vec![]) };
    if !can_pay(g, p, x as u32, "", false) {
        return Ok(vec![]);
    }
    let u = (ol - 1.2 * ml) / 2.0 - 0.5;
    if u < 0.8 {
        return Ok(vec![]);
    }
    Ok(vec![ability(u.min(8.0), &format!("Steel Hellkite (X={x})"), src, hellkite_go, x as i64)])
}

/// {X}: destroy each nonland permanent with mana value X whose controller was dealt combat damage by this creature
/// this turn. Activate only once each turn.
pub(crate) fn hellkite_go(g: &mut Game, src: PermId, p: PlayerId, x: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).data.get(DataKey::Used) == Some(&Val::Stamp(g.turn_stamp())) {
        return Ok(false);
    }
    let hit = hit_players(g, src);
    if hit.is_empty() || !pay(g, p, x as u32, "", false)? {
        return Ok(false);
    }
    let st = g.turn_stamp();
    g.perm_mut(src).data.set(DataKey::Used, Val::Stamp(st));
    if ability_window(g, p, Some(src), &format!("destroy each nonland permanent with mana value {x}"), Some(7.0), None)?
    {
        g.batch += 1;
        let prev_batch = g.cur_batch.replace(g.batch);
        let prev = g.destroyer.replace(p);
        g.resolving += 1;
        let doomed: Vec<PermId> =
            all_perms(g).into_iter().filter(|&(q, m)| hit.contains(&q) && mv(g, m) == x as i32).map(|y| y.1).collect();
        let mut r = Ok(());
        for m in doomed {
            if live(g, m) {
                r = r.and(die(g, m, "destroy"));
            }
        }
        g.cur_batch = prev_batch;
        g.destroyer = prev;
        g.resolving -= 1;
        r?;
        if g.resolving == 0 && !g.trig_queue.is_empty() {
            crate::engine::stack::flush_triggers(g)?;
        }
    }
    Ok(true)
}

// ================================================================== Cityscape Leveler
const LEVELER: &str = "Cityscape Leveler";

/// the permanent Leveler destroys: the opposing nonland permanent worth most (3 or more)
fn leveler_target(g: &Game, p: PlayerId) -> Option<PermId> {
    best_opp_nonland(g, p, |x| !indestructible(g, x)).filter(|&t| pval(g, t) >= 3.0)
}

/// destroy it; its controller creates a tapped Powerstone token
fn leveler_destroy(g: &mut Game, p: PlayerId, t: PermId) -> Res {
    let q = g.perm(t).owner;
    crate::engine::removal::apply_removal(g, Some(p), t, "destroy", None)?;
    if !g.perm(t).on_bf && g.player(q).alive {
        make_powerstone(g, q, 1, true)?;
    }
    Ok(())
}

/// When you cast this spell, destroy up to one target nonland permanent. Its controller creates a tapped Powerstone token.
fn leveler_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    if &*g.db.get(c).name != LEVELER {
        return Ok(());
    }
    if let Some(t) = leveler_target(g, p) {
        crate::glog!(g, "    Cityscape Leveler (cast): destroys {}", name_of(g, t));
        leveler_destroy(g, p, t)?;
    }
    Ok(())
}

/// Whenever it attacks, destroy up to one target nonland permanent (the same pick).
fn leveler_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p != owner(g, src) || !atk.contains(&src) {
        return Ok(vec![]);
    }
    if leveler_target(g, p).is_none() || !trigger_window(g, p, Some(src), "destroy a nonland permanent", Some(6.0))? {
        return Ok(vec![]);
    }
    if let Some(t) = leveler_target(g, p) {
        crate::glog!(g, "    Cityscape Leveler (attack): destroys {}", name_of(g, t));
        leveler_destroy(g, p, t)?;
    }
    Ok(vec![])
}

/// Unearth {8}: from the graveyard at sorcery speed, once per death (it is exiled at the end step)
fn leveler_unearth_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || !g.player(p).gy.contains(&c) || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 8, "", false) {
        return Ok(vec![]);
    }
    let u = 1.0 + leveler_target(g, p).map_or(0.0, |t| 0.3 * pval(g, t).min(8.0));
    Ok(vec![Opt {
        utility: u,
        label: "unearth Cityscape Leveler".into(),
        act: Some(Action::Plan { f: leveler_unearth, arg: c.0 as i64 }),
    }])
}

fn leveler_unearth(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !pay(g, p, 8, "", false)? {
        return Ok(false);
    }
    if !crate::engine::stack::ability_window_card(g, p, c, "unearth", None, None)? || !g.player(p).gy.contains(&c) {
        return Ok(true);
    }
    let pl = g.player_mut(p);
    if let Some(i) = pl.gy.iter().position(|&x| x == c) {
        pl.gy.remove(i);
    }
    let m = enter(g, p, c, Enter::default())?;
    g.perm_mut(m).sick = false; // haste
    g.perm_mut(m).data.set(DataKey::Unearth, Val::Bool(true));
    crate::glog!(g, "  {} unearths Cityscape Leveler", g.player(p).name);
    Ok(true)
}

/// the unearthed Leveler is exiled at the beginning of the end step
fn leveler_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !g.perm(src).data.truthy(DataKey::Unearth) || !live(g, src) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "exile it (unearth)", Some(2.0))? || !live(g, src) {
        return Ok(());
    }
    exile_perm(g, src)
}

// ================================================================== Ugin, the Ineffable
const UGIN: &str = "Ugin, the Ineffable";

fn ugin_plus_val(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(1.5)
}

/// +1: exile the top card of your library face down and look at it. Create a 2/2 colourless Spirit creature token.
/// When that token leaves the battlefield, put the exiled card into your hand.
fn ugin_plus(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let top = g.player_mut(p).library.pop();
    let mut spec = Tokens::new(1, 2);
    spec.tgh = Some(2);
    spec.color = Some(Colors::NONE);
    spec.types = vec!["spirit"];
    let toks = make_tokens(g, p, spec)?;
    let Some(c) = top else { return Ok(()) };
    match toks.first() {
        Some(&tok) => {
            g.player_mut(p).exile.push(c);
            let mut v: Vec<Val> = match g.perm(src).data.get(DataKey::Held) {
                Some(Val::List(v)) => v.clone(),
                _ => vec![],
            };
            v.push(Val::Perm(tok));
            v.push(Val::Card(c));
            g.perm_mut(src).data.set(DataKey::Held, Val::List(v));
            crate::glog!(g, "    Ugin exiles the top card of the library face down; a Spirit token");
        }
        None => g.player_mut(p).hand.push(c), // no token: the "leaves" trigger never happens; the card is not lost
    }
    Ok(())
}

/// when a Spirit made by Ugin leaves the battlefield, its exiled card goes to its owner's hand (Ugin's dies hook
/// covers a token that dies; `ugin_leaves` hands over every card still waiting when Ugin leaves)
fn ugin_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let p = owner(g, src);
    let Some(Val::List(v)) = g.perm(src).data.get(DataKey::Held).cloned() else { return Ok(()) };
    let mut keep = vec![];
    for x in v.chunks(2) {
        if let [Val::Perm(tok), Val::Card(c)] = x {
            if *tok == m {
                let pl = g.player_mut(p);
                if let Some(i) = pl.exile.iter().position(|y| y == c) {
                    pl.exile.remove(i);
                    pl.hand.push(*c);
                    crate::glog!(g, "    Ugin's Spirit leaves: {} returns the exiled card to hand", g.player(p).name);
                }
            } else {
                keep.extend_from_slice(x);
            }
        }
    }
    g.perm_mut(src).data.set(DataKey::Held, Val::List(keep));
    Ok(())
}

/// Ugin left: the cards still held under Spirits go to hand (the tokens' own departure is not tracked after this)
fn ugin_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    let p = g.perm(src).orig;
    let Some(Val::List(v)) = g.perm(src).data.get(DataKey::Held).cloned() else { return Ok(()) };
    for x in v.chunks(2) {
        if let [_, Val::Card(c)] = x {
            let pl = g.player_mut(p);
            if let Some(i) = pl.exile.iter().position(|y| y == c) {
                pl.exile.remove(i);
                pl.hand.push(*c);
            }
        }
    }
    g.perm_mut(src).data.set(DataKey::Held, Val::List(vec![]));
    Ok(())
}

fn ugin_target(g: &Game, p: PlayerId) -> Option<PermId> {
    best_opp_nonland(g, p, |x| !colors_of(g, x).is_empty()).filter(|&t| pval(g, t) >= 3.0)
}

fn ugin_minus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    let t = ugin_target(g, p)?;
    if g.perm(src).loyalty.unwrap_or(0) < 3 {
        return None;
    }
    Some(0.8 + 0.3 * pval(g, t).min(8.0))
}

/// −3: destroy target permanent that's one or more colours
fn ugin_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = ugin_target(g, p) {
        crate::glog!(g, "    Ugin destroys {}", name_of(g, t));
        crate::engine::removal::apply_removal(g, Some(p), t, "destroy", None)?;
    }
    Ok(())
}

const UGIN_ABS: [WalkerAb; 2] = [
    WalkerAb { delta: 1, label: "exile the top card, a 2/2 Spirit", val: ugin_plus_val, eff: ugin_plus },
    WalkerAb { delta: -3, label: "destroy a coloured permanent", val: ugin_minus_val, eff: ugin_minus },
];

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Nevinyrral's Disk")?;
    c.as_enters = Some(disk_enters);
    c.options = Some(disk_options);
    r.card(db, "Oblivion Stone")?.options = Some(stone_options);
    r.card(db, "Ratchet Bomb")?.options = Some(bomb_options);
    r.card(db, "Darksteel Forge")?.grant_kw = Some(forge_kw);
    let c = r.card(db, LIBERATOR)?;
    c.cast = Some(liberator_cast);
    r.card(db, "Steel Overseer")?.options = Some(overseer_options);
    r.card(db, "Triplicate Titan")?.self_dies = Some(titan_dies);
    r.card(db, "Duplicant")?.etb = Some(duplicant_etb);
    r.card(db, "Kuldotha Forgemaster")?.options = Some(forgemaster_options);
    r.card(db, REPLICATOR)?.cast = Some(replicator_cast);
    let c = r.card(db, "Portal to Phyrexia")?;
    c.etb = Some(portal_etb);
    c.upkeep = Some(portal_upkeep);
    let c = r.card(db, "Spine of Ish Sah")?;
    c.etb = Some(spine_etb);
    c.self_dies = Some(spine_dies);
    r.card(db, "Unstable Obelisk")?.options = Some(obelisk_options);
    let c = r.card(db, HELLKITE)?;
    c.combat_damage = Some(hellkite_hit);
    c.options = Some(hellkite_options);
    let c = r.card(db, LEVELER)?;
    c.self_cast = Some(leveler_cast);
    c.attack = Some(leveler_attack);
    c.end_step = Some(leveler_end);
    r.card(db, "Cityscape Leveler")?.gy_options = Some(leveler_unearth_opts);
    walker(r, db, UGIN, &UGIN_ABS)?;
    let c = r.card(db, UGIN)?;
    c.cost = Some(ugin_cost);
    c.dies = Some(ugin_dies);
    c.leaves = Some(ugin_leaves);
    Ok(())
}
