//! The card ability language's interpreter (Python's `cards/dsl.py`): compiled abilities (`model.rs`) run by the
//! engine. Hand-tagged cards keep the engine's own code; both kinds play together in one game.
//!
//!   - `fire`: triggered abilities (enters, dies, attacks, cast, draw, upkeep ...), put on the stack as known triggers
//!   - `resolve_spell`: instants and sorceries
//!   - `pt`, `has_kw`, `protection`, `cost_delta`, `no_lifegain`: static abilities (the token and counter doublers are
//!     read by `engine::repl`)
//!   - `ability_options`: activated, loyalty and equip abilities as choices for the AI
//!   - `card_value`, `value_of`: how much the AI wants to cast a card it has no hand-written priority for

pub mod model;

use crate::ai::hand_key; // --- phase4-hand-off
use crate::cards::{Colors, Types};
use crate::engine::pool::Kind;
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::repl::{self, Ctr, On, Path};
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
    /// permanents this ability destroyed so far ("for each creature destroyed this way")
    pub destroyed: i32,
    /// the mana value of the permanent sacrificed to pay for this ability
    pub sacrificed_mv: i32,
    /// `x` is the stand-in 3 (no X was chosen): reading it as "X" is counted in `x_fallbacks`
    pub x_default: bool,
    /// an attack trigger: attacking tokens it makes join the attack
    pub attack: bool,
    /// --- Tiers 1/2 (isshin): an attack trigger: the creatures attacking as it triggered (0 outside one)
    pub atk_count: i32,
}

impl DslCtx {
    pub fn base() -> DslCtx {
        DslCtx { x: 3, x_default: true, ..DslCtx::default() }
    }
}

thread_local! {
    /// how many times a compiled "X" amount was read without any X chosen, so it ran as the stand-in 3 (this thread)
    static X_FALLBACKS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// the mana value of the permanent the last paid ability cost sacrificed (0 if none): `activate` hands it to the ability
    static LAST_SAC_MV: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
    /// the X the last paid ability cost with an {X} chose (0 if none): `activate` hands it to the effects
    static LAST_X: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
    /// --- ghave: the creature an ability's "sacrifice a creature" cost takes (`activate_sacrificing`), not the AI's pick
    static FORCED_FODDER: std::cell::Cell<Option<PermId>> = const { std::cell::Cell::new(None) };
}

/// --- ghave: the compiled cost text of Ghave, Guru of Spores's first ability (cardgen keeps it as `other`)
pub const REMOVE_PLUS_COST: &str = "remove a +1/+1 counter from a creature you control";

/// Reads and clears this thread's count of "X" amounts that ran as the stand-in 3. Cardgen's audit flags the cards
/// that can do it (a bare "X" amount on a card with no {X}); tests over whole games assert it stays zero.
pub fn x_fallbacks() -> u32 {
    X_FALLBACKS.with(|c| c.replace(0))
}

pub fn active(g: &Game) -> bool {
    g.dsl_on
}

fn abilities(g: &Game, c: CardId) -> Arc<[Ability]> {
    g.db.get(c).abilities.clone()
}

/// Phase 3 (one arena): q's lands whose compiled abilities work, as ability sources after q's other permanents
/// (`player.perms` first, so everything that ran before keeps its order). Blood Moon: a nonbasic land that is a plain
/// Mountain has no abilities of its own.
pub fn land_sources(g: &Game, q: PlayerId) -> impl Iterator<Item = PermId> + '_ {
    g.player(q).lands.iter().copied().filter(move |&l| {
        let d = g.db.get(g.perm(l).card());
        land_live(d) && g.perm(l).on_bf && !crate::engine::mana::moon_land(g, l)
    })
}

/// Does land card c have a compiled ability that does something (not only a note)? Such a land turns the ability
/// language on as it enters (`Game::add_land`), as a permanent with abilities does (`zones::enter`).
pub fn land_live(d: &crate::cards::CardDef) -> bool {
    d.abilities.iter().any(|a| !matches!(a, Ability::Static { kind, .. } if kind == "note"))
}

/// A channel ability ("discard this card"), compiled as an activated ability with a discard cost and no {T}: it is
/// used from the hand (card code `hand_options`: Boseiju, Otawara), never from the battlefield.
fn channel(cost: &model::Cost) -> bool {
    cost.discard.is_some() && cost.tap != Some(true)
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
        "X" => {
            if ctx.x_default {
                X_FALLBACKS.with(|c| c.set(c.get() + 1));
            }
            ctx.x.max(1)
        }
        "creatures_died_this_turn" => {
            (g.died_count.0 == Some(g.turn_stamp())).then_some(g.died_count.1 as i32).unwrap_or(0)
        }
        "destroyed_this_way" => ctx.destroyed,
        // --- Tiers 1/2 (isshin): "the number of (other) attacking creatures" (Mishra, Claimed by Gix; Commissar
        // Severina Raine): the attack the trigger saw, else the creatures in combat now
        "attacking_creatures" | "other_attacking_creatures" => {
            let n = if ctx.atk_count > 0 {
                ctx.atk_count
            } else {
                g.in_combat.iter().filter(|&&m| g.perm(m).on_bf).count() as i32 // every player's (Galadhrim Ambush)
            };
            // "other": the source itself attacks (its own attack trigger)
            let me = name == "other_attacking_creatures"
                && src.is_some_and(|m| ctx.atk_count > 0 || g.in_combat.contains(&m));
            (n - me as i32).max(0)
        }
        "sacrificed_mv" => ctx.sacrificed_mv,
        "gy_instants_sorceries" => pl.gy.iter().filter(|&&c| g.db.get(c).instant || g.db.get(c).sorcery).count() as i32,
        // --- Tiers 1/2 (brago-meren): "each creature card in your graveyard" (Wight of the Reliquary)
        "gy_creatures" => pl.gy.iter().filter(|&&c| g.db.get(c).creature).count() as i32,
        "colors_among_permanents" => pl
            .perms
            .iter()
            .filter(|&&m| g.perm(m).on_bf && !g.perm(m).phased)
            .fold(Colors::NONE, |acc, &m| acc.union(colors_of(g, m)))
            .count() as i32,
        "creatures_you_control" => pl.perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() as i32,
        "cards_in_hand" => pl.hand.len() as i32,
        "lands_you_control" => pl.lands.len() as i32, // --- Tiers 1/2 (windgrace)
        "event_player_hand" => ctx.event_player.map_or(0, |q| g.player(q).hand.len() as i32), // that player's hand
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
        // --- Tiers 1/2 (lathril): "you gain life equal to its toughness" in an enters trigger (Wolverine Riders)
        "event_toughness" => ctx.event_perm.map_or(0, |m| etgh(g, m).max(0)),
        "event_power" => ctx.event_perm.map_or(0, |m| epow(g, m).max(0)),
        // --- Tiers 1/2 (tatyova-teysa): capped, so a doubling each landfall (Mossborn Hydra) never overflows
        "counters_on_self" => src.map_or(0, |m| g.perm(m).plus.clamp(0, 1_000_000)),
        // --- Tier 3 (marwyn): "each attacking Elf you control" (Dwynen): the creatures p attacked with this turn
        // (`combat::attack_triggers` records them) still on the battlefield
        s if s.starts_with("attacking_type:") => {
            let t = &s[15..];
            if pl.attacking != Some(g.turn_stamp()) {
                return 0;
            }
            let live = |m: PermId| g.perm(m).on_bf && g.perm(m).owner == p && has_type(g, m, t);
            pl.attackers.iter().filter(|&&m| live(m)).count() as i32
        }
        // --- Tiers 1/2 (lathril): "Elves you control plus the number of Elf cards in your graveyard" (Abomination)
        s if s.starts_with("type_you_gy:") => {
            let t = &s[12..];
            let bf = pl.perms.iter().filter(|&&m| !g.perm(m).phased && has_type(g, m, t)).count();
            let gy = pl.gy.iter().filter(|&&c| g.db.get(c).has_subtype(t) || g.db.get(c).has_kw("changeling")).count();
            (bf + gy) as i32
        }
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
        // --- Tiers 4/5 (krenko): "the number of Goblins that entered the battlefield under your control this turn"
        s if s.starts_with("type_entered_you:") => crate::impls::t45_krenko::entered_this_turn(g, p, &s[17..]),
        s if s.starts_with("target_") => 2,
        _ => 2,
    }
}

