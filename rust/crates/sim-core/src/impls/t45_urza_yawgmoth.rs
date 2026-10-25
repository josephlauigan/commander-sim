//! Tiers 4/5 pool decks `urza-mono-blue-artifacts` and `yawgmoth-mono-black-aristocrats` (EDHREC's bracket-4 average
//! lists): the cards the ability language doesn't cover (rust/agents/t45-urza-yawgmoth.md). Ayara's enter drain is
//! compiled (dsl_parse.rs, `// --- Tiers 4/5 (urza-yawgmoth)`); the rest is here.
//!
//! Urza:
//! - Winter Orb, Static Orb, Back to Basics: the untap step (`untap_lock` before it, `untap_step` after it, called
//!   from turn::step_start). An Orb counts when it is untapped as the step begins. The active player keeps its best
//!   land (Winter Orb) or its best two permanents, lands and tapped Treasures included (Static Orb), untapped.
//!   Left out: the land kept is the one making the most colours, not the one the hand needs.
//! - Unwinding Clock: during each other player's untap step, its controller's artifacts untap (artifact lands and
//!   tapped Treasures too). Urza never taps an Orb for mana (t5::urza_mana), so the lock stays on.
//! - Welding Jar: sacrificed to regenerate an artifact of its controller's worth 1.5 or more that is being destroyed
//!   (zones::die). Left out: it never saves itself or a cheaper artifact.
//! - Forensic Gadgeteer: investigate is compiled; artifacts' activated abilities cost {1} less (not below one mana):
//!   cracking a Clue ({1}), and Basalt Monolith's untap ({2}: the combo with a mana sink, below). Left out: the other
//!   artifacts' activations (Isochron Scepter's {2}, Walking Ballista's {4}) keep their printed cost.
//! - Padeem, Consul of Innovation: artifacts you control have hexproof; your upkeep, draw if you control the artifact
//!   with the greatest mana value (or tied).
//! - The Reality Chip: reconfigure {2}{U} as a sorcery onto your best other creature; while attached, lands are played
//!   and spells cast from the top of the library (the AI knows the top card). Left out: it stays a 0/4 creature while
//!   attached.
//! - Thought Monitor, Thoughtcast: affinity for artifacts (as Emry: one less per artifact and Treasure).
//! - Uthros, Titanic Godcore: station (tap another untapped creature as a sorcery: charge counters equal to its power;
//!   the AI does it after combat, or before it with a creature that can't attack); at 12+ it taps for {U} per
//!   artifact less the {U} the ability costs.
//!
//! Yawgmoth:
//! - Entomb: the outside decks' AI casts it (end of an opponent's turn, or in a main phase): the best creature for a
//!   reanimation spell in hand (Unearth: mana value 3 or less), else a creature that returns by itself (Bloodghast,
//!   Gravecrawler, Reassembling Skeleton, Nether Traitor, Forsaken Miner). JD's decks keep their own Entomb plan.
//! - Genesis Chamber: a nontoken creature entering while it is untapped gives its controller a 1/1 Myr artifact
//!   creature token.
//! - Blight Mound: your nontoken creatures dying make 1/1 black-green Pests that gain you 1 life as they die
//!   (zones::die_rest); attacking Pests you control get +1/+0 and menace (until end of turn).
//! - Forsaken Miner: whenever you commit a crime (a spell that targets an opponent or their things, Yawgmoth's
//!   counter on an opposing creature, the other abilities `bites::crime` sees), pay {B}: it returns from your graveyard.
//! - Hancock, Ghoulish Mayor: your other Zombies and Mutants get +X/+X (X: the counters on Hancock); undying is the
//!   engine's.
//! - Necroskitter: wither (combat damage to creatures is -1/-1 counters, combat.rs); an opposing creature with a
//!   -1/-1 counter dying returns under your control.
//! - Nine-Lives Familiar: eight revival counters when cast; dying with one, it returns at the next end step with one
//!   fewer.
//! - The Meathook Massacre: X is paid as it enters (the X that kills the most opposing value for the least of yours,
//!   with the mana left); every creature gets -X/-X until end of turn. Its drain halves are compiled. Left out:
//!   opponents' protection spells in answer to the enter trigger.
//!
//! Combos (chained into combos::all): Basalt Monolith + Forensic Gadgeteer (a mana sink wins); Hullbreaker Horror with
//! Urza or Sol Ring and free artifacts cast between each other; Reassembling Skeleton / Gravecrawler + Pitiless
//! Plunderer + an altar or Warren Soultrader + a death payoff.

