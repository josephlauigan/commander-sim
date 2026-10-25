//! Tier 4 pool deck `heliod-mono-white-stax` and Tier 5 `zur-esper-enchantment-control` (EDHREC's bracket-4 average
//! lists): the cards the ability language doesn't cover (rust/agents/t45-heliod-zur.md). Compiled under
//! `// --- Tiers 4/5 (heliod-zur)` in dsl_parse.rs: Authority of the Consuls (opponents' creatures enter tapped),
//! Suture Priest ("have that player lose 1 life"), Oketra's Monument (white creature spells cost {1} less), The Wind
//! Crystal (life gain doubled), Cleric Class's "plus 1" and Daxos's "enters or dies" life gain; the "whenever you
//! gain life" triggers now fire (engine/life.rs). The rest is here.
//!
//! Heliod:
//! - Cosmos Elixir: your end step, draw a card above your starting life total (40), else gain 2 life.
//! - Heliod's Intervention: destroy X opposing artifacts and/or enchantments, or a player gains twice X life.
//! - Well of Lost Dreams: whenever you gain life, pay up to that much and draw that many.
//! - Ajani, Strength of the Pride: +1 life per creature and planeswalker; -2 an Ajani's Pridemate token (a token copy
//!   of the card, so its compiled trigger comes along); 0 at 55+ life: exile Ajani and each opposing artifact and
//!   creature.
//! - Altar of the Pantheon: +1 devotion to each colour (rules2::devotion); tapped for mana with a God, Demigod or
//!   legendary enchantment out: gain 1 life.
//! - Archivist of Oghma: an opponent's library search (each shuffle after a search, tutors::shuffle_library): gain 1
//!   life and draw. It runs with the state checks rather than on the stack. Left out: searches that don't shuffle.
//! - Cleric Class: levels 2 ({3}{W}: whenever you gain life, a +1/+1 counter on a creature you control) and 3 ({4}{W}:
//!   the best creature card from your graveyard returns, gain its toughness). Level 1's "plus 1" is compiled.
//! - Daxos, Blessed by the Sun: toughness equal to your devotion to white (its life gain is compiled).
//! - Felidar Sovereign: your upkeep with 40 or more life: you win.
//! - Haliya, Guided by Light: it or another creature or artifact of yours enters: gain 1; your end step, draw if you
//!   gained 3 or more this turn. Left out: warp, and Treasure / Food / Clue tokens (counters here) as artifacts.
//! - Linden, the Steadfast Queen: each white creature of yours that attacks: gain 1 life.
//! - Serra Ascendant: +5/+5 and flying with 30 or more life.
//! - Voice of the Blessed: flying and vigilance with 4+ counters, indestructible with 10+ (its counters are compiled).
//! - Buried Ruin: {2},{T}, sacrifice: an artifact card from the graveyard to hand.
//! - Hall of Heliod's Generosity: {1}{W},{T}: an enchantment card from the graveyard on top of the library.
//! - Heliod, Sun-Crowned + Triskelion: the lifelink loop (combo catalogue, `COMBOS`).
//!
//! Zur:
//! - Diplomatic Immunity, Steel of the Godhead: Auras (common::aura); Steel's bonus by the host's colours.
//! - Estrid's Invocation: enters as a copy of your best nonlegendary, non-Aura enchantment. Left out: the upkeep
//!   blink the copy gains.
//! - Solemnity: no counters on players, artifacts, creatures, enchantments or lands (`solemnity`, read where +1/+1,
//!   -1/-1 and poison counters are put).
//! - Delney, Streetwise Lookout: your power-2-or-less creatures can't be blocked by power 3 or more
//!   (cardcode::evasion_blocked); their triggered abilities trigger an additional time (stack::queue_triggers).
//! - Eidolon of Rhetoric: one spell per player per turn (common's spell limits).
//! - Imprisoned in the Moon: removal kind `moon` (data/card_overrides.toml): the creature or planeswalker becomes a
//!   colourless land (a Wastes) for good.

