//! Tier 4 pool deck `krenko-mono-red-goblins` (EDHREC's bracket-4 average list): the cards the ability language
//! doesn't cover (rust/agents/t45-krenko.md). Compiled under `// --- Tiers 4/5 (krenko)` in dsl_parse.rs: Goblin War
//! Strike, Massive Raid, Brightstone Ritual, Hobgoblin Bandit Lord's count, General Kreat's attack trigger and Goblin
//! King's anthem. The rest is here:
//!
//! - Boggart Shenanigans: another Goblin of yours dies (tokens too): 1 damage to an opponent (a lethal one first, else
//!   the biggest threat). Left out: planeswalkers as targets.
//! - Mana Echoes: a creature enters (anyone's; Krenko's tokens as one batch): {C} equal to your creatures that share
//!   a creature type with it, into the pool (it lasts the turn here).
//! - Goblin Chirurgeon: a nontoken creature of yours worth keeping would be destroyed: sacrifice a Goblin (a token
//!   first) to regenerate it (tapped). Left out: regenerating tokens.
//! - Goblin King: other Goblins (every player's) have mountainwalk: unblockable while the defender controls a Mountain
//!   (a nonbasic land under Blood Moon is one). The +1/+1 is compiled.
//! - Hexing Squelcher: "Ward—Pay 2 life" on itself and your other creatures: an opponent's removal spell costs its
//!   caster 2 life, or is countered when they are at 4 life or less. The uncounterable parts are compiled.
//! - Hobgoblin Bandit Lord: the compiled {R},{T} ping reads the Goblins that entered under your control this turn
//!   (`note_entered` counts them as they enter; tokens sacrificed since still count).
//! - Howlsquad Heavy: start your engines (speed 1 as it enters; once on each of your turns an opponent losing life
//!   raises it, up to 4); other Goblins you control have haste; at the beginning of combat on your turn, a 1/1 red
//!   Goblin that attacks if able; at max speed {T}: {R} for each Goblin you control (a mana ability).
//! - Marvin, Murderous Mimic: also has Krenko, Mob Boss's and Kiki-Jiki, Mirror Breaker's {T} abilities while you
//!   control them (on top of the Niv-Mizzet / Fangtail / Blightspeaker / Rummaging Goblin copies and Ioreth's untaps).
//! - Patriar's Seal / Thousand-Year Elixir: {1},{T}: untap a creature of yours whose {T} ability is worth using again
//!   now (Krenko, Kiki-Jiki, Marvin; the Seal only a legendary one); the Elixir also lets your creatures use {T}
//!   abilities as though they had haste (the ability language's tap costs and Krenko read `elixir`).
//! - Sting, the Glinting Dagger: first strike while blocking or blocked by a Goblin or Orc (`combat` reads
//!   `sting_first_strike`); its untap is in t6_godo.rs.
//! - Treasure Nabber: an opponent taps an artifact for mana: you control it until the end of your next turn (it
//!   comes back in your end step: an artifact taken on your own turn goes back that same turn).
//! - Warren Instigator: double strike (keyword); deals combat damage to an opponent: put a Goblin creature card from
//!   your hand onto the battlefield (the most expensive).
//! - Impact Tremors and Purphoros, God of the Forge: Krenko's (and every card-code) Goblin tokens trigger them too
//!   (`tokens_enter`: their enter hooks don't see tokens made by card code), one trigger for the batch.
//!
//! Combos: Krenko + an untapper that pays with colourless (Umbral Mantle, Staff of Domination) + Ashnod's Altar or
//! Mana Echoes (unbounded Goblins and mana; it wins through a payoff or hasty Goblins before combat).

use crate::cards::{CardDb, Colors, Types};
use crate::engine::pool::Kind;
use crate::engine::combat::has_haste;
use crate::engine::control::{Duration, gain_control};
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::{card_name, has_type, pval, subtypes, threat};
use crate::engine::zones::{Enter, Tokens, die, enter, make_tokens, max_by, min_by, sac_worth};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready};
use crate::state::{DataKey, Game, Step, Val};
use crate::sym::Sym;

