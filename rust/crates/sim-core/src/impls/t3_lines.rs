//! Tier 3's combo and alternate-win lines that the catalogue missed (rust/agents/t3-misses.md). Each line uses only
//! cards already in the pool lists (decklists/pool/t3-high-b3/), checked against their Oracle text.
//!
//! Combos (chained into `combos::all()`):
//! - **Helm of the Host + Aurelia, the Warleader / Combat Celebrant** (Aurelia): at the beginning of combat Helm makes
//!   a hasty, nonlegendary token copy of the equipped creature. An Aurelia copy's first attack untaps the team and adds
//!   a combat; a Celebrant copy exerts for the same. Each new combat makes a new copy: unbounded combats. The base
//!   catalogue had only Celebrant with the Helm already on it; Aurelia is the commander, and the line now pays the
//!   equip ({5}) as its first step.
//! - **Chatterfang + Pitiless Plunderer + a sacrifice outlet + a death payoff** (Korvold): a creature dies, Plunderer
//!   makes a Treasure, Chatterfang adds a Squirrel; sacrifice the Squirrel and it repeats. Goblin Bombardment is both
//!   the outlet and the payoff. With Korvold out each sacrifice also draws a card (not optional), so the loop must kill
//!   before the library runs out.
//! - **Staff of Domination + an Elf that taps for five or more + a finisher** (Marwyn): tap the Elf, {3}: untap it,
//!   {1}: untap the Staff, net mana each time; with unbounded mana the Staff's {5}: draw finds Ezuri, Finale of
//!   Devastation or Craterhoof Behemoth.
//!
//! Cards the Aurelia deck never played (no priority, no equip, its activation was Sauron's alone):
//! - **Sword of Feast and Famine**: cast and equipped (its combat damage untapping the lands is the engine's).
//! - **Aggravated Assault**: {3}{R}{R}: untap your creatures, an additional combat (`assault_again`, read by
//!   `engine::combat`). With the Sword connecting, the untapped lands pay for the next one.
//!
//! Tergrid: Torment of Hailfire is cast (its option was Sauron's alone: `threats::torment_options`).

use crate::cards::CardDb;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, equip_to};
use crate::engine::values::{epow, has_type, stopped};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready, Step, StepKind, done, no_keys, no_src};
use crate::state::{Game, Step as TurnStep};
use crate::tag::Tag;

pub const AURELIA_KEY: &str = "aurelia-boros-extra-combats";
pub const TERGRID_KEY: &str = "tergrid-mono-black-disruption";

const HELM: &str = "Helm of the Host";
const AURELIA: &str = "Aurelia, the Warleader";
const CELEBRANT: &str = "Combat Celebrant";
const SOFF: &str = "Sword of Feast and Famine";
const ASSAULT: &str = "Aggravated Assault";

const CHATTER: &str = "Chatterfang, Squirrel General";
const PLUNDERER: &str = "Pitiless Plunderer";
const KORVOLD: &str = "Korvold, Fae-Cursed King";
const BOMBARDMENT: &str = "Goblin Bombardment";
const SOULTRADER: &str = "Warren Soultrader";
/// free sacrifice outlets (Warren Soultrader costs 1 life a time, paid back by its own Treasure's Squirrel's payoff)
const OUTLETS: &[&str] = &["Viscera Seer", BOMBARDMENT, "Phyrexian Altar", "Ashnod's Altar", SOULTRADER];
/// death payoffs that hit each opponent once per creature death
const EACH_OPP: &[&str] = &[
    "Zulaport Cutthroat",
    "Marionette Apprentice",
    "Cruel Celebrant",
    "Bastion of Remembrance",
    "The Meathook Massacre",
];
/// death payoffs that hit one target per creature death
const ONE_TARGET: &[&str] = &["Blood Artist", "Mayhem Devil", "Falkenrath Noble"];

const STAFF: &str = "Staff of Domination";
const EZURI: &str = "Ezuri, Renegade Leader";
const FINALE: &str = "Finale of Devastation";
const HOOF: &str = "Craterhoof Behemoth";

const NOT_READY: fn() -> Ready = || (false, vec![], vec![]);

// ------------------------------------------------------------------ helpers
fn named(g: &Game, m: PermId, name: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == name)
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// p's live permanent of this name
fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && named(g, m, name))
}

/// a card of this name in p's hand or library (with unbounded mana and a Staff draw, both are within reach)
fn in_hand_or_library(g: &Game, p: PlayerId, name: &str) -> bool {
    let pl = g.player(p);
    pl.hand.iter().chain(&pl.library).any(|&c| &*g.db.get(c).name == name)
}