use crate::cards::{CardDb, Types};
use crate::engine::cast::cast_card;
use crate::engine::life::gain;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::trigger_window;
use crate::engine::tutors::card_worth;
use crate::engine::values::{epow, pval, untargetable};
use crate::engine::zones::{Enter, Tokens, draw, enter, enter_token_copy, exile_perm, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::common::{AURA, AuraSpec, WalkerAb, aura, walker};
use crate::impls::lands::{can_pay_without, pay_without, sac_land};
use crate::state::{CopyExc, CopyOf, Ctx, DataKey, Game, Step, Val};
use crate::sym::{Sym, intern};

/// the commander starting life total
const START_LIFE: i32 = 40;
const ARCHIVIST: &str = "Archivist of Oghma";
const SOLEMNITY: &str = "Solemnity";
const DELNEY: &str = "Delney, Streetwise Lookout";
const INTERVENTION: &str = "Heliod's Intervention";
const CLERIC: &str = "Cleric Class";
const ALTAR: &str = "Altar of the Pantheon";

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn name_is(g: &Game, m: PermId, name: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| cname(g, c) == name)
}

/// any live, unneutered permanent of this name (any player's)
fn anyone_has(g: &Game, name: &str) -> bool {
    g.players.iter().any(|q| q.perms.iter().any(|&m| live(g, m) && !g.perm(m).neutered && name_is(g, m, name)))
}

/// p's live, unneutered permanents of this name
fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && !g.perm(m).neutered && name_is(g, m, name)).collect()
}

fn card_has(g: &Game, m: PermId, t: Types) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(t))
}

/// a permanent that is an artifact (a card's type, or an artifact token)
fn is_artifact(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    card_has(g, m, Types::ARTIFACT) || (x.cd.is_none() && x.ttypes.contains(&"artifact"))
}

/// p's live creatures
fn creatures_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

/// the creature a +1/+1 counter does most for: a Walking Ballista or Triskelion first (a ping), else the best one
fn counter_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs = creatures_of(g, p);
    cs.iter()
        .copied()
        .find(|&m| name_is(g, m, "Walking Ballista") || name_is(g, m, "Triskelion"))
        .or_else(|| max_by(&cs, |m| pval(g, m) + if g.perm(m).fly { 0.5 } else { 0.0 }))
}

fn is_my_main(g: &Game, p: PlayerId) -> bool {
    g.active == Some(p) && matches!(g.step, Step::Main1 | Step::Main2)
}

// ================================================================== Solemnity
/// **Solemnity**: players can't get counters, and counters can't be put on artifacts, creatures, enchantments or
/// lands. Read where +1/+1 and -1/-1 counters are put (cardcode::put_plus, the ability language's counters, enters
/// with counters, proliferate) and poison counters are given (values::melira). Loyalty counters (planeswalkers) still
/// go on. Left out: lore, charge, oil and other kept counters.
pub fn solemnity(g: &Game) -> bool {
    anyone_has(g, SOLEMNITY)
}

// ================================================================== Altar of the Pantheon
/// your devotion to each colour is one more per Altar of the Pantheon (rules2::devotion)
pub fn devotion_bonus(g: &Game, p: PlayerId) -> usize {
    named(g, p, ALTAR).len()
}

/// {T}: mana; with a God, a Demigod or a legendary enchantment out, gain 1 life
fn altar_tap(g: &mut Game, p: PlayerId, m: PermId, _n: u32) -> Res {
    if owner(g, m) != p || g.perm(m).neutered {
        return Ok(());
    }
    let ok = g.player(p).perms.iter().any(|&x| {
        live(g, x)
            && g.perm(x).cd.is_some_and(|c| {
                let d = g.db.get(c);
                d.has_subtype("god")
                    || d.has_subtype("demigod")
                    || (d.tag(crate::tag::Tag::Leg) && d.types.has(Types::ENCHANTMENT))
            })
    });
    if ok {
        gain(g, p, 1)?;
    }
    Ok(())
}

// ================================================================== Cosmos Elixir
/// your end step: above your starting life total, draw a card; otherwise gain 2 life
fn elixir_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !live(g, src) {
        return Ok(());
    }
    let label = if g.player(p).life > START_LIFE { "draw a card" } else { "gain 2 life" };
    if !trigger_window(g, p, Some(src), label, Some(2.0))? {
        return Ok(());
    }
    if g.player(p).life > START_LIFE { draw(g, p, 1, false) } else { gain(g, p, 2) }
}