const KRENKO: &str = "Krenko, Mob Boss";
const KIKI: &str = "Kiki-Jiki, Mirror Breaker";
const MARVIN: &str = "Marvin, Murderous Mimic";
const SEAL: &str = "Patriar's Seal";
const ELIXIR: &str = "Thousand-Year Elixir";
const SQUELCHER: &str = "Hexing Squelcher";
const KING: &str = "Goblin King";
const CHIRURGEON: &str = "Goblin Chirurgeon";
const HOWLSQUAD: &str = "Howlsquad Heavy";
const STING: &str = "Sting, the Glinting Dagger";
const MANTLE: &str = "Umbral Mantle";
const STAFF: &str = "Staff of Domination";
const ALTAR: &str = "Ashnod's Altar";
const ECHOES: &str = "Mana Echoes";

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn named(g: &Game, m: PermId, name: &str) -> bool {
    card_name(g, m) == Some(name)
}

/// p's live permanent with this name
fn mine(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && named(g, m, name))
}

/// n damage to an opponent of p: one it kills first, else the biggest threat ("target player")
fn ping_player(g: &mut Game, p: PlayerId, n: i32) -> Res {
    let os: Vec<PlayerId> = g.opps(p).collect();
    let q = match os.iter().copied().find(|&q| g.player(q).life <= n) {
        Some(q) => Some(q),
        None => max_by(&os, |q| threat(g, p, q)),
    };
    match q {
        Some(q) => lose_life(g, q, n, Some(p), "triggers", None),
        None => Ok(()),
    }
}

// ------------------------------------------------------------------ Hobgoblin Bandit Lord
/// the engine's enter paths (a card, a token copy, a batch of tokens): count the Goblins entering under p's control
pub fn note_entered(g: &mut Game, p: PlayerId, ms: &[PermId]) {
    let k = ms.iter().filter(|&&m| g.is_creature(m) && has_type(g, m, "goblin")).count() as u32;
    if k == 0 {
        return;
    }
    let st = g.turn_stamp();
    let pl = g.player_mut(p);
    pl.goblins_in = if pl.goblins_in.0 == Some(st) { (Some(st), pl.goblins_in.1 + k) } else { (Some(st), k) };
}

/// "the number of <type>s that entered the battlefield under your control this turn": Goblins as counted; another
/// type by the ones still here that entered this turn
pub fn entered_this_turn(g: &Game, p: PlayerId, t: &str) -> i32 {
    if t == "goblin" {
        let x = g.player(p).goblins_in;
        return if x.0 == Some(g.turn_stamp()) { x.1 as i32 } else { 0 };
    }
    let new = |m: PermId| g.perm(m).born > g.turn_enter && live(g, m) && g.is_creature(m) && has_type(g, m, t);
    g.player(p).perms.iter().filter(|&&m| new(m)).count() as i32
}

// ------------------------------------------------------------------ Boggart Shenanigans
fn shenanigans(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, src);
    if m == src || owner(g, m) != o || !g.is_creature(m) || !has_type(g, m, "goblin") || !g.perm(src).on_bf {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "1 damage to target player", None)? {
        ping_player(g, o, 1)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Mana Echoes
/// p's creatures that share a creature type with m (m too, if it is p's)
fn sharing(g: &Game, p: PlayerId, m: PermId) -> u32 {
    let ts = subtypes(g, m);
    if ts.is_empty() {
        return 0;
    }
    let shares = |x: PermId| live(g, x) && g.is_creature(x) && subtypes(g, x).iter().any(|t| ts.contains(t));
    g.player(p).perms.iter().filter(|&&x| shares(x)).count() as u32
}

fn echoes_add(g: &mut Game, src: Src, ms: &[PermId]) -> Res {
    let o = owner(g, src);
    let ms: Vec<PermId> = ms.iter().copied().filter(|&m| g.perm(m).on_bf && g.is_creature(m)).collect();
    if ms.is_empty() || !live(g, src) {
        return Ok(());
    }
    let label = if ms.len() == 1 {
        "add {C} per creature sharing a type".to_string()
    } else {
        format!("{} triggers: add {{C}}", ms.len())
    };
    if !trigger_window(g, o, Some(src), &label, None)? {
        return Ok(());
    }
    let n: u32 = ms.iter().map(|&m| sharing(g, o, m)).sum();
    if n > 0 {
        g.player_mut(o).mana.add(Kind::C, n);
        crate::glog!(g, "    Mana Echoes adds {} colourless", n);
    }
    Ok(())
}

fn echoes_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        return Ok(());
    }
    echoes_add(g, src, &[m])
}

