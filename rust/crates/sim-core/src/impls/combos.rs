//! Python's `cards/impl/combos.py`: combos for the opponent pools. A combo is assembled from pieces; when a player
//! has everything it needs, the AI goes for it: any spell still to be cast goes through the normal counter window,
//! then each opponent gets one chance to answer a key piece with instant-speed removal (or with a counter / ability
//! counter where noted). If nothing stops it, the loop is executed abstractly: the player wins (or, for locks, the
//! lock applies).
//!
//! Opponents value the pieces of a combo that is one step from completion much higher (removal, counterspells), and
//! combo decks get tutor wish lists for their missing pieces. search.deck_combos and search.combo_progress (the
//! look-ahead's reading of a combo in the making) are here too.

use crate::ai::plans::{DRAIN_PAYOFFS, drain_payoff};
use crate::cards::CardDb;
use crate::engine::cast::{cast_card, castable};
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::removal::legal_targets;
use crate::engine::tutors::card_worth;
use crate::engine::values::{has_type, untargetable};
use crate::engine::zones::{Enter, die, draw, enter, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::tag::Tag;
use std::cell::RefCell;
use std::collections::HashMap;

/// what a combo's `ready` finds: (ready, its key permanents, the cards still to cast from hand)
type Ready = (bool, Vec<PermId>, Vec<CardId>);

const NOT_READY: fn() -> Ready = || (false, vec![], vec![]);

/// combos.Combo: groups of card names (one of each group is needed); `ready` checks the position; `mana` is paid
/// for the loop; `finish` is what happens (None: p wins); `xcost`: mana spent on X for X spells cast as part of the
/// combo (Walking Ballista needs X >= 1 to survive)
pub struct Combo {
    pub name: &'static str,
    pub groups: &'static [&'static [&'static str]],
    ready: fn(&Game, PlayerId) -> Ready,
    mana: (u32, &'static str),
    finish: Option<fn(&mut Game, PlayerId) -> Res>,
    xcost: &'static [(&'static str, u32)],
}

impl Combo {
    fn x_for(&self, name: &str) -> u32 {
        self.xcost.iter().find(|x| x.0 == name).map_or(0, |x| x.1)
    }
}

// ------------------------------------------------------------------ the pieces
const KIKI: &str = "Kiki-Jiki, Mirror Breaker";
const FABLE: &str = "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki";
const KIKI_TARGETS: &[&str] =
    &["Zealous Conscripts", "Felidar Guardian", "Restoration Angel", "Pestermite", "Deceiver Exarch"];
const MANA_SINKS: &[&str] =
    &["Walking Ballista", "Urza, Lord High Artificer", "Kinnan, Bonder Prodigy", "Thassa's Oracle"];
const BOUNCERS: &[&str] = &["Shrieking Drake", "Whitemane Lion", "Kor Skyfisher", "Man-o'-War"];
const UNDYING_LOOP: &[&str] =
    &["Mikaeus, the Unhallowed", "Geralf's Messenger", "Butcher Ghoul", "Young Wolf", "Nether Traitor"];
const STAFF_OUTLETS: &[&str] = &["Skirk Prospector", "Goblin Bombardment", "Ashnod's Altar", "Phyrexian Altar"];

/// combos.COMBOS, in Python's order
pub static COMBOS: [Combo; 14] = [
    Combo {
        name: "Kiki-Jiki + Zealous Conscripts / Felidar / Resto",
        groups: &[&[KIKI, FABLE], KIKI_TARGETS],
        ready: kiki,
        mana: (0, ""),
        finish: None,
        xcost: &[],
    },
    Combo {
        name: "Isochron Scepter + Dramatic Reversal",
        groups: &[&["Isochron Scepter"], &["Dramatic Reversal"], MANA_SINKS],
        ready: scepter,
        mana: (0, ""),
        finish: None,
        xcost: &[],
    },
    Combo {
        name: "Power Artifact / Rings of Brighthearth + Monolith",
        groups: &[&["Power Artifact", "Rings of Brighthearth"], &["Basalt Monolith", "Grim Monolith"], MANA_SINKS],
        ready: power_artifact,
        mana: (0, ""),
        finish: None,
        xcost: &[],
    },
    Combo {
        name: "Kinnan + Basalt Monolith",
        groups: &[&["Kinnan, Bonder Prodigy"], &["Basalt Monolith"]],
        ready: kinnan_basalt,
        mana: (0, ""),
        finish: None,
        xcost: &[],
    },
    Combo {
        name: "Kinnan + Freed from the Real / Pemmin's Aura",
        groups: &[&["Kinnan, Bonder Prodigy"], &["Freed from the Real", "Pemmin's Aura"]],
        ready: kinnan_aura,
        mana: (0, ""),
        finish: None,
        xcost: &[],
    },
    Combo {
        name: "Aluren + Chulane + a bouncing creature",
        groups: &[&["Aluren"], &["Chulane, Teller of Tales"], BOUNCERS],
        ready: aluren,
        mana: (0, ""),
        finish: None,
        xcost: &[],
    },
    Combo {
        name: "Sanguine Bond + Exquisite Blood",
        groups: &[&["Sanguine Bond"], &["Exquisite Blood"]],
        ready: bond,
        mana: (0, ""),
        finish: None,
        xcost: &[],
    },
    Combo {
        name: "Thassa's Oracle + Demonic Consultation / Tainted Pact",
        groups: &[&["Thassa's Oracle"], &["Demonic Consultation", "Tainted Pact"]],
        ready: oracle,
        mana: (0, ""),
        finish: None,
        xcost: &[],
    },
    Combo {
        name: "Helm of the Host + Combat Celebrant",
        groups: &[&["Helm of the Host"], &["Combat Celebrant"]],
        ready: helm,
        mana: (0, ""),
        finish: None,
        xcost: &[],
    },
    Combo {
        name: "Gravecrawler + Phyrexian Altar + a Zombie + a death payoff",
        groups: &[&["Gravecrawler"], &["Phyrexian Altar"], &DRAIN_PAYOFFS],
        ready: gravecrawler,
        mana: (0, ""),
        finish: None,
        xcost: &[],
    },
    Combo {
        name: "Yawgmoth + undying loop + a death payoff",
        groups: &[&["Yawgmoth, Thran Physician"], UNDYING_LOOP, &DRAIN_PAYOFFS],
        ready: yawg,
        mana: (0, ""),
        finish: Some(yawg_finish),
        xcost: &[],
    },
    Combo {
        name: "Krenko + Thornbite Staff + a sacrifice outlet",
        groups: &[&["Krenko, Mob Boss"], &["Thornbite Staff"], STAFF_OUTLETS],
        ready: staff,
        mana: (0, ""),
        finish: None,
        xcost: &[],
    },
    Combo {
        name: "Karn, the Great Creator + Mycosynth Lattice",
        groups: &[&["Karn, the Great Creator"], &["Mycosynth Lattice"]],
        ready: karn,
        mana: (0, ""),
        finish: Some(lock_lattice),
        xcost: &[],
    },
    Combo {
        name: "Heliod, Sun-Crowned + Walking Ballista",
        groups: &[&["Heliod, Sun-Crowned"], &["Walking Ballista"]],
        ready: heliod,
        mana: (1, "W"),
        finish: None,
        xcost: &[("Walking Ballista", 2)],
    },
];

/// combos.PIECES: is this card name a piece of a modeled combo?
pub fn is_piece_name(name: &str) -> bool {
    COMBOS.iter().any(|c| c.groups.iter().any(|grp| grp.contains(&name)))
}

/// combos.PIECES as card ids (the names in the card database)
pub fn pieces(g: &Game) -> Vec<CardId> {
    let mut out = vec![];
    for cmb in &COMBOS {
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
    grp.iter().any(|n| on_bf(g, p, n).is_some() || in_hand(g, p, n).is_some())
}

/// combos.free_outlet: a sacrifice outlet of p's (common.outlets)
fn free_outlet(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.cd.is_some_and(|c| {
            let n = name_of(g, c);
            crate::cardcode::is_sac_outlet(n) && !crate::engine::values::stopped(g, n)
        }) && !x.phased
    })
}