// ================================================================== Well of Lost Dreams
/// whenever you gain life, you may pay {X} (X up to the life gained) to draw X: the AI pays what it can, keeping three
/// mana in its own turn before its second main phase
fn well_gain(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    if p != owner(g, src) || n <= 0 || !live(g, src) {
        return Ok(());
    }
    let keep = |g: &Game| {
        if g.active == Some(p) && matches!(g.step, Step::Start | Step::Main1 | Step::Combat) { 3 } else { 0 }
    };
    let x = |g: &Game| (total_mana(g, p, false) as i32 - keep(g)).min(n).max(0) as u32;
    if x(g) == 0 || !trigger_window(g, p, Some(src), "pay X, draw X", Some(2.5))? {
        return Ok(());
    }
    let x = x(g);
    if x == 0 || !pay(g, p, x, "", false)? {
        return Ok(());
    }
    crate::glog!(g, "    {} pays {} to draw {} (Well of Lost Dreams)", pname(g, p), x, x);
    draw(g, p, x, false)
}

// ================================================================== Heliod's Intervention
/// the opposing artifacts and enchantments worth destroying, best first
fn intervention_targets(g: &Game, p: PlayerId) -> Vec<PermId> {
    let mut ts: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && (is_artifact(g, m) || card_has(g, m, Types::ENCHANTMENT)) && !untargetable(g, m))
        .filter(|&m| pval(g, m) >= 1.5)
        .collect();
    ts.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
    ts
}

/// the most X p can pay for the spell now
fn intervention_max_x(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn, &pips, false) {
        return -1;
    }
    total_mana(g, p, false) as i32 - gn as i32 - pips.chars().count() as i32
}

/// mode 1 when two or more opposing artifacts / enchantments are worth it (one with little else to do late), mode 2
/// to reach Felidar Sovereign's 40 life before your upkeep
fn intervention_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    let max_x = intervention_max_x(g, p, c);
    if max_x < 1 {
        return Ok(vec![]);
    }
    let mut out = vec![];
    let ts = intervention_targets(g, p);
    let x = (ts.len() as i32).min(max_x);
    if x >= 2 || (x == 1 && pval(g, ts[0]) >= 4.0) {
        let worth: f64 = ts.iter().take(x as usize).map(|&m| pval(g, m)).sum();
        out.push(Opt {
            utility: 0.8 + 0.45 * worth,
            label: format!("{INTERVENTION}: destroy {x}"),
            act: Some(Action::Plan { f: intervention_go, arg: ((c.0 as i64) << 8) | x as i64 }),
        });
    }
    // life: at the end of an opponent's turn, for Felidar Sovereign's 40
    let felidar = !named(g, p, "Felidar Sovereign").is_empty();
    let life = g.player(p).life;
    if post.is_none() && g.active != Some(p) && felidar && life < 40 && life + 2 * max_x >= 40 {
        let x = (40 - life + 1) / 2;
        out.push(Opt {
            utility: 6.0,
            label: format!("{INTERVENTION}: gain {}", 2 * x),
            act: Some(Action::Plan { f: intervention_go, arg: ((c.0 as i64) << 8) | 0x80 | x as i64 }),
        });
    }
    Ok(out)
}

fn intervention_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg >> 8) as u16);
    let (life_mode, x) = (arg & 0x80 != 0, (arg & 0x7f) as i32);
    if !g.player(p).hand.contains(&c) || x < 1 || intervention_max_x(g, p, c) < x {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, p, c);
    if !pay(g, p, gn + x as u32, &pips, false)? {
        return Ok(false);
    }
    let rem = if life_mode { None } else { Some(intern("destroy")) };
    cast_card(g, p, c, "hand", Ctx { x, rem_kind: rem, ..Ctx::default() })?;
    Ok(true)
}