use crate::ai::brain::{Situation, card_utility, do_cast_with};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable, grants_flash};
use crate::engine::life::{check_state, gain};
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, etgh, has_type, pval};
use crate::engine::zones::{Enter, Tokens, agent_for, agent_take, die, enter, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::combos::{Combo, QUIET, Ready};
use crate::impls::partials::at_once;
use crate::impls::t5::is_artifact;
use crate::state::{Ctx, DataKey, Game, Perm, PermData, Step, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

const WINTER: &str = "Winter Orb";
const STATIC: &str = "Static Orb";
const B2B: &str = "Back to Basics";
const CLOCK: &str = "Unwinding Clock";
const JAR: &str = "Welding Jar";
const GADGETEER: &str = "Forensic Gadgeteer";
const PADEEM: &str = "Padeem, Consul of Innovation";
const CHIP: &str = "The Reality Chip";
const UTHROS: &str = "Uthros, Titanic Godcore";
const ENTOMB: &str = "Entomb";
const CHAMBER: &str = "Genesis Chamber";
const MOUND: &str = "Blight Mound";
const MINER: &str = "Forsaken Miner";
const HANCOCK: &str = "Hancock, Ghoulish Mayor";
const SKITTER: &str = "Necroskitter";
const NINE: &str = "Nine-Lives Familiar";
const MEATHOOK: &str = "The Meathook Massacre";
const BASALT: &str = "Basalt Monolith";
const HORROR: &str = "Hullbreaker Horror";
const URZA: &str = "Urza, Lord High Artificer";
const PLUNDERER: &str = "Pitiless Plunderer";
const SKELETON: &str = "Reassembling Skeleton";
const GRAVECRAWLER: &str = "Gravecrawler";
/// the keyword mark on Blight Mound's Pest tokens ("When this token dies, you gain 1 life.")
const PEST_MARK: &str = "t45_pest_dies";

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn named(g: &Game, m: PermId, name: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == name)
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

/// p's live permanents of this name
fn all_named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && named(g, m, name)).collect()
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && named(g, m, name))
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| cname(g, c) == name)
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

fn plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

/// p's own main phase (sorcery timing)
fn sorcery_time(g: &Game, p: PlayerId) -> bool {
    g.active == Some(p) && matches!(g.step, Step::Main1 | Step::Main2)
}

// ================================================================== the untap step
/// What restricts the active player's untap step this turn, read before it begins (an Orb counts when untapped then),
/// with what was tapped (only those untap, so only those can be kept tapped).
pub struct UntapLock {
    winter: bool,
    stat: bool,
    b2b: bool,
    lands: Vec<LandId>,
    perms: Vec<PermId>,
    treasures: u32,
}

/// None (the common case) when no Winter Orb, Static Orb or Back to Basics is on the battlefield
pub fn untap_lock(g: &Game, p: PlayerId) -> Option<UntapLock> {
    let (mut winter, mut stat, mut b2b) = (false, false, false);
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in &q.perms {
            let x = g.perm(m);
            if !x.on_bf || x.phased {
                continue;
            }
            match &*x.name {
                WINTER if !x.tapped => winter = true,
                STATIC if !x.tapped => stat = true,
                B2B => b2b = true,
                _ => {}
            }
        }
    }
    if !(winter || stat || b2b) {
        return None;
    }
    let pl = g.player(p);
    let lands = pl.lands.iter().copied().filter(|&l| g.land(l).tapped).collect();
    let perms = pl.perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).tapped).collect();
    Some(UntapLock { winter, stat, b2b, lands, perms, treasures: pl.tapped_treasures })
}

/// how much p wants land l untapped: one mana, a little more for each colour it makes
fn land_value(g: &Game, p: PlayerId, l: LandId) -> f64 {
    let cols = crate::engine::mana::land_colors_now(g, p, l);
    1.0 + 0.05 * cols.count() as f64 + if g.db.get(g.land(l).cd).basic { 0.01 } else { 0.0 }
}

/// how much p wants permanent m untapped (Static Orb): mana rocks and dorks first, then creatures
fn perm_value(g: &Game, m: PermId) -> f64 {
    let Some(c) = g.perm(m).cd else { return if g.is_creature(m) { 0.7 } else { 0.3 } };
    let d = g.db.get(c);
    if matches!(&*d.name, "Sol Ring" | "Mana Vault" | "Basalt Monolith" | "Grim Monolith") {
        return 2.5;
    }
    if d.tag(Tag::Rock) || d.tag(Tag::Dork) {
        return 1.2;
    }
    if g.is_creature(m) { 0.8 + 0.01 * pval(g, m) } else { 0.5 }
}

/// after p's untap step: the Orbs and Back to Basics keep what can't untap tapped; Unwinding Clock untaps the other
/// players' artifacts
pub fn untap_step(g: &mut Game, p: PlayerId, lock: Option<UntapLock>) {
    if let Some(k) = lock {
        apply_lock(g, p, k);
    }
    unwinding_clock(g, p);
}

#[derive(Clone, Copy)]
enum Thing {
    Land(LandId),
    Perm(PermId),
    Treasure,
}

