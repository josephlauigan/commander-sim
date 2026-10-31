//! Python's `cards/impl/combos.py`: combos for the opponent pools. A combo is assembled from pieces; when a player
//! has everything it needs, the AI goes for it: any spell still to be cast goes through the normal counter window,
//! then each opponent gets one chance to answer a key piece with instant-speed removal (or with a counter / ability
//! counter where noted). If nothing stops it, the loop is executed abstractly: the player wins (or, for locks, the
//! lock applies).
//!
//! Opponents value the pieces of a combo that is one step from completion much higher (removal, counterspells), and
//! combo decks get tutor wish lists for their missing pieces. search.deck_combos and search.combo_progress (the
//! look-ahead's reading of a combo in the making) are here too.

use crate::engine::repl;
use crate::ai::plans::DRAIN_PAYOFFS;
use crate::cards::CardDb;
use crate::engine::cast::{cast_card, castable};
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, can_pay_with_rituals, cost_of, pay, seq_cost}; // --- ai-fix: C
use crate::engine::removal::legal_targets;
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{has_type, untargetable};
use crate::engine::zones::{Enter, die, draw, enter, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Step as TurnStep, Val};
use crate::tag::Tag;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::LazyLock;

/// what a combo's `ready` finds: (ready, its key permanents, the cards still to cast from hand)
pub type Ready = (bool, Vec<PermId>, Vec<CardId>);

const NOT_READY: fn() -> Ready = || (false, vec![], vec![]);

/// combos.Combo: groups of card names (one of each group is needed); `ready` checks the position; `mana` is paid
/// for the loop; `finish` is what happens (None: p wins); `xcost`: mana spent on X for X spells cast as part of the
/// combo (Walking Ballista needs X >= 1 to survive)
pub struct Combo {
    pub name: &'static str,
    pub groups: &'static [&'static [&'static str]],
    pub ready: fn(&Game, PlayerId) -> Ready,
    pub mana: (u32, &'static str),
    pub finish: Option<fn(&mut Game, PlayerId) -> Res>,
    pub xcost: &'static [(&'static str, u32)],
    /// the steps of the line, each with its own answer window (empty: the old single window at the end)
    pub steps: &'static [Step],
}

impl Combo {
    fn x_for(&self, name: &str) -> u32 {
        self.xcost.iter().find(|x| x.0 == name).map_or(0, |x| x.1)
    }
}

/// how the opponents may answer a step of a combo
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepKind {
    /// a spell: `run` casts it (normal counter window, as any spell); false if it was countered or can't be cast
    Spell,
    /// a triggered ability: its cost is paid, it goes on the stack with `src` as its source (the ability-answer
    /// path: Ertai, Louisoix, Tidebinder, Stifle-likes), and `run` is its effect if it resolves
    Trigger,
    /// an activated ability: as Trigger (`ability_window`)
    Ability,
    /// a permanent that must survive: `run` happens, then each opponent may remove or bounce one of `keys` at instant
    /// speed (`interrupted`)
    Permanent,
    /// something the real engine does with its own windows (a cast Godo's enter trigger, a copy's enters effect):
    /// `run` makes it happen and answers whether it came out as the line needs
    Engine,
}

/// one step of a combo's line (see `Combo::steps`): an answered step stops the combo
#[derive(Clone, Copy)]
pub struct Step {
    pub label: &'static str,
    pub kind: StepKind,
    /// Trigger / Ability: the permanent the ability belongs to
    pub src: fn(&Game, PlayerId) -> Option<PermId>,
    /// Trigger / Ability: the cost, paid before the ability goes on the stack
    pub pay: (u32, &'static str),
    pub run: fn(&mut Game, PlayerId) -> Res<bool>,
    /// Permanent: what opponents may answer, looked up after `run`
    pub keys: fn(&Game, PlayerId) -> Vec<PermId>,
}

pub fn no_src(_g: &Game, _p: PlayerId) -> Option<PermId> {
    None
}

pub fn no_keys(_g: &Game, _p: PlayerId) -> Vec<PermId> {
    vec![]
}

pub fn done(_g: &mut Game, _p: PlayerId) -> Res<bool> {
    Ok(true)
}

/// every modeled combo, in order: `BASE` (Python's), then each Tier 6 module's slice. A new `t6_combos_*.rs` module
/// exports `pub static COMBOS: [Combo; N]` and is chained here (the index of a combo is its place in this list)
pub fn all() -> &'static [&'static Combo] {
    static ALL: LazyLock<Vec<&'static Combo>> = LazyLock::new(|| {
        BASE.iter()
            .chain(super::t6_combos_godo::COMBOS.iter())
            .chain(super::t6_combos_t6::COMBOS.iter())
            .chain(super::t6_combos_extra::COMBOS.iter())
            .chain(super::t3_marwyn::COMBOS.iter()) // --- Tier 3 (marwyn)
            .chain(super::t45_krenko::COMBOS.iter()) // --- Tiers 4/5 (krenko)
            .chain(super::t45_prosper::COMBOS.iter()) // --- Tiers 4/5 (prosper)
            .chain(super::t45_chulane_kinnan::COMBOS.iter()) // --- Tiers 4/5 (chulane-kinnan)
            .chain(super::t45_winota::COMBOS.iter()) // --- Tiers 4/5 (winota)
            .chain(super::t45_heliod_zur::COMBOS.iter()) // --- Tiers 4/5 (heliod-zur)
            .chain(super::t45_urza_yawgmoth::COMBOS.iter()) // --- Tiers 4/5 (urza-yawgmoth)
            .chain(super::t45_edgar::COMBOS.iter()) // --- Tiers 4/5 (edgar)
            .chain(super::t12_windgrace::COMBOS.iter()) // --- Tiers 1/2 (windgrace)
            .chain(super::t12_brago_meren::COMBOS.iter()) // --- Tiers 1/2 (brago-meren)
            .chain(super::t45_miirym::COMBOS.iter()) // --- Tiers 4/5 (miirym)
            .chain(super::t3_lines::COMBOS.iter()) // --- t3 lines
            .chain(super::cc_brago::COMBOS.iter()) // --- casual combos (brago)
            .chain(super::cc_teysa::COMBOS.iter()) // --- casual combos (teysa)
            .chain(super::t6_combos_scion::COMBOS.iter()) // --- scion
            .chain(super::ghave::COMBOS.iter()) // --- ghave
            .collect()
    });
    &ALL
}

// ------------------------------------------------------------------ the pieces
const KIKI: &str = "Kiki-Jiki, Mirror Breaker";
const FABLE: &str = "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki";
const KIKI_TARGETS: &[&str] =
    &["Zealous Conscripts", "Felidar Guardian", "Restoration Angel", "Pestermite", "Deceiver Exarch"];
const MANA_SINKS: &[&str] =
    &["Walking Ballista", "Urza, Lord High Artificer", "Kinnan, Bonder Prodigy", "Thassa's Oracle"];
const BOUNCERS: &[&str] = &["Shrieking Drake", "Whitemane Lion", "Kor Skyfisher", "Man-o'-War"];
// --- Tiers 4/5 (urza-yawgmoth): Hancock, Ghoulish Mayor has undying
const UNDYING_LOOP: &[&str] = &[
    "Mikaeus, the Unhallowed",
    "Geralf's Messenger",
    "Butcher Ghoul",
    "Young Wolf",
    "Nether Traitor",
    "Hancock, Ghoulish Mayor",
];
const STAFF_OUTLETS: &[&str] = &["Skirk Prospector", "Goblin Bombardment", "Ashnod's Altar", "Phyrexian Altar"];

/// combos.COMBOS, in Python's order
pub static BASE: [Combo; 14] = [
    Combo {
        name: "Kiki-Jiki + Zealous Conscripts / Felidar / Resto",
        groups: &[&[KIKI, FABLE], KIKI_TARGETS],
        ready: kiki,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Isochron Scepter + Dramatic Reversal",
        groups: &[&["Isochron Scepter"], &["Dramatic Reversal"], MANA_SINKS],
        ready: scepter,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Power Artifact / Rings of Brighthearth + Monolith",
        groups: &[&["Power Artifact", "Rings of Brighthearth"], &["Basalt Monolith", "Grim Monolith"], MANA_SINKS],
        ready: power_artifact,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Kinnan + Basalt Monolith",
        groups: &[&["Kinnan, Bonder Prodigy"], &["Basalt Monolith"]],
        ready: kinnan_basalt,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &super::t6_combos_extra::KINNAN_BASALT_STEPS,
    },
    Combo {
        name: "Kinnan + Freed from the Real / Pemmin's Aura",
        groups: &[&["Kinnan, Bonder Prodigy"], &["Freed from the Real", "Pemmin's Aura"]],
        ready: kinnan_aura,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Aluren + Chulane + a bouncing creature",
        groups: &[&["Aluren"], &["Chulane, Teller of Tales"], BOUNCERS],
        ready: aluren,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Sanguine Bond + Exquisite Blood",
        groups: &[&["Sanguine Bond"], &["Exquisite Blood"]],
        ready: bond,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Thassa's Oracle + Demonic Consultation / Tainted Pact",
        groups: &[&["Thassa's Oracle"], &["Demonic Consultation", "Tainted Pact"]],
        ready: oracle,
        mana: (0, ""),
        finish: Some(super::t6_combos_extra::oracle_finish),
        xcost: &[],
        steps: &super::t6_combos_extra::ORACLE_STEPS,
    },
    Combo {
        name: "Helm of the Host + Combat Celebrant",
        groups: &[&["Helm of the Host"], &["Combat Celebrant"]],
        ready: helm,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Gravecrawler + Phyrexian Altar + a Zombie + a death payoff",
        groups: &[&["Gravecrawler"], &["Phyrexian Altar"], &DRAIN_PAYOFFS],
        ready: gravecrawler,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Yawgmoth + undying loop + a death payoff",
        groups: &[&["Yawgmoth, Thran Physician"], UNDYING_LOOP, &DRAIN_PAYOFFS],
        ready: yawg,
        mana: (0, ""),
        finish: Some(yawg_finish),
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Krenko + Thornbite Staff + a sacrifice outlet",
        groups: &[&["Krenko, Mob Boss"], &["Thornbite Staff"], STAFF_OUTLETS],
        ready: staff,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Karn, the Great Creator + Mycosynth Lattice",
        groups: &[&["Karn, the Great Creator"], &["Mycosynth Lattice"]],
        ready: karn,
        mana: (0, ""),
        finish: Some(lock_lattice),
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Heliod, Sun-Crowned + Walking Ballista",
        groups: &[&["Heliod, Sun-Crowned"], &["Walking Ballista"]],
        ready: heliod,
        mana: (1, "W"),
        finish: None,
        xcost: &[("Walking Ballista", 2)],
        steps: &[],
    },
];