fn echoes_tokens(g: &mut Game, src: Src, _p: PlayerId, toks: &[PermId]) -> Res {
    echoes_add(g, src, toks)
}

// ------------------------------------------------------------------ Impact Tremors, Purphoros: tokens
/// Impact Tremors (1 each) and Purphoros (2 each, while it is out) see the tokens card code makes: one trigger for the
/// batch, damage per creature token
fn tremor_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    let o = owner(g, src);
    if p != o || !live(g, src) {
        return Ok(());
    }
    let k = toks.iter().filter(|&&m| g.perm(m).on_bf && g.is_creature(m)).count() as i32;
    if k == 0 {
        return Ok(());
    }
    let per = if named(g, src, "Purphoros, God of the Forge") { 2 } else { 1 };
    let label = format!("{} damage to each opponent", per * k);
    if trigger_window(g, o, Some(src), &label, None)? {
        for q in g.opps(o).collect::<Vec<_>>() {
            lose_life(g, q, per * k, Some(o), "triggers", Some(true))?;
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ Goblin Chirurgeon
/// m (nontoken, being destroyed) regenerates: its controller's Chirurgeon sacrifices a Goblin (a token first; not m)
pub fn chirurgeon_regen(g: &mut Game, m: PermId) -> Res<bool> {
    let p = owner(g, m);
    let Some(ch) = mine(g, p, CHIRURGEON) else { return Ok(false) };
    if pval(g, m) < 3.0 || crate::engine::values::stopped(g, CHIRURGEON) {
        return Ok(false);
    }
    let fod: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&x| x != m && live(g, x) && g.is_creature(x) && has_type(g, x, "goblin"))
        .collect();
    let Some(f) = min_by(&fod, |x| sac_worth(g, x) + if g.perm(x).token { 0.0 } else { 1.0 }) else {
        return Ok(false);
    };
    if f != m && pval(g, f) >= pval(g, m) {
        return Ok(false);
    }
    crate::glog!(g, "    {} sacrifices {} to Goblin Chirurgeon", g.player(p).name, g.perm(f).name);
    die(g, f, "sac")?;
    let label = format!("regenerate {}", g.perm(m).name);
    if !ability_window(g, p, Some(ch), &label, Some(4.0), Some(m))? || !g.perm(m).on_bf {
        return Ok(false);
    }
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "    {} regenerates", g.perm(m).name);
    Ok(true)
}

// ------------------------------------------------------------------ Goblin King
fn king_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "mountainwalk" && m != src && g.is_creature(m) && has_type(g, m, "goblin")
}

/// a Mountain of q's: a land with the type, or a nonbasic one under Blood Moon
fn has_mountain(g: &Game, q: PlayerId) -> bool {
    g.player(q).lands.iter().any(|&l| g.perm(l).on_bf && crate::engine::chars::land_types(g, l).has('R')) // layer 4
}

/// mountainwalk (Goblin King, any player's): b can't block the Goblin a while b's controller controls a Mountain
pub fn mountainwalk(g: &Game, b: PermId, a: PermId) -> bool {
    if g.hooks.is_empty() || !has_type(g, a, "goblin") {
        return false;
    }
    let king = g.hooks.iter().any(|&k| k != a && live(g, k) && named(g, k, KING));
    king && has_mountain(g, owner(g, b))
}

// ------------------------------------------------------------------ Hexing Squelcher
/// "Ward—Pay 2 life" on m (Hexing Squelcher, or another creature of its controller's): actor pays 2 life, or the
/// spell is countered (false) when actor is at 4 life or less
pub fn life_ward(g: &mut Game, actor: PlayerId, m: PermId) -> Res<bool> {
    if !g.is_creature(m) {
        return Ok(true);
    }
    let o = owner(g, m);
    if mine(g, o, SQUELCHER).is_none() {
        return Ok(true);
    }
    if g.player(actor).life <= 4 {
        return Ok(false);
    }
    crate::glog!(g, "    {} pays 2 life for ward ({})", g.player(actor).name, g.perm(m).name);
    lose_life(g, actor, 2, Some(actor), "life", Some(false))?;
    Ok(true)
}