fn intervention_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if ctx.rem_kind == Some("destroy") {
        for m in intervention_targets(g, p).into_iter().take(ctx.x.max(0) as usize) {
            if live(g, m) {
                apply_removal(g, Some(p), m, "destroy", Some(c))?;
            }
        }
    } else {
        gain(g, p, 2 * ctx.x.max(0))?;
    }
    Ok(intern("gy"))
}

// ================================================================== Ajani, Strength of the Pride
fn ajani_count(g: &Game, p: PlayerId) -> i32 {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| live(g, m) && (g.is_creature(m) || card_has(g, m, Types::PLANESWALKER)))
        .count() as i32
}

fn ajani_plus_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    Some(0.8 + 0.25 * ajani_count(g, p) as f64)
}

/// +1: gain life equal to the creatures plus the planeswalkers you control
fn ajani_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let n = ajani_count(g, p);
    gain(g, p, n)
}

fn ajani_minus_v(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(1.6)
}

/// -2: a 2/2 white Cat Soldier named Ajani's Pridemate that grows when you gain life (a token copy of the card)
fn ajani_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    match g.db.id("Ajani's Pridemate") {
        Some(c) => {
            enter_token_copy(g, p, c)?;
        }
        None => {
            make_tokens(g, p, Tokens { types: vec!["cat", "soldier"], ..Tokens::new(1, 2) })?;
        }
    }
    Ok(())
}

/// the opposing artifacts and creatures Ajani's 0 exiles
fn ajani_victims(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && (g.is_creature(m) || is_artifact(g, m)))
        .collect()
}

fn ajani_zero_v(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    if g.player(p).life < START_LIFE + 15 {
        return None;
    }
    let worth: f64 = ajani_victims(g, p).iter().map(|&m| pval(g, m)).sum();
    let mine = pval(g, src);
    (worth >= 4.0 && worth >= 0.75 * mine).then_some(1.0 + 0.5 * worth)
}

/// 0: with 15 or more life above your starting total, exile Ajani and each artifact and creature your opponents
/// control
fn ajani_zero(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    if g.player(p).life < START_LIFE + 15 {
        return Ok(());
    }
    crate::glog!(g, "    Ajani exiles itself and each opposing artifact and creature");
    if live(g, src) {
        exile_perm(g, src)?;
    }
    for m in ajani_victims(g, p) {
        if live(g, m) {
            exile_perm(g, m)?;
        }
    }
    Ok(())
}

static AJANI: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "gain life", val: ajani_plus_v, eff: ajani_plus },
    WalkerAb { delta: -2, label: "Ajani's Pridemate", val: ajani_minus_v, eff: ajani_minus },
    WalkerAb { delta: 0, label: "exile their artifacts and creatures", val: ajani_zero_v, eff: ajani_zero },
];

// ================================================================== Archivist of Oghma
/// an opponent of each Archivist searched its library (tutors::shuffle_library, the shuffle after a search): the
/// Archivist answers with the state checks (`archivist_sba`)
pub fn searched(g: &mut Game, p: PlayerId) {
    if g.hooks.is_empty() {
        return;
    }
    let arcs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && !g.perm(m).neutered && name_is(g, m, ARCHIVIST))
        .collect();
    for m in arcs {
        let n = g.perm(m).data.int(DataKey::T45Search);
        g.perm_mut(m).data.set(DataKey::T45Search, Val::Int(n + 1));
    }
}

/// gain 1 life and draw a card for each search still to answer
fn archivist_sba(g: &mut Game, src: Src) -> Res {
    let n = g.perm(src).data.int(DataKey::T45Search);
    if n == 0 {
        return Ok(());
    }
    g.perm_mut(src).data.remove(DataKey::T45Search);
    if !live(g, src) {
        return Ok(());
    }
    let p = owner(g, src);
    for _ in 0..n {
        crate::glog!(g, "    {} gains 1 life and draws (Archivist of Oghma)", pname(g, p));
        gain(g, p, 1)?;
        draw(g, p, 1, false)?;
    }
    Ok(())
}

// ================================================================== Cleric Class
fn class_level(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.get(DataKey::Level).map_or(1, Val::int)
}