/// the first group of a "quiet" combo: a line that is offered and played (options, steps, answers) but whose cards are
/// not registered as pieces (no tutor wish lists, discard keeps, piece threats or deck progress: a deck outside Tier 6
/// that happens to play Sol Ring, Underworld Breach or Goblin Welder must not change). The sentinel is no card, so
/// `deck_combos` and `missing_pieces` never see the combo either.
pub const QUIET: &[&str] = &["~quiet"];

pub fn quiet(c: &Combo) -> bool {
    c.groups.first().is_some_and(|g| *g == QUIET)
}

/// combos.PIECES: is this card name a piece of a modeled combo?
pub fn is_piece_name(name: &str) -> bool {
    // a set built once: this runs for every permanent each time the AI values a board
    static SET: std::sync::OnceLock<std::collections::HashSet<&'static str>> = std::sync::OnceLock::new();
    SET.get_or_init(|| {
        all().iter().filter(|c| !quiet(c)).flat_map(|c| c.groups.iter().flat_map(|grp| grp.iter().copied())).collect()
    })
    .contains(name)
}

/// combos.PIECES as card ids (the names in the card database)
pub fn pieces(g: &Game) -> Vec<CardId> {
    let mut out = vec![];
    for cmb in all() {
        if quiet(cmb) {
            continue;
        }
        for grp in cmb.groups {
            for n in grp.iter() {
                if let Some(c) = g.db.id(n)
                    && !out.contains(&c)
                {
                    out.push(c);
                }
            }
        }
    }
    out
}

// ------------------------------------------------------------------ helpers
fn name_of(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// combos.on_bf: p's permanent of this name (not phased out; untapped, or able to tap now, if asked)
fn on_bf_with(g: &Game, p: PlayerId, name: &str, untapped: bool, unsick: bool) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.cd.is_some_and(|c| name_of(g, c) == name)
            && !x.phased
            && (!untapped || !x.tapped)
            && (!unsick || !x.sick || crate::dsl::has_kw(g, m, "haste"))
    })
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    on_bf_with(g, p, name, false, false)
}

/// combos.in_hand
fn in_hand(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| name_of(g, c) == name)
}

fn any_bf(g: &Game, p: PlayerId, names: &[&str]) -> Option<PermId> {
    names.iter().find_map(|n| on_bf(g, p, n))
}

fn any_hand(g: &Game, p: PlayerId, names: &[&str]) -> Option<CardId> {
    names.iter().find_map(|n| in_hand(g, p, n))
}

fn has_piece(g: &Game, p: PlayerId, grp: &[&str]) -> bool {
    // --- Tiers 1/2 (windgrace): a land piece on the battlefield (Dark Depths, Thespian's Stage)
    let land = |n: &str| g.player(p).lands.iter().any(|&l| name_of(g, g.perm(l).card()) == n);
    grp.iter().any(|n| on_bf(g, p, n).is_some() || in_hand(g, p, n).is_some() || land(n))
}

/// combos.free_outlet: a sacrifice outlet of p's (common.outlets); `only`: one of these
fn free_outlet(g: &Game, p: PlayerId, only: Option<&[&str]>) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.cd.is_some_and(|c| {
            let n = name_of(g, c);
            crate::cardcode::is_sac_outlet(n)
                && only.is_none_or(|o| o.contains(&n))
                && !crate::engine::values::stopped(g, n)
        }) && !x.phased
    })
}

/// p's turn, before combat: a combo that wins by attacking needs a combat still to come
fn precombat(g: &Game, p: PlayerId) -> bool {
    g.active == Some(p) && matches!(g.step, TurnStep::Start | TurnStep::Main1)
}

/// a payoff that drains on each death of a loop. Goblin Bombardment isn't one: it is an outlet (its damage comes
/// from sacrificing to it, which no loop here does), and sacrificing to it makes no mana to keep a loop going.
pub(crate) fn loop_payoff(g: &Game, p: PlayerId) -> Option<PermId> {
    let bomb = |m: PermId| g.perm(m).cd.is_some_and(|c| name_of(g, c) == "Goblin Bombardment");
    DRAIN_PAYOFFS.iter().filter_map(|n| on_bf(g, p, n)).find(|&m| !bomb(m)).or_else(|| {
        g.player(p).perms.iter().copied().find(|&m| {
            !bomb(m) && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Bartist) || g.db.get(c).tag(Tag::Drain))
        })
    })
}

/// unbounded mana of any kind wins: Walking Ballista (on the battlefield, or cast from hand for any X), Urza (his
/// {5} plays the whole library), or a tutor in hand that puts the Ballista in hand. Not Kinnan: his {5}{G}{U} needs
/// green and blue each time, and digging the library puts creatures onto the battlefield (a Ballista found that way
/// enters with no counters). Not Thassa's Oracle: mana doesn't empty the library.
pub(crate) fn mana_sink(g: &Game, p: PlayerId) -> bool {
    if any_bf(g, p, &["Walking Ballista", "Urza, Lord High Artificer"]).is_some()
        || in_hand(g, p, "Walking Ballista").is_some()
    {
        return true;
    }
    let ballista_in_library = g.player(p).library.iter().any(|&c| name_of(g, c) == "Walking Ballista");
    // Thrasios, Triton Hero turns unbounded mana into the whole library: each {4} puts the top card in hand (a land goes
    // onto the battlefield), so a Ballista in the library is found, then cast for any X (Rust only, and only JD's `thrasios` seat: the Tier 6 pool decks keep their lines)
    if ballista_in_library && g.player(p).key == "thrasios" && on_bf(g, p, "Thrasios, Triton Hero").is_some() {
        return true;
    }
    any_hand(g, p, &SINK_TUTORS).is_some() && ballista_in_library
}

fn can_pay_cost(g: &Game, p: PlayerId, c: CardId) -> bool {
    let (gn, pips) = cost_of(g, p, c);
    can_pay(g, p, gn, &pips, false)
}

/// combos.mana_need: the loop's own mana plus the pieces still to cast, in order, at what the engine will charge for them
/// (`seq_cost`: taxes such as Thorn of Amethyst, Defense Grid and Damping Sphere included), X included
pub fn mana_need(g: &Game, p: PlayerId, cmb: &Combo, casts: &[CardId]) -> (u32, String) {
    let (gn, pips) = seq_cost(g, p, casts);
    let x: u32 = casts.iter().map(|&c| cmb.x_for(name_of(g, c))).sum();
    (cmb.mana.0 + gn + x, format!("{}{pips}", cmb.mana.1))
}

/// the combo and its mana (the loop's own and the pieces' still to cast) can be paid for
fn affordable(g: &Game, p: PlayerId, cmb: &Combo, casts: &[CardId]) -> bool {
    let (gn, pips) = mana_need(g, p, cmb, casts);
    // --- ai-fix: C: with the rituals in hand cast first (each once; never a piece of the line, nor a card it casts)
    can_pay_with_rituals(g, p, gn, &pips, &line_cards(g, p, cmb, casts))
}

// --- ai-fix: C
/// the cards in p's hand a line uses: the pieces of its groups and the cards it casts (no ritual among them is spent
/// as mana for the line)
fn line_cards(g: &Game, p: PlayerId, cmb: &Combo, casts: &[CardId]) -> Vec<CardId> {
    let mut v: Vec<CardId> = casts.to_vec();
    v.extend(
        g.player(p).hand.iter().copied().filter(|&c| cmb.groups.iter().any(|grp| grp.contains(&name_of(g, c)))),
    );
    v
}

// --- ai-fix: C
/// before a line runs: cast the rituals it needs (`affordable` counted them), one at a time, until its mana can be paid
/// from what is open. (the mana is there, a ritual was cast)
fn cast_rituals_for(g: &mut Game, p: PlayerId, cmb: &Combo, casts: &[CardId]) -> Res<(bool, bool)> {
    let mut cast = false;
    loop {
        let (gn, pips) = mana_need(g, p, cmb, casts);
        if can_pay(g, p, gn, &pips, false) {
            return Ok((true, cast));
        }
        let skip = line_cards(g, p, cmb, casts);
        let Some(&r) = crate::engine::mana::rituals_in_hand(g, p, &skip).first() else { return Ok((false, cast)) };
        crate::glog!(g, "    {} casts {} for {}", g.player(p).name, name_of(g, r), cmb.name);
        if !crate::ai::brain::do_cast(g, p, r, None)? {
            return Ok((false, cast));
        }
        cast = true;
        if g.over {
            return Ok((false, cast));
        }
    }
}

// ------------------------------------------------------------------ execution
/// the index in COMBOS of the first combo whose name contains `part` (tests: Python's
/// `next(c for c in COMBOS if part in c.name)`)
pub fn combo_index(part: &str) -> Option<usize> {
    all().iter().position(|c| c.name.contains(part))
}

/// COMBOS[i].ready(g, p)[0]: p has everything COMBOS[i] needs
pub fn ready(g: &Game, p: PlayerId, i: usize) -> bool {
    (all()[i].ready)(g, p).0
}