/// p's turn, before combat (no combat yet this turn)
fn precombat(g: &Game, p: PlayerId) -> bool {
    g.active == Some(p) && matches!(g.step, TurnStep::Start | TurnStep::Main1) && g.player(p).combat_no == 0
}

fn can_attack(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    live(g, m) && g.is_creature(m) && !x.noatk && (!x.sick || crate::dsl::has_kw(g, m, "haste"))
}

/// the mana a creature taps for (a mana creature's own count: Priest of Titania's Elves, Circle of Dreams Druid's
/// creatures, Marwyn's power)
fn dork_mana(g: &Game, p: PlayerId, m: PermId) -> u32 {
    let Some(c) = g.perm(m).cd else { return 0 };
    if !g.db.get(c).tag(Tag::Dork) {
        return 0;
    }
    crate::cardcode::dyn_mana_perm(g, p, m).unwrap_or(1)
}

const fn step(label: &'static str, kind: StepKind, run: fn(&mut Game, PlayerId) -> Res<bool>) -> Step {
    Step { label, kind, src: no_src, pay: (0, ""), run, keys: no_keys }
}

// ================================================================== Helm of the Host + Aurelia / Combat Celebrant
/// the creature Helm goes on: the one it is on already, else Aurelia, else Combat Celebrant; and the equip cost left
fn helm_host(g: &Game, p: PlayerId) -> Option<(PermId, PermId, u32)> {
    let helm = on_bf(g, p, HELM)?;
    let hosts: Vec<PermId> = [AURELIA, CELEBRANT]
        .iter()
        .filter_map(|n| on_bf(g, p, n))
        .filter(|&m| g.is_creature(m) && g.perm(m).owner == p)
        .collect();
    if let Some(&h) = hosts.iter().find(|&&m| g.perm(helm).attached == Some(m)) {
        return Some((helm, h, 0));
    }
    let &h = hosts.first()?;
    if stopped(g, HELM) {
        return None; // Disruptor Flute: no equip
    }
    Some((helm, h, crate::impls::cloud::equip_cost(g, p, helm, h, 5)))
}

/// before combat on p's turn: Helm of the Host and Aurelia or Combat Celebrant, with the equip paid for if Helm is
/// elsewhere
fn helm_ready(g: &Game, p: PlayerId) -> Ready {
    if !precombat(g, p) {
        return NOT_READY();
    }
    let Some((helm, host, cost)) = helm_host(g, p) else { return NOT_READY() };
    if cost > 0 && !can_pay(g, p, cost, "", false) {
        return NOT_READY();
    }
    (true, vec![helm, host], vec![])
}

fn helm_src(g: &Game, p: PlayerId) -> Option<PermId> {
    on_bf(g, p, HELM)
}

fn helm_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    helm_host(g, p).map_or(vec![], |(h, m, _)| vec![h, m])
}

/// equip Helm to its host (an activated ability: it can be answered); already on: nothing to do
fn equip_helm(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some((helm, host, cost)) = helm_host(g, p) else { return Ok(false) };
    if g.perm(helm).attached == Some(host) {
        return Ok(true);
    }
    if !can_pay(g, p, cost, "", false) {
        return Ok(false);
    }
    crate::glog!(g, "    {} equips {HELM} to {}", g.player(p).name, g.perm(host).name);
    equip_to(g, p, helm, host, cost)?;
    Ok(g.perm(helm).on_bf && g.perm(host).on_bf && g.perm(helm).attached == Some(host))
}

const HELM_STEPS: [Step; 3] = [
    step("equip Helm of the Host", StepKind::Engine, equip_helm),
    Step {
        label: "Helm of the Host makes a hasty token copy at the beginning of combat",
        kind: StepKind::Trigger,
        src: helm_src,
        pay: (0, ""),
        run: done,
        keys: no_keys,
    },
    Step {
        label: "Helm and its host stay: each new copy's attack adds a combat",
        kind: StepKind::Permanent,
        src: no_src,
        pay: (0, ""),
        run: done,
        keys: helm_keys,
    },
];

// ================================================================== Chatterfang + Pitiless Plunderer
/// the loop's damage per Squirrel sacrificed: (each-opponent payoffs, one-target payoffs)
fn chatter_rate(g: &Game, p: PlayerId, outlet: PermId) -> (u32, u32) {
    let count = |names: &[&str]| names.iter().filter(|n| on_bf(g, p, n).is_some()).count() as u32;
    let each = count(EACH_OPP);
    let one = count(ONE_TARGET) + named(g, outlet, BOMBARDMENT) as u32;
    (each, one)
}