/// level 2: whenever you gain life, a +1/+1 counter on target creature you control
fn cleric_gain(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    if p != owner(g, src) || n <= 0 || class_level(g, src) < 2 || counter_target(g, p).is_none() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "a +1/+1 counter on a creature", None)? {
        return Ok(());
    }
    if let Some(t) = counter_target(g, p) {
        crate::cardcode::put_plus(g, t, 1);
    }
    Ok(())
}

/// the creature card level 3 returns
fn cleric_reanimation(g: &Game, p: PlayerId) -> Option<CardId> {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    max_by(&cs, |c| card_worth(g, p, c, true))
}

/// level 2 ({3}{W}) and level 3 ({4}{W}), one at a time as a sorcery: level 2 with two creatures out, level 3 with a
/// creature card worth returning
fn cleric_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !is_my_main(g, p) || owner(g, src) != p || !live(g, src) {
        return Ok(vec![]);
    }
    let level = class_level(g, src) + 1;
    let generic = match level {
        2 => 3,
        3 => 4,
        _ => return Ok(vec![]),
    };
    if !can_pay(g, p, generic, "W", false) {
        return Ok(vec![]);
    }
    let utility = if level == 2 {
        let n = creatures_of(g, p).len();
        if n < 2 {
            return Ok(vec![]);
        }
        0.9 + 0.2 * n as f64
    } else {
        let Some(c) = cleric_reanimation(g, p) else { return Ok(vec![]) };
        let w = card_worth(g, p, c, true);
        if w <= 0.0 && g.db.get(c).cmc < 3 {
            return Ok(vec![]);
        }
        1.2 + (w / 25.0).clamp(0.0, 3.0)
    };
    Ok(vec![Opt {
        utility,
        label: format!("{CLERIC} level {level}"),
        act: Some(Action::Ability { src, f: cleric_level_up, arg: level }),
    }])
}

fn cleric_level_up(g: &mut Game, src: PermId, p: PlayerId, level: i64) -> Res<bool> {
    let generic = if level == 2 { 3 } else { 4 };
    if !live(g, src) || owner(g, src) != p || class_level(g, src) != level - 1 || !can_pay(g, p, generic, "W", false) {
        return Ok(false);
    }
    pay(g, p, generic, "W", false)?;
    g.perm_mut(src).data.set(DataKey::Level, Val::Int(level));
    crate::glog!(g, "  {} levels up Cleric Class to {}", pname(g, p), level);
    if level == 3 {
        cleric_three(g, src, p)?;
    }
    Ok(true)
}

/// "When this Class becomes level 3": return target creature card from your graveyard; gain life equal to its
/// toughness
fn cleric_three(g: &mut Game, src: PermId, p: PlayerId) -> Res {
    if cleric_reanimation(g, p).is_none() || !trigger_window(g, p, Some(src), "return a creature card", Some(4.0))? {
        return Ok(());
    }
    let Some(c) = cleric_reanimation(g, p) else { return Ok(()) };
    let gy = &mut g.player_mut(p).gy;
    let Some(i) = gy.iter().position(|&x| x == c) else { return Ok(()) };
    gy.remove(i);
    crate::glog!(g, "    {} returns {} (Cleric Class)", pname(g, p), cname(g, c));
    let m = enter(g, p, c, Enter::default())?;
    let t = if g.perm(m).on_bf { crate::engine::values::etgh(g, m) } else { g.db.get(c).tgh };
    gain(g, p, t.max(0))
}

// ================================================================== Daxos, Serra Ascendant, Voice of the Blessed
/// Daxos's toughness is your devotion to white (its printed 2 is the stand-in)
fn daxos_pt(g: &Game, p: PlayerId, m: PermId) -> (i32, i32) {
    let dev = crate::impls::rules2::devotion(g, p, 'W') as i32;
    (0, dev - g.perm(m).tgh)
}

/// as long as you have 30 or more life: +5/+5
fn ascendant_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    if g.player(p).life >= 30 { (5, 5) } else { (0, 0) }
}

/// ... and flying
fn ascendant_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    m == src && kw == "flying" && g.player(owner(g, src)).life >= 30
}