/// combos.attempt: p goes for COMBOS[i]
pub fn attempt(g: &mut Game, p: PlayerId, i: i64) -> Res<bool> {
    if !crate::impls::t6_plans::is_t6(g, p) {
        return attempt_run(g, p, i);
    }
    // Tier 6: protection first (Silence / Orim's Chant), then every spell and ability of the line is a win step the
    // table answers with everything it has
    let cmb = all()[i as usize];
    let (ok, _, casts) = (cmb.ready)(g, p);
    let turns = g.player(p).turns;
    if ok && affordable(g, p, cmb, &casts) && g.player(p).combo_turn != Some((turns, cmb.name)) {
        let need = mana_need(g, p, cmb, &casts);
        crate::ai::t6_table::protect_before_go(g, p, need)?;
    }
    g.t6_going = Some(p);
    let r = attempt_run(g, p, i);
    g.t6_going = None;
    r
}

fn attempt_run(g: &mut Game, p: PlayerId, i: i64) -> Res<bool> {
    let cmb = all()[i as usize];
    let (ok, keys, casts) = (cmb.ready)(g, p);
    if !ok || !affordable(g, p, cmb, &casts) {
        return Ok(false);
    }
    let turns = g.player(p).turns;
    if g.player(p).combo_turn == Some((turns, cmb.name)) {
        return Ok(false);
    }
    // --- ai-fix: C: the rituals the line counted on are cast first; then the line is read again
    let (gn, pips) = mana_need(g, p, cmb, &casts);
    let (keys, casts) = if can_pay(g, p, gn, &pips, false) {
        (keys, casts)
    } else {
        let (paid, cast) = cast_rituals_for(g, p, cmb, &casts)?;
        if !paid {
            return Ok(cast); // a ritual cast (or countered) changed the turn state
        }
        let (ok, keys, casts) = (cmb.ready)(g, p);
        if !ok || !affordable(g, p, cmb, &casts) {
            return Ok(true);
        }
        (keys, casts)
    };
    let pl = g.player_mut(p);
    pl.combo_turn = Some((turns, cmb.name));
    pl.stat("combo_attempt", 1);
    pl.milestone.entry("combo").or_insert(turns);
    crate::t6stats::combo_go(g, p, cmb.name);
    g.last_counter = None; // unread by the game; the report names the card that stops a line
    crate::glog!(g, "  {} goes for {}", g.player(p).name, cmb.name);
    if !cmb.steps.is_empty() {
        return run_steps(g, p, cmb);
    }
    for (i, &c) in casts.iter().enumerate() {
        // the last pieces are cast normally: they can be countered. They cost what any cast costs (taxes included)
        let (gn, pips) = cost_of(g, p, c);
        if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") || !can_pay(g, p, gn, &pips, false) {
            if i > 0 {
                crate::glog!(g, "    ...{} fizzles ({} is gone)", cmb.name, name_of(g, c));
                g.player_mut(p).stat("combo_stopped", 1);
                crate::t6stats::combo_stop(g, p, cmb.name, "fizzled");
            }
            return Ok(i > 0); // something already happened: the turn state changed
        }
        let x = cmb.x_for(name_of(g, c));
        pay(g, p, gn + x, &pips, false)?;
        if x > 0 {
            g.last_x = x as i32;
        }
        g.combo_spell = true;
        let r = cast_card(g, p, c, "hand", Ctx::default());
        g.combo_spell = false;
        if !r? {
            crate::glog!(g, "    ...{} is countered: {} stopped", name_of(g, c), cmb.name);
            g.player_mut(p).stat("combo_stopped", 1);
            let by = crate::t6stats::last_counter_name(g);
            crate::t6stats::combo_stop(g, p, cmb.name, &by);
            return Ok(true);
        }
    }
    let (gn, pips) = cmb.mana;
    if gn > 0 || !pips.is_empty() {
        pay(g, p, gn, pips, false)?;
    }
    let keys: Vec<PermId> = keys.into_iter().filter(|&k| g.perm(k).on_bf).collect();
    if interrupted(g, p, &keys)? {
        crate::glog!(g, "    ...{} is stopped", cmb.name);
        g.player_mut(p).stat("combo_stopped", 1);
        let by = crate::t6stats::last_counter_name(g);
        crate::t6stats::combo_stop(g, p, cmb.name, &by);
        return Ok(true);
    }
    match cmb.finish {
        Some(f) => f(g, p)?,
        None => crate::ai::win(g, p, "combo", Some(!cmb.name.contains("Oracle")))?, // Oracle: an alternate win
    }
    if g.over && g.winner == Some(p) {
        crate::t6stats::combo_win(g, p, cmb.name);
    }
    Ok(true)
}

/// a combo with declared steps: each step is answerable in its own way (`StepKind`); an answered step stops the combo
/// (the pieces already played stay, and p may go for another line later), and once the last one passes the loop is
/// executed abstractly
pub fn run_steps(g: &mut Game, p: PlayerId, cmb: &Combo) -> Res<bool> {
    let (gn, pips) = cmb.mana;
    if gn > 0 || !pips.is_empty() {
        pay(g, p, gn, pips, false)?;
    }
    for (n, st) in cmb.steps.iter().enumerate() {
        g.last_counter = None;
        let ok = match st.kind {
            StepKind::Spell | StepKind::Engine => (st.run)(g, p)?,
            StepKind::Trigger | StepKind::Ability => {
                let (cg, cp) = st.pay;
                if !can_pay(g, p, cg, cp, false) {
                    false
                } else {
                    if cg > 0 || !cp.is_empty() {
                        pay(g, p, cg, cp, false)?;
                    }
                    let src = (st.src)(g, p);
                    let resolved = if st.kind == StepKind::Trigger {
                        trigger_window(g, p, src, st.label, Some(6.0))?
                    } else {
                        ability_window(g, p, src, st.label, Some(6.0), None)?
                    };
                    if resolved && !g.over { (st.run)(g, p)? } else { resolved }
                }
            }
            StepKind::Permanent => {
                if !(st.run)(g, p)? {
                    false
                } else {
                    let keys: Vec<PermId> = (st.keys)(g, p).into_iter().filter(|&k| g.perm(k).on_bf).collect();
                    !interrupted(g, p, &keys)?
                }
            }
        };
        if g.over {
            return Ok(true);
        }
        if !ok {
            crate::glog!(g, "    ...{} stops at step {} ({})", cmb.name, n + 1, st.label);
            g.player_mut(p).stat("combo_stopped", 1);
            let by = crate::t6stats::last_counter_name(g);
            crate::t6stats::combo_stop(g, p, cmb.name, &by);
            return Ok(true);
        }
    }
    match cmb.finish {
        Some(f) => f(g, p)?,
        None => crate::ai::win(g, p, "combo", Some(true))?,
    }
    if g.over && g.winner == Some(p) {
        crate::t6stats::combo_win(g, p, cmb.name);
    }
    Ok(true)
}