fn type_ok(g: &Game, m: PermId, t: &str) -> Option<bool> {
    let ty = g.perm(m).cd.map(|_| crate::engine::values::ptypes(g, m));
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
    if f.tapped == Some(true) && !x.tapped {
        return false;
    }
    if f.nontoken == Some(true) && x.token {
        return false;
    }
    if f.attacking == Some(true) && !g.in_combat.contains(&m) {
        return false; // Tier 3 (aurelia): "attacking creatures you control" (Blade Historian)
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
            s if s.starts_with("non-") => !has_type(g, m, &s[4..]), // --- Tiers 4/5 (urdragon): Crux of Fate
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

/// `_ask_target`: the person picks among cands. A cancelled session (an `Err` from the helper) picks nothing here:
/// this function cannot return the error, and the next engine step stops the game (`tick`).
fn ask_target(
    g: &Game,
    p: PlayerId,
    cands: &[PermId],
    sel: &Sel,
    src: Option<PermId>,
    spell: Option<CardId>,
) -> Option<PermId> {
    let what = match spell.or(src.and_then(|m| g.perm(m).cd)) {
        Some(c) => g.db.get(c).name.to_string(),
        None => "An ability".to_string(),
    };
    let labels: Vec<String> =
        cands.iter().map(|&m| crate::human::legal::describe_target(g, p, crate::human::Target::Perm(m))).collect();
    let cancel = if sel.upto == Some(true) { Some("no target") } else { None };
    let k = crate::human::choices::choose(g, p, "target", &format!("{what}: choose a target"), &labels, cancel, None)
        .ok()
        .flatten()?;
    Some(cands[k])
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
    let cols = src_colors(g, src, spell);
    let cands: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| {
            matches(g, p, m, f, src)
                && !(g.perm(m).owner != p
                    && (untargetable(g, m)
                        || protected_from(g, m, cols)
                        || crate::impls::tymna_cards::hexproof_from(g, m, cols)))
        })
        .collect();
    if crate::human::is_human(g, p) && !cands.is_empty() {
        return ask_target(g, p, &cands, sel, src, spell); // practice mode: the person picks
    }
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
    // "you may X, then Y" / "you may X. If you do, Z": a person who says no to X skips Y and Z. The AI is never
    // asked and runs X and Y as ever; Z is nested in X (`if_you_do`) and only a person who said yes runs it.
    let human = crate::human::is_human(g, p);
    let mut declined = false;
    for e in effects {
        if g.over || !g.player(p).alive {
            return Ok(());
        }
        if e.cont == Some(true) && declined {
            continue;
        }
        if e.optional == Some(true) && human {
            declined = !ask_may(g, p, e, src, spell)?;
            if declined {
                continue;
            }
        }
        run(g, p, e, src, ctx, spell, depth)?;
        if human && !e.if_you_do.is_empty() && !g.over && g.player(p).alive {
            execute(g, p, &e.if_you_do, src, ctx, spell, depth + 1)?;
        }
    }
    check_state(g)
}

/// "you may ...": the person's yes or no
fn ask_may(g: &mut Game, p: PlayerId, e: &Effect, src: Option<PermId>, spell: Option<CardId>) -> Res<bool> {
    let what = match spell.or(src.and_then(|m| g.perm(m).cd)) {
        Some(c) => g.db.get(c).name.to_string(),
        None => "An effect".to_string(),
    };
    let does = match e.kind.as_str() {
        "search" => "search your library".to_string(),
        "reanimate" => "return a creature card from your graveyard to the battlefield".to_string(),
        "regrow" => "return a card from your graveyard to your hand".to_string(),
        "destroy" => "destroy it".to_string(),
        "exile" => "exile it".to_string(),
        "bounce" => "return it to its owner's hand".to_string(),
        "tuck" => "put it into its owner's library".to_string(),
        "blink" => "exile it and return it to the battlefield".to_string(),
        "put_land" => "put a land card from your hand onto the battlefield".to_string(),
        "gain_life" => "gain life".to_string(),
        "lose_life" => "lose life".to_string(),
        "discard" => "discard a card".to_string(),
        "draw" => "draw".to_string(),
        "sacrifice" => "sacrifice".to_string(),
        "token" => "create the token".to_string(),
        "counters" => "put the counters".to_string(),
        "mill" => "mill".to_string(),
        k => k.replace('_', " "),
    };
    crate::human::choices::yes_no(g, p, &format!("{what}: {does}?"))
}

/// a count read for each player an effect reaches, not for its controller ("equal to the number of nonbasic lands that
/// player controls", Price of Progress)
fn per_player_n(e: &Effect) -> Option<fn(&Game, PlayerId) -> i32> {
    match &e.n {
        Some(Num::Name(s)) if s == "that_player_nonbasic_lands" => Some(nonbasic_lands),
        _ => None,
    }
}

fn nonbasic_lands(g: &Game, q: PlayerId) -> i32 {
    g.player(q).lands.iter().filter(|&&m| g.perm(m).cd.is_some_and(|c| !g.db.get(c).basic)).count() as i32
}

/// p has a use for its own graveyard now: a reanimation spell or Yawgmoth's Will in hand, or a graveyard engine out
fn gy_payoff(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    pl.hand.iter().any(|&c| g.db.get(c).tag(Tag::Rean) || g.db.get(c).tag(Tag::Yawg))
        || pl.perms.iter().any(|&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Rean)))
}

/// "Target player mills N cards" (Glimpse the Unthinkable): the AI mills itself when its graveyard pays off now,
/// else the opponent closest to milling out (a library of N or fewer: that player loses at the next draw), else the
/// one it fears most
fn mill_target(g: &Game, p: PlayerId, k: i32) -> Option<PlayerId> {
    if gy_payoff(g, p) && g.player(p).library.len() as i32 > k + 5 {
        return Some(p);
    }
    let opps: Vec<PlayerId> =
        g.opps(p).filter(|&q| !crate::engine::values::player_hexproof(g, q)).collect();
    if let Some(q) = min_by(&opps, |q| g.player(q).library.len() as f64).filter(|&q| g.player(q).library.len() as i32 <= k)
    {
        return Some(q);
    }
    max_by(&opps, |q| threat(g, p, q))
}