/// four or more +1/+1 counters: flying and vigilance; ten or more: indestructible
fn voice_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    if m != src {
        return false;
    }
    let n = g.perm(src).plus;
    match &*kw {
        "flying" | "vigilance" => n >= 4,
        "indestructible" => n >= 10,
        _ => false,
    }
}

/// a card's own power/toughness rule goes live as it enters
fn selfpt_on(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

// ================================================================== Felidar Sovereign
/// your upkeep, if you have 40 or more life: you win the game
fn felidar_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || g.player(p).life < 40 || !live(g, src) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "win the game", Some(9.0))? || g.player(p).life < 40 {
        return Ok(()); // the "if" is checked again as it resolves
    }
    crate::ai::win(g, p, "Felidar Sovereign", Some(false))
}

// ================================================================== Haliya, Guided by Light
/// it or another creature or artifact you control enters: gain 1 life
fn haliya_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if p != owner(g, src) || !(m == src || g.is_creature(m) || is_artifact(g, m)) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "gain 1 life", Some(1.0))? {
        return Ok(());
    }
    gain(g, p, 1)
}

/// your end step: draw a card if you gained 3 or more life this turn
fn haliya_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !live(g, src) {
        return Ok(());
    }
    let st = g.turn_stamp();
    let gained = |g: &Game| g.player(p).gained_turn.filter(|x| x.0 == st).map_or(0, |x| x.1);
    if gained(g) < 3 || !trigger_window(g, p, Some(src), "draw a card", Some(2.0))? {
        return Ok(());
    }
    draw(g, p, 1, false)
}

// ================================================================== Linden, the Steadfast Queen
/// whenever a white creature you control attacks, gain 1 life (once per such creature)
fn linden_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p != owner(g, src) {
        return Ok(vec![]);
    }
    let white = atk.iter().filter(|&&m| crate::engine::values::colors_of(g, m).has('W')).count();
    if white == 0 || !trigger_window(g, p, Some(src), "gain life", Some(1.0))? {
        return Ok(vec![]);
    }
    for _ in 0..white {
        gain(g, p, 1)?;
    }
    Ok(vec![])
}

// ================================================================== Buried Ruin, Hall of Heliod's Generosity
fn best_gy(g: &Game, p: PlayerId, t: Types) -> Option<CardId> {
    let cs: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).types.has(t) && !g.db.get(c).land).collect();
    max_by(&cs, |c| card_worth(g, p, c, false))
}

fn pack(l: LandId, c: CardId) -> i64 {
    ((c.0 as i64) << 32) | l.0 as i64
}

fn unpack(arg: i64) -> (LandId, CardId) {
    (LandId((arg & 0xffff_ffff) as u32), CardId((arg >> 32) as u16))
}

/// {2},{T}, sacrifice: the best artifact card from your graveyard to your hand (at the end-of-turn window)
fn buried_ruin(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.land(l).tapped || !can_pay_without(g, p, l, 2, "") {
        return Ok(vec![]);
    }
    let Some(c) = best_gy(g, p, Types::ARTIFACT) else { return Ok(vec![]) };
    if card_worth(g, p, c, false) < 40.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.4,
        label: "Buried Ruin".into(),
        act: Some(Action::Plan { f: buried_go, arg: pack(l, c) }),
    }])
}

fn buried_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, c) = unpack(arg);
    if !g.player(p).lands.contains(&l) || g.land(l).tapped || !g.player(p).gy.contains(&c) {
        return Ok(false);
    }
    if !pay_without(g, p, l, 2, "")? {
        return Ok(false);
    }
    let ruin = g.land(l).cd;
    sac_land(g, p, l)?;
    let what = format!("{} to hand", cname(g, c));
    if crate::engine::stack::ability_window_card(g, p, ruin, &what, None, None)? {
        let gy = &mut g.player_mut(p).gy;
        if let Some(i) = gy.iter().position(|&x| x == c) {
            gy.remove(i);
            g.player_mut(p).hand.push(c);
            crate::glog!(g, "  {} returns {} with Buried Ruin", pname(g, p), cname(g, c));
        }
    }
    Ok(true)
}