/// combos.interrupted: each opponent may answer one key piece with instant-speed removal (or a stifle effect)
/// before the loop runs
fn interrupted(g: &mut Game, p: PlayerId, keys: &[PermId]) -> Res<bool> {
    if keys.is_empty() {
        return Ok(false);
    }
    for q in g.after(p).collect::<Vec<_>>() {
        if !g.player(q).alive || q == p || g.over {
            continue;
        }
        if !castable_any(g, q) {
            continue;
        }
        if g.rng.random() > 0.95 {
            continue;
        }
        for c in g.player(q).hand.clone() {
            let d = g.db.get(c);
            let Some(rem) = d.tags.str(Tag::Rem) else { continue };
            if !(d.instant || d.tag(Tag::Flash)) || !castable(g, q, c, "hand") {
                continue;
            }
            let tgt = d.tags.str(Tag::Tgt).unwrap_or("c");
            let tg: Vec<PermId> = legal_targets(g, q, rem, tgt, d.tag(Tag::Mv4), Some(c))
                .into_iter()
                .filter(|m| keys.contains(m))
                .collect();
            if tg.is_empty() || !can_pay_cost(g, q, c) {
                continue;
            }
            let (gn, pips) = cost_of(g, q, c);
            pay(g, q, gn, &pips, false)?;
            cast_card(g, q, c, "hand", Ctx { target: Some(tg[0]), ..Ctx::default() })?;
            g.last_counter = Some(c);
            if !g.perm(tg[0]).on_bf || g.perm(tg[0]).phased {
                g.player_mut(q).stat("combo_stops_removal", 1);
                return Ok(true);
            }
            break;
        }
        // Tishana's Tidebinder / Siren Stormtamer style: counter the activated ability
        for c in g.player(q).hand.clone() {
            if name_of(g, c) == "Tishana's Tidebinder" && can_pay(g, q, 2, "U", false) && castable(g, q, c, "hand") {
                pay(g, q, 2, "U", false)?;
                let hand = &mut g.player_mut(q).hand;
                if let Some(i) = hand.iter().position(|&x| x == c) {
                    hand.remove(i);
                }
                enter(g, q, c, Enter::default())?;
                g.player_mut(q).stat("combo_stops_ability", 1);
                g.last_counter = Some(c);
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// combos.castable_any: may q cast an instant now (no lock forbids it)? Python asks the locks about a blank
/// instant with mana value 0; Pact of Negation (an instant with mana value 0 that no lock singles out) stands in.
fn castable_any(g: &Game, q: PlayerId) -> bool {
    if g.hooks.is_empty() {
        return true;
    }
    match g.db.id("Pact of Negation") {
        Some(c) => crate::engine::hooks::allowed(g, q, c, "hand"),
        None => true,
    }
}

/// combos.combo_options: go for a ready combo (an outside deck's; ctr_risk: the chance a spell in it is countered)
pub fn combo_options(g: &mut Game, p: PlayerId, ctr_risk: f64, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.player(p).t6 && post.is_some() {
        t6_measure(g, p); // Tier 6 speed measurement: stats only
    }
    let key = g.player(p).key;
    if post.is_none() || (crate::ai::is_main(key) && key != "karn") {
        return Ok(vec![]); // g.combo_decks: the outside decks (and the Karn deck: Basalt Monolith + Rings of Brighthearth)
    }
    let mut out = vec![];
    // the names of p's cards in hand and on the battlefield, built once: the same test as `has_piece` for every combo
    let held: std::collections::HashSet<&str> = {
        let pl = g.player(p);
        let mut s: std::collections::HashSet<&str> = pl.hand.iter().map(|&c| name_of(g, c)).collect();
        s.extend(pl.perms.iter().filter(|&&m| !g.perm(m).phased).filter_map(|&m| g.perm(m).cd).map(|c| name_of(g, c)));
        s
    };
    for (i, cmb) in all().iter().enumerate() {
        // a line that starts from the command zone or the library (Godo) has no piece in hand to gate on: ask `ready`
        if !cmb.groups.iter().any(|grp| grp.iter().any(|n| held.contains(n)))
            && (cmb.steps.is_empty() || !(cmb.ready)(g, p).0)
        {
            continue;
        }
        if g.player(p).combo_turn == Some((g.player(p).turns, cmb.name)) {
            continue; // already gone for this turn: `attempt_run` refuses a second try
        }
        let (ok, _, casts) = (cmb.ready)(g, p);
        if !ok || !affordable(g, p, cmb, &casts) {
            continue;
        }
        if crate::impls::t6_plans::is_t6(g, p) && !crate::ai::t6_table::go_ok(g, p, cmb.name) {
            continue; // Tier 6: the table can still stop it (or the plan slot says wait)
        }
        let risk = if casts.is_empty() { 0.0 } else { ctr_risk };
        out.push(Opt {
            utility: 14.0 - 6.0 * risk,
            label: format!("combo: {}", cmb.name),
            act: Some(Action::Plan { f: attempt, arg: i as i64 }),
        });
    }
    Ok(out)
}

// --- ai-fix: C
/// What ritual `r` is worth to p's catalogued lines: the utility of going for a line that is ready now, can't be paid
/// for from the open mana, and can once `r` is cast (`after`: p's mana units then), gated as `combo_options` gates its
/// offer (a piece held, not gone for this turn, the Tier 6 table's `go_ok`). None if no line needs it. A ritual that is
/// itself a piece of the line, or one of its casts, doesn't count.
pub fn line_enabled_by(g: &Game, p: PlayerId, r: CardId, after: &[crate::engine::mana::Unit]) -> Option<f64> {
    let key = g.player(p).key;
    if crate::ai::is_main(key) && key != "karn" {
        return None; // JD's decks run their lines from their own code (`combo_options` offers them none)
    }
    let rn = name_of(g, r);
    let held: std::collections::HashSet<&str> = {
        let pl = g.player(p);
        let mut s: std::collections::HashSet<&str> = pl.hand.iter().map(|&c| name_of(g, c)).collect();
        s.extend(pl.perms.iter().filter(|&&m| !g.perm(m).phased).filter_map(|&m| g.perm(m).cd).map(|c| name_of(g, c)));
        s
    };
    for cmb in all() {
        let piece_held = cmb.groups.iter().any(|grp| grp.iter().any(|n| held.contains(n)));
        if !piece_held || cmb.groups.iter().any(|grp| grp.contains(&rn)) {
            continue;
        }
        if g.player(p).combo_turn == Some((g.player(p).turns, cmb.name)) {
            continue;
        }
        let (ok, _, casts) = (cmb.ready)(g, p);
        if !ok || casts.contains(&r) {
            continue;
        }
        let (gn, pips) = mana_need(g, p, cmb, &casts);
        if can_pay(g, p, gn, &pips, false) || crate::engine::mana::plan_pay(g, after, gn, &pips).is_none() {
            continue;
        }
        if crate::impls::t6_plans::is_t6(g, p) && !crate::ai::t6_table::go_ok(g, p, cmb.name) {
            continue;
        }
        return Some(14.0);
    }
    None
}

/// any modeled combo is ready (Ad Nauseam stops there)
pub fn combo_ready(g: &Game, p: PlayerId) -> bool {
    all().iter().any(|cmb| (cmb.ready)(g, p).0)
}

/// combos.full_deck_names: the cards of p's deck in its library, hand, graveyard and on the battlefield (lands
/// aside)
fn full_deck_names(g: &Game, p: PlayerId) -> Vec<&str> {
    let pl = g.player(p);
    let mut out: Vec<&str> = pl.library.iter().chain(&pl.hand).chain(&pl.gy).map(|&c| name_of(g, c)).collect();
    out.extend(pl.perms.iter().filter_map(|&m| g.perm(m).cd).map(|c| name_of(g, c)));
    out
}

/// combos.missing_pieces: for tutors, the pieces of this player's closest combo that it doesn't have (any combo
/// whose pieces are in its deck; a combo with some pieces already in hand or play first)
pub fn missing_pieces(g: &Game, p: PlayerId) -> Vec<CardId> {
    // --- ai-fix: B: a line complete but for its mana wants the mana first
    let mana = mana_for_lines(g, p);
    if !mana.is_empty() {
        return mana;
    }
    let deck = full_deck_names(g, p);
    // --- ai-fix: B: a card p put on top of its library itself is on its way: not missing
    let top: Vec<&str> = crate::engine::tutors::known_top(g, p).into_iter().map(|c| name_of(g, c)).collect();
    let mut best: Option<(usize, Vec<&str>)> = None;
    for cmb in all() {
        if !cmb.groups.iter().all(|grp| grp.iter().any(|n| deck.contains(n))) {
            continue;
        }
        let miss: Vec<&[&str]> = cmb
            .groups
            .iter()
            .copied()
            .filter(|grp| !has_piece(g, p, grp) && !grp.iter().any(|n| top.contains(n)))
            .collect();
        if miss.is_empty() {
            continue;
        }
        let lib: Vec<&str> = g.player(p).library.iter().map(|&c| name_of(g, c)).collect();
        let want: Vec<&str> = miss.iter().flat_map(|grp| grp.iter().copied()).filter(|n| lib.contains(n)).collect();
        if want.is_empty() {
            continue;
        }
        if best.as_ref().is_none_or(|b| miss.len() < b.0) {
            best = Some((miss.len(), want));
        }
    }
    best.map_or(vec![], |b| b.1.iter().filter_map(|n| g.db.id(n)).collect())
}

// --- ai-fix: B
/// For tutors: the mana to fetch for a line of p's deck whose every piece p holds but which it can't pay for (rituals
/// in hand counted). First the library cards that make it payable this turn once in hand: a ritual or a noncreature
/// rock cast from the open mana, or a land while p's land drop is still to come. Then, when p's mana next turn
/// (`next_turn_mana`) falls short of the line, the cards that close the gap by then: a land if it holds none, a rock
/// that makes more than it costs. Empty when no line waits for mana, or no card helps.
pub(crate) fn mana_for_lines(g: &Game, p: PlayerId) -> Vec<CardId> {
    use crate::engine::mana::{
        land_card_unit, mana_units, next_turn_mana, ritual_out, units_after_cast, units_pay_with_rituals,
    };
    let pl = g.player(p);
    let u = mana_units(g, p, false);
    let drop_now = g.active == Some(p) && pl.land_turn != pl.turns as i32;
    let land_in_hand = pl.hand.iter().any(|&c| g.db.get(c).land);
    let next = next_turn_mana(g, p);
    let (mut now, mut later): (Vec<CardId>, Vec<CardId>) = (vec![], vec![]);
    for i in lines_of(g, p).iter().copied() {
        let cmb = all()[i];
        let (ok, _, casts) = (cmb.ready)(g, p);
        if !ok || affordable(g, p, cmb, &casts) {
            continue;
        }
        let (gn, pips) = mana_need(g, p, cmb, &casts);
        let total = gn + pips.chars().count() as u32;
        let skip = line_cards(g, p, cmb, &casts);
        let mut seen: Vec<&str> = vec![];
        for &c in &pl.library {
            let d = g.db.get(c);
            if seen.contains(&&*d.name) {
                continue;
            }
            seen.push(&*d.name);
            let rock = !d.creature && d.tags.str(Tag::Rock).is_some();
            if d.land {
                let mut v = u.clone();
                v.push(land_card_unit(g, p, c));
                if drop_now && units_pay_with_rituals(g, p, &v, gn, &pips, &skip) {
                    now.push(c);
                } else if !land_in_hand && next < total && next + land_card_unit(g, p, c).amt >= total {
                    later.push(c);
                }
            } else if rock || ritual_out(g, p, c).is_some() {
                let after = units_after_cast(g, p, c, &u);
                if after.is_some_and(|v| units_pay_with_rituals(g, p, &v, gn, &pips, &skip)) {
                    now.push(c);
                } else if rock && next < total {
                    let (cg, cp) = cost_of(g, p, c);
                    let out: u32 = d.tags.str(Tag::Rock).and_then(|r| r.split(':').next()?.parse().ok()).unwrap_or(1);
                    if next + out >= total + cg + cp.chars().count() as u32 {
                        later.push(c);
                    }
                }
            }
        }
    }
    for c in later {
        if !now.contains(&c) {
            now.push(c);
        }
    }
    now
}

/// combos.piece_threat: extra value of m if it's a piece of a combo whose other pieces its controller has on the
/// battlefield (7), or all but one of them in hand or play (2.5)
pub fn piece_threat(g: &Game, m: PermId) -> f64 {
    let Some(c) = g.perm(m).cd else { return 0.0 };
    let name = name_of(g, c);
    if !is_piece_name(name) {
        return 0.0;
    }
    let p = g.perm(m).owner;
    for cmb in all() {
        if quiet(cmb) || !cmb.groups.iter().any(|grp| grp.contains(&name)) {
            continue;
        }
        let others: Vec<&[&str]> = cmb.groups.iter().copied().filter(|grp| !grp.contains(&name)).collect();
        if others.iter().all(|grp| grp.iter().any(|n| on_bf(g, p, n).is_some())) {
            return 7.0;
        }
        if others.iter().filter(|grp| has_piece(g, p, grp)).count() + 1 >= others.len() {
            return 2.5;
        }
    }
    0.0
}

/// combos.combo_imp: importance of a spell for counterspell decisions: it completes (9) or nearly completes (7) a
/// combo
pub fn combo_imp(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let name = name_of(g, c);
    if !is_piece_name(name) {
        return 0.0;
    }
    for cmb in all() {
        if quiet(cmb) || !cmb.groups.iter().any(|grp| grp.contains(&name)) {
            continue;
        }
        let others: Vec<&[&str]> = cmb.groups.iter().copied().filter(|grp| !grp.contains(&name)).collect();
        if others.iter().all(|grp| grp.iter().any(|n| on_bf(g, p, n).is_some())) {
            return 9.0;
        }
        if others.iter().all(|grp| has_piece(g, p, grp)) {
            return 7.0;
        }
    }
    0.0
}

thread_local! {
    /// search._DECK_COMBOS: each deck's combos, by deck key (fixed per deck)
    static DECK_COMBOS: RefCell<HashMap<&'static str, Vec<usize>>> = RefCell::new(HashMap::new());
}

/// search.deck_combos: the modeled combos (indices in COMBOS) whose pieces are all in q's deck. Python reads the
/// deck from its zones the first time (`full_deck_names` and the commander); its whole list is the same thing.
pub fn deck_combos(g: &Game, q: PlayerId) -> Vec<usize> {
    let key = g.player(q).key;
    if let Some(v) = DECK_COMBOS.with(|m| m.borrow().get(key).cloned()) {
        return v;
    }
    let pl = g.player(q);
    let mut names: Vec<&str> = pl.deck_names.iter().map(|&c| name_of(g, c)).collect();
    names.extend(pl.commanders().map(|c| name_of(g, c))); // `| {q.cmd.name}`
    let v: Vec<usize> = all()
        .iter()
        .enumerate()
        .filter(|(_, c)| c.groups.iter().all(|grp| grp.iter().any(|n| names.contains(n))))
        .map(|x| x.0)
        .collect();
    DECK_COMBOS.with(|m| m.borrow_mut().insert(key, v.clone()));
    v
}

/// search.combo_progress: the share of q's closest combo that q holds (in hand, on the battlefield, or the
/// commander in the command zone), squared: the last pieces count most. 0 for a deck without a modeled combo.
pub fn combo_progress(g: &Game, q: PlayerId) -> f64 {
    // --- t6 quiet progress: a Tier 6 deck reads its quiet (stepped) lines too, by their real groups (`lines_of`), and
    // its land pieces on the battlefield (Gaea's Cradle). Other decks are unchanged (`deck_combos`). The generic
    // executor (`combo_options`) never reads this.
    let t6 = crate::impls::t6_plans::is_t6(g, q);
    let cmbs: Vec<usize> = if t6 { lines_of(g, q).to_vec() } else { deck_combos(g, q) };
    if cmbs.is_empty() {
        return 0.0;
    }
    let pl = g.player(q);
    let mut have: Vec<&str> = pl
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased)
        .filter_map(|&m| g.perm(m).cd)
        .chain(pl.hand.iter().copied())
        .map(|c| name_of(g, c))
        .collect();
    for c in pl.cmds_in_zone() {
        have.push(name_of(g, c));
    }
    if t6 {
        have.extend(pl.lands.iter().filter(|&&l| g.perm(l).on_bf).map(|&l| name_of(g, g.perm(l).card())));
    }
    let mut best = 0.0f64; // --- t6 quiet progress: (was -inf) every line skipped is 0, not inf
    for i in cmbs {
        let grps = real_groups(all()[i]); // the whole list for a line that is not quiet
        if quiet(all()[i]) && grps.len() < 2 {
            continue; // a quiet line of the commander alone (Dack Fayden) is always "held": no progress to read
        }
        let n = grps.iter().filter(|grp| grp.iter().any(|x| have.contains(x))).count();
        best = best.max(n as f64 / grps.len() as f64);
    }
    best * best
}

// ================================================================== the combos
/// infinite hasty copies: they win by attacking, so only before combat on p's turn (the tokens are sacrificed at
/// the end step)
fn kiki(g: &Game, p: PlayerId) -> Ready {
    if !precombat(g, p) {
        return NOT_READY();
    }
    let mut k = on_bf_with(g, p, KIKI, true, true);
    if k.is_none()
        && let Some(f) = on_bf_with(g, p, FABLE, true, false)
    {
        let d = &g.perm(f).data;
        let turns = g.player(p).turns as i64;
        if d.truthy(DataKey::Reflection) && d.get(DataKey::Flipped).map_or(turns, Val::int) < turns {
            k = Some(f);
        }
    }
    let Some(k) = k else { return NOT_READY() };
    if let Some(t) = any_bf(g, p, KIKI_TARGETS) {
        return (true, vec![k, t], vec![]);
    }
    if let Some(c) = any_hand(g, p, KIKI_TARGETS)
        && can_pay_cost(g, p, c)
    {
        return (true, vec![k], vec![c]);
    }
    NOT_READY()
}

/// infinite mana with 3+ mana from nonland permanents; wins with a mana sink (Walking Ballista, Urza)
fn scepter(g: &Game, p: PlayerId) -> Ready {
    let sc = on_bf(g, p, "Isochron Scepter");
    let mut casts = vec![];
    match sc {
        Some(m) => {
            let imprinted = match g.perm(m).data.get(DataKey::Imprint) {
                Some(Val::Str(s)) => *s == "Dramatic Reversal",
                Some(Val::Card(c)) => name_of(g, *c) == "Dramatic Reversal",
                _ => false,
            };
            if !imprinted {
                return NOT_READY();
            }
        }
        None => {
            // cast the Scepter now and imprint the Reversal from hand
            let (Some(c), Some(_)) = (in_hand(g, p, "Isochron Scepter"), in_hand(g, p, "Dramatic Reversal")) else {
                return NOT_READY();
            };
            casts.push(c);
        }
    }
    let generic: u32 = casts.iter().map(|&c| g.db.get(c).generic).sum();
    if nonland_mana(g, p) < 3 || !can_pay(g, p, 2 + generic, "U", false) {
        return NOT_READY();
    }
    if !mana_sink(g, p) {
        return NOT_READY();
    }
    (true, sc.into_iter().collect(), casts)
}

/// combos.nonland_mana: mana p's nonland permanents make, tapped or not (Dramatic Reversal untaps them all)
fn nonland_mana(g: &Game, p: PlayerId) -> i32 {
    use crate::engine::hooks::total_count;
    use crate::hooks::Event;
    let hooks = !g.hooks.is_empty();
    if hooks && total_count(g, Event::ManaLock, p) != 0 {
        return 0;
    }
    let mut n = 0;
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        let Some(c) = x.cd else { continue };
        if x.phased {
            continue;
        }
        let t = &g.db.get(c).tags;
        if let Some(rock) = t.str(Tag::Rock) {
            if hooks && total_count(g, Event::NoArtifactMana, p) != 0 {
                continue;
            }
            n += rock.split(':').next().unwrap_or("0").parse::<i32>().unwrap_or(0);
        } else if t.has(Tag::Dork) && !x.sick && !(hooks && total_count(g, Event::NoCreatureMana, p) != 0) {
            n += crate::cardcode::dyn_mana_perm(g, p, m).map_or(1, |v| v as i32);
        }
    }
    n
}

/// combos.SINK_TUTORS: tutors that put Walking Ballista into hand (not to the top of the library, like Enlightened
/// Tutor, and not an instant or sorcery, like Mystical Tutor and Spellseeker)
const SINK_TUTORS: [&str; 3] = ["Recruiter of the Guard", "Demonic Tutor", "Diabolic Intent"];

/// infinite colourless mana; wins with Urza / Walking Ballista
fn power_artifact(g: &Game, p: PlayerId) -> Ready {
    let pa = any_bf(g, p, &["Power Artifact", "Rings of Brighthearth"]);
    let mono = any_bf(g, p, &["Basalt Monolith", "Grim Monolith"]);
    let (Some(pa), Some(mono)) = (pa, mono) else { return NOT_READY() };
    let cd_name = |m: PermId| g.perm(m).cd.map_or("", |c| name_of(g, c));
    if cd_name(pa) == "Rings of Brighthearth" && cd_name(mono) != "Basalt Monolith" {
        return NOT_READY();
    }
    // the first loop (Rust only; Python started it with no mana): Basalt taps for {C}{C}{C}, {3} untaps it and Rings' copy
    // costs {2}, so the copy and the original untap it twice and it taps twice more: {6} for {5}, one more mana each loop. It
    // takes {5} in all (Basalt's own three, untapped, plus {2} from elsewhere; or {5} from elsewhere when it is tapped).
    // Power Artifact makes the untap {1}, so a tapped Monolith only needs {1}.
    if cd_name(mono) == "Basalt Monolith" {
        let need = if cd_name(pa) == "Rings of Brighthearth" { 5 } else { 1 };
        if (g.perm(mono).tapped || need == 5) && crate::engine::mana::total_mana(g, p, false) < need {
            return NOT_READY();
        }
    }
    if !mana_sink(g, p) {
        return NOT_READY();
    }
    (true, vec![pa, mono], vec![])
}

/// Kinnan's extra mana makes Basalt Monolith net one colourless per untap: infinite colourless mana (a tapped
/// Monolith needs {3} to start); wins with Urza / Walking Ballista
fn kinnan_basalt(g: &Game, p: PlayerId) -> Ready {
    let (Some(k), Some(b)) = (on_bf(g, p, "Kinnan, Bonder Prodigy"), on_bf(g, p, "Basalt Monolith")) else {
        return NOT_READY();
    };
    if g.perm(b).tapped && !can_pay(g, p, 3, "", false) || !mana_sink(g, p) {
        return NOT_READY();
    }
    (true, vec![k, b], vec![])
}

/// the aura goes on a mana creature that makes blue: two mana with Kinnan, {U} to untap it, so infinite mana; wins
/// with Urza / Walking Ballista
fn kinnan_aura(g: &Game, p: PlayerId) -> Ready {
    let (Some(k), Some(a)) =
        (on_bf(g, p, "Kinnan, Bonder Prodigy"), any_bf(g, p, &["Freed from the Real", "Pemmin's Aura"]))
    else {
        return NOT_READY();
    };
    let dork = g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        !x.phased
            && g.is_creature(m)
            && (!x.sick || crate::dsl::has_kw(g, m, "haste"))
            && x.cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Dork).is_some_and(|s| s.contains(['U', 'A'])))
    });
    let Some(d) = dork else { return NOT_READY() };
    if !mana_sink(g, p) {
        return NOT_READY();
    }
    (true, vec![k, a, d], vec![])
}