fn can_pay_cost(g: &Game, p: PlayerId, c: CardId) -> bool {
    let (gn, pips) = cost_of(g, p, c);
    can_pay(g, p, gn, &pips, false)
}

/// combos.cast_generic: generic mana for the pieces still to be cast, X included
fn cast_generic(g: &Game, cmb: &Combo, casts: &[CardId]) -> u32 {
    casts.iter().map(|&c| g.db.get(c).generic + cmb.x_for(name_of(g, c))).sum()
}

fn cast_pips(g: &Game, cmb: &Combo, casts: &[CardId]) -> String {
    let mut s = cmb.mana.1.to_string();
    for &c in casts {
        s.push_str(&g.db.get(c).pips);
    }
    s
}

/// the combo and its mana (the loop's own and the pieces' still to cast) can be paid for
fn affordable(g: &Game, p: PlayerId, cmb: &Combo, casts: &[CardId]) -> bool {
    can_pay(g, p, cmb.mana.0 + cast_generic(g, cmb, casts), &cast_pips(g, cmb, casts), false)
}

// ------------------------------------------------------------------ execution
/// the index in COMBOS of the first combo whose name contains `part` (tests: Python's
/// `next(c for c in COMBOS if part in c.name)`)
pub fn combo_index(part: &str) -> Option<usize> {
    COMBOS.iter().position(|c| c.name.contains(part))
}