/// {1}{W},{T}: the best enchantment card from your graveyard on top of your library (at the end-of-turn window)
fn hall(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.land(l).tapped || !can_pay_without(g, p, l, 1, "W") {
        return Ok(vec![]);
    }
    let Some(c) = best_gy(g, p, Types::ENCHANTMENT) else { return Ok(vec![]) };
    if card_worth(g, p, c, false) < 40.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.3,
        label: "Hall of Heliod's Generosity".into(),
        act: Some(Action::Plan { f: hall_go, arg: pack(l, c) }),
    }])
}

fn hall_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, c) = unpack(arg);
    if !g.player(p).lands.contains(&l) || g.land(l).tapped || !g.player(p).gy.contains(&c) {
        return Ok(false);
    }
    if !pay_without(g, p, l, 1, "W")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    let what = format!("{} on top", cname(g, c));
    if crate::engine::stack::ability_window_card(g, p, g.land(l).cd, &what, None, None)? {
        let gy = &mut g.player_mut(p).gy;
        if let Some(i) = gy.iter().position(|&x| x == c) {
            gy.remove(i);
            g.player_mut(p).library.push(c);
            crate::glog!(g, "  {} puts {} on top with Hall of Heliod's Generosity", pname(g, p), cname(g, c));
        }
    }
    Ok(true)
}

// ================================================================== Steel of the Godhead, Diplomatic Immunity
/// white: +1/+1 (and lifelink); blue: +1/+1 (and can't be blocked)
fn steel_bonus(g: &Game, _p: PlayerId, _a: PermId, m: PermId) -> (i32, i32) {
    let cols = crate::engine::values::colors_of(g, m);
    let k = cols.has('W') as i32 + cols.has('U') as i32;
    (k, k)
}

fn steel_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    if g.perm(src).attached != Some(m) || !live(g, src) {
        return false;
    }
    let cols = crate::engine::values::colors_of(g, m);
    match &*kw {
        "lifelink" => cols.has('W'),
        "unblockable" => cols.has('U'),
        _ => false,
    }
}

static STEEL: AuraSpec = AuraSpec { bonus: Some(steel_bonus), ..AURA };
static IMMUNITY: AuraSpec = AuraSpec { kws: &["shroud"], ..AURA };

// ================================================================== Estrid's Invocation
/// enters as a copy of the best enchantment you control (not an Aura, not legendary: the copy would be put into the
/// graveyard by the legend rule)
fn estrid_copy(g: &mut Game, p: PlayerId, _c: CardId) -> Option<CopyOf> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&x| {
            live(g, x)
                && g.perm(x).cd.is_some_and(|c| {
                    let d = g.db.get(c);
                    d.types.has(Types::ENCHANTMENT) && !d.has_subtype("aura") && !d.tag(crate::tag::Tag::Leg)
                })
        })
        .collect();
    let best = max_by(&cs, |x| {
        pval(g, x) + if name_is(g, x, "Necropotence") || name_is(g, x, "Rhystic Study") { 3.0 } else { 0.0 }
    })?;
    Some(CopyOf { cd: g.perm(best).cd?, exc: CopyExc::default() })
}

// ================================================================== Delney, Streetwise Lookout
fn delney_count(g: &Game, p: PlayerId) -> usize {
    named(g, p, DELNEY).len()
}

/// creatures you control with power 2 or less can't be blocked by creatures with power 3 or greater
/// (cardcode::evasion_blocked)
pub fn delney_blocked(g: &Game, b: PermId, a: PermId) -> bool {
    let p = owner(g, a);
    delney_count(g, p) > 0 && g.is_creature(a) && epow(g, a) <= 2 && epow(g, b) >= 3
}

/// a triggered ability of a creature you control with power 2 or less triggers an additional time, once per Delney
/// (stack::queue_triggers)
pub fn delney_extra(g: &Game, src: PermId) -> usize {
    let x = g.perm(src);
    if !x.on_bf || x.phased || !g.is_creature(src) || epow(g, src) > 2 {
        return 0;
    }
    delney_count(g, x.owner)
}