/// what "target player mills N" is worth to p now (value_of)
fn mill_value(g: &Game, p: PlayerId, k: i32) -> f64 {
    match mill_target(g, p, k) {
        Some(q) if q == p => 0.25 * k as f64,
        Some(q) if g.player(q).library.len() as i32 <= k => 15.0,
        Some(_) => 0.05 * k as f64,
        None => 0.0,
    }
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
                // "you may draw": a person chooses (the draw by someone else's "may" is asked of that player here).
                // The AI ignores the flag (it was never set before cardgen flagged every "you may" effect).
                if e.optional == Some(true) && q != p && crate::human::is_human(g, q) {
                    let what = match spell.or(src.and_then(|m| g.perm(m).cd)) {
                        Some(c) => g.db.get(c).name.to_string(),
                        None => "An effect".to_string(),
                    };
                    if !crate::human::choices::yes_no(g, q, &format!("{what}: draw?"))? {
                        continue;
                    }
                }
                let k = n(g, ctx);
                zones::draw(g, q, k.max(0) as u32, false)?;
            }
        }
        "damage" => {
            let dmg = (n(g, ctx) + has(g, p, Tag::Thor) as i32) * crate::impls::kuja::flare_star(g, src); // Trance Kuja: a Wizard's damage is doubled
            let eff = dmg;
            let to = match &e.to {
                Some(To::Sel(s)) => (**s).clone(),
                _ => Sel::default(),
            };
            match to.sel.as_deref() {
                Some("any_target") if crate::human::is_human(g, p) => {
                    let what = match spell.or(src.and_then(|m| g.perm(m).cd)) {
                        Some(c) => g.db.get(c).name.to_string(),
                        None => "Damage".to_string(),
                    };
                    crate::human::choices::deal_damage(g, p, dmg, &what, "triggers")?;
                }
                Some("any_target") => {
                    let cols = src_colors(g, src, spell);
                    let tg: Vec<PermId> = opps
                        .iter()
                        .flat_map(|&q| g.player(q).perms.iter().copied())
                        .filter(|&m| {
                            g.is_creature(m)
                                && !untargetable(g, m)
                                && etgh(g, m) <= eff
                                && !protected_from(g, m, cols)
                                && !crate::impls::tymna_cards::hexproof_from(g, m, cols)
                        })
                        .collect();
                    let best = max_by(&tg, |m| pval(g, m));
                    let lethal: Vec<PlayerId> = opps.iter().copied().filter(|&q| g.player(q).life <= eff).collect();
                    // Kefka flipped with a draw-trigger engine out and a library too short for its loop: no face pings
                    let guard = crate::impls::kefka::loop_guard(g, p);
                    if let Some(&q) = lethal.first() {
                        lose_life(g, q, dmg, Some(p), kind, None)?;
                    } else if let Some(b) = best.filter(|&b| guard || pval(g, b) >= 3.0) {
                        snapshot(g, b, ctx);
                        apply_removal(g, Some(p), b, &format!("dmg{dmg}"), spell)?;
                    } else if guard {
                        // the trigger is mandatory and an opponent's face would start the loop: ping yourself
                        lose_life(g, p, dmg, Some(p), kind, None)?;
                    } else if let Some(q) = min_by(&opps, |q| g.player(q).life as f64) {
                        lose_life(g, q, dmg, Some(p), kind, None)?;
                    }
                }
                Some("player") if per_player_n(e).is_some() => {
                    // each player takes its own count, you too (Price of Progress)
                    for q in players(g, p, Some(to.who.as_deref().unwrap_or("each_player")), ctx) {
                        let k = per_player_n(e).map_or(0, |f| f(g, q)) * e.mult.unwrap_or(1);
                        if k > 0 {
                            lose_life(g, q, k, Some(p), kind, None)?;
                        }
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
                        let amt = if g.perm(m).owner != p { eff } else { dmg };
                        if etgh(g, m) <= amt {
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
            let mut ms = select(g, p, &sel, true, ctx, src, spell);
            if e.kind == "bounce"
                && let Some(t) = src.and_then(|s| crate::impls::garland::ropemaster_target(g, p, s))
            {
                ms = vec![t]; // Peerless Ropemaster re-crowns: our tapped Garland is the target
            }
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
                let prev_batch = g.cur_batch.replace(g.batch);
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
                            "destroy" => {
                                zones::die(g, m, "destroy")?;
                                ctx.destroyed += i32::from(!g.perm(m).on_bf);
                            }
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
                g.cur_batch = prev_batch;
                g.destroyer = prev;
                r?;
            } else {
                for m in ms {
                    snapshot(g, m, ctx);
                    apply_removal(g, Some(p), m, &e.kind, spell)?;
                    ctx.destroyed += i32::from(e.kind == "destroy" && !g.perm(m).on_bf);
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
                color: e.color.as_deref().map(Colors::from_letters), // --- Tiers 1/2 (tatyova-teysa)
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
            if repl::solemnity(g) {
                return Ok(()); // --- Tiers 4/5 (heliod-zur): Solemnity
            }
            let k = n(g, ctx);
            let sel = e.what.clone().unwrap_or(Sel { sel: Some("self".into()), ..Sel::default() });
            for m in select(g, p, &sel, false, ctx, src, spell) {
                // the counters step: Hardened Scales, the controller's doublers, Vorinclex (Captain Marvel)
                repl::put_counters(g, p, m, k);
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
            if crate::impls::t45_winota::jailed(g) {
                return Ok(()); // --- Tiers 4/5 (winota): Soulless Jailer: permanent cards in graveyards can't enter
            }
            let f = e.filter.clone().unwrap_or(Filter { kind: Some("creature".into()), ..Filter::default() });
            if e.all == Some(true) {
                // --- Tiers 1/2 (kaalia-sythis): "return all ... cards from your graveyard to the battlefield"
                let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| card_matches(g, c, Some(&f))).collect();
                let tapped = e.tapped == Some(true);
                let mut lands = 0;
                for c in cs {
                    let gy = &mut g.player_mut(p).gy;
                    let Some(i) = gy.iter().position(|&x| x == c) else { continue };
                    gy.remove(i);
                    if g.db.get(c).land {
                        g.add_land_entering(p, c, tapped);
                        lands += 1;
                    } else {
                        let m = zones::enter(g, p, c, Enter::default())?;
                        if tapped {
                            g.perm_mut(m).tapped = true;
                        }
                    }
                    crate::glog!(g, "    {} returns to the battlefield", g.db.get(c).name);
                }
                for _ in 0..lands {
                    zones::landfall(g, p)?;
                }
                return Ok(());
            }
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
            let qs = if e.who.as_deref() == Some("target_player") {
                mill_target(g, p, n(g, ctx)).into_iter().collect()
            } else {
                players(g, p, Some(e.who.as_deref().unwrap_or("you")), ctx)
            };
            for q in qs {
                let k = n(g, ctx);
                zones::mill(g, q, k.max(0) as usize)?;
            }
        }
        "exile_library" => {
            // "exile all but the bottom card of target player's library" (Nicol Bolas, the Arisen)
            for q in players(g, p, Some(e.who.as_deref().unwrap_or("target_player")), ctx) {
                let lib = std::mem::take(&mut g.player_mut(q).library);
                let mut it = lib.into_iter();
                let bottom = it.next();
                for c in it {
                    g.player_mut(q).exile.push(c);
                }
                g.player_mut(q).library.extend(bottom);
            }
        }
        "sacrifice" => {
            let mut f = e.filter.clone().unwrap_or(Filter { kind: Some("creature".into()), ..Filter::default() });
            f.controller = None;
            if f.kind.as_deref() == Some("land") {
                // --- Tier 3 (korvold): "each player sacrifices a land of their choice" (Smallpox): lands are not
                // permanents here, each player gives up its least useful one
                for q in players(g, p, Some(e.who.as_deref().unwrap_or("each_opponent")), ctx) {
                    for _ in 0..num(g, p, Some(e.n.as_ref().unwrap_or(&Num::Int(1))), ctx, src) {
                        crate::impls::t3_korvold::sac_worst_land(g, q)?;
                    }
                }
                return Ok(());
            }
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
            let k = n(g, ctx).max(0) as u32;
            // Savage Ventmaw: "until end of turn, you don't lose this mana as steps and phases end" (the parser drops it)
            let kept = src.and_then(|m| g.perm(m).cd).is_some_and(|c| &*g.db.get(c).name == "Savage Ventmaw");
            if kept { g.player_mut(p).mana.add_kept(Kind::Any, k) } else { g.player_mut(p).mana.add(Kind::Any, k) }
        }
        "extra_combat" => g.player_mut(p).extra_combats += 1,
        "ring_tempt" => crate::cardcode::ring_tempt(g, p)?, // --- Tiers 4/5 (winota): "the Ring tempts you"
        "proliferate" => {
            // the engine's full proliferate: +1/+1 and loyalty counters, Sagas, lands' counters and opponents' poison
            // (this effect used to add +1/+1 counters to your own permanents only)
            let times = if e.n.is_some() { n(g, ctx).max(0) as u32 } else { 1 };
            crate::impls::common::proliferate(g, p, times)?;
        }
        "poison" => {
            // each opponent / each player / target opponent gets N poison counters (Melira stops them; Vorinclex
            // doubles the ones you give)
            let k = if e.n.is_some() { n(g, ctx).max(0) } else { 1 };
            for q in players(g, p, Some(e.who.as_deref().unwrap_or("each_opponent")), ctx) {
                if k == 0 || !repl::poison_ok(g, q) {
                    continue;
                }
                let k = repl::give_poison(g, p, q, k, Path::VORINCLEX);
                crate::glog!(g, "    {} gets {} poison counter(s)", g.player(q).name, k);
            }
            crate::engine::life::check_state(g)?;
        }
        "double_counters" => {
            for m in g.player(p).perms.clone() {
                if g.perm(m).plus > 0 {
                    // doubling puts that many more (the counters step: Hardened Scales, Vorinclex ...)
                    repl::put_counters(g, p, m, g.perm(m).plus);
                }
            }
        }
        "amass" => {
            let k = n(g, ctx);
            zones::amass(g, p, k)?;
        }
        "untap" => {
            let sel = sel_or_empty(&e.what);
            if sel.filter.as_ref().and_then(|f| f.kind.as_deref()) == Some("land") && e.n.is_some() {
                // --- Tiers 4/5 (chulane-kinnan): "untap up to five lands": the tapped lands making the most first
                let k = n(g, ctx).max(0) as usize;
                let mut tapped: Vec<_> = g.player(p).lands.iter().copied().filter(|&l| g.perm(l).tapped).collect();
                let worth = |l| {
                    let t = &g.db.get(g.perm(l).card()).tags;
                    10 * t.int(crate::tag::Tag::Amt).unwrap_or(1) as i32
                        + crate::engine::mana::land_colors_now(g, p, l).count() as i32
                };
                tapped.sort_by_key(|&l| -worth(l));
                for l in tapped.into_iter().take(k) {
                    g.perm_mut(l).tapped = false;
                }
            } else if sel.filter.as_ref().and_then(|f| f.kind.as_deref()) == Some("land") {
                for l in g.player(p).lands.clone() {
                    g.perm_mut(l).tapped = false;
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
                let pl_c = g.player(p);
                let (cmd, cmd2) = (pl_c.cmd, pl_c.cmd2);
                let cands: Vec<PermId> = g
                    .player(p)
                    .perms
                    .iter()
                    .copied()
                    .filter(|&m| {
                        let x = g.perm(m);
                        g.is_creature(m)
                            && x.cd.is_some_and(|c| c != cmd && Some(c) != cmd2 && etb_value(g, c) > 0)
                            && !x.phased
                            && matches(g, p, m, sel.filter.as_ref(), src)
                    })
                    .collect();
                let mut ms: Vec<PermId> =
                    max_by(&cands, |m| etb_value(g, g.perm(m).cd.unwrap()) as f64).into_iter().collect();
                if crate::human::is_human(g, p) {
                    // practice mode: any of yours, or none
                    let mine: Vec<PermId> = g
                        .player(p)
                        .perms
                        .iter()
                        .copied()
                        .filter(|&m| {
                            g.is_creature(m) && !g.perm(m).phased && matches(g, p, m, sel.filter.as_ref(), src)
                        })
                        .collect();
                    let up = Sel { upto: Some(true), ..sel.clone() };
                    let t = if mine.is_empty() { None } else { ask_target(g, p, &mine, &up, src, spell) };
                    ms = t.into_iter().collect();
                }
                ms
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
            if crate::human::is_human(g, p) {
                let what = match spell.or(src.and_then(|m| g.perm(m).cd)) {
                    Some(c) => g.db.get(c).name.to_string(),
                    None => "Choose".to_string(),
                };
                let picked = crate::human::choices::pick_modes(g, p, &e.modes, k, &what)?;
                for i in picked {
                    execute(g, p, &e.modes[i], src, ctx, spell, depth + 1)?;
                }
            } else {
                let mut order: Vec<(usize, f64)> =
                    e.modes.iter().enumerate().map(|(i, ms)| (i, value_of(g, p, ms, spell))).collect();
                order.sort_by(|a, b| b.1.total_cmp(&a.1)); // stable, best first
                for &(i, _) in order.iter().take(k) {
                    execute(g, p, &e.modes[i], src, ctx, spell, depth + 1)?;
                }
            }
        }
        "blight" => {
            // --- Tiers 1/2 (lathril): "each opponent blights 1" (High Perfect Morcant)
            let k = n(g, ctx);
            for q in players(g, p, Some(e.who.as_deref().unwrap_or("each_opponent")), ctx) {
                crate::impls::t12_lathril::blight(g, q, k)?;
            }
        }
        "creature_backlash" => {
            // --- Tiers 4/5 (prosper): each creature deals 1 damage to its controller (Rakdos Charm)
            for q in g.players.iter().filter(|x| x.alive).map(|x| x.id).collect::<Vec<_>>() {
                let k = g.player(q).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() as i32;
                lose_life(g, q, k, Some(q), "triggers", Some(true))?;
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
        "impulse" => {
            // exile the top n cards of your library; you may play them this turn (held in hand, `impulse`: the end
            // step exiles those not played)
            let k = (n(g, ctx).max(0) as usize).min(g.player(p).library.len());
            let pl = g.player_mut(p);
            let top: Vec<CardId> = (0..k).map(|_| pl.library.pop().unwrap()).collect();
            for &c in &top {
                pl.hand.push(c);
                pl.seen.insert(c);
            }
            pl.impulse.extend(&top);
        }
        "extra_land" => {
            // you may play an additional land this turn. The AI plays it at once (it plays no lands after combat);
            // a person plays it in a main phase
            // --- Tiers 1/2 (tatyova-teysa): "up to three additional lands" (Summer Bloom): n of them (1 unnamed)
            let k = n(g, ctx).max(1) as u32;
            g.player_mut(p).extra_land_now += k;
            if !crate::human::is_human(g, p) && g.active == Some(p) {
                crate::engine::turn::more_lands(g, p)?;
            }
        }
        "free_gy_spell" => {
            // you may cast an instant or sorcery card with mana value n or less from your graveyard free; exiled
            // instead of going to the graveyard (Sword of Once and Future)
            let k = n(g, ctx).max(0) as u32;
            crate::impls::t0_stella::free_from_gy(g, p, k)?;
        }
        "put_land" => {
            // put a land card from your hand onto the battlefield
            let mut ls: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
            // --- Tiers 1/2 (tatyova-teysa): "from your hand or graveyard" (Bonny Pall): a graveyard land first (it
            // costs no card from hand), the best of each
            let gy_ok = e.from.as_deref() == Some("hand_or_gy");
            if gy_ok {
                ls.extend(g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).land));
            }
            if let Some(c) = max_by(&ls, |c| {
                let t = &g.db.get(c).tags;
                let from_gy = gy_ok && g.player(p).gy.contains(&c);
                t.str(Tag::C).map_or(0, |s| s.chars().count()) as f64 * 10.0
                    + t.has(Tag::F) as i32 as f64
                    + if from_gy { 100.0 } else { 0.0 }
            }) {
                let pl = g.player_mut(p);
                match pl.hand.iter().position(|&x| x == c) {
                    Some(i) => pl.hand.remove(i),
                    None => {
                        let i = pl.gy.iter().position(|&x| x == c).unwrap();
                        pl.gy.remove(i)
                    }
                };
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
        // the karn deck's nonbasic land searches (Expedition Map, Urza's Cave) take the land it wants (`karn::search_pick`)
        let karn_pick = f.basic == Some(false) && crate::impls::karn::is_deck(hand_key(g, p));
        if f.kind.as_deref() == Some("land") && to == "battlefield" && !karn_pick {
            zones::land_ramp(g, p, 1, tapped)?;
            continue;
        }
        // a basic land to hand; "a land card" (Sylvan Scrying) takes any land through the pick below (it took a basic
        // only, so a land tutor never found Gaea's Cradle)
        if f.kind.as_deref() == Some("land") && to == "hand" && !karn_pick && f.basic == Some(true) {
            zones::land_to_hand(g, p)?;
            continue;
        }
        let cands: Vec<CardId> =
            zones::searchable(g, p).into_iter().filter(|&c| card_matches(g, c, Some(&f))).collect();
        if cands.is_empty() {
            return Ok(());
        }
        let c = if crate::human::is_human(g, p) {
            // practice mode: the person searches
            let fc = f.clone();
            let mut prompt = "Search your library".to_string();
            if let Some(t) = f.kind.as_deref().filter(|t| *t != "any") {
                prompt.push_str(&format!(" for {t} card"));
            }
            let got = crate::human::choices::search(
                g,
                p,
                &|g: &Game, x: CardId| card_matches(g, x, Some(&fc)),
                1,
                &prompt,
                true,
            )?;
            let Some(&c) = got.first() else { return Ok(()) };
            g.player_mut(p).library.push(c); // put back: removed below like the AI's pick
            c
        } else {
            let pick = crate::ai::dsl_search_pick(g, p, &f, &cands);
            pick.unwrap_or_else(|| max_by(&cands, |c| card_value(g, p, c) * 1000.0 + g.db.get(c).cmc as f64).unwrap())
        };
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
                crate::engine::tutors::put_known_top(g, p, c); // --- ai-fix: B: p knows the card it put there
            }
            _ => {
                g.player_mut(p).hand.push(c);
                g.player_mut(p).seen.insert(c);
            }
        }
        g.player_mut(p).stat("tutored", 1);
        if !matches!(to, "battlefield" | "graveyard") {
            crate::ai::rules::mark_tutored(g, p, c); // --- phase4-rules
        }
        let (pn, cn) = (g.player(p).name, g.db.get(c).name.to_string());
        if to == "hand" || to == "top" {
            g.log_secret(p, &format!("    {pn} searches for a card"), &format!("    {pn} searches for {cn}"));
        } else {
            crate::glog!(g, "    {} searches for {}", pn, cn);
        }
    }
    Ok(())
}

/// an instant or sorcery with compiled abilities resolves
pub fn resolve_spell(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res {
    let mut dctx = DslCtx { x: ctx.x, target: ctx.target, ..DslCtx::default() };
    if dctx.x == 0 {
        dctx.x = 3; // Python's ctx.get('x', 3) when no X was chosen
        dctx.x_default = true;
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
    /// --- Tier 3 (korvold): the combat damage dealt ("create that many Treasure tokens": X in the trigger)
    pub dmg: Option<i32>,
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
        srcs.extend(land_sources(g, q)); // Phase 3: lands' compiled triggers, after the other permanents'
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
        n += repl::trigger_copies(g, q, "dies", kw.perm);
    }
    if event == "etb" {
        n += repl::trigger_copies(g, q, "etb", kw.perm);
    }
    n
}

/// one ctx per time the ability triggers
fn trigger_matches(g: &mut Game, q: PlayerId, src: PermId, a: &Ability, kw: &Fired) -> Vec<DslCtx> {
    let Ability::Triggered { event, source, who, spell, nth, other, if_cast, min_pow, subtype, min_atk, color, .. } = a
    else {
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
            if color.as_deref().and_then(|c| c.chars().next()).is_some_and(|c| !colors_of(g, m).has(c)) {
                return out; // --- Tiers 4/5 (urza-yawgmoth): "another black creature" (Ayara)
            }
            // --- Tiers 1/2 (lathril): "an Elf you control dies", "another Elf you control enters"
            if let Some(t) = subtype.as_deref().filter(|t| !t.is_empty())
                && !(has_type(g, m, t) || (t == "warrior" && g.perm(m).warrior))
            {
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
            let ctx = DslCtx { event_player: kw.defender, attack: true, atk_count: kw.attackers.len() as i32, ..base };
            if min_atk.is_some_and(|k| (kw.attackers.len() as u32) < k) {
                return out; // Tier 3 (aurelia): battalion, "attacks with three or more creatures"
            }
            match s {
                Some("self") if kw.attackers.contains(&src) => out.push(ctx),
                // --- Tiers 4/5 (krenko): "one or more Goblins you control attack" (General Kreat): one of that type
                Some("you_any")
                    if kw.player == Some(q)
                        && subtype.as_deref().is_some_and(|t| t != "warrior")
                        && !kw.attackers.iter().any(|&m| has_type(g, m, subtype.as_deref().unwrap_or(""))) => {}
                Some("you_any") if kw.player == Some(q) && !kw.attackers.is_empty() => out.push(ctx),
                Some("any_player") if !kw.attackers.is_empty() => out.push(ctx), // Tier 3 (aurelia): "a player attacks"
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
            let mut ctx = DslCtx { event_player: kw.defender, event_perm: m, ..base };
            if let Some(d) = kw.dmg {
                // --- Tier 3 (korvold): "that many" is the damage dealt
                ctx.x = d;
                ctx.x_default = false;
            }
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
        "draw" | "upkeep" | "end_step" | "proliferate" => {
            let pl = kw.player;
            match w {
                Some("you") if pl == Some(q) => out.push(base),
                Some("opponent") if pl.is_some() && pl != Some(q) => out.push(DslCtx { event_player: pl, ..base }),
                Some("any") if event != "draw" => out.push(DslCtx { event_player: pl, ..base }),
                _ => {}
            }
        }
        "landfall" if kw.player == Some(q) => out.push(base),
        // --- Tiers 4/5 (heliod-zur): "whenever you gain life" (Ajani's Pridemate, Voice of the Blessed, Archangel of
        // Thune): fired by engine/life.rs `gain`, once per life-gain event
        "gain_life" => match w {
            Some("you") | None if kw.player == Some(q) => out.push(base),
            Some("opponent") if kw.player.is_some_and(|x| x != q) => out.push(DslCtx { event_player: kw.player, ..base }),
            _ => {}
        },
        _ => {}
    }
    out
}

// ------------------------------------------------------------------ static abilities
/// (controller, source, ability) for every static or replacement ability of this kind on the battlefield
pub(crate) fn statics<'a>(g: &'a Game, kind: &'a str) -> impl Iterator<Item = (PlayerId, PermId, Arc<[Ability]>, usize)> + 'a {
    g.players.iter().filter(|q| q.alive).flat_map(move |q| {
        q.perms
            .iter()
            .copied()
            .chain(land_sources(g, q.id)) // Phase 3: lands' statics, after the other permanents'
            .filter_map(move |src| {
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

/// Power and toughness changes beyond printed stats and counters (Python's `dsl.pt`): `chars::pt_changes`
pub fn pt(g: &Game, m: PermId) -> (i32, i32) {
    crate::engine::chars::pt_changes(g, m)
}

/// Does m have keyword kw (lower case)? Python's `dsl.has_kw`: `chars::has_keyword`
#[inline]
pub fn has_kw(g: &Game, m: PermId, kw: &str) -> bool {
    crate::engine::chars::has_keyword(g, m, kw)
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
            // --- Tiers 1/2 (lightpaws): Transcendent Envoy, Starnheim Courser
            "aura" => d.has_subtype("aura"),
            "artifact_or_enchantment" => d.types.has(Types::ARTIFACT) || d.types.has(Types::ENCHANTMENT),
            // --- Tier 3 (marwyn): the Medallions ("green spells you cast cost {1} less")
            "white" => d.colors.has('W'),
            "blue" => d.colors.has('U'),
            "black" => d.colors.has('B'),
            "red" => d.colors.has('R'),
            "green" => d.colors.has('G'),
            // --- Tiers 4/5 (urdragon): "Dragon spells you cast cost {1} less" (a creature type; changelings too)
            s if s.starts_with("sub_") => d.has_subtype(&s[4..]) || d.has_kw("changeling"),
            // --- Tiers 4/5 (heliod-zur): "white creature spells you cast cost {1} less" (Oketra's Monument)
            s if s.ends_with("_creature") => {
                d.creature && s.split('_').next().and_then(color_letter).is_some_and(|ch| d.colors.has(ch))
            }
            _ => true,
        };
        if ok {
            delta += amount.unwrap_or(0);
        }
    }
    delta
}

/// m's own toxic N (compiled from "toxic N" on its keyword line): poison counters to a player it deals combat
/// damage to, on top of the damage
pub fn toxic_amount(g: &Game, m: PermId) -> i32 {
    let x = g.perm(m);
    let Some(cd) = x.cd.filter(|_| !x.neutered) else { return 0 };
    g.db.get(cd)
        .abilities
        .iter()
        .filter_map(|a| match a {
            Ability::Static { kind, amount, .. } if kind == "toxic" => *amount,
            _ => None,
        })
        .sum()
}

/// "players can't gain life" statics of other players
pub fn no_lifegain(g: &Game, p: PlayerId) -> bool {
    active(g) && statics(g, "no_lifegain").any(|(q, ..)| q != p)
}

// --- Tier 3 (aurelia)
/// a keyword static whose filter names a type wider than creatures ("permanents you control have indestructible")
/// reaches noncreature permanents too
pub(crate) fn noncreature_ok(f: Option<&Filter>) -> bool {
    f.and_then(|f| f.kind.as_deref()).is_some_and(|k| k != "creature")
}

/// do p's lands have keyword kw from a "permanents you control have ..." static (Avacyn, Angel of Hope: lands are
/// not permanents in the engine, so land destruction asks here)
pub fn lands_have_kw(g: &Game, p: PlayerId, kw: &str) -> bool {
    active(g)
        && statics(g, "keyword").any(|(q, _, ab, i)| {
            q == p
                && matches!(&ab[i], Ability::Static { keyword, filter, .. }
                    if keyword.as_deref() == Some(kw)
                        && filter.as_ref().is_some_and(|f| f.kind.as_deref() == Some("permanent")
                            && f.controller.as_deref() == Some("you")))
        })
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
        "poison" => 0.6, // per counter on each player it reaches: ten put a player out
        "extra_combat" => 3.0,
        "ring_tempt" => 0.8, // --- Tiers 4/5 (winota)
        "add_mana" => 0.5,
        "mill" => 0.1,
        "exile_library" => 8.0, // a library of one card: that player is out at the next draw but one
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
            // --- Tiers 4/5 (krenko): a count of your creatures or of a creature type is read now (Goblin War
            // Strike, Massive Raid, Brightstone Ritual)
            Some(Num::Name(s)) if s.starts_with("type_") || s == "creatures_you_control" => {
                num(g, p, e.n.as_ref(), &DslCtx::default(), None) as f64
            }
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
                    Some("player") if d == "damage" && per_player_n(e).is_some() => {
                        // each player's own count, yours against you; an opponent it kills is worth a lot
                        let f = per_player_n(e).unwrap();
                        let k = |q: PlayerId| (f(g, q) * e.mult.unwrap_or(1)) as f64;
                        for q in g.opps(p) {
                            v += 0.6 * k(q) + if k(q) >= g.player(q).life as f64 { 20.0 } else { 0.0 };
                        }
                        v -= 0.9 * k(p) + if k(p) >= g.player(p).life as f64 { 100.0 } else { 0.0 };
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
            "blight" => v += 0.5 * n * g.opps(p).count() as f64, // --- Tiers 1/2 (lathril)
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
            "creature_backlash" => {
                // --- Tiers 4/5 (prosper): a point of life per creature, an opponent out worth a lot
                let n = |q: PlayerId| g.player(q).perms.iter().filter(|&&m| g.is_creature(m)).count() as i32;
                for q in g.opps(p) {
                    v += 0.15 * n(q) as f64 + if n(q) >= g.player(q).life { 20.0 } else { 0.0 };
                }
                v -= 0.2 * n(p) as f64;
            }
            "sacrifice" => v += if e.who.as_deref() != Some("you") { 2.0 } else { -1.0 },
            "extra_land" => {
                // --- Tiers 1/2 (tatyova-teysa): worth the land cards in hand that the drops left this turn can't play
                let pl = g.player(p);
                let lands = pl.hand.iter().filter(|&&c| g.db.get(c).land).count() as f64;
                let left = (1 + crate::cardcode::extra_lands(g, p) + pl.extra_land_now) as f64 - pl.lands_played as f64;
                v += 0.3 + 1.2 * n.min((lands - left.max(0.0)).max(0.0));
            }
            "discard" => v += if e.who.as_deref() != Some("you") { 0.8 } else { -0.5 },
            "mill" if e.who.as_deref() == Some("target_player") => v += mill_value(g, p, n as i32),
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

/// --- Tiers 1/2 (windgrace): the cards in p's hand of type t ("land", "creature", "artifact", "enchantment"), for a
/// typed discard cost
fn discard_typed(g: &Game, p: PlayerId, t: &str) -> Vec<CardId> {
    let ok = |c: CardId| {
        let d = g.db.get(c);
        match t {
            "land" => d.land,
            "creature" => d.creature,
            "artifact" => d.types.has(Types::ARTIFACT),
            "enchantment" => d.types.has(Types::ENCHANTMENT),
            _ => true,
        }
    };
    g.player(p).hand.iter().copied().filter(|&c| ok(c)).collect()
}

fn pay_ability_cost(g: &mut Game, p: PlayerId, src: PermId, cost: &model::Cost, dry: bool) -> Res<bool> {
    let (gn, pips) = cost.mana.as_deref().map(crate::engine::cast::parse_cost).unwrap_or((0, String::new()));
    let x = g.perm(src);
    // --- Tiers 4/5 (krenko): Thousand-Year Elixir, "as though those creatures had haste"
    let sick = x.sick && !crate::impls::t45_krenko::elixir(g, p);
    if cost.tap == Some(true) && (x.tapped || (g.is_creature(src) && sick)) {
        return Ok(false);
    }
    if cost.life.is_some_and(|l| g.player(p).life <= l + 5) {
        return Ok(false);
    }
    if cost.discard.is_some_and(|k| (g.player(p).hand.len() as u32) < k) {
        return Ok(false);
    }
    // --- Tiers 1/2 (windgrace): "discard a land card": enough cards of that type in hand
    if let (Some(k), Some(t)) = (cost.discard, cost.discard_type.as_deref())
        && (discard_typed(g, p, t).len() as u32) < k
    {
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
            .filter(|&m| {
                m != src
                    && matches(g, p, m, Some(&f), None)
                    && (g.perm(m).token || pval(g, m) < 3.0)
                    && (!crate::ai::is_main(hand_key(g, p))
                        || (!g.perm(m).is_cmd
                            && !crate::engine::values::no_sac(g, m)
                            && !crate::impls::common::is_payoff(g, m)))
            })
            .collect();
        // --- ghave: a response sacrifices the creature it is saving (`activate_sacrificing`)
        let forced = FORCED_FODDER.with(|c| c.get()).filter(|&m| m != src && matches(g, p, m, Some(&f), None));
        let Some(f0) = forced.or_else(|| min_by(&fods, |m| pval(g, m))) else { return Ok(false) };
        fod = Some(f0);
    }
    // --- ghave: "remove a +1/+1 counter from a creature you control" (Ghave, Guru of Spores): the counter comes off
    // the creature that misses it least (the most counters, one that survives it; a token before a card)
    let mut ctr_from = None;
    if cost.other.as_deref() == Some(REMOVE_PLUS_COST) {
        let cands: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| g.is_creature(m) && !g.perm(m).phased && g.perm(m).plus > 0 && etgh(g, m) > 1)
            .collect();
        let Some(m) = max_by(&cands, |m| g.perm(m).plus as f64 + if g.perm(m).token { 0.5 } else { 0.0 }) else {
            return Ok(false);
        };
        ctr_from = Some(m);
    } else if cost.other.is_some() || cost.tap_other.is_some() || cost.energy.is_some() {
        return Ok(false);
    }
    // Phase 3: a land whose ability costs {T} can't also tap for the mana that pays it
    let land = g.player(p).lands.contains(&src);
    let land_tap = land && cost.tap == Some(true);
    let can = |g: &mut Game, n: u32| {
        if land_tap { crate::impls::lands::can_pay_without(g, p, src, n, &pips) } else { can_pay(g, p, n, &pips, false) }
    };
    let mut gn = gn;
    if let Some(k) = cost.x.filter(|&k| k > 0) {
        // an {X} cost: the largest X the mana pays (at least 1: X = 0 does nothing worth a card)
        let xv = (1..=30u32).take_while(|&x| can(&mut *g, gn + k * x)).last().unwrap_or(0);
        if xv == 0 {
            return Ok(false);
        }
        gn += k * xv;
        LAST_X.with(|c| c.set(xv as i32));
    }
    let payable = can(g, gn);
    if !payable {
        return Ok(false);
    }
    if dry {
        return Ok(true);
    }
    if land_tap {
        crate::impls::lands::pay_without(g, p, src, gn, &pips)?;
    } else {
        pay(g, p, gn, &pips, false)?;
    }
    if cost.tap == Some(true) {
        g.perm_mut(src).tapped = true;
        crate::impls::t0_captain::became_tapped(g, src); // Captain America, Living Legend
        crate::impls::t3_marwyn::on_tapped(g, src)?; // --- Tier 3 (marwyn): Dionus, Legolas
    }
    if let Some(l) = cost.life {
        lose_life(g, p, l, Some(p), "other", None)?;
    }
    if let Some(k) = cost.discard {
        match cost.discard_type.as_deref() {
            Some(t) => {
                for _ in 0..k {
                    let cs = discard_typed(g, p, t);
                    let Some(c) = min_by(&cs, |c| crate::engine::tutors::card_worth(g, p, c, false)) else { break };
                    zones::discard_cards(g, p, &[c])?;
                }
            }
            None => crate::engine::cast::discard_worst(g, p, k)?,
        }
    }
    if let Some(m) = ctr_from {
        g.perm_mut(m).plus -= 1;
    }
    LAST_SAC_MV.with(|c| c.set(fod.and_then(|f| g.perm(f).cd).map_or(0, |c| g.db.get(c).cmc as i32)));
    if let Some(f) = fod {
        zones::die(g, f, "sac")?;
    }
    if cost.sac.as_deref() == Some("self") {
        if land {
            crate::impls::lands::sac_land(g, p, src)?; // Phase 3: a land with "sacrifice it" goes as lands do
        } else {
            zones::die(g, src, "sac")?;
        }
    }
    Ok(true)
}

/// an effect with its "X" amounts read as the X paid (for valuing an {X} ability before it is activated)
fn x_paid(mut e: Effect, x: i32) -> Effect {
    let fix = |n: &mut Option<Num>| {
        if matches!(n, Some(Num::Name(s)) if s == "X") {
            *n = Some(Num::Int(x));
        }
    };
    fix(&mut e.n);
    fix(&mut e.pow);
    fix(&mut e.tgh);
    e.modes = e.modes.into_iter().map(|m| m.into_iter().map(|f| x_paid(f, x)).collect()).collect();
    e
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
    // Phase 3: p's lands after p's other permanents, but not those whose abilities run through another path: a
    // fetch land (the f tag: turn::crack_fetch) and a land whose card code offers its plays (land_options)
    let lands: Vec<PermId> = land_sources(g, p)
        .filter(|&l| {
            let c = g.perm(l).card();
            !g.db.get(c).tag(Tag::F) && g.registry.get(c).is_none_or(|i| i.land_options.is_none())
        })
        .collect();
    let mut srcs = g.player(p).perms.clone();
    srcs.extend(lands);
    for src in srcs {
        let x = g.perm(src);
        let Some(cd) = x.cd else { continue };
        if x.phased || !g.db.get(cd).has_dsl() || crate::engine::values::stopped(g, &g.db.get(cd).name) {
            continue; // (Disruptor Flute)
        }
        if crate::cardcode::ability_locked(g, src, p) {
            continue;
        }
        let name = g.db.get(cd).name.to_string();
        let held = crate::impls::karn::hold_ability(g, p, &name); // the karn deck keeps its Baubles
        let land = g.db.get(cd).land;
        for (i, a) in abilities(g, cd).iter().enumerate() {
            match a {
                Ability::Activated { cost, effects, sorcery, once, ai_min } => {
                    if held || (land && channel(cost)) {
                        continue; // a land's channel ability is used from the hand
                    }
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
                    // an {X} cost: valued at the X the mana pays now; at the end of the turn before yours the mana
                    // would go unused, so only the fixed part of the cost counts
                    let xv = if cost.x.is_some() { LAST_X.with(|c| c.take()) } else { 0 };
                    let with_x: Vec<Effect>;
                    let effects: &[Effect] = if xv > 0 {
                        with_x = effects.iter().cloned().map(|e| x_paid(e, xv)).collect();
                        &with_x
                    } else {
                        effects
                    };
                    let x_cost = if sorcery_ok { 0.25 * (cost.x.unwrap_or(0) as i32 * xv) as f64 } else { 0.0 };
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
                    cost_v += x_cost;
                    if cost.other.as_deref() == Some(REMOVE_PLUS_COST) {
                        cost_v += 0.6; // --- ghave: the counter is a point of power and toughness
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

/// The compiled activated abilities of p's permanent src a person can use now (practice mode): (label, index for
/// `activate`). Sorcery-speed ones only on p's own turn; whether the cost can be paid is checked when activated (from
/// the person's pool), so an ability shows even before its mana is in the pool.
pub fn human_abilities(g: &Game, p: PlayerId, src: PermId) -> Vec<(String, u16)> {
    let x = g.perm(src);
    let Some(cd) = x.cd else { return vec![] };
    if !active(g) || x.phased || !g.db.get(cd).has_dsl() || crate::cardcode::ability_locked(g, src, p) {
        return vec![];
    }
    let mut out = vec![];
    for (i, a) in abilities(g, cd).iter().enumerate() {
        let Ability::Activated { cost, effects, sorcery, .. } = a else { continue };
        if *sorcery == Some(true) && g.active != Some(p) {
            continue;
        }
        out.push((format!("{}: {}", cost_text(cost), effects_text(effects)), i as u16));
    }
    out
}

/// "{2}, {T}, sacrifice it" from a compiled cost
fn cost_text(cost: &model::Cost) -> String {
    let mut parts = vec![];
    if let Some(m) = cost.mana.as_deref() {
        let (gn, pips) = crate::engine::cast::parse_cost(m);
        let xs = "{X}".repeat(cost.x.unwrap_or(0) as usize);
        let mut s = if gn > 0 || (pips.is_empty() && xs.is_empty()) { format!("{{{gn}}}") } else { String::new() };
        s.insert_str(0, &xs);
        for ch in pips.chars() {
            s.push_str(&format!("{{{ch}}}"));
        }
        parts.push(s);
    }
    if cost.tap == Some(true) {
        parts.push("{T}".into());
    }
    match cost.sac.as_deref() {
        Some("self") => parts.push("sacrifice it".into()),
        Some(k) => parts.push(format!("sacrifice a {k}")),
        None => {}
    }
    if let Some(l) = cost.life {
        parts.push(format!("pay {l} life"));
    }
    if let Some(k) = cost.discard {
        match cost.discard_type.as_deref() {
            Some(t) => parts.push(format!("discard {k} {t}")), // --- Tiers 1/2 (windgrace)
            None => parts.push(format!("discard {k}")),
        }
    }
    if parts.is_empty() { "{0}".into() } else { parts.join(", ") }
}

/// a short reading of compiled effects ("search", "draw") for an ability's label
fn effects_text(effects: &[model::Effect]) -> String {
    let words: Vec<String> = effects.iter().map(|e| e.kind.replace('_', " ")).collect();
    if words.is_empty() { "its effect".into() } else { words.join(", then ") }
}

/// p activates src's compiled ability idx (an option from ability_options); false if it can't now
pub fn activate(g: &mut Game, p: PlayerId, src: PermId, idx: u16) -> Res<bool> {
    let Some(cd) = g.perm(src).cd else { return Ok(false) };
    let ab = abilities(g, cd);
    let Ability::Activated { cost, effects, .. } = &ab[idx as usize] else { return Ok(false) };
    if g.locks_used
        && !effects.iter().all(|e| e.kind == "add_mana")
        && (crate::engine::cast::abilities_locked(g, p) || crate::engine::cast::split_second_up(g))
    {
        return Ok(false); // Abeyance, split second: no non-mana abilities
    }
    if !g.perm(src).on_bf || g.perm(src).owner != p || !pay_ability_cost(g, p, src, cost, false)? {
        return Ok(false);
    }
    let sacrificed_mv = LAST_SAC_MV.with(|c| c.take()); // before the stack window runs other costs
    let paid_x = if cost.x.is_some() { LAST_X.with(|c| c.take()) } else { 0 };
    let stamp = round_stamp(g, p);
    let e = g.player_mut(p).act_uses.entry((src, idx)).or_insert((stamp, 0));
    *e = if e.0 == stamp { (stamp, e.1 + 1) } else { (stamp, 1) };
    crate::glog!(g, "  {} activates {}", g.player(p).name, g.perm(src).name);
    if !effects.iter().all(|e| e.kind == "add_mana")
        && !crate::engine::stack::ability_window(g, p, Some(src), "ability", None, None)?
    {
        return Ok(true); // countered (mana abilities skip the stack)
    }
    let mut dctx = DslCtx { sacrificed_mv, ..DslCtx::base() };
    if cost.x.is_some() {
        (dctx.x, dctx.x_default) = (paid_x, false);
    }
    execute(g, p, effects, Some(src), &mut dctx, None, 0)?;
    Ok(true)
}

/// --- ghave: p's compiled "{N}, sacrifice a creature: ..." ability that can sacrifice `m` now (Ghave, Guru of Spores):
/// a paid sacrifice outlet for the answers to removal and wipes (`common::sac_in_response`, `sac_before_wipe`), which
/// otherwise know only the free outlets. Mana-only costs; not m's own ability.
pub fn paid_outlet(g: &Game, p: PlayerId, m: PermId) -> Option<(PermId, u16)> {
    if !active(g) {
        return None;
    }
    for &src in &g.player(p).perms {
        let x = g.perm(src);
        let Some(cd) = x.cd else { continue };
        if src == m || x.phased || !g.db.get(cd).has_dsl() || crate::engine::values::stopped(g, &g.db.get(cd).name) {
            continue;
        }
        if crate::cardcode::ability_locked(g, src, p) {
            continue;
        }
        for (i, a) in abilities(g, cd).iter().enumerate() {
            let Ability::Activated { cost, sorcery, .. } = a else { continue };
            let plain = cost.tap.is_none_or(|t| !t)
                && cost.life.is_none()
                && cost.discard.is_none()
                && cost.other.is_none()
                && cost.tap_other.is_none()
                && cost.energy.is_none();
            if *sorcery == Some(true) || !plain || cost.sac.as_deref() != Some("creature") {
                continue;
            }
            let (gn, pips) = cost.mana.as_deref().map(crate::engine::cast::parse_cost).unwrap_or((0, String::new()));
            if can_pay(g, p, gn, &pips, false) {
                return Some((src, i as u16));
            }
        }
    }
    None
}

/// --- ghave: p activates src's compiled ability idx sacrificing `m` for its cost (the creature a response saves)
pub fn activate_sacrificing(g: &mut Game, p: PlayerId, src: PermId, idx: u16, m: PermId) -> Res<bool> {
    FORCED_FODDER.with(|c| c.set(Some(m)));
    let r = activate(g, p, src, idx);
    FORCED_FODDER.with(|c| c.set(None));
    r
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
    let loyalty = if *loyalty > 0 {
        repl::counters(g, p, On::Perm(src), Ctr::Loyalty, *loyalty, Path::VORINCLEX)
    } else {
        *loyalty
    };
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

/// how much better (in `equip_options` score) a creature must be to take an Equipment off its current holder
const MOVE_MARGIN: f64 = 2.0;

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
        if x.attached.is_some_and(|t| g.perm(t).on_bf && g.perm(t).owner == p && !g.perm(t).phased)
            && !crate::impls::cloud::wants_move(g, p, src)
        {
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
            .filter(|&m| {
                g.is_creature(m)
                    && !g.perm(m).phased
                    && !g.perm(m).noatk
                    && !untargetable(g, m)
                    && x.attached != Some(m)
            })
            .collect();
        let base = n;
        let cands: Vec<PermId> = cands
            .into_iter()
            .filter(|&m| can_pay(g, p, crate::impls::cloud::equip_cost(g, p, src, m, base), "", false))
            .collect(); // equip cost changes (cloud.rs)
        if cands.is_empty() {
            continue;
        }
        let score = |m: PermId| {
            let y = g.perm(m);
            let flies = y.fly || has_kw(g, m, "flying");
            (if y.army { 3.0 } else { 0.0 }) + if flies { 2.0 } else { 0.0 } + 0.3 * epow(g, m) as f64
                - if y.sick { 1.0 } else { 0.0 }
                + crate::impls::cloud::holder_bonus(g, p, m)
                + crate::impls::tymna_cards::equip_holder_value(g, p, src, m)
        };
        let best = max_by(&cands, score).unwrap();
        // moving a worn piece: only to a clearly better holder (else it ping-pongs between equals)
        if let Some(cur) = x.attached.filter(|&t| g.perm(t).on_bf && g.perm(t).owner == p && !g.perm(t).phased)
            && score(best) < score(cur) + MOVE_MARGIN
        {
            continue;
        }
        let n = crate::impls::cloud::equip_cost(g, p, src, best, base);
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
        let u = 2.5 + 0.5 * bonus as f64 + 2.0 * trig as f64 - 0.3 * n as f64
            + crate::impls::cloud::equip_utility_bonus(g, p, src, best)
            + crate::impls::tymna_cards::equip_utility_value(g, p, src, best);
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

// ------------------------------------------------------------------ borrowed abilities (Marvin, Murderous Mimic)
/// Marvin: the `{T}` abilities (no mana cost) of the compiled creature card `from`, as (ability index, AI value)
/// for `src`, a permanent that has them; only those `src` can pay for right now
pub fn borrowed_tap_options(g: &mut Game, p: PlayerId, src: PermId, from: CardId) -> Res<Vec<(u16, f64)>> {
    let mut out = vec![];
    if !active(g) {
        return Ok(out);
    }
    for (i, a) in abilities(g, from).iter().enumerate() {
        let Ability::Activated { cost, effects, sorcery, .. } = a else { continue };
        if cost.tap != Some(true)
            || cost.mana.is_some()
            || cost.sac.is_some()
            || cost.life.is_some()
            || *sorcery == Some(true)
        {
            continue;
        }
        if !pay_ability_cost(g, p, src, cost, true)? {
            continue;
        }
        let u = value_of(g, p, effects, None) - if cost.discard.is_some() { 0.5 } else { 0.0 };
        out.push((i as u16, u));
    }
    Ok(out)
}

/// p activates ability `idx` of card `from` with permanent `src` (which doesn't have it printed): pays the cost from
/// `src`, puts it on the stack and resolves it with `src` as the source. False if it can't be paid now.
pub fn activate_borrowed(g: &mut Game, p: PlayerId, src: PermId, from: CardId, idx: u16) -> Res<bool> {
    let ab = abilities(g, from);
    let Some(Ability::Activated { cost, effects, .. }) = ab.get(idx as usize) else { return Ok(false) };
    if !g.perm(src).on_bf || g.perm(src).owner != p || !pay_ability_cost(g, p, src, cost, false)? {
        return Ok(false);
    }
    crate::glog!(g, "  {} activates {} (copied ability)", g.player(p).name, g.perm(src).name);
    if !crate::engine::stack::ability_window(g, p, Some(src), "ability", None, None)? {
        return Ok(true);
    }
    execute(g, p, effects, Some(src), &mut DslCtx::base(), None, 0)?;
    Ok(true)
}

// --- Tiers 4/5 (heliod-zur)
/// a colour word's letter ("white" -> 'W')
fn color_letter(w: &str) -> Option<char> {
    match w {
        "white" => Some('W'),
        "blue" => Some('U'),
        "black" => Some('B'),
        "red" => Some('R'),
        "green" => Some('G'),
        _ => None,
    }
}

/// "creatures your opponents control enter tapped" (Authority of the Consuls): does m, entering under p's control,
/// enter tapped? (None: a Treasure / Food / Clue, never a creature)
pub fn opp_enters_tapped(g: &Game, p: PlayerId, m: Option<PermId>) -> bool {
    let Some(m) = m else { return false };
    active(g)
        && g.is_creature(m)
        && statics(g, "enters_tapped").any(|(q, src, ab, i)| {
            matches!(&ab[i], Ability::Static { who, .. } if who.as_deref() == Some("opponents")) && q != p && src != m
        })
}