fn apply_lock(g: &mut Game, p: PlayerId, k: UntapLock) {
    let mut lands: Vec<LandId> = k.lands.into_iter().filter(|&l| g.land(l).on_bf && !g.land(l).tapped).collect();
    let before = lands.len();
    if k.b2b {
        lands.retain(|&l| {
            let basic = g.db.get(g.land(l).cd).basic;
            if !basic {
                g.land_mut(l).tapped = true;
            }
            basic
        });
    }
    lands.sort_by(|&a, &b| land_value(g, p, b).total_cmp(&land_value(g, p, a)));
    if k.winter {
        for &l in lands.iter().skip(1) {
            g.land_mut(l).tapped = true;
        }
        lands.truncate(1);
    }
    let mut kept_treasures = k.treasures;
    if k.stat {
        let mut all: Vec<(f64, Thing)> = lands.iter().map(|&l| (land_value(g, p, l), Thing::Land(l))).collect();
        for &m in &k.perms {
            if g.perm(m).on_bf && !g.perm(m).tapped {
                all.push((perm_value(g, m), Thing::Perm(m)));
            }
        }
        all.extend((0..k.treasures).map(|_| (0.9, Thing::Treasure)));
        all.sort_by(|a, b| b.0.total_cmp(&a.0));
        kept_treasures = 0;
        for (i, &(_, t)) in all.iter().enumerate() {
            match t {
                Thing::Land(l) if i >= 2 => g.land_mut(l).tapped = true,
                Thing::Perm(m) if i >= 2 => g.perm_mut(m).tapped = true,
                Thing::Treasure if i < 2 => kept_treasures += 1,
                _ => {}
            }
        }
        g.player_mut(p).tapped_treasures = k.treasures - kept_treasures;
    }
    let after = g.player(p).lands.iter().filter(|&&l| !g.land(l).tapped).count();
    crate::glog!(
        g,
        "    untap step under {}: {} of {} tapped lands untap{}",
        [(k.winter, WINTER), (k.stat, STATIC), (k.b2b, B2B)]
            .iter()
            .filter(|x| x.0)
            .map(|x| x.1)
            .collect::<Vec<_>>()
            .join(", "),
        lands.len().min(after),
        before,
        if k.stat { format!(", {kept_treasures} Treasures") } else { String::new() }
    );
}

/// Unwinding Clock: during p's untap step, each other player with a Clock untaps its artifacts
fn unwinding_clock(g: &mut Game, p: PlayerId) {
    let n = g.players.len();
    for i in 0..n {
        let q = PlayerId(i as u8);
        if q == p || !g.player(q).alive || on_bf(g, q, CLOCK).is_none() {
            continue;
        }
        for m in g.player(q).perms.clone() {
            if g.perm(m).on_bf && g.perm(m).tapped && is_artifact(g, m) && g.perm(m).data.int(DataKey::Stun) == 0 {
                g.perm_mut(m).tapped = false;
            }
        }
        for l in g.player(q).lands.clone() {
            if g.land(l).tapped && g.db.get(g.land(l).cd).types.has(Types::ARTIFACT) {
                g.land_mut(l).tapped = false;
            }
        }
        g.player_mut(q).tapped_treasures = 0;
        crate::glog!(g, "    Unwinding Clock untaps {}'s artifacts", pname(g, q));
    }
}

/// Urza taps no Orb for mana (it would switch its own lock off)
pub fn orb(g: &Game, m: PermId) -> bool {
    named(g, m, WINTER) || named(g, m, STATIC)
}

// ================================================================== Welding Jar
/// zones::die: artifact m is being destroyed; its controller sacrifices a Welding Jar to regenerate it (tapped)
pub fn welding_jar(g: &mut Game, m: PermId) -> Res<bool> {
    let p = owner(g, m);
    if !is_artifact(g, m) || named(g, m, JAR) || pval(g, m) < 1.5 {
        return Ok(false);
    }
    let Some(jar) = all_named(g, p, JAR).into_iter().find(|&j| j != m) else { return Ok(false) };
    let label = format!("Welding Jar: regenerate {}", g.perm(m).name);
    die(g, jar, intern("sac"))?;
    if !ability_window(g, p, Some(jar), &label, Some(pval(g, m)), Some(m))? || !g.perm(m).on_bf {
        return Ok(false);
    }
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "    {} regenerates (Welding Jar)", g.perm(m).name);
    Ok(true)
}

// ================================================================== Forensic Gadgeteer
/// how much less p's artifacts' activated abilities cost (one per Forensic Gadgeteer)
pub fn artifact_discount(g: &Game, p: PlayerId) -> u32 {
    all_named(g, p, GADGETEER).len() as u32
}

/// the generic cost of an artifact's activation under Forensic Gadgeteer ("can't reduce the mana in that cost to less
/// than one mana": `colored` is the coloured part)
pub fn artifact_cost(g: &Game, p: PlayerId, generic: u32, colored: u32) -> u32 {
    let d = artifact_discount(g, p);
    if d == 0 {
        return generic;
    }
    let floor = if colored > 0 { 0 } else { 1 };
    generic.saturating_sub(d).max(floor.min(generic))
}

/// Clue: {2}, sacrifice: draw a card
pub fn clue_cost(g: &Game, p: PlayerId) -> u32 {
    artifact_cost(g, p, 2, 0)
}

// ================================================================== Padeem, Consul of Innovation
fn padeem_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "hexproof" && owner(g, m) == owner(g, src) && live(g, src) && is_artifact(g, m)
}

fn mv(g: &Game, m: PermId) -> u32 {
    g.perm(m).cd.map_or(0, |c| g.db.get(c).cmc)
}