// ================================================================== combos
/// Heliod gives Triskelion lifelink ({1}{W}); each counter it removes pings for 1, the life gained puts a counter
/// back (Heliod): unlimited damage
fn heliod_triskelion(g: &Game, p: PlayerId) -> crate::impls::combos::Ready {
    let on = |n: &str| named(g, p, n).into_iter().next();
    let Some(h) = on("Heliod, Sun-Crowned") else { return (false, vec![], vec![]) };
    if let Some(t) = named(g, p, "Triskelion").into_iter().find(|&m| g.perm(m).plus > 0) {
        return (true, vec![h, t], vec![]);
    }
    if let Some(c) = g.player(p).hand.iter().copied().find(|&c| cname(g, c) == "Triskelion") {
        return (true, vec![h], vec![c]);
    }
    (false, vec![], vec![])
}

pub static COMBOS: [crate::impls::combos::Combo; 1] = [crate::impls::combos::Combo {
    name: "Heliod, Sun-Crowned + Triskelion",
    groups: &[&["Heliod, Sun-Crowned"], &["Triskelion"]],
    ready: heliod_triskelion,
    mana: (1, "W"),
    finish: None,
    xcost: &[],
    steps: &[],
}];

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let has = |n: &str| db.id(n).is_some();
    if has("Cosmos Elixir") {
        r.card(db, "Cosmos Elixir")?.end_step = Some(elixir_end);
    }
    if has("Well of Lost Dreams") {
        r.card(db, "Well of Lost Dreams")?.gain_life = Some(well_gain);
    }
    if has(INTERVENTION) {
        let c = r.card(db, INTERVENTION)?;
        c.resolve = Some(intervention_resolve);
        c.hand_options = Some(intervention_options);
        c.prio = Some(|_, _, _| 0);
    }
    if has("Ajani, Strength of the Pride") {
        walker(r, db, "Ajani, Strength of the Pride", &AJANI)?;
    }
    if has(ALTAR) {
        r.card(db, ALTAR)?.on_tap_perm = Some(altar_tap);
    }
    if has(ARCHIVIST) {
        r.card(db, ARCHIVIST)?.sba = Some(archivist_sba);
    }
    if has(CLERIC) {
        let c = r.card(db, CLERIC)?;
        c.gain_life = Some(cleric_gain);
        c.options = Some(cleric_options);
    }
    for (name, f) in
        [("Daxos, Blessed by the Sun", daxos_pt as crate::hooks::SelfPtFn), ("Serra Ascendant", ascendant_pt)]
    {
        if has(name) {
            let c = r.card(db, name)?;
            c.self_pt = Some(f);
            c.etb = Some(selfpt_on);
            *c = c.at_once(Event::Etb);
        }
    }
    if has("Serra Ascendant") {
        r.card(db, "Serra Ascendant")?.grant_kw = Some(ascendant_kw);
    }
    if has("Voice of the Blessed") {
        r.card(db, "Voice of the Blessed")?.grant_kw = Some(voice_kw);
    }
    if has("Felidar Sovereign") {
        r.card(db, "Felidar Sovereign")?.upkeep = Some(felidar_upkeep);
    }
    if has("Haliya, Guided by Light") {
        let c = r.card(db, "Haliya, Guided by Light")?;
        c.etb = Some(haliya_etb);
        c.end_step = Some(haliya_end);
    }
    if has("Linden, the Steadfast Queen") {
        r.card(db, "Linden, the Steadfast Queen")?.attack = Some(linden_attack);
    }
    if has("Buried Ruin") {
        r.card(db, "Buried Ruin")?.land_options = Some(buried_ruin);
    }
    if has("Hall of Heliod's Generosity") {
        r.card(db, "Hall of Heliod's Generosity")?.land_options = Some(hall);
    }
    // Zur
    if has("Steel of the Godhead") {
        aura(r, db, "Steel of the Godhead", &STEEL)?;
        r.card(db, "Steel of the Godhead")?.grant_kw = Some(steel_kw);
    }
    if has("Diplomatic Immunity") {
        aura(r, db, "Diplomatic Immunity", &IMMUNITY)?;
    }
    if has("Estrid's Invocation") {
        r.card(db, "Estrid's Invocation")?.enter_copy = Some(estrid_copy);
    }
    Ok(())
}