/// free creatures bounce themselves: infinite draws, land drops and ETBs
fn aluren(g: &Game, p: PlayerId) -> Ready {
    let (Some(a), Some(ch)) = (on_bf(g, p, "Aluren"), on_bf(g, p, "Chulane, Teller of Tales")) else {
        return NOT_READY();
    };
    if any_bf(g, p, BOUNCERS).is_some() {
        return (true, vec![a, ch], vec![]);
    }
    if let Some(c) = any_hand(g, p, BOUNCERS) {
        return (true, vec![a, ch], vec![c]);
    }
    NOT_READY()
}

/// infinite drain on any life change
fn bond(g: &Game, p: PlayerId) -> Ready {
    let a = on_bf(g, p, "Sanguine Bond");
    let b = on_bf(g, p, "Exquisite Blood");
    if let (Some(a), Some(b)) = (a, b) {
        return (true, vec![a, b], vec![]);
    }
    if let Some(a) = a
        && let Some(c) = in_hand(g, p, "Exquisite Blood")
    {
        return (true, vec![a], vec![c]);
    }
    if let Some(b) = b
        && let Some(c) = in_hand(g, p, "Sanguine Bond")
    {
        return (true, vec![b], vec![c]);
    }
    NOT_READY()
}

/// exile the library, then Oracle wins; both are spells (counterable)
fn oracle(g: &Game, p: PlayerId) -> Ready {
    let o = in_hand(g, p, "Thassa's Oracle");
    let t = any_hand(g, p, &["Demonic Consultation", "Tainted Pact"]);
    let (Some(o), Some(t)) = (o, t) else { return NOT_READY() };
    if name_of(g, t) == "Tainted Pact" {
        let lib = &g.player(p).library;
        let mut distinct: Vec<CardId> = lib.clone();
        distinct.sort_unstable();
        distinct.dedup();
        if distinct.len() < lib.len() {
            return NOT_READY(); // Pact stops at the first duplicate name (two Islands): no win
        }
    }
    // the Oracle first, the exile spell with its trigger on the stack: and Oracle's {U}{U} is paid before the {1}
    (true, vec![], vec![o, t])
}