/// your upkeep: draw if you control the artifact with the greatest mana value (or one tied for it)
fn padeem_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if p != o || !live(g, src) {
        return Ok(());
    }
    let best =
        |q: PlayerId| g.player(q).perms.iter().filter(|&&m| live(g, m) && is_artifact(g, m)).map(|&m| mv(g, m)).max();
    let Some(mine) = best(o) else { return Ok(()) };
    if g.players.iter().filter(|q| q.alive && q.id != o).any(|q| best(q.id).is_some_and(|v| v > mine)) {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "Padeem: draw a card", None)? {
        crate::engine::zones::draw(g, o, 1, false)?;
    }
    Ok(())
}

// ================================================================== The Reality Chip
fn chip_on(g: &Game, chip: PermId) -> bool {
    live(g, chip)
        && g.perm(chip).attached.is_some_and(|h| live(g, h) && owner(g, h) == owner(g, chip) && g.is_creature(h))
}

fn chip_lands(g: &Game, src: Src, p: PlayerId) -> i32 {
    (owner(g, src) == p && chip_on(g, src)) as i32
}

/// the creature the Chip attaches to: the toughest other creature (it stays longest)
fn chip_host(g: &Game, p: PlayerId, chip: PermId) -> Option<PermId> {
    let cs: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| m != chip && live(g, m) && g.is_creature(m)).collect();
    max_by(&cs, |m| etgh(g, m) as f64 + 0.1 * pval(g, m) + if g.perm(m).token { 0.0 } else { 1.0 })
}

fn chip_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if owner(g, src) != p || !live(g, src) {
        return Ok(out);
    }
    if chip_on(g, src) {
        // cast the top card when it would be cast from the hand
        if let Some(&top) = g.player(p).library.last()
            && !g.db.get(top).land
            && (post.is_some() || g.db.get(top).instant || grants_flash(g, p, top))
        {
            let s = Situation::new(g, p);
            if let Some(u) = card_utility(g, p, &s, top) {
                let (cg, cp) = cost_of(g, p, top);
                g.pay_for = Some(top);
                let ok = can_pay(g, p, cg, &cp, false) && (!g.cast_checks() || castable(g, p, top, "lib"));
                g.pay_for = None;
                if ok {
                    let label = format!("cast {} from the top of the library (The Reality Chip)", g.db.get(top).name);
                    out.push(ability(u + 0.6, label, src, chip_cast, top.0 as i64));
                }
            }
        }
    } else if post.is_some()
        && sorcery_time(g, p)
        && can_pay(g, p, 2, "U", false)
        && let Some(h) = chip_host(g, p, src)
    {
        let label = format!("The Reality Chip: reconfigure onto {}", g.perm(h).name);
        out.push(ability(2.2, label, src, chip_attach, h.0 as i64));
    }
    Ok(out)
}

fn chip_attach(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let h = PermId(arg as u32);
    if !live(g, h) || owner(g, h) != p || !can_pay(g, p, 2, "U", false) {
        return Ok(false);
    }
    pay(g, p, 2, "U", false)?;
    if !ability_window(g, p, Some(src), "reconfigure", None, None)? || !live(g, h) {
        return Ok(true);
    }
    g.perm_mut(src).attached = Some(h);
    crate::glog!(g, "  The Reality Chip attaches to {}", g.perm(h).name);
    Ok(true)
}

fn chip_cast(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let top = CardId(arg as u16);
    if g.player(p).library.last() != Some(&top) || !chip_on(g, src) {
        return Ok(false);
    }
    g.player_mut(p).library.pop();
    crate::glog!(g, "  {} casts {} from the top of the library", pname(g, p), g.db.get(top).name);
    let ok = do_cast_with(g, p, top, Some(intern("lib")), Ctx::default())?;
    if !ok {
        g.player_mut(p).library.push(top);
    }
    Ok(ok)
}

// ================================================================== affinity for artifacts
fn affinity(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::impls::alela::emry_cost(g, p, c)
}

// ================================================================== Uthros, Titanic Godcore
fn charge(g: &Game, l: LandId) -> i64 {
    g.land(l).data.int(DataKey::Charge)
}

fn artifacts_of(g: &Game, p: PlayerId) -> u32 {
    let pl = g.player(p);
    let perms = pl.perms.iter().filter(|&&m| live(g, m) && is_artifact(g, m)).count() as u32;
    let lands = pl.lands.iter().filter(|&&l| g.db.get(g.land(l).cd).types.has(Types::ARTIFACT)).count() as u32;
    perms + lands + pl.treasures
}

/// 12+: {U},{T}: {U} for each artifact you control (net: one less, for the {U} it costs); else {T}: {U}
fn uthros_mana(g: &Game, p: PlayerId, l: LandId) -> u32 {
    if charge(g, l) >= 12 { artifacts_of(g, p).saturating_sub(1).max(1) } else { 1 }
}

/// station as a sorcery: the strongest untapped creature that won't attack this turn (after combat, or summoning sick)
fn station_pick(g: &Game, p: PlayerId) -> Option<PermId> {
    let after = g.step == Step::Main2;
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            live(g, m) && g.is_creature(m) && !x.tapped && epow(g, m) >= 1 && (after || x.sick)
        })
        .collect();
    max_by(&cs, |m| epow(g, m) as f64 - 0.1 * pval(g, m))
}