/// Korvold draws a card for each sacrifice (not a "may"): the Squirrels the loop needs, at its damage per Squirrel,
/// must fit in the library
fn korvold_fits(g: &Game, p: PlayerId, each: u32, one: u32) -> bool {
    if on_bf(g, p, KORVOLD).is_none() {
        return true;
    }
    let lives: Vec<i64> = g.opps(p).map(|q| g.player(q).life.max(0) as i64).collect();
    let n = lives.len() as i64;
    let (each, one) = (each as i64, one as i64);
    if n == 0 || each + one == 0 {
        return false;
    }
    let max = *lives.iter().max().unwrap();
    let sum: i64 = lives.iter().sum();
    let need = ((max + each + one - 1) / (each + one)).max((sum + each * n + one - 1) / (each * n + one));
    need <= g.player(p).library.len() as i64
}

/// Chatterfang, Pitiless Plunderer, a free sacrifice outlet, a death payoff (Goblin Bombardment is its own) and a
/// creature to start with
fn chatter_ready(g: &Game, p: PlayerId) -> Ready {
    let (Some(chat), Some(pl)) = (on_bf(g, p, CHATTER), on_bf(g, p, PLUNDERER)) else { return NOT_READY() };
    let Some(outlet) = OUTLETS.iter().filter(|n| !stopped(g, n)).find_map(|n| on_bf(g, p, n)) else {
        return NOT_READY();
    };
    if named(g, outlet, SOULTRADER) && g.player(p).life < 10 {
        return NOT_READY();
    }
    let payoff = crate::impls::combos::loop_payoff(g, p).or_else(|| named(g, outlet, BOMBARDMENT).then_some(outlet));
    let Some(payoff) = payoff else { return NOT_READY() };
    let fodder =
        g.player(p).perms.iter().any(|&m| {
            live(g, m) && g.is_creature(m) && ![chat, pl, outlet, payoff].contains(&m) && !named(g, m, KORVOLD)
        });
    if !fodder {
        return NOT_READY();
    }
    let (each, one) = chatter_rate(g, p, outlet);
    if !korvold_fits(g, p, each, one) {
        return NOT_READY();
    }
    let mut keys = vec![chat, pl, outlet];
    if payoff != outlet {
        keys.push(payoff);
    }
    (true, keys, vec![])
}

// ================================================================== Staff of Domination + a big Elf
/// p's creature that taps for five or more and can tap now (the Staff untaps a tapped one for {3}, plus {1} for the
/// Staff itself if it is tapped)
fn staff_dork(g: &Game, p: PlayerId, staff: PermId) -> Option<PermId> {
    let staff_tapped = g.perm(staff).tapped as u32;
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            live(g, m)
                && g.is_creature(m)
                && (!x.sick || crate::dsl::has_kw(g, m, "haste"))
                && dork_mana(g, p, m) >= 5
                && (!x.tapped || can_pay(g, p, 3 + staff_tapped, "", false))
                && (x.tapped || staff_tapped == 0 || dork_mana(g, p, m) >= 6 || can_pay(g, p, 1, "", false))
        })
        .collect();
    crate::engine::zones::max_by(&cs, |m| dork_mana(g, p, m) as f64)
}

/// unbounded mana before combat: Ezuri (with an Elf to attack), Finale of Devastation (X = 10 or more: haste for all)
/// or Craterhoof Behemoth (with a creature to attack), in hand, in the library (the Staff draws it) or Ezuri out
fn staff_ready(g: &Game, p: PlayerId) -> Ready {
    if !precombat(g, p) || stopped(g, STAFF) {
        return NOT_READY();
    }
    let Some(staff) = on_bf(g, p, STAFF) else { return NOT_READY() };
    let Some(dork) = staff_dork(g, p, staff) else { return NOT_READY() };
    let perms = &g.player(p).perms;
    let elf_attacker =
        perms.iter().any(|&m| (m == dork || can_attack(g, m)) && has_type(g, m, "elf") && !g.perm(m).noatk);
    let attacker = perms.iter().any(|&m| (m == dork || can_attack(g, m)) && !g.perm(m).noatk);
    let ezuri = on_bf(g, p, EZURI).is_some() || in_hand_or_library(g, p, EZURI);
    let win = (ezuri && elf_attacker)
        || (in_hand_or_library(g, p, FINALE) && perms.iter().any(|&m| live(g, m) && g.is_creature(m)))
        || (in_hand_or_library(g, p, HOOF) && attacker);
    if !win {
        return NOT_READY();
    }
    (true, vec![staff, dork], vec![])
}