/// combos.oracle_devotion: p's devotion to blue
fn oracle_devotion(g: &Game, p: PlayerId) -> usize {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased)
        .filter_map(|&m| g.perm(m).cd)
        .filter(|&c| g.db.get(c).perm)
        .map(|c| g.db.get(c).pips.matches('U').count())
        .sum()
}

/// ETB: look at the top X (devotion to blue), win if X is at least the library size; otherwise keep the best one on
/// top. Cast alone only when that wins, otherwise held for Demonic Consultation / Tainted Pact.
fn oracle_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || g.combo_spell {
        return Ok(()); // in the combo the exile spell comes next
    }
    let x = oracle_devotion(g, p);
    if x >= g.player(p).library.len() {
        // the win is a triggered ability on the stack (importance 9): the table can counter it
        if !crate::engine::stack::trigger_window(
            g,
            p,
            Some(src),
            "look at the top cards: win with an empty library",
            Some(9.0),
        )? {
            return Ok(());
        }
        return crate::ai::win(g, p, "combo", Some(false));
    }
    let lib = &mut g.player_mut(p).library;
    let k = x.min(lib.len());
    let mut top: Vec<CardId> = (0..k).map(|_| lib.pop().unwrap()).collect();
    if let Some(best) = max_by(&top, |c| card_worth(g, p, c, false)) {
        let i = top.iter().position(|&c| c == best).unwrap();
        top.remove(i);
        g.rng.shuffle(&mut top);
        let lib = &mut g.player_mut(p).library;
        lib.splice(0..0, top);
        lib.push(best);
    }
    Ok(())
}

/// Thassa's Oracle: cast when it wins on its own
fn oracle_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if oracle_devotion(g, p) + 2 >= g.player(p).library.len() { 85 } else { 0 }
}

/// a fresh Celebrant each combat: infinite combats
fn helm(g: &Game, p: PlayerId) -> Ready {
    let (Some(h), Some(c)) = (on_bf(g, p, "Helm of the Host"), on_bf(g, p, "Combat Celebrant")) else {
        return NOT_READY();
    };
    if g.perm(h).attached != Some(c) || g.active != Some(p) || g.player(p).combat_no > 0 {
        return NOT_READY();
    }
    if !can_pay(g, p, 0, "", false) {
        return NOT_READY();
    }
    (true, vec![c], vec![])
}

/// infinite death triggers
fn gravecrawler(g: &Game, p: PlayerId) -> Ready {
    let Some(altar) = on_bf(g, p, "Phyrexian Altar") else { return NOT_READY() };
    // on the battlefield the Altar pays for the first recast; from the graveyard it needs its own {B}
    let gc = on_bf(g, p, "Gravecrawler").is_some()
        || g.player(p).gy.iter().any(|&c| name_of(g, c) == "Gravecrawler") && can_pay(g, p, 0, "B", false);
    if !gc {
        return NOT_READY();
    }
    let zombies = g
        .player(p)
        .perms
        .iter()
        .any(|&m| has_type(g, m, "zombie") && !g.perm(m).cd.is_some_and(|c| name_of(g, c) == "Gravecrawler"));
    let Some(payoff) = loop_payoff(g, p) else { return NOT_READY() };
    if !zombies {
        return NOT_READY();
    }
    (true, vec![altar, payoff], vec![])
}

/// combos._yawg_payoff: a death payoff for the Yawgmoth loop; Geralf's Messenger in the loop is its own (each
/// return drains 2)
fn yawg_payoff(g: &Game, p: PlayerId) -> Option<PermId> {
    loop_payoff(g, p).or_else(|| on_bf(g, p, "Geralf's Messenger"))
}

/// the loop with a death payoff wins; without one it kills every opposing creature and draws ten
fn yawg_finish(g: &mut Game, p: PlayerId) -> Res {
    if yawg_payoff(g, p).is_some() {
        return crate::ai::win(g, p, "combo", None);
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        for m in g.player(q).perms.clone() {
            if g.is_creature(m) && !untargetable(g, m) {
                g.perm_mut(m).plus -= 99;
                die(g, m, "sba")?;
            }
        }
    }
    let n = 10.min(g.player(p).library.len().saturating_sub(5)) as u32;
    draw(g, p, n, false)?;
    lose_life(g, p, 5, Some(p), "other", None)?;
    let pl = g.player_mut(p);
    pl.yawg_loop = Some(pl.turns);
    pl.combo_turn = None; // a payoff drawn by the loop lets it go again this turn (ready() stops a second payoff-less loop)
    crate::glog!(g, "    Yawgmoth loop: opposing creatures die, {} draws", g.player(p).name);
    Ok(())
}

/// sacrifice / -1/-1 counter loop: draws and drains (without a payoff: wipes their creatures, draws ten)
fn yawg(g: &Game, p: PlayerId) -> Ready {
    let Some(y) = on_bf(g, p, "Yawgmoth, Thran Physician") else { return NOT_READY() };
    let pl = g.player(p);
    if pl.life < 12 {
        return NOT_READY();
    }
    if pl.yawg_loop == Some(pl.turns) && yawg_payoff(g, p).is_none() {
        return NOT_READY(); // looped already; a payoff drawn since wins
    }
    let und = pl
        .perms
        .iter()
        .filter(|&&m| g.is_creature(m) && m != y && g.perm(m).cd.is_some_and(|c| g.db.get(c).has_kw("undying")))
        .count();
    let mik = on_bf(g, p, "Mikaeus, the Unhallowed");
    let fodder =
        pl.perms.iter().filter(|&&m| g.is_creature(m) && m != y && Some(m) != mik && !has_type(g, m, "human")).count();
    let looping = und >= 2 || (mik.is_some() && fodder >= 1);
    if !looping {
        return NOT_READY();
    }
    if yawg_payoff(g, p).is_none() && !g.opps(p).any(|q| g.player(q).perms.iter().any(|&m| g.is_creature(m))) {
        return NOT_READY();
    }
    let mut keys = vec![y];
    keys.extend(mik);
    (true, keys, vec![])
}

/// each Goblin sacrificed untaps Krenko: infinite Goblins (summoning sick). They win through an outlet that makes
/// mana for the Staff's {2} ping or deals damage itself, or through a death payoff; Viscera Seer alone only scries.
fn staff(g: &Game, p: PlayerId) -> Ready {
    let k = on_bf_with(g, p, "Krenko, Mob Boss", true, true);
    let st = on_bf(g, p, "Thornbite Staff");
    let (Some(k), Some(st)) = (k, st) else { return NOT_READY() };
    let wins = free_outlet(g, p, Some(STAFF_OUTLETS)).is_some()
        || free_outlet(g, p, None).is_some() && loop_payoff(g, p).is_some();
    if g.perm(st).attached != Some(k) || !wins {
        return NOT_READY();
    }
    (true, vec![k], vec![])
}

/// lock: every permanent is an artifact and opponents can't activate artifacts (their lands)
fn lock_lattice(g: &mut Game, p: PlayerId) -> Res {
    g.lattice_lock = Some(p);
    crate::glog!(g, "    Karn + Mycosynth Lattice: opponents can't use mana from permanents");
    Ok(())
}