fn uthros_options(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.land(l).owner != p || post.is_none() || !sorcery_time(g, p) || charge(g, l) >= 12 {
        return Ok(vec![]);
    }
    let Some(m) = station_pick(g, p) else { return Ok(vec![]) };
    let label = format!("Uthros: station with {} ({} charge counters)", g.perm(m).name, epow(g, m));
    Ok(vec![plan(1.0 + 0.05 * epow(g, m) as f64, label, station_go, l.0 as i64 | ((m.0 as i64) << 32))])
}

fn station_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, m) = (LandId((arg & 0xffff_ffff) as u32), PermId((arg >> 32) as u32));
    if !g.land(l).on_bf || !live(g, m) || g.perm(m).tapped || owner(g, m) != p {
        return Ok(false);
    }
    g.perm_mut(m).tapped = true;
    let n = charge(g, l) + epow(g, m).max(0) as i64;
    g.land_mut(l).data.set(DataKey::Charge, Val::Int(n));
    crate::glog!(g, "  {} stations Uthros with {}: {} charge counters", pname(g, p), g.perm(m).name, n);
    Ok(true)
}

// ================================================================== Entomb
const REANIMATE: [&str; 5] = ["Reanimate", "Animate Dead", "Necromancy", "Victimize", "Dread Return"];
const RECURSIVE: [&str; 5] =
    ["Gravecrawler", "Bloodghast", "Reassembling Skeleton", "Nether Traitor", "Forsaken Miner"];

/// the card Entomb puts into p's graveyard (None: not worth casting now)
fn entomb_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let pl = g.player(p);
    let lib: Vec<CardId> = pl.library.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    let big = |c: CardId| card_worth(g, p, c, true) + epow_card(g, c) as f64 * 0.5 + g.db.get(c).cmc as f64 * 0.2;
    if pl.hand.iter().any(|&c| REANIMATE.contains(&cname(g, c))) {
        return max_by(&lib, big);
    }
    if in_hand(g, p, "Unearth").is_some() {
        let small: Vec<CardId> = lib.iter().copied().filter(|&c| g.db.get(c).cmc <= 3).collect();
        if let Some(c) = max_by(&small, big) {
            return Some(c);
        }
    }
    RECURSIVE.iter().find_map(|n| lib.iter().copied().find(|&c| cname(g, c) == *n))
}

fn epow_card(g: &Game, c: CardId) -> i32 {
    g.db.get(c).pow
}

fn entomb_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if crate::ai::is_main(g.player(p).key) || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    let (cg, cp) = cost_of(g, p, c);
    if !can_pay(g, p, cg, &cp, false) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let Some(pick) = entomb_pick(g, p) else { return Ok(vec![]) };
    let u = if post.is_none() { 2.4 } else { 1.4 };
    Ok(vec![plan(u, format!("Entomb ({})", cname(g, pick)), entomb_go, c.0 as i64)])
}

fn entomb_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (cg, cp) = cost_of(g, p, c);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, cg, &cp, false) {
        return Ok(false);
    }
    g.pay_for = Some(c);
    let ok = pay(g, p, cg, &cp, false)?;
    g.pay_for = None;
    if !ok {
        return Ok(false);
    }
    cast_card(g, p, c, intern("hand"), Ctx::default())?;
    Ok(true)
}

fn entomb_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if crate::ai::is_main(g.player(p).key) || crate::human::is_human(g, p) {
        crate::ai::seph_fill_resolve(g, p, intern("entomb"), ctx)?; // JD's decks: their own Entomb plan
        return Ok(intern("gy"));
    }
    let Some(pick) = entomb_pick(g, p) else {
        shuffle_library(g, p);
        return Ok(intern("gy"));
    };
    remove_card(&mut g.player_mut(p).library, pick);
    if let Some(a) = agent_for(g, p) {
        agent_take(g, a, p, pick); // Opposition Agent takes what they search for
    } else {
        g.player_mut(p).gy.push(pick);
        crate::impls::t12_brago_meren::note_gy(g, pick); // --- Tiers 1/2 (brago-meren)
        crate::glog!(g, "    {} puts {} into the graveyard (Entomb)", pname(g, p), cname(g, pick));
    }
    shuffle_library(g, p);
    Ok(intern("gy"))
}

// ================================================================== Genesis Chamber
fn chamber_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if !live(g, src) || g.perm(src).tapped || g.perm(m).token || !g.is_creature(m) || !g.perm(m).on_bf {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "Genesis Chamber: a Myr", None)? || g.perm(src).tapped {
        return Ok(());
    }
    let mut data = PermData::default();
    data.set(DataKey::Artifact, Val::Bool(true));
    let myr = Tokens { color: Some(Colors::NONE), types: vec![intern("myr")], data, ..Tokens::new(1, 1) };
    make_tokens(g, p, myr)?;
    Ok(())
}

// ================================================================== Blight Mound
/// zones::die_rest and cardcode::keyword_attack: one of Blight Mound's Pests
pub fn mound_pest(x: &Perm) -> bool {
    x.token
        && matches!(x.data.get(DataKey::Kws), Some(Val::List(v)) if v.iter().any(|k| matches!(k, Val::Str(s) if *s == PEST_MARK)))
}