/// COMBOS[i].ready(g, p)[0]: p has everything COMBOS[i] needs
pub fn ready(g: &Game, p: PlayerId, i: usize) -> bool {
    (COMBOS[i].ready)(g, p).0
}

/// combos.attempt: p goes for COMBOS[i]
pub fn attempt(g: &mut Game, p: PlayerId, i: i64) -> Res<bool> {
    let cmb = &COMBOS[i as usize];
    let (ok, keys, casts) = (cmb.ready)(g, p);
    if !ok || !affordable(g, p, cmb, &casts) {
        return Ok(false);
    }
    let turns = g.player(p).turns;
    if g.player(p).combo_turn == Some((turns, cmb.name)) {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    pl.combo_turn = Some((turns, cmb.name));
    pl.stat("combo_attempt", 1);
    pl.milestone.entry("combo").or_insert(turns);
    crate::glog!(g, "  {} goes for {}", g.player(p).name, cmb.name);
    for (i, &c) in casts.iter().enumerate() {
        // the last pieces are cast normally: they can be countered
        let (gn, pips) = (g.db.get(c).generic, g.db.get(c).pips.to_string());
        if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") || !can_pay(g, p, gn, &pips, false) {
            if i > 0 {
                crate::glog!(g, "    ...{} fizzles ({} is gone)", cmb.name, name_of(g, c));
                g.player_mut(p).stat("combo_stopped", 1);
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
        return Ok(true);
    }
    match cmb.finish {
        Some(f) => f(g, p)?,
        None => crate::ai::win(g, p, "combo", Some(!cmb.name.contains("Oracle")))?, // Oracle: an alternate win
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
    if post.is_none() || crate::ai::is_main(g.player(p).key) {
        return Ok(vec![]); // g.combo_decks: the outside decks
    }
    let mut out = vec![];
    for (i, cmb) in COMBOS.iter().enumerate() {
        if !cmb.groups.iter().any(|grp| has_piece(g, p, grp)) {
            continue;
        }
        let (ok, _, casts) = (cmb.ready)(g, p);
        if !ok || !affordable(g, p, cmb, &casts) {
            continue;
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

/// any modeled combo is ready (Ad Nauseam stops there)
pub fn combo_ready(g: &Game, p: PlayerId) -> bool {
    COMBOS.iter().any(|cmb| (cmb.ready)(g, p).0)
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
    let deck = full_deck_names(g, p);
    let mut best: Option<(usize, Vec<&str>)> = None;
    for cmb in &COMBOS {
        if !cmb.groups.iter().all(|grp| grp.iter().any(|n| deck.contains(n))) {
            continue;
        }
        let miss: Vec<&[&str]> = cmb.groups.iter().copied().filter(|grp| !has_piece(g, p, grp)).collect();
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

/// combos.piece_threat: extra value of m if it's a piece of a combo whose other pieces its controller has on the
/// battlefield (7), or all but one of them in hand or play (2.5)
pub fn piece_threat(g: &Game, m: PermId) -> f64 {
    let Some(c) = g.perm(m).cd else { return 0.0 };
    let name = name_of(g, c);
    if !is_piece_name(name) {
        return 0.0;
    }
    let p = g.perm(m).owner;
    for cmb in &COMBOS {
        if !cmb.groups.iter().any(|grp| grp.contains(&name)) {
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
    for cmb in &COMBOS {
        if !cmb.groups.iter().any(|grp| grp.contains(&name)) {
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
    names.push(name_of(g, pl.cmd)); // `| {q.cmd.name}`
    let v: Vec<usize> = COMBOS
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
    let cmbs = deck_combos(g, q);
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
    if pl.cmd_in_zone {
        have.push(name_of(g, pl.cmd));
    }
    let mut best = f64::NEG_INFINITY;
    for i in cmbs {
        let c = &COMBOS[i];
        let n = c.groups.iter().filter(|grp| grp.iter().any(|x| have.contains(x))).count();
        best = best.max(n as f64 / c.groups.len() as f64);
    }
    best * best
}

// ================================================================== the combos
/// infinite hasty copies
fn kiki(g: &Game, p: PlayerId) -> Ready {
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

/// infinite mana with 3+ mana from nonland permanents; wins with a mana sink (Walking Ballista, Urza, Kinnan,
/// Thassa's Oracle in hand ...)
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

/// combos.SINK_TUTORS: tutors that find a mana sink (None: any sink)
const SINK_TUTORS: [(&str, Option<&str>); 6] = [
    ("Recruiter of the Guard", Some("Walking Ballista")),
    ("Enlightened Tutor", Some("Walking Ballista")),
    ("Demonic Tutor", None),
    ("Diabolic Intent", None),
    ("Mystical Tutor", None),
    ("Spellseeker", None),
];

/// combos.mana_sink: infinite mana wins: a sink on the battlefield or in hand, or a tutor in hand that finds one
/// (with infinite mana, Recruiter of the Guard or Enlightened Tutor fetches Walking Ballista and casts it for any X)
fn mana_sink(g: &Game, p: PlayerId) -> bool {
    if any_bf(g, p, &["Walking Ballista", "Urza, Lord High Artificer", "Kinnan, Bonder Prodigy"]).is_some() {
        return true;
    }
    if in_hand(g, p, "Walking Ballista").is_some() || in_hand(g, p, "Thassa's Oracle").is_some() {
        return true;
    }
    let lib: Vec<&str> = g.player(p).library.iter().map(|&c| name_of(g, c)).collect();
    for &c in &g.player(p).hand {
        if let Some((_, t)) = SINK_TUTORS.iter().find(|x| x.0 == name_of(g, c)) {
            let found = match t {
                Some(t) => lib.contains(t),
                None => ["Walking Ballista", "Thassa's Oracle"].iter().any(|n| lib.contains(n)),
            };
            if found {
                return true;
            }
        }
    }
    false
}

/// infinite colourless mana; wins with Urza / Kinnan / Walking Ballista
fn power_artifact(g: &Game, p: PlayerId) -> Ready {
    let pa = any_bf(g, p, &["Power Artifact", "Rings of Brighthearth"]);
    let mono = any_bf(g, p, &["Basalt Monolith", "Grim Monolith"]);
    let (Some(pa), Some(mono)) = (pa, mono) else { return NOT_READY() };
    let cd_name = |m: PermId| g.perm(m).cd.map_or("", |c| name_of(g, c));
    if cd_name(pa) == "Rings of Brighthearth" && cd_name(mono) != "Basalt Monolith" {
        return NOT_READY();
    }
    let sink = any_bf(g, p, &["Urza, Lord High Artificer", "Kinnan, Bonder Prodigy", "Walking Ballista"]).is_some()
        || in_hand(g, p, "Walking Ballista").is_some();
    if !sink {
        return NOT_READY();
    }
    (true, vec![pa, mono], vec![])
}

/// infinite colourless mana; Kinnan's activation finds a winner
fn kinnan_basalt(g: &Game, p: PlayerId) -> Ready {
    match (on_bf(g, p, "Kinnan, Bonder Prodigy"), on_bf(g, p, "Basalt Monolith")) {
        (Some(k), Some(b)) => (true, vec![k, b], vec![]),
        _ => NOT_READY(),
    }
}

/// a mana creature that nets extra mana untaps for U: infinite mana; Kinnan's activation finds a winner
fn kinnan_aura(g: &Game, p: PlayerId) -> Ready {
    match (on_bf(g, p, "Kinnan, Bonder Prodigy"), any_bf(g, p, &["Freed from the Real", "Pemmin's Aura"])) {
        (Some(k), Some(a)) => (true, vec![k, a], vec![]),
        _ => NOT_READY(),
    }
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
    let gc = on_bf(g, p, "Gravecrawler").is_some() || g.player(p).gy.iter().any(|&c| name_of(g, c) == "Gravecrawler");
    if !gc {
        return NOT_READY();
    }
    let zombies = g
        .player(p)
        .perms
        .iter()
        .any(|&m| has_type(g, m, "zombie") && !g.perm(m).cd.is_some_and(|c| name_of(g, c) == "Gravecrawler"));
    let Some(payoff) = drain_payoff(g, p) else { return NOT_READY() };
    if !zombies {
        return NOT_READY();
    }
    (true, vec![altar, payoff], vec![])
}

/// combos._yawg_payoff: a death payoff for the Yawgmoth loop; Geralf's Messenger in the loop is its own (each
/// return drains 2)
fn yawg_payoff(g: &Game, p: PlayerId) -> Option<PermId> {
    drain_payoff(g, p).or_else(|| on_bf(g, p, "Geralf's Messenger"))
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

/// each Goblin sacrificed untaps Krenko: infinite Goblins
fn staff(g: &Game, p: PlayerId) -> Ready {
    let k = on_bf_with(g, p, "Krenko, Mob Boss", true, true);
    let st = on_bf(g, p, "Thornbite Staff");
    let (Some(k), Some(st)) = (k, st) else { return NOT_READY() };
    if g.perm(st).attached != Some(k) || free_outlet(g, p).is_none() {
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