// ------------------------------------------------------------------ Howlsquad Heavy: speed
/// an opponent of the active player lost life: the active player's speed rises (once a turn, to 4)
pub fn speed_up(g: &mut Game, loser: PlayerId) {
    let Some(a) = g.active else { return };
    if a == loser || g.player(a).speed == 0 || g.player(a).speed >= 4 {
        return;
    }
    let st = g.turn_stamp();
    if g.player(a).speed_rose == Some(st) {
        return;
    }
    let pl = g.player_mut(a);
    pl.speed += 1;
    pl.speed_rose = Some(st);
}

fn engines(g: &mut Game, p: PlayerId, _m: PermId) -> Res {
    let pl = g.player_mut(p);
    pl.speed = pl.speed.max(1); // start your engines!
    Ok(())
}

fn howl_haste(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "haste" && m != src && owner(g, m) == owner(g, src) && g.is_creature(m) && has_type(g, m, "goblin")
}

/// at the beginning of combat on your turn: a 1/1 red Goblin that attacks this combat if able
fn howl_combat(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if p != o || g.active != Some(o) || !live(g, src) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "create a 1/1 Goblin that attacks", None)? {
        return Ok(());
    }
    let spec = Tokens { color: Some(Colors::from_letters("R")), types: vec!["goblin"], ..Tokens::new(1, 1) };
    let now = Val::Stamp(g.turn_stamp());
    for t in make_tokens(g, o, spec)? {
        g.perm_mut(t).data.set(DataKey::MustAttack, now.clone());
    }
    Ok(())
}

/// max speed: {T}: {R} for each Goblin you control (nothing below max speed)
fn howl_mana(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    if g.player(p).speed < 4 {
        return 0;
    }
    crate::impls::t1::count_type(g, p, "goblin", false).max(0) as u32
}

// ------------------------------------------------------------------ Marvin, Murderous Mimic
/// Marvin's options: the Tier 6 / Kefka copies, then Krenko's and Kiki-Jiki's {T} abilities while p controls them
fn marvin(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = super::t6_legends1b::marvin_both(g, src, p, post)?;
    if owner(g, src) != p || !live(g, src) || g.perm(src).tapped {
        return Ok(out);
    }
    if g.perm(src).sick && !has_haste(g, src) && !elixir(g, p) {
        return Ok(out);
    }
    let mut add = |v: Vec<Opt>, who: &str| {
        for mut o in v {
            o.label = format!("Marvin ({who}): {}", o.label);
            out.push(o);
        }
    };
    if mine(g, p, KRENKO).is_some() {
        let v = super::t4::krenko(g, src, p, post)?;
        add(v, "Krenko");
    }
    if mine(g, p, KIKI).is_some() {
        let v = super::t5::kiki(g, src, p, post)?;
        add(v, "Kiki-Jiki");
    }
    Ok(out)
}

// ------------------------------------------------------------------ Patriar's Seal, Thousand-Year Elixir
/// Thousand-Year Elixir: p's creatures use {T} abilities as though they had haste
pub fn elixir(g: &Game, p: PlayerId) -> bool {
    !g.hooks.is_empty() && mine(g, p, ELIXIR).is_some()
}

/// how much untapping m is worth now: the best of its own {T} options (Krenko, Kiki-Jiki, Marvin), untapped
fn untap_gain(g: &mut Game, p: PlayerId, m: PermId, post: Option<bool>) -> Res<f64> {
    let Some(cd) = g.perm(m).cd else { return Ok(0.0) };
    if !(named(g, m, KRENKO) || named(g, m, KIKI) || named(g, m, MARVIN)) {
        return Ok(0.0);
    }
    let Some(f) = g.registry.get(cd).and_then(|i| i.options) else { return Ok(0.0) };
    g.perm_mut(m).tapped = false;
    let r = f(g, m, p, post);
    g.perm_mut(m).tapped = true;
    let best = r?
        .iter()
        .filter(|o| matches!(o.act, Some(Action::Ability { src, .. }) if src == m))
        .map(|o| o.utility)
        .fold(0.0, f64::max);
    Ok(best)
}