/// zones::die_rest: a Pest of Blight Mound's died: its controller gains 1 life
pub fn pest_dies(g: &mut Game, x: &Perm) -> Res {
    if mound_pest(x) && g.player(x.owner).alive {
        gain(g, x.owner, 1)?;
    }
    Ok(())
}

fn mound_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, src);
    if m == src || !live(g, src) || owner(g, m) != o || g.perm(m).token || !g.is_creature(m) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "Blight Mound: a Pest", None)? {
        return Ok(());
    }
    let mut data = PermData::default();
    data.set(DataKey::Kws, Val::List(vec![Val::Str(intern(PEST_MARK))]));
    let pest =
        Tokens { color: Some(Colors::from_letters("BG")), types: vec![intern("pest")], data, ..Tokens::new(1, 1) };
    make_tokens(g, o, pest)?;
    Ok(())
}

/// attacking Pests you control get +1/+0 and have menace (until end of turn here)
fn mound_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !live(g, src) {
        return Ok(vec![]);
    }
    for &m in atk {
        if g.perm(m).on_bf && has_type(g, m, "pest") {
            let x = g.perm_mut(m);
            x.eot_pt.0 += 1;
            let menace = intern("menace");
            if !x.eot_kw.contains(&menace) {
                x.eot_kw.push(menace);
            }
        }
    }
    Ok(vec![])
}

// ================================================================== Forsaken Miner
/// p committed a crime: each Forsaken Miner in p's graveyard may return for {B}
pub fn miner_crime(g: &mut Game, p: PlayerId) -> Res {
    let Some(miner) = g.db.id(MINER) else { return Ok(()) };
    while g.player(p).gy.contains(&miner) && can_pay(g, p, 0, "B", false) {
        if !trigger_window(g, p, None, "Forsaken Miner: pay {B} to return it", None)? {
            return Ok(());
        }
        if !g.player(p).gy.contains(&miner) || !can_pay(g, p, 0, "B", false) {
            return Ok(());
        }
        pay(g, p, 0, "B", false)?;
        remove_card(&mut g.player_mut(p).gy, miner);
        crate::glog!(g, "    a crime: {} returns Forsaken Miner", pname(g, p));
        enter(g, p, miner, Enter::default())?;
    }
    Ok(())
}

/// engine::cast, as a spell is cast: one that targets an opponent or their things is a crime (Forsaken Miner)
pub fn spell_crime(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let Some(miner) = g.db.id(MINER) else { return Ok(()) };
    if !g.player(p).gy.contains(&miner) {
        return Ok(());
    }
    let Some((cc, ctx, _)) = g.cur_cast.clone() else { return Ok(()) };
    if cc == c && (ctx.target.is_some_and(|t| g.perm(t).owner != p) || ctx.face.is_some_and(|q| q != p)) {
        miner_crime(g, p)?;
    }
    Ok(())
}

// ================================================================== Hancock, Ghoulish Mayor
fn selfpt_on(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

fn hancock_pt(g: &Game, m: PermId) -> (i32, i32) {
    if !g.is_creature(m) || !(has_type(g, m, "zombie") || has_type(g, m, "mutant")) {
        return (0, 0);
    }
    let x: i32 = all_named(g, owner(g, m), HANCOCK).into_iter().filter(|&h| h != m).map(|h| g.perm(h).plus.abs()).sum();
    (x, x)
}

// ================================================================== Necroskitter
/// an opposing creature with a -1/-1 counter on it died: its card returns under your control
fn skitter_gy(g: &mut Game, src: Src, m: PermId) -> Res {
    let o = owner(g, src);
    let x = g.perm(m);
    if !live(g, src) || x.owner == o || x.token || x.plus >= 0 {
        return Ok(());
    }
    let (Some(cd), orig) = (x.phys.or(x.cd), x.orig) else { return Ok(()) };
    if !g.player(orig).gy.contains(&cd) || g.player(orig).is_commander(cd) {
        return Ok(());
    }
    let label = format!("Necroskitter: return {}", cname(g, cd));
    if !trigger_window(g, o, Some(src), &label, Some(card_worth(g, o, cd, true)))? {
        return Ok(());
    }
    if !remove_card(&mut g.player_mut(orig).gy, cd) {
        return Ok(());
    }
    crate::glog!(g, "    {} returns {} under its control (Necroskitter)", pname(g, o), cname(g, cd));
    enter(g, o, cd, Enter { orig: Some(orig), ..Enter::default() })?;
    Ok(())
}

// ================================================================== Nine-Lives Familiar
fn nine_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src && g.last_cast_etb {
        g.perm_mut(m).data.set(DataKey::Revival, Val::Int(8));
    }
    Ok(())
}

fn nine_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let x = g.perm(m);
    let n = x.data.int(DataKey::Revival);
    if m != src || x.token || n <= 0 {
        return Ok(());
    }
    let (Some(cd), orig) = (x.phys.or(x.cd), x.orig) else { return Ok(()) };
    g.t45_due.push((orig, cd, n - 1));
    crate::glog!(g, "    {} returns at the beginning of the next end step ({} revival counters)", cname(g, cd), n - 1);
    Ok(())
}