fn karn(g: &Game, p: PlayerId) -> Ready {
    if g.lattice_lock == Some(p) {
        return NOT_READY();
    }
    let Some(k) = on_bf(g, p, "Karn, the Great Creator") else { return NOT_READY() };
    if let Some(lat) = on_bf(g, p, "Mycosynth Lattice") {
        return (true, vec![k, lat], vec![]);
    }
    if let Some(c) = in_hand(g, p, "Mycosynth Lattice") {
        return (true, vec![k], vec![c]);
    }
    NOT_READY()
}

/// Heliod gives the Ballista lifelink ({1}{W}); each ping gains life, which adds a counter: unlimited pings
fn heliod(g: &Game, p: PlayerId) -> Ready {
    if repl::solemnity(g) {
        return NOT_READY(); // --- Tiers 4/5 (heliod-zur): Solemnity: the Ballista gets no counters
    }
    let Some(h) = on_bf(g, p, "Heliod, Sun-Crowned") else { return NOT_READY() };
    let b = g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.cd.is_some_and(|c| name_of(g, c) == "Walking Ballista") && x.plus > 0 && !x.phased
    });
    if let Some(b) = b {
        return (true, vec![h, b], vec![]);
    }
    if let Some(c) = in_hand(g, p, "Walking Ballista") {
        return (true, vec![h], vec![c]);
    }
    NOT_READY()
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let oracle = r.card(db, "Thassa's Oracle")?;
    oracle.etb = Some(oracle_etb);
    oracle.prio = Some(oracle_prio);
    *oracle = oracle.at_once(crate::hooks::Event::Etb); // no trigger window: it runs as it enters
    Ok(())
}

// ------------------------------------------------------------------ Tier 6 table reading (ai/t6_table.rs)
/// a line's real piece groups (a quiet line's first group is the `QUIET` marker, not a card)
fn real_groups(c: &Combo) -> &[&'static [&'static str]] {
    if quiet(c) { &c.groups[1..] } else { &c.groups[..] }
}

/// a piece the table can see is live: on q's battlefield, or one of q's commanders in the command zone that q can
/// cast now from its open mana (a commander is public, but a line of one commander alone is not "ready" while it
/// can't be cast)
fn public_piece(g: &Game, q: PlayerId, grp: &[&str]) -> bool {
    let pl = g.player(q);
    grp.iter().any(|n| {
        on_bf(g, q, n).is_some()
            || pl.commanders().any(|c| {
                pl.cmd_zone_of(c) && name_of(g, c) == *n && {
                    let (gn, pips) = cost_of(g, q, c);
                    can_pay(g, q, gn + pl.tax_of(c), &pips, false)
                }
            })
    })
}

/// How far q's best modeled line is, read from public information only (opponents can't see q's hand): (the largest
/// fraction of its piece groups on the battlefield, whether some line has every group public). Only lines whose every
/// group is somewhere in q's cards count; quiet lines count too (this is the Tier 6 reading, which needs them).
pub fn line_progress(g: &Game, q: PlayerId) -> (f64, bool) {
    let (mut best, mut ready) = (0.0f64, false);
    for i in lines_of(g, q).iter().copied() {
        let grps = real_groups(all()[i]);
        let on = grps.iter().filter(|grp| grp.iter().any(|n| on_bf(g, q, n).is_some())).count();
        best = best.max(on as f64 / grps.len() as f64);
        ready |= grps.iter().all(|grp| public_piece(g, q, grp));
    }
    (best, ready)
}

/// How far the card named `name` would take q's best line if it resolved now: the largest fraction of a line's piece
/// groups that would be on the battlefield (or a castable commander) with it, over the lines whose groups include it
/// (quiet lines too). 0.0 when it is no piece of q's lines. Public information only: q's hand is not read.
pub fn piece_gain(g: &Game, q: PlayerId, name: &str) -> f64 {
    let mut best = 0.0f64;
    for i in lines_of(g, q).iter().copied() {
        let grps = real_groups(all()[i]);
        if !grps.iter().any(|grp| grp.contains(&name)) {
            continue;
        }
        let on = grps.iter().filter(|grp| grp.contains(&name) || public_piece(g, q, grp)).count();
        best = best.max(on as f64 / grps.len() as f64);
    }
    best
}

/// the indices in `all()` of the lines whose every piece group has a card in q's deck (or commanders), cached per
/// deck key
fn lines_of(g: &Game, q: PlayerId) -> std::rc::Rc<Vec<usize>> {
    thread_local! {
        static CACHE: RefCell<HashMap<(&'static str, usize), std::rc::Rc<Vec<usize>>>> = RefCell::new(HashMap::new());
    }
    let pl = g.player(q);
    let key = (pl.key, pl.deck_names.len());
    if let Some(v) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return v;
    }
    let mut deck: std::collections::HashSet<&str> = pl.deck_names.iter().map(|&c| name_of(g, c)).collect();
    deck.extend(pl.commanders().map(|c| name_of(g, c)));
    let v: Vec<usize> = all()
        .iter()
        .enumerate()
        .filter(|(_, cmb)| {
            !real_groups(cmb).is_empty() && real_groups(cmb).iter().all(|grp| grp.iter().any(|n| deck.contains(n)))
        })
        .map(|(i, _)| i)
        .collect();
    let v = std::rc::Rc::new(v);
    CACHE.with(|c| c.borrow_mut().insert(key, v.clone()));
    v
}

// --- phase4-rules
/// the piece groups of each of q's catalogued lines (`lines_of`, quiet markers left out) that has a group naming this
/// card (the combo guard, `ai::rules::combo_ok`)
pub fn deck_lines_with(g: &Game, q: PlayerId, name: &str) -> Vec<&'static [&'static [&'static str]]> {
    lines_of(g, q)
        .iter()
        .map(|&i| real_groups(all()[i]))
        .filter(|grps| grps.iter().any(|grp| grp.contains(&name)))
        .collect()
}

/// how many answerable steps the line has: its declared steps that are not plain permanents, else its spells to cast
pub fn answerable_steps(cmb: &Combo, casts: &[CardId]) -> usize {
    if cmb.steps.is_empty() {
        casts.len().max(1)
    } else {
        cmb.steps.iter().filter(|s| s.kind != StepKind::Engine).count().max(1)
    }
}

// ------------------------------------------------------------------ windows for lines played out by normal actions
/// A Tier 6 player's action with `acting` (an equip, an activation, a trigger such as Helm of the Host's) that works on a
/// piece of one of its modeled lines whose every piece group is already on the battlefield is a win step like any step
/// of `attempt`: each opponent may answer a piece of the line with instant-speed removal or bounce. Normal play never
/// goes through `attempt`, so without this nobody could answer Godo + Helm of the Host played out turn by turn.
/// True: the line was stopped, the action does not happen. Not used while the player is in `attempt` (its steps have
/// their own windows).
pub fn normal_play_window(g: &mut Game, p: PlayerId, acting: PermId) -> Res<bool> {
    if g.over || g.trig_probe || g.t6_going == Some(p) || !crate::impls::t6_plans::is_t6(g, p) || !g.perm(acting).on_bf {
        return Ok(false);
    }
    let Some(cd) = g.perm(acting).cd else { return Ok(false) };
    let an = name_of(g, cd).to_string();
    let mut found: Option<(&'static str, Vec<PermId>)> = None;
    for i in lines_of(g, p).iter().copied() {
        let grps = real_groups(all()[i]);
        if !grps.iter().any(|grp| grp.contains(&an.as_str())) {
            continue;
        }
        let keys: Vec<PermId> = grps.iter().filter_map(|grp| grp.iter().find_map(|n| on_bf(g, p, n))).collect();
        if keys.len() == grps.len() {
            found = Some((all()[i].name, keys));
            break;
        }
    }
    let Some((line, keys)) = found else { return Ok(false) };
    g.last_counter = None;
    if interrupted(g, p, &keys)? {
        crate::glog!(g, "    ...{} is stopped", line);
        g.player_mut(p).stat("combo_stopped", 1);
        let by = crate::t6stats::last_counter_name(g);
        crate::t6stats::combo_stop(g, p, line, &by);
        return Ok(true);
    }
    Ok(false)
}

// ------------------------------------------------------------------ Tier 6 speed measurement (stats only)
/// For the Tier 6 speed project: at each main-phase decision of a Tier 6 player, note the first round one of its deck's
/// lines was ready (every piece in hand or in play and the mana to run it, now) and the first round one was a tutor
/// away (exactly one piece group missing, a piece of it in the library, a tutor in hand, mana for tutor + that piece +
/// the line). Hidden information is used: this is a measurement. Writes only `sp|` stats; no decisions, no RNG.
fn t6_measure(g: &mut Game, p: PlayerId) {
    use crate::t6stats::{speed_flag, speed_mark};
    let st = |g: &Game, k: &str| g.player(p).stats.contains_key(&crate::sym::intern(k));
    let (has_ready, gone) = (st(g, "sp|ready"), st(g, "sp|go"));
    if has_ready && gone {
        return;
    }
    let mut any_ready = false;
    let mut reach = false; // one tutor away now (checked at every decision, for `sp|away_now`)
    // --- t6 rograkh-thrasios: `lines_of` (real groups), not `deck_combos`, whose `QUIET` group is in no deck
    let mine = lines_of(g, p);
    let held: std::collections::HashSet<&str> = {
        let pl = g.player(p);
        let mut s: std::collections::HashSet<&str> = pl.hand.iter().map(|&c| name_of(g, c)).collect();
        s.extend(pl.perms.iter().filter(|&&m| !g.perm(m).phased).filter_map(|&m| g.perm(m).cd).map(|c| name_of(g, c)));
        s
    };
    for (i, &cmb) in all().iter().enumerate() {
        // the same gate as `combo_options` (a piece held, or a stepped line), plus the lines of p's deck
        let in_deck = mine.contains(&i);
        if !in_deck && !cmb.groups.iter().any(|grp| grp.iter().any(|n| held.contains(n))) && cmb.steps.is_empty() {
            continue;
        }
        let (ok, _, casts) = (cmb.ready)(g, p);
        if ok && affordable(g, p, cmb, &casts) {
            any_ready = true;
            break;
        }
        if in_deck && !reach {
            reach = one_tutor_away(g, p, cmb);
        }
    }
    speed_flag(g, p, any_ready && !gone);
    crate::t6stats::away_flag(g, p, reach && !any_ready && !gone);
    if any_ready {
        speed_mark(g, p, "ready");
        speed_mark(g, p, "reach");
    } else if reach {
        speed_mark(g, p, "reach");
    }
}

// --- ai-audit: read-only accessors for the goldfish loss accounting (gf_audit.rs), stats only
/// the first line that is ready and affordable for p now, by the same test as `t6_measure`
pub fn audit_ready(g: &Game, p: PlayerId) -> Option<&'static str> {
    let mine = lines_of(g, p);
    let held: std::collections::HashSet<&str> = {
        let pl = g.player(p);
        let mut s: std::collections::HashSet<&str> = pl.hand.iter().map(|&c| name_of(g, c)).collect();
        s.extend(pl.perms.iter().filter(|&&m| !g.perm(m).phased).filter_map(|&m| g.perm(m).cd).map(|c| name_of(g, c)));
        s
    };
    for (i, &cmb) in all().iter().enumerate() {
        if !mine.contains(&i) && !cmb.groups.iter().any(|grp| grp.iter().any(|n| held.contains(n))) && cmb.steps.is_empty()
        {
            continue;
        }
        let (ok, _, casts) = (cmb.ready)(g, p);
        if ok && affordable(g, p, cmb, &casts) {
            return Some(cmb.name);
        }
    }
    None
}