fn untapper_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>, legendary: bool) -> Res<Vec<Opt>> {
    if owner(g, src) != p || !live(g, src) || g.perm(src).tapped || !can_pay(g, p, 1, "", false) {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            live(g, m)
                && g.is_creature(m)
                && g.perm(m).tapped
                && (!legendary || g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(crate::tag::Tag::Leg)))
        })
        .collect();
    let mut best: Option<(PermId, f64)> = None;
    for m in cands {
        let u = untap_gain(g, p, m, post)?;
        if u >= 1.0 && best.is_none_or(|b| u > b.1) {
            best = Some((m, u));
        }
    }
    let Some((m, u)) = best else { return Ok(vec![]) };
    let label = format!("{}: untap {}", g.perm(src).name, g.perm(m).name);
    Ok(vec![Opt { utility: u - 0.3, label, act: Some(Action::Ability { src, f: untap_go, arg: m.0 as i64 }) }])
}

fn seal_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    untapper_options(g, src, p, post, true)
}

fn elixir_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    untapper_options(g, src, p, post, false)
}

fn untap_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if g.perm(src).tapped || !live(g, m) || owner(g, m) != p || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    g.perm_mut(src).tapped = true;
    let label = format!("untap {}", g.perm(m).name);
    if ability_window(g, p, Some(src), &label, None, Some(m))? && live(g, m) {
        g.perm_mut(m).tapped = false;
        crate::glog!(g, "    {} untaps {}", g.perm(src).name, g.perm(m).name);
    }
    Ok(true)
}

// ------------------------------------------------------------------ Sting, the Glinting Dagger
/// first strike for m while it fights `other`, a Goblin or an Orc, wearing Sting (read by the combat damage step)
pub fn sting_first_strike(g: &Game, m: PermId, other: PermId) -> bool {
    if !(has_type(g, other, "goblin") || has_type(g, other, "orc")) {
        return false;
    }
    g.player(owner(g, m)).perms.iter().any(|&e| g.perm(e).attached == Some(m) && named(g, e, STING))
}

// ------------------------------------------------------------------ Treasure Nabber
fn is_artifact(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT))
}

/// an opponent tapped an artifact for mana: take it until the end of your next turn (it stays tapped; the borrowed
/// list hands it back in your end step)
fn nabber(g: &mut Game, src: Src, p: PlayerId, m: PermId, _n: u32) -> Res {
    let o = owner(g, src);
    if p == o || !live(g, src) || !g.perm(m).on_bf || owner(g, m) != p || !is_artifact(g, m) {
        return Ok(());
    }
    if gain_control(g, o, m, Duration::EndOfTurn) {
        g.perm_mut(m).tapped = true;
    }
    Ok(())
}

// ------------------------------------------------------------------ Warren Instigator
fn instigator_hit(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if a != src || d == p {
        return Ok(());
    }
    let gobs = |g: &Game| -> Vec<CardId> {
        g.player(p)
            .hand
            .iter()
            .copied()
            .filter(|&c| g.db.get(c).creature && g.db.get(c).has_subtype("goblin"))
            .collect()
    };
    if gobs(g).is_empty() || !trigger_window(g, p, Some(src), "put a Goblin onto the battlefield", None)? {
        return Ok(());
    }
    let gs = gobs(g);
    if let Some(c) = max_by(&gs, |c| g.db.get(c).cmc as f64 + 0.01 * g.db.get(c).bomb as f64) {
        if let Some(i) = g.player(p).hand.iter().position(|&x| x == c) {
            g.player_mut(p).hand.remove(i);
        }
        enter(g, p, c, Enter::default())?;
    }
    Ok(())
}

// ------------------------------------------------------------------ combos
fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    mine(g, p, name)
}

/// the Goblin tokens can attack: a haste giver before combat on p's turn
fn hasty_goblins(g: &Game, p: PlayerId) -> bool {
    let pre = g.active == Some(p) && matches!(g.step, Step::Start | Step::Main1);
    pre && ["Goblin Warchief", "Goblin Chieftain", HOWLSQUAD, "Fervor", "Legion Loyalist"]
        .iter()
        .any(|n| on_bf(g, p, n).is_some())
}