/// cardcode::delayed_end_step: the Familiars waiting in their owners' graveyards return
pub fn revival_returns(g: &mut Game) -> Res {
    for (p, cd, n) in std::mem::take(&mut g.t45_due) {
        if g.over || !g.player(p).alive || !remove_card(&mut g.player_mut(p).gy, cd) {
            continue; // exiled or returned by something else meanwhile
        }
        let m = enter(g, p, cd, Enter::default())?;
        g.perm_mut(m).data.set(DataKey::Revival, Val::Int(n));
        crate::glog!(g, "  {} returns to the battlefield ({n} revival counters)", cname(g, cd));
    }
    check_state(g)
}

// ================================================================== The Meathook Massacre
/// (X, value): the X that kills the most opposing worth for the least of p's own (deaths drain with Meathook out)
fn meathook_x(g: &Game, p: PlayerId, mana: u32) -> (i32, f64) {
    let creatures: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m))
        .collect();
    let mut best = (0, 0.0);
    for x in 1..=mana.min(12) as i32 {
        let mut v = 0.0;
        for &m in &creatures {
            if etgh(g, m) > x {
                continue;
            }
            v += if owner(g, m) == p { 0.5 - 0.7 * pval(g, m) } else { 1.0 + pval(g, m) };
        }
        v -= 0.15 * x as f64; // mana spent
        if v > best.1 {
            best = (x, v);
        }
    }
    best
}

fn meathook_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let (cg, cp) = cost_of(g, p, c);
    let spare = total_mana(g, p, false).saturating_sub(cg + cp.len() as u32);
    let (x, v) = meathook_x(g, p, spare);
    if x > 0 && v >= 6.0 { 68 } else { 44 }
}

/// as it enters (cast): X from the mana left, then each creature gets -X/-X until end of turn
fn meathook_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || !g.last_cast_etb {
        return Ok(());
    }
    let (x, _) = meathook_x(g, p, total_mana(g, p, false));
    if x <= 0 || !pay(g, p, x as u32, "", false)? {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), &format!("The Meathook Massacre: each creature gets -{x}/-{x}"), None)? {
        return Ok(());
    }
    crate::glog!(g, "    The Meathook Massacre: X = {x}");
    g.batch += 1; // they die together
    let prev = g.cur_batch.replace(g.batch);
    let victims: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&v| live(g, v) && g.is_creature(v))
        .collect();
    let mut r = Ok(());
    for v in victims {
        if !g.perm(v).on_bf {
            continue;
        }
        let y = g.perm_mut(v);
        y.eot_pt = (y.eot_pt.0 - x, y.eot_pt.1 - x);
        if etgh(g, v) <= 0 {
            r = die(g, v, intern("sba"));
            if r.is_err() {
                break;
            }
        }
    }
    g.cur_batch = prev;
    r
}

// ================================================================== combos
/// zero-cost artifacts the Urza list casts between each other under Hullbreaker Horror
const FREE: &[&str] =
    &["Ornithopter", "Mishra's Bauble", "Mox Opal", "Chrome Mox", "Lotus Petal", "Welding Jar", "Tormod's Crypt"];
const ALTARS: &[&str] = &["Ashnod's Altar", "Phyrexian Altar", "Warren Soultrader"];

const NOT_READY: fn() -> Ready = || (false, vec![], vec![]);

fn free(g: &Game, c: CardId) -> bool {
    g.db.get(c).types.has(Types::ARTIFACT) && g.db.get(c).cmc == 0 && !g.db.get(c).land
}

/// Forensic Gadgeteer makes Basalt Monolith's untap {2}: it taps for {C}{C}{C}, so each loop nets one: unbounded
/// colourless mana, a win with a mana sink (a tapped Monolith needs {2} to start)
fn gadgeteer_basalt(g: &Game, p: PlayerId) -> Ready {
    let Some(b) = on_bf(g, p, BASALT) else { return NOT_READY() };
    let gad = on_bf(g, p, GADGETEER);
    let mut casts = vec![];
    let mut need = if g.perm(b).tapped { 2 } else { 0 };
    if gad.is_none() {
        let Some(c) = in_hand(g, p, GADGETEER) else { return NOT_READY() };
        if !castable(g, p, c, "hand") {
            return NOT_READY();
        }
        need += g.db.get(c).generic;
        casts.push(c);
    }
    if !crate::impls::combos::mana_sink(g, p) || !can_pay(g, p, need, if casts.is_empty() { "" } else { "U" }, false) {
        return NOT_READY();
    }
    (true, [Some(b), gad].into_iter().flatten().collect(), casts)
}