pub static COMBOS: [Combo; 3] = [
    Combo {
        name: "Helm of the Host + Aurelia, the Warleader / Combat Celebrant (equipped now)",
        groups: &[&[HELM], &[AURELIA, CELEBRANT]],
        ready: helm_ready,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &HELM_STEPS,
    },
    Combo {
        name: "Chatterfang + Pitiless Plunderer + a sacrifice outlet + a death payoff",
        groups: &[
            &[CHATTER],
            &[PLUNDERER],
            OUTLETS,
            &["Zulaport Cutthroat", "Marionette Apprentice", "Blood Artist", "Mayhem Devil", BOMBARDMENT],
        ],
        ready: chatter_ready,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Staff of Domination + an Elf that taps for five + Ezuri / Finale / Craterhoof",
        groups: &[
            &[STAFF],
            &["Priest of Titania", "Elvish Archdruid", "Circle of Dreams Druid", "Marwyn, the Nurturer"],
            &[EZURI, FINALE, HOOF],
        ],
        ready: staff_ready,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
];

// ================================================================== Aurelia: Sword of Feast and Famine, Aggravated Assault
fn aurelia(g: &Game, p: PlayerId) -> bool {
    g.player(p).key == AURELIA_KEY && !crate::human::is_human(g, p)
}

fn has_card(g: &Game, p: PlayerId, name: &str) -> bool {
    on_bf(g, p, name).is_some() || g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

/// the Aurelia deck casts the Sword like the other cheap Swords (it had no priority: tag only, no compiled ability)
fn soff_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if aurelia(g, p) { 60 } else { 0 }
}

/// Aggravated Assault: an extra combat a turn; with the Sword (its lands untap on a hit) a combat after every hit
fn assault_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if !aurelia(g, p) {
        return 0;
    }
    if has_card(g, p, SOFF) { 58 } else { 45 }
}

/// the Sword's wearer: the creature most likely to connect (flying first), then the biggest
fn soff_wearer(g: &Game, p: PlayerId, e: PermId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m) && !g.perm(m).noatk && g.perm(e).attached != Some(m))
        .collect();
    crate::engine::zones::max_by(&cs, |m| {
        let fly = g.perm(m).fly || crate::dsl::has_kw(g, m, "flying");
        (if fly { 3.0 } else { 0.0 }) + 0.3 * epow(g, m) as f64 - if g.perm(m).sick { 1.5 } else { 0.0 }
            + if g.perm(m).is_cmd { 1.0 } else { 0.0 }
    })
}

/// equip {2}: the Sword onto its wearer when it is on nothing of p's
fn soff_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !aurelia(g, p) || g.perm(src).owner != p || !live(g, src) || stopped(g, SOFF) {
        return Ok(vec![]);
    }
    if g.perm(src).attached.is_some_and(|t| live(g, t) && g.perm(t).owner == p) {
        return Ok(vec![]);
    }
    let Some(m) = soff_wearer(g, p, src) else { return Ok(vec![]) };
    let n = crate::impls::cloud::equip_cost(g, p, src, m, 2);
    if !can_pay(g, p, n, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: if post == Some(false) { 5.5 } else { 3.0 },
        label: format!("equip {SOFF}"),
        act: Some(Action::Equip { src, target: m, n }),
    }])
}

/// After combat `ncomb` (no other extra combat pending): the Aurelia deck activates Aggravated Assault ({3}{R}{R}:
/// untap its creatures, another combat). Again after any combat where the Sword of Feast and Famine connected (its
/// lands untapped: the loop), else once a turn with enough power to swing. True: the combat comes (the caller untaps
/// the creatures).
pub fn assault_again(g: &mut Game, p: PlayerId, ncomb: u32, conn: &[PermId]) -> Res<bool> {
    if !aurelia(g, p) || ncomb >= 12 || stopped(g, ASSAULT) || g.over {
        return Ok(false);
    }
    let Some(a) = on_bf(g, p, ASSAULT) else { return Ok(false) };
    if !can_pay(g, p, 3, "RR", false) || g.opps(p).next().is_none() {
        return Ok(false);
    }
    let sword_hit = conn.iter().any(|&m| {
        g.perm(m).on_bf
            && g.player(p).perms.iter().any(|&e| live(g, e) && named(g, e, SOFF) && g.perm(e).attached == Some(m))
    });
    let power: i32 = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| live(g, m) && g.is_creature(m) && !g.perm(m).noatk)
        .map(|&m| epow(g, m))
        .sum();
    if !sword_hit && (ncomb > 2 || power < 6) {
        return Ok(false);
    }
    pay(g, p, 3, "RR", false)?;
    crate::glog!(g, "  {} activates {ASSAULT}: untap, an additional combat", g.player(p).name);
    ability_window(g, p, Some(a), "untap, an additional combat", Some(7.0), None)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if db.id(SOFF).is_some() {
        let c = r.card(db, SOFF)?;
        c.prio = Some(soff_prio);
        c.options = Some(soff_options);
    }
    if db.id(ASSAULT).is_some() {
        r.card(db, ASSAULT)?.prio = Some(assault_prio);
    }
    Ok(())
}