/// unbounded Goblins (and mana) win: Goblin Bombardment, an enters payoff, a death payoff with an outlet out, or
/// hasty Goblins before combat
fn goblins_win(g: &Game, p: PlayerId) -> bool {
    let any = |ns: &[&str]| ns.iter().any(|n| on_bf(g, p, n).is_some());
    if any(&["Goblin Bombardment", "Impact Tremors", "Purphoros, God of the Forge", "General Kreat, the Boltbringer"]) {
        return true;
    }
    let outlet = any(&[ALTAR, "Skirk Prospector", "Phyrexian Altar"]);
    if outlet && any(&["Boggart Shenanigans", "Goblin Sharpshooter", "Pashalik Mons"]) {
        return true;
    }
    hasty_goblins(g, p)
}

/// Krenko (untapped, able to use {T}) with Umbral Mantle ({3}: untap) or Staff of Domination ({1} + {3}: untap him),
/// paid by Ashnod's Altar (two Goblins: {C}{C}{C}{C}) or Mana Echoes (each batch of Goblins: {C} per Goblin each).
/// Three Goblins start the loop growing (or the mana for one untap). Unbounded Goblins and mana; see `goblins_win`.
fn krenko_untap_ready(g: &Game, p: PlayerId) -> Ready {
    let no = || (false, vec![], vec![]);
    let Some(k) = on_bf(g, p, KRENKO) else { return no() };
    let x = g.perm(k);
    if x.tapped || (x.sick && !has_haste(g, k) && !elixir(g, p)) {
        return no();
    }
    let Some(u) = on_bf(g, p, MANTLE).or_else(|| on_bf(g, p, STAFF)) else { return no() };
    let Some(e) = on_bf(g, p, ALTAR).or_else(|| on_bf(g, p, ECHOES)) else { return no() };
    let gobs = crate::impls::t1::count_type(g, p, "goblin", false);
    let cost = if named(g, u, MANTLE) { 3 } else { 4 };
    if (gobs < 3 && !can_pay(g, p, cost, "", false)) || !goblins_win(g, p) {
        return no();
    }
    (true, vec![k, u, e], vec![])
}

pub static COMBOS: [Combo; 1] = [Combo {
    name: "Krenko + Umbral Mantle / Staff of Domination + Ashnod's Altar / Mana Echoes",
    groups: &[&[KRENKO], &[MANTLE, STAFF], &[ALTAR, ECHOES]],
    ready: krenko_untap_ready,
    mana: (0, ""),
    finish: None,
    xcost: &[],
    steps: &[],
}];

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let has = |n: &str| db.id(n).is_some();
    if has("Boggart Shenanigans") {
        r.card(db, "Boggart Shenanigans")?.dies = Some(shenanigans);
    }
    if has(ECHOES) {
        let c = r.card(db, ECHOES)?;
        c.etb = Some(echoes_etb);
        c.tokens_enter = Some(echoes_tokens);
    }
    for n in ["Impact Tremors", "Purphoros, God of the Forge"] {
        if has(n) {
            r.card(db, n)?.tokens_enter = Some(tremor_tokens);
        }
    }
    if has(KING) {
        r.card(db, KING)?.grant_kw = Some(king_kw);
    }
    if has(HOWLSQUAD) {
        let c = r.card(db, HOWLSQUAD)?;
        c.as_enters = Some(engines);
        c.grant_kw = Some(howl_haste);
        c.each_combat = Some(howl_combat);
        c.dyn_mana_perm = Some(howl_mana);
    }
    if has(MARVIN) {
        r.card(db, MARVIN)?.options = Some(marvin);
    }
    if has(SEAL) {
        r.card(db, SEAL)?.options = Some(seal_options);
    }
    if has(ELIXIR) {
        r.card(db, ELIXIR)?.options = Some(elixir_options);
    }
    if has("Treasure Nabber") {
        r.card(db, "Treasure Nabber")?.mana_tapped = Some(nabber);
    }
    if has("Warren Instigator") {
        r.card(db, "Warren Instigator")?.combat_damage = Some(instigator_hit);
    }
    Ok(())
}