/// Hullbreaker Horror: each spell cast returns a permanent to hand. With Urza, two free artifacts cast between each
/// other (each taps for {U} first) make unbounded blue mana; with Sol Ring and a free spell, Sol Ring recast nets
/// {1} each loop. Unbounded mana with a sink wins (Urza's {5} plays the library, or Walking Ballista).
fn horror_loop(g: &Game, p: PlayerId) -> Ready {
    let horror = on_bf(g, p, HORROR);
    let mut casts = vec![];
    if horror.is_none() {
        let Some(h) = in_hand(g, p, HORROR) else { return NOT_READY() };
        if !castable(g, p, h, "hand") || !can_pay(g, p, g.db.get(h).generic, &g.db.get(h).pips, false) {
            return NOT_READY();
        }
        casts.push(h);
    }
    let pl = g.player(p);
    let free_hand: Vec<CardId> = pl.hand.iter().copied().filter(|&c| free(g, c) && castable(g, p, c, "hand")).collect();
    let free_bf = pl.perms.iter().any(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| free(g, c)));
    let urza = on_bf(g, p, URZA);
    let sol = on_bf(g, p, "Sol Ring");
    let looping = !free_hand.is_empty() && ((urza.is_some() && (free_bf || free_hand.len() >= 2)) || sol.is_some());
    if !looping || !crate::impls::combos::mana_sink(g, p) {
        return NOT_READY();
    }
    let keys: Vec<PermId> = [horror, urza.or(sol)].into_iter().flatten().collect();
    (true, keys, casts)
}

/// Reassembling Skeleton (returns for {1}{B}) or Gravecrawler (cast from the graveyard for {B} with another Zombie)
/// sacrificed to an altar (Phyrexian Altar: one mana; Ashnod's: two; Warren Soultrader: 1 life, a Treasure) with
/// Pitiless Plunderer's Treasure pays for its own return: unbounded deaths, a win with a death payoff
fn plunderer_loop(g: &Game, p: PlayerId) -> Ready {
    let (Some(pl), Some(altar)) = (on_bf(g, p, PLUNDERER), ALTARS.iter().find_map(|n| on_bf(g, p, n))) else {
        return NOT_READY();
    };
    if named(g, altar, "Warren Soultrader") && g.player(p).life < 10 {
        return NOT_READY();
    }
    let in_gy = |n: &str| g.player(p).gy.iter().any(|&c| cname(g, c) == n);
    let skel = on_bf(g, p, SKELETON).is_some() || (in_gy(SKELETON) && can_pay(g, p, 1, "B", false));
    let zombie =
        g.player(p).perms.iter().any(|&m| live(g, m) && has_type(g, m, "zombie") && !named(g, m, GRAVECRAWLER));
    let crawler =
        zombie && (on_bf(g, p, GRAVECRAWLER).is_some() || (in_gy(GRAVECRAWLER) && can_pay(g, p, 0, "B", false)));
    if !(skel || crawler) {
        return NOT_READY();
    }
    let Some(payoff) = crate::impls::combos::loop_payoff(g, p) else { return NOT_READY() };
    (true, vec![pl, altar, payoff], vec![])
}

pub static COMBOS: [Combo; 3] = [
    Combo {
        name: "Basalt Monolith + Forensic Gadgeteer",
        groups: &[&[BASALT], &[GADGETEER]],
        ready: gadgeteer_basalt,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Hullbreaker Horror + Urza / Sol Ring + free artifacts",
        groups: &[QUIET, &[HORROR], &[URZA, "Sol Ring"], FREE], // quiet: Sol Ring and the Horror are in many lists
        ready: horror_loop,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff",
        groups: &[&[PLUNDERER], &[SKELETON, GRAVECRAWLER], ALTARS, &crate::ai::plans::DRAIN_PAYOFFS],
        ready: plunderer_loop,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
];

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, PADEEM)?;
    c.grant_kw = Some(padeem_kw);
    c.upkeep = Some(padeem_upkeep);
    let c = r.card(db, CHIP)?;
    c.options = Some(chip_options);
    c.lands_from_top = Some(chip_lands);
    r.card(db, "Thought Monitor")?.self_cost = Some(affinity);
    r.card(db, "Thoughtcast")?.self_cost = Some(affinity);
    let c = r.card(db, UTHROS)?;
    c.land_options = Some(uthros_options);
    c.dyn_mana_land = Some(uthros_mana);
    let c = r.card(db, ENTOMB)?;
    c.hand_options = Some(entomb_options);
    c.resolve = Some(entomb_resolve);
    r.card(db, CHAMBER)?.etb = Some(chamber_etb);
    let c = r.card(db, MOUND)?;
    c.dies = Some(mound_dies);
    c.attack = Some(mound_attack);
    at_once(c, Event::Attack, true);
    let c = r.card(db, HANCOCK)?;
    c.etb = Some(selfpt_on);
    at_once(c, Event::Etb, true);
    r.creature_pt.push(hancock_pt);
    r.card(db, SKITTER)?.creature_to_gy = Some(skitter_gy);
    let c = r.card(db, NINE)?;
    c.etb = Some(nine_etb);
    at_once(c, Event::Etb, true);
    c.self_dies = Some(nine_dies);
    at_once(c, Event::SelfDies, true);
    let c = r.card(db, MEATHOOK)?;
    c.etb = Some(meathook_etb);
    at_once(c, Event::Etb, true);
    c.prio = Some(meathook_prio);
    Ok(())
}