// --- ai-fix: D
/// every line ready and affordable for p now (the test `audit_ready` makes), each with its mana need and the cards in
/// hand it uses (its pieces and casts): what a forced play must leave payable
pub fn ready_lines(g: &Game, p: PlayerId) -> Vec<((u32, String), Vec<CardId>)> {
    let mine = lines_of(g, p);
    let held: std::collections::HashSet<&str> = {
        let pl = g.player(p);
        let mut s: std::collections::HashSet<&str> = pl.hand.iter().map(|&c| name_of(g, c)).collect();
        s.extend(pl.perms.iter().filter(|&&m| !g.perm(m).phased).filter_map(|&m| g.perm(m).cd).map(|c| name_of(g, c)));
        s
    };
    let mut out = vec![];
    for (i, &cmb) in all().iter().enumerate() {
        let piece_held = cmb.groups.iter().any(|grp| grp.iter().any(|n| held.contains(n)));
        if !mine.contains(&i) && !piece_held && cmb.steps.is_empty() {
            continue;
        }
        let (ok, _, casts) = (cmb.ready)(g, p);
        if ok && affordable(g, p, cmb, &casts) {
            out.push((mana_need(g, p, cmb, &casts), line_cards(g, p, cmb, &casts)));
        }
    }
    out
}

/// the catalogued lines of p's deck (indices into `all()`)
pub fn audit_lines(g: &Game, p: PlayerId) -> Vec<usize> {
    lines_of(g, p).to_vec()
}

/// a line's piece groups (a quiet line's marker left out)
pub fn audit_groups(i: usize) -> &'static [&'static [&'static str]] {
    real_groups(all()[i])
}

/// a line's own loop mana
pub fn audit_mana(i: usize) -> u32 {
    all()[i].mana.0
}

/// --- ai-planner: a line's own loop mana, its coloured part
pub fn audit_loop_pips(i: usize) -> &'static str {
    all()[i].mana.1
}

pub fn audit_name(i: usize) -> &'static str {
    all()[i].name
}

// --- ai-fix: G
/// The cards in p's hand to keep as pieces of p's own catalogued lines (`lines_of`), each with the share of its
/// line's piece groups p holds (the closest line's pieces rank first). A group already held on the battlefield, among
/// p's lands or as a commander in the command zone needs nothing from the hand; otherwise its cheapest card in hand is
/// kept, and a second card of the same group is spare.
pub fn line_keepers(g: &Game, p: PlayerId) -> Vec<(CardId, f64)> {
    let pl = g.player(p);
    let mut out_names: std::collections::HashSet<&str> =
        pl.perms.iter().filter(|&&m| !g.perm(m).phased).filter_map(|&m| g.perm(m).cd).map(|c| name_of(g, c)).collect();
    out_names.extend(pl.lands.iter().filter(|&&l| g.perm(l).on_bf).map(|&l| name_of(g, g.perm(l).card())));
    out_names.extend(pl.cmds_in_zone().map(|c| name_of(g, c)));
    let in_hand = |grp: &[&str]| -> Option<CardId> {
        pl.hand.iter().copied().filter(|&c| grp.contains(&name_of(g, c))).min_by_key(|&c| g.db.get(c).cmc)
    };
    let mut keep: Vec<(CardId, f64)> = vec![];
    for i in lines_of(g, p).iter().copied() {
        let grps = real_groups(all()[i]);
        let out = |grp: &[&str]| grp.iter().any(|n| out_names.contains(n));
        let held = grps.iter().filter(|grp| out(grp) || in_hand(grp).is_some()).count();
        let share = held as f64 / grps.len() as f64;
        for grp in grps.iter().filter(|grp| !out(grp)) {
            let Some(c) = in_hand(grp) else { continue };
            match keep.iter_mut().find(|x| x.0 == c) {
                Some(x) => x.1 = x.1.max(share),
                None => keep.push((c, share)),
            }
        }
    }
    keep
}

/// is `cmb` one tutor away for p (see `t6_measure`)?
fn one_tutor_away(g: &Game, p: PlayerId, cmb: &Combo) -> bool {
    let pl = g.player(p);
    let cmd_in_zone: Vec<&str> = pl.cmds_in_zone().map(|c| name_of(g, c)).collect();
    // --- t6 rograkh-thrasios: the real groups (a quiet line's `QUIET` marker is never held, so no quiet line was
    // ever one tutor away), and a land piece on the battlefield is held (lands are not on `perms`: Gaea's Cradle)
    let land_out = |grp: &[&str]| {
        pl.lands.iter().any(|&l| g.perm(l).on_bf && grp.contains(&name_of(g, g.perm(l).card())))
    };
    let miss: Vec<&[&str]> = real_groups(cmb)
        .iter()
        .copied()
        .filter(|grp| !has_piece(g, p, grp) && !land_out(grp) && !grp.iter().any(|n| cmd_in_zone.contains(n)))
        .collect();
    if miss.len() != 1 {
        return false;
    }
    let piece: Vec<CardId> =
        pl.library.iter().copied().filter(|&c| miss[0].contains(&name_of(g, c))).collect();
    if piece.is_empty() {
        return false;
    }
    let pc = piece.iter().map(|&c| cost_of(g, p, c)).min_by_key(|(gn, pips)| *gn + pips.len() as u32).unwrap();
    for &t in &pl.hand {
        let d = g.db.get(t);
        if d.land || !(d.tags.has(Tag::Tut) || d.tags.has(Tag::Xtutor)) {
            continue;
        }
        let tc = cost_of(g, p, t);
        if can_pay(g, p, tc.0 + pc.0 + cmb.mana.0, &format!("{}{}{}", tc.1, pc.1, cmb.mana.1), false) {
            return true;
        }
    }
    false
}

// --- ai-planner: read-only accessors for the line planner (`ai::planner`)
/// The lines p could go for now, as `combo_options` offers them: a piece held (or a stepped line that is ready), not
/// gone for this turn, ready and affordable (rituals in hand counted), and at Tier 6 cleared by the table and the plan
/// slot (`go_ok`) when `table` is true. Each comes with the cards in p's hand it uses (its pieces and casts).
pub fn plan_lines(g: &Game, p: PlayerId, table: bool) -> Vec<(usize, Vec<CardId>)> {
    let held: std::collections::HashSet<&str> = {
        let pl = g.player(p);
        let mut s: std::collections::HashSet<&str> = pl.hand.iter().map(|&c| name_of(g, c)).collect();
        s.extend(pl.perms.iter().filter(|&&m| !g.perm(m).phased).filter_map(|&m| g.perm(m).cd).map(|c| name_of(g, c)));
        s
    };
    let t6 = crate::impls::t6_plans::is_t6(g, p);
    let mut out = vec![];
    for (i, cmb) in all().iter().enumerate() {
        if !cmb.groups.iter().any(|grp| grp.iter().any(|n| held.contains(n)))
            && (cmb.steps.is_empty() || !(cmb.ready)(g, p).0)
        {
            continue;
        }
        if g.player(p).combo_turn == Some((g.player(p).turns, cmb.name)) {
            continue;
        }
        let (ok, _, casts) = (cmb.ready)(g, p);
        if !ok || !affordable(g, p, cmb, &casts) {
            continue;
        }
        if table && t6 && !crate::ai::t6_table::go_ok(g, p, cmb.name) {
            continue;
        }
        out.push((i, line_cards(g, p, cmb, &casts)));
    }
    out
}

/// is a card of this piece group held by p: on the battlefield, in hand, among its lands, or a commander in the
/// command zone (the line's own `ready` decides whether it can be used)
pub fn group_held(g: &Game, p: PlayerId, grp: &[&str]) -> bool {
    has_piece(g, p, grp) || g.player(p).cmds_in_zone().any(|c| grp.contains(&name_of(g, c)))
}

/// is a card of this piece group on p's battlefield (a permanent, or a land)
pub fn group_out(g: &Game, p: PlayerId, grp: &[&str]) -> bool {
    grp.iter().any(|n| on_bf(g, p, n).is_some())
        || g.player(p).lands.iter().any(|&l| g.perm(l).on_bf && grp.contains(&name_of(g, g.perm(l).card())))
}

/// the mana cards a tutor should fetch for a line complete but for its mana (`mana_for_lines`)
pub fn plan_mana_targets(g: &Game, p: PlayerId) -> Vec<CardId> {
    mana_for_lines(g, p)
}
