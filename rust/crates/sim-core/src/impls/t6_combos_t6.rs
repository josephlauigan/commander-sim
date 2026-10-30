//! Tier 6 combo shortcuts for the Dack, Sisay and Rograkh & Thrasios decks (audit/t6/<deck>/SPEC.md, "Win lines"), as
//! `combos::Combo`s with declared steps (`combos::Step`). Each step has the answer window that fits it; an answered
//! step stops the combo (the pieces already played stay) and the real loop is not played out once the last step
//! passes: the player wins, as for every other combo.
//!
//! - **Dack**: Preston + Felidar Guardian + Radiant Solar (or a Preston copy of Solar), all under the player's
//!   control. Started by a flicker spell (Cloudshift / Ephemerate), or already running (the engine's Preston loop cap
//!   `t6_dack::PRESTON_CAP` was hit this turn: the loop is unbounded and only the cap stopped it). The cap stays: real
//!   play (a Dack flip) reaches the loop without this combo. Steps: the three pieces (Permanent), the flicker spell
//!   (Spell), the first Felidar trigger (Trigger).
//! - **Dack first flip**: a resolved Dack flips three of the seven creatures; a table of the 35 piles (`pile_wins`) decides
//!   whether the turn is won (see the section "Dack: the first flip").
//! - **Sisay**: Ioreth + Marvin + a legend that taps for mana (the DDR line; Shang-Chi or Tyvar give the pieces
//!   haste), and Derevi + Emiel + Gaea's Cradle / a rainbow Bloom Tender (the Flash Unicorn line); both finish with
//!   Mount Doom pings. Ioreth's own cap `t6_legends1b::UNTAP_CAP` (6 untaps a turn) stays for the same reason.
//! - **Rograkh & Thrasios**: Oboro Breezecaller + Gaea's Cradle (6+ creatures) + Talon Gates of Madara (Plan A: no
//!   spells, every opposing creature phases out), and Eternal Witness + Snap + Cradle (5+ creatures); infinite mana
//!   into Thrasios draws the library, then Finale of Devastation, or Faerie Mastermind with Green Sun's Zenith, wins.
//!
//! Left out: the Dack two-banisher trade (a way to get pieces back, not a line of its own), Sisay's Selvala chain,
//! Deadpool + Forbidden Orchard and the Mana Vault starter, Rograkh's Badgermole, Springheart/Vibrance lines.

use super::combos::{Combo, QUIET, Ready, Step, StepKind, done, no_keys, no_src};
use crate::engine::cast::{castable, on_cast};
use crate::engine::life::check_state;
use crate::engine::mana::{can_pay, cost_of, pay, seq_cost};
use crate::engine::stack::{ability_window, ability_window_card, counter_window};
use crate::engine::values::colors_of;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Game, Step as TurnStep};
use crate::sym::intern;
use crate::tag::Tag;

// ------------------------------------------------------------------ names
const PRESTON: &str = "Preston, the Vanisher";
const FELIDAR: &str = "Felidar Guardian";
const SOLAR: &str = "Radiant Solar";
const FLICKERS: &[&str] = &["Cloudshift", "Ephemerate"];
const DACK: &str = "Dack Fayden, Helping Hand";

const IORETH: &str = "Ioreth of the Healing House";
const MARVIN: &str = "Marvin, Murderous Mimic";
const SHANG: &str = "Shang-Chi, Master of Kung Fu";
const TYVAR: &str = "Tyvar, Jubilant Brawler";
const SELVALA: &str = "Selvala, Heart of the Wilds";
const HASTERS: &[&str] = &[SHANG, TYVAR];
const ENABLERS: &[&str] = &["Enduring Vitality", "Cryptolith Rite", "Relic of Legends"];
const DEREVI: &str = "Derevi, Empyrial Tactician";
const EMIEL: &str = "Emiel the Blessed";
const TENDER: &str = "Bloom Tender";
const DOOM: &str = "Mount Doom";
const CRADLE: &str = "Gaea's Cradle";

const OBORO: &str = "Oboro Breezecaller";
const GATES: &str = "Talon Gates of Madara";
const WITNESS: &str = "Eternal Witness";
const SNAP: &str = "Snap";
const THRASIOS: &str = "Thrasios, Triton Hero";
const FINALE: &str = "Finale of Devastation";
const MASTERMIND: &str = "Faerie Mastermind";
const GSZ: &str = "Green Sun's Zenith";

// ------------------------------------------------------------------ helpers
fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn nm(g: &Game, m: PermId) -> &str {
    &g.perm(m).name
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && nm(g, m) == name)
}

fn bf_any(g: &Game, p: PlayerId, names: &[&str]) -> Option<PermId> {
    names.iter().find_map(|n| bf(g, p, n))
}

fn hand_card(g: &Game, p: PlayerId, names: &[&str]) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| names.contains(&cname(g, c)))
}

fn lib_card(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).library.iter().copied().find(|&c| cname(g, c) == name)
}

/// p's land of this name; Gaea's Cradle also matches Itlimoc, Cradle of the Sun (a transformed Growing Rites taps the same
/// way, and the lines' untaps work on it)
fn land_on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    let rites = super::t6_green2::RITES;
    g.player(p).lands.iter().copied().find(|&l| {
        let n = cname(g, g.perm(l).card());
        g.perm(l).on_bf && (n == name || (name == CRADLE && n == rites))
    })
}

fn creatures(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| live(g, m) && g.is_creature(m)).count()
}

/// can p pay for all these, cast one after another (`seq_cost`: taxes included)?
fn affordable(g: &Game, p: PlayerId, casts: &[CardId], extra: u32) -> bool {
    let (gn, pips) = seq_cost(g, p, casts);
    can_pay(g, p, gn + extra, &pips, false)
}

fn ready_with(ok: bool) -> Ready {
    (ok, vec![], vec![])
}

/// cast the card by name from hand through the normal AI cast (counter window and real enter effects); true if the
/// permanent is there afterwards
fn cast_named(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    let name = cname(g, c).to_string();
    crate::ai::brain::do_cast(g, p, c, None)?;
    Ok(bf(g, p, &name).is_some())
}

/// pay for and cast an instant or sorcery as a spell whose effect is the shortcut (`free`: the cost is paid by the
/// infinite mana): the counter window is the normal one; false if it was countered or can't be cast
fn cast_abstract(g: &mut Game, p: PlayerId, c: CardId, free: bool) -> Res<bool> {
    let (gn, pips) = cost_of(g, p, c);
    if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") || (!free && !can_pay(g, p, gn, &pips, false)) {
        return Ok(false);
    }
    if !free {
        pay(g, p, gn, &pips, false)?;
    }
    let hand = &mut g.player_mut(p).hand;
    if let Some(i) = hand.iter().position(|&x| x == c) {
        hand.remove(i);
    }
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts {}", g.player(p).name, cname(g, c));
    on_cast(g, p, c)?;
    if g.over || !counter_window(g, p, c, 6.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(false);
    }
    g.player_mut(p).gy.push(c);
    check_state(g)?;
    Ok(true)
}

const fn step(label: &'static str, kind: StepKind, run: fn(&mut Game, PlayerId) -> Res<bool>) -> Step {
    Step { label, kind, src: no_src, pay: (0, ""), run, keys: no_keys }
}

const fn trigger(label: &'static str, src: fn(&Game, PlayerId) -> Option<PermId>) -> Step {
    Step { label, kind: StepKind::Trigger, src, pay: (0, ""), run: done, keys: no_keys }
}

const fn ability(label: &'static str, src: fn(&Game, PlayerId) -> Option<PermId>, cost: (u32, &'static str)) -> Step {
    Step { label, kind: StepKind::Ability, src, pay: cost, run: done, keys: no_keys }
}

const fn pieces(label: &'static str, keys: fn(&Game, PlayerId) -> Vec<PermId>) -> Step {
    Step { label, kind: StepKind::Permanent, src: no_src, pay: (0, ""), run: done, keys }
}

const fn combo(
    name: &'static str,
    groups: &'static [&'static [&'static str]],
    ready: fn(&Game, PlayerId) -> Ready,
    steps: &'static [Step],
) -> Combo {
    Combo { name, groups, ready, mana: (0, ""), finish: None, xcost: &[], steps }
}

// ================================================================== Dack: Preston + Felidar Guardian + Radiant Solar
/// Preston, a nontoken Felidar Guardian and a Radiant Solar (or a token copy of it), all p's
fn dack_pieces(g: &Game, p: PlayerId) -> Option<[PermId; 3]> {
    let preston = bf(g, p, PRESTON)?;
    let felidar = g.player(p).perms.iter().copied().find(|&m| live(g, m) && nm(g, m) == FELIDAR && !g.perm(m).token)?;
    let solar = bf(g, p, SOLAR)?;
    Some([preston, felidar, solar])
}

fn flicker_spell(g: &Game, p: PlayerId) -> Option<CardId> {
    let c = hand_card(g, p, FLICKERS)?;
    (castable(g, p, c, "hand") && affordable(g, p, &[c], 0)).then_some(c)
}

/// the engine's Preston loop cap was hit this turn: the loop is unbounded (`t6_dack::preston_etb`)
fn preston_loop_capped(g: &Game, p: PlayerId) -> bool {
    let stamp = (g.round as f64) * 8.0 + g.active.map_or(0.0, |a| a.0 as f64);
    let pl = g.player(p);
    pl.grudge.get(&intern("preston_cap_round")).copied() == Some(stamp)
        && pl.grudge.get(&intern("preston_cap_n")).copied().unwrap_or(0.0) > super::t6_dack::PRESTON_CAP
}

fn ready_dack_spell(g: &Game, p: PlayerId) -> Ready {
    ready_with(dack_pieces(g, p).is_some() && flicker_spell(g, p).is_some())
}

fn ready_dack_running(g: &Game, p: PlayerId) -> Ready {
    ready_with(dack_pieces(g, p).is_some() && flicker_spell(g, p).is_none() && preston_loop_capped(g, p))
}

fn dack_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    dack_pieces(g, p).map(|x| x.to_vec()).unwrap_or_default()
}

fn felidar_src(g: &Game, p: PlayerId) -> Option<PermId> {
    dack_pieces(g, p).map(|x| x[1])
}

fn cast_flicker(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(c) = flicker_spell(g, p) else { return Ok(false) };
    cast_abstract(g, p, c, false)
}

const S_DACK_PIECES: Step = pieces("Preston, Felidar Guardian and Radiant Solar are under control", dack_keys);
const S_DACK_FLICKER: Step =
    step("a flicker spell on Felidar Guardian (it re-enters uncast)", StepKind::Spell, cast_flicker);
const S_DACK_LOOP: Step = trigger(
    "Felidar's flicker trigger: each token copy flickers the real Felidar, Solar ventures every time",
    felidar_src,
);

// ================================================================== Dack: the first flip (35 piles)
// A resolved Dack Fayden flips three of the deck's seven creatures and gives them away. What the hand traces found for
// each of the 35 possible piles (audit/t6/dack-fayden-white-stax/flips-*.md, SPEC.md "First-flip trace") is a table here:
// the real engine plays Dack's enters trigger (with its trigger window: Ertai, Louisoix and Tidebinder can counter it),
// `t6_dack::record_first_flip` notes the pile and the creature the next flip would leave in the library, and this quiet
// combo then decides from the table whether the banisher / Felidar / Preston lines win the turn. Loops are not played out.

pub const AB: usize = 0;
pub const ST: usize = 1;
pub const FH: usize = 2;
pub const SO: usize = 3;
pub const WF: usize = 4;
pub const PR: usize = 5;
pub const FE: usize = 6;

/// the three creatures of a pile as a bit set
pub const fn bits(a: usize, b: usize, c: usize) -> u32 {
    (1 << a) | (1 << b) | (1 << c)
}

/// pile numbers 1..=35 as in the traces: 1-5 Preston + Felidar + one of Ab/St/FH/So/Wf; 6-15 Felidar + a pair of those
/// five (Ab-St, Ab-FH, Ab-So, Ab-Wf, St-FH, St-So, St-Wf, FH-So, FH-Wf, So-Wf); 16-25 Preston + a pair; 26-35 three of the five
pub fn pile_number(pile: u32) -> Option<u32> {
    let five = [AB, ST, FH, SO, WF];
    let mut pairs = vec![];
    for i in 0..5 {
        for j in i + 1..5 {
            pairs.push((five[i], five[j]));
        }
    }
    let mut all: Vec<u32> = five.iter().map(|&x| bits(PR, FE, x)).collect();
    all.extend(pairs.iter().map(|&(a, b)| bits(FE, a, b)));
    all.extend(pairs.iter().map(|&(a, b)| bits(PR, a, b)));
    for i in 0..5 {
        for j in i + 1..5 {
            for k in j + 1..5 {
                all.push(bits(five[i], five[j], five[k]));
            }
        }
    }
    all.iter().position(|&b| b == pile).map(|i| i as u32 + 1)
}

/// what a winning pile needs besides the board: a flicker spell on Dack, and {1}{W} for Preston's sacrifice ability
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DackLine {
    pub spell: bool,
    pub fetch: bool,
}

/// Does `left` (the creature a second flip would leave in the library, None if unknown) let pile `n` win, with or
/// without a flicker spell on Dack? The outcomes are the traces' summaries (SPEC.md "First-flip trace"):
/// - 1-5, 16, 17, 19, 20, 22, 24: win whatever the second flip is (Felidar or the banisher tokens make it themselves).
/// - 6-15 (Felidar, no Preston): the second flip is certain (Felidar flickers Dack) and wins unless Preston is the one
///   left in the library; piles 10, 12 and 14 only for the cards listed in their traces.
/// - 18, 21, 23, 25: no win without a spell; with one, 23 wins always, 21 and 25 when Abdel or Fiend Hunter is left,
///   18 when Felidar is not left (18's trace: "WIN+1 in most flips, not fully traced"; the no-Felidar flip is unverified).
/// - 26-31: win unless Preston is left; 32-35: only when Felidar or Abdel is left (Preston and the third banisher
///   or Solar both flip). With a spell all of 26-35 win (the "lap trick" and "spend the banishers first" lines).
/// Doubts flagged in the traces stay as traced: "no line found" is a loss here, not a proof.
pub fn pile_wins(n: u32, left: Option<usize>, spell: bool) -> bool {
    let any = |l: &[usize]| left.is_some_and(|x| l.contains(&x));
    let not_pr = left.is_some_and(|x| x != PR);
    match n {
        1..=5 | 16 | 17 | 19 | 20 | 22 | 24 => true,
        6..=9 | 11 | 13 | 15 => not_pr,
        10 | 14 => any(if spell { &[SO, AB] } else { &[SO] }),
        12 => any(&[SO, AB]),
        18 => spell && left.is_some_and(|x| x != FE),
        21 | 25 => spell && any(&[AB, FH]),
        23 => spell,
        26..=31 => spell || not_pr,
        32..=35 => spell || any(&[FE, AB]),
        _ => false,
    }
}

/// the line for a pile and a card left in the library: without a spell if that wins, else with one if `spell_ok`
pub fn dack_line(pile: u32, left: Option<usize>, spell_ok: bool) -> Option<DackLine> {
    let n = pile_number(pile)?;
    let spell = if pile_wins(n, left, false) {
        false
    } else if spell_ok && pile_wins(n, left, true) {
        true
    } else {
        return None;
    };
    // Solar was donated (in the pile) or is the last one left: the trace takes it back with Preston's {1}{W} ability.
    // Only the Felidar-without-Preston piles (6-15) do; the others make a Solar token or have none to need.
    let fetch = (6..=15).contains(&n) && (pile & (1 << SO) != 0 || left == Some(SO));
    Some(DackLine { spell, fetch })
}

/// this turn's first flip: (pile bits, card left in the library)
fn dack_flip(g: &Game, p: PlayerId) -> Option<(u32, Option<usize>)> {
    let stamp = (g.round as f64) * 8.0 + g.active.map_or(0.0, |a| a.0 as f64);
    let pl = g.player(p);
    if pl.grudge.get(&intern(super::t6_dack::FLIP_STAMP_KEY)).copied() != Some(stamp) {
        return None;
    }
    let v = pl.grudge.get(&intern(super::t6_dack::FLIP_KEY)).copied()? as u32;
    let left = (v >> 7) as usize;
    Some((v & 0x7f, left.checked_sub(1)))
}

/// the line Dack's first flip gives right now (Dack still out); `spell_ok`: a flicker spell may be used
fn current_line(g: &Game, p: PlayerId, spell_ok: bool) -> Option<DackLine> {
    bf(g, p, DACK)?;
    let (pile, left) = dack_flip(g, p)?;
    dack_line(pile, left, spell_ok)
}

fn ready_dack_flip(g: &Game, p: PlayerId) -> Ready {
    let spell = flicker_spell(g, p);
    let Some(line) = current_line(g, p, spell.is_some()) else { return ready_with(false) };
    let ok = match (line.spell, spell) {
        (true, Some(c)) => {
            let (gn, pips) = cost_of(g, p, c);
            let (gn, pips) = if line.fetch { (gn + 1, format!("{pips}W")) } else { (gn, pips) };
            can_pay(g, p, gn, &pips, false)
        }
        (true, None) => false,
        (false, _) => !line.fetch || can_pay(g, p, 1, "W", false),
    };
    (ok, flip_keys(g, p), vec![])
}

/// the pile creatures that are still out (their holders are opponents) and matter to a line: Preston, Felidar and the banishers
fn flip_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    let Some((pile, _)) = dack_flip(g, p) else { return vec![] };
    let want: Vec<&str> =
        [PR, FE, ST, FH, WF].iter().filter(|&&i| pile & (1 << i) != 0).map(|&i| super::t6_dack::SEVEN[i]).collect();
    g.players
        .iter()
        .flat_map(|pl| pl.perms.iter().copied())
        .filter(|&m| live(g, m) && g.perm(m).orig == p && !g.perm(m).token && want.contains(&nm(g, m)))
        .collect()
}

fn cast_flip_spell(g: &mut Game, p: PlayerId) -> Res<bool> {
    match current_line(g, p, true) {
        Some(DackLine { spell: true, .. }) => cast_flicker(g, p),
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

fn preston_src(g: &Game, p: PlayerId) -> Option<PermId> {
    flip_keys(g, p).into_iter().find(|&m| nm(g, m) == PRESTON)
}

fn preston_fetch(g: &mut Game, p: PlayerId) -> Res<bool> {
    match current_line(g, p, true) {
        Some(DackLine { fetch: true, .. }) => {
            if !can_pay(g, p, 1, "W", false) {
                return Ok(false);
            }
            pay(g, p, 1, "W", false)?;
            let src = preston_src(g, p);
            ability_window(g, p, src, "Preston's ability takes Radiant Solar back", Some(6.0), None)
        }
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

/// the first flicker trigger: Felidar's (donated, still out) or, without it, Dack's own re-entry
fn flip_src(g: &Game, p: PlayerId) -> Option<PermId> {
    flip_keys(g, p).into_iter().find(|&m| nm(g, m) == FELIDAR).or_else(|| bf(g, p, DACK))
}

const S_FLIP_PIECES: Step = pieces("the flipped Preston, Felidar and banishers stay out", flip_keys);
const S_FLIP_SPELL: Step = step("a flicker spell on Dack when the pile needs it", StepKind::Spell, cast_flip_spell);
const S_FLIP_FETCH: Step =
    step("Preston's {1}{W} ability takes Solar back when the pile needs it", StepKind::Engine, preston_fetch);
const S_FLIP_LOOP: Step = trigger("the first flicker trigger (Felidar's, or Dack's re-entry)", flip_src);
/// the real loop has more windows than one: each of these is a trigger the table can answer (Dack's second enters
/// trigger, Preston's copy triggers, Radiant Solar's ventures, the next flicker)
fn flip_dack_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, DACK)
}

fn flip_solar_src(g: &Game, p: PlayerId) -> Option<PermId> {
    g.players
        .iter()
        .flat_map(|pl| pl.perms.iter().copied())
        .find(|&m| live(g, m) && g.perm(m).orig == p && nm(g, m) == SOLAR)
        .or_else(|| bf(g, p, DACK))
}

fn flip_preston_src(g: &Game, p: PlayerId) -> Option<PermId> {
    preston_src(g, p).or_else(|| bf(g, p, DACK))
}

const S_FLIP_DACK2: Step = trigger("Dack's second enters trigger (the rest of the creatures flip)", flip_dack_src);
const S_FLIP_PRESTON: Step = trigger("Preston copies a creature that entered uncast", flip_preston_src);
const S_FLIP_SOLAR: Step = trigger("Radiant Solar ventures (a nontoken creature entered)", flip_solar_src);

// ================================================================== Sisay
fn is_legend_creature(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.creature && d.tag(Tag::Leg) && !matches!(&*d.name, IORETH | MARVIN)
}

/// taps for mana without help: Selvala, or a legendary mana creature (not Shang-Chi: his mana only pays creature abilities)
fn natural_source(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    &*d.name == SELVALA || d.tags.str(Tag::Dork).is_some_and(|k| k != "C")
}

fn pb_cd(g: &Game, m: PermId) -> Option<CardId> {
    g.perm(m).cd
}

/// can m use a {T} ability now: untapped (unless `any_tap`: something else untaps it first), and not summoning sick unless
/// it has haste or its controller has Shang-Chi / Tyvar
fn can_tap(g: &Game, m: PermId, haste: bool, any_tap: bool) -> bool {
    let x = g.perm(m);
    (any_tap || !x.tapped) && (!x.sick || haste || crate::dsl::has_kw(g, m, "haste"))
}

fn land_drop_open(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    pl.lands_played < 1 + crate::cardcode::extra_lands(g, p) + pl.extra_land_now
}

/// Mount Doom is the finisher: out, or playable
fn doom_available(g: &Game, p: PlayerId) -> bool {
    land_on_bf(g, p, DOOM).is_some() || (hand_card(g, p, &[DOOM]).is_some() && land_drop_open(g, p))
}

/// Mount Doom for the plan: out, or in hand with a land drop (the Doom step plays it), or Sisay fetches it
fn doom_piece(g: &Game, p: PlayerId, plan: &mut Plan) -> bool {
    if doom_available(g, p) {
        return true;
    }
    match get_piece(g, p, &[DOOM]) {
        Some(Src::Fetch(c)) => {
            plan.fetches.push(c);
            true
        }
        _ => false,
    }
}

const SISAY: &str = "Sisay, Weatherlight Captain";

/// A plan for a Sisay line: the cards to cast from hand and the cards Sisay fetches from the library ({W}{U}{B}{R}{G}
/// each, an ability: no spell counter can stop it, an ability answer can), both in order.
#[derive(Default)]
struct Plan {
    casts: Vec<CardId>,
    fetches: Vec<CardId>,
}

/// where a missing piece comes from: the hand (cast), or Sisay's fetch (she is out and the card's mana value is below
/// her power)
enum Src {
    Cast(CardId),
    Fetch(CardId),
}

fn get_piece(g: &Game, p: PlayerId, names: &[&str]) -> Option<Src> {
    if let Some(c) = hand_card(g, p, names) {
        return Some(Src::Cast(c));
    }
    let sis = bf(g, p, SISAY)?;
    let power = crate::engine::values::epow(g, sis);
    names.iter().filter_map(|n| lib_card(g, p, n)).find(|&c| (g.db.get(c).cmc as i32) < power).map(Src::Fetch)
}

impl Plan {
    fn add(&mut self, s: Src) {
        match s {
            Src::Cast(c) => self.casts.push(c),
            Src::Fetch(c) => self.fetches.push(c),
        }
    }

    /// can p pay for the casts (one after another, `seq_cost`) plus every fetch (5 coloured mana each)?
    fn affordable(&self, g: &Game, p: PlayerId, extra: u32) -> bool {
        let (gn, mut pips) = seq_cost(g, p, &self.casts);
        for _ in &self.fetches {
            pips.push_str("WUBRG");
        }
        can_pay(g, p, gn + extra, &pips, false)
    }
}

/// Sisay fetches the next planned piece (a step of the Sisay lines)
fn fetch_next(g: &mut Game, p: PlayerId, plan: Option<Plan>) -> Res<bool> {
    let Some(plan) = plan else {
        crate::glog!(g, "    (the line is no longer ready: the plan found no way to finish it)");
        return Ok(false);
    };
    let Some(&c) = plan.fetches.first() else { return Ok(true) };
    super::legends::sisay_fetch_card(g, p, c)
}

fn fetch_next_ddr(g: &mut Game, p: PlayerId) -> Res<bool> {
    keeping_ioreth(g, p, |g| fetch_next(g, p, ddr_plan(g, p, true)))
}

/// run a DDR step with Ioreth kept out of the mana payment: with Enduring Vitality or Relic of Legends (and Tyvar's
/// haste) a fetched Ioreth is a mana source, and paying Sisay's next {W}{U}{B}{R}{G} with it taps the untapper the loop
/// needs (the line then stopped as "no longer ready")
fn keeping_ioreth(g: &mut Game, p: PlayerId, f: impl FnOnce(&mut Game) -> Res<bool>) -> Res<bool> {
    let ioreth: Vec<PermId> = bf(g, p, IORETH).into_iter().collect();
    let keep = std::mem::replace(&mut g.keep_untapped, ioreth);
    let r = f(g);
    g.keep_untapped = keep;
    r
}

fn fetch_next_uni(g: &mut Game, p: PlayerId) -> Res<bool> {
    fetch_next(g, p, uni_plan(g, p, true))
}

/// plan slot hook: does the ready Sisay line need a spell cast (so a spell counter could stop it)?
/// None = the line is not ready
pub fn line_needs_cast(g: &Game, p: PlayerId, combo: &str) -> Option<bool> {
    let plan = if combo.contains("DDR") {
        ddr_plan(g, p, false)
    } else if combo.contains("Flash Unicorn") {
        uni_plan(g, p, false)
    } else {
        None
    }?;
    Some(!plan.casts.is_empty())
}

/// (the cards to cast from hand, in order) for Ioreth + Marvin + a legend that taps for mana + Mount Doom, or None
/// `in_line`: a step of the line already running: the mana was checked when it began (each fetch or cast pays for itself and
/// fails if it can't), and a piece fetched on the way (a haste legend) changes what the plan would need now
fn ddr_plan(g: &Game, p: PlayerId, in_line: bool) -> Option<Plan> {
    let mut plan = Plan::default();
    let (ioreth, marvin) = (bf(g, p, IORETH), bf(g, p, MARVIN));
    if ioreth.is_none() {
        // an Ioreth still to come enters tapped under Blind Obedience (and the like): its untap can't start the loop
        // this turn (the line went, fetched Ioreth tapped and stopped)
        if crate::engine::hooks::enters_tapped(g, p, None) {
            return None;
        }
        plan.add(get_piece(g, p, &[IORETH])?);
    }
    if marvin.is_none() {
        plan.add(get_piece(g, p, &[MARVIN])?);
    }
    // the legend that makes the mana
    let legends_bf: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m) && pb_cd(g, m).is_some_and(|c| is_legend_creature(g, c)))
        .collect();
    let legends_hand: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| is_legend_creature(g, c)).collect();
    let enab_bf = bf_any(g, p, ENABLERS).is_some();
    let enab_hand = hand_card(g, p, ENABLERS);
    let mut source: Option<PermId> =
        legends_bf.iter().copied().find(|&m| pb_cd(g, m).is_some_and(|c| natural_source(g, c)));
    if source.is_none() && enab_bf {
        source = legends_bf.first().copied();
    }
    if source.is_none() {
        if let Some(c) = legends_hand.iter().copied().find(|&c| natural_source(g, c)) {
            plan.casts.push(c);
        } else if let Some(Src::Fetch(c)) = get_piece(g, p, &[SELVALA]) {
            plan.fetches.push(c); // Sisay fetches the mana legend
        } else {
            // any other legend, with a Vitality / Cryptolith Rite / Relic of Legends to make it tap for mana
            let enab = if enab_bf { None } else { Some(enab_hand?) };
            if let Some(&m) = legends_bf.first() {
                source = Some(m);
            } else {
                plan.casts.push(*legends_hand.first()?);
            }
            plan.casts.extend(enab);
        }
    }
    // abilities need haste when a creature was cast this turn (Shang-Chi, Tyvar), or a bf piece is summoning sick
    let creature_cast = plan.casts.iter().chain(&plan.fetches).any(|&c| g.db.get(c).creature);
    let cast_haster = plan.casts.iter().any(|&c| HASTERS.contains(&cname(g, c)));
    let mut haste = super::t6_legends1b::ability_haste(g, p) || cast_haster;
    let usable = |g: &Game, haste: bool| {
        ioreth.is_none_or(|m| can_tap(g, m, haste, false))
            && marvin.is_none_or(|m| can_tap(g, m, haste, true))
            && source.is_none_or(|m| can_tap(g, m, haste, true))
    };
    let sick_bf = !usable(g, haste);
    if (creature_cast || sick_bf) && !haste {
        plan.add(get_piece(g, p, HASTERS)?);
        haste = true;
    }
    if !usable(g, haste) {
        return None;
    }
    (doom_piece(g, p, &mut plan) && (in_line || plan.affordable(g, p, 0))).then_some(plan)
}

fn ready_ddr(g: &Game, p: PlayerId) -> Ready {
    ready_with(ddr_plan(g, p, false).is_some())
}

fn cast_next_ddr(g: &mut Game, p: PlayerId) -> Res<bool> {
    keeping_ioreth(g, p, |g| {
        let Some(plan) = ddr_plan(g, p, true) else { return Ok(false) };
        let Some(&c) = plan.casts.first() else { return Ok(true) };
        cast_named(g, p, c)
    })
}

/// The pieces an answer can stop: Ioreth, Marvin and ONE mana legend (the loop uses a single source; any other legend
/// on the table, Sisay among them, is not a piece of the line). A natural mana source is preferred; with only an
/// enabler (Enduring Vitality ...) the first legend creature serves.
fn ddr_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    let mut v: Vec<PermId> = [bf(g, p, IORETH), bf(g, p, MARVIN)].into_iter().flatten().collect();
    let legends: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m) && pb_cd(g, m).is_some_and(|c| is_legend_creature(g, c)))
        .collect();
    let src = legends
        .iter()
        .copied()
        .find(|&m| pb_cd(g, m).is_some_and(|c| natural_source(g, c)))
        .or(legends.first().copied());
    v.extend(src);
    v
}

fn ioreth_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, IORETH)
}

/// Mount Doom out (the land drop, if it was in hand)
fn doom_in(g: &mut Game, p: PlayerId) -> Res<bool> {
    if land_on_bf(g, p, DOOM).is_some() {
        return Ok(true);
    }
    let Some(c) = hand_card(g, p, &[DOOM]) else { return Ok(false) };
    let hand = &mut g.player_mut(p).hand;
    if let Some(i) = hand.iter().position(|&x| x == c) {
        hand.remove(i);
    }
    crate::engine::turn::play_land_card(g, p, c, "plays")?;
    Ok(land_on_bf(g, p, DOOM).is_some())
}

/// Mount Doom's ping (40 of them, each untapped by the loop): the ability answer window is its answer
fn doom_ping(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(l) = land_on_bf(g, p, DOOM) else { return Ok(false) };
    Ok(ability_window_card(g, p, g.perm(l).card(), "1 damage to each opponent (again and again)", None, None)?
        && land_on_bf(g, p, DOOM).is_some())
}

const S_FETCH_DDR: Step = step("Sisay fetches a missing piece", StepKind::Engine, fetch_next_ddr);
const S_CAST_NEXT_DDR: Step = step("cast a missing piece", StepKind::Spell, cast_next_ddr);
const S_DOOM_IN: Step = step("Mount Doom is on the battlefield", StepKind::Engine, doom_in);
const S_DOOM_PING: Step = step("Mount Doom pings, untapped by the loop", StepKind::Engine, doom_ping);

// --- Flash Unicorn: Derevi + Emiel + Cradle / rainbow Bloom Tender
/// Bloom Tender's colours: those among p's permanents and the cards about to be cast
fn colors_among(g: &Game, p: PlayerId, casts: &[CardId]) -> u32 {
    let mut cs = crate::cards::Colors::NONE;
    for &m in &g.player(p).perms {
        if live(g, m) {
            cs = cs.union(colors_of(g, m));
        }
    }
    for &c in casts {
        cs = cs.union(g.db.get(c).colors);
    }
    cs.count()
}

/// `in_line`: a step of the line already running. The mana for earlier steps may have tapped the Tender, and Derevi's
/// enter trigger untaps it again, so a tapped Tender no longer makes the line not ready (it stopped Unicorn "for no
/// game reason" after the first fetch)
fn uni_plan(g: &Game, p: PlayerId, in_line: bool) -> Option<Plan> {
    let mut plan = Plan::default();
    if bf(g, p, DEREVI).is_none() {
        plan.add(get_piece(g, p, &[DEREVI])?);
    }
    if bf(g, p, EMIEL).is_none() {
        plan.add(get_piece(g, p, &[EMIEL])?);
    }
    let engine_ok = if land_on_bf(g, p, CRADLE).is_some() {
        creatures(g, p) + plan.casts.len() + plan.fetches.len() >= 4 // 4 G a tap, {3} a blink
    } else {
        let haste = super::t6_legends1b::ability_haste(g, p);
        bf(g, p, TENDER).is_some_and(|m| can_tap(g, m, haste, in_line)) && colors_among(g, p, &plan.casts) >= 4
    };
    (engine_ok && doom_piece(g, p, &mut plan) && plan.affordable(g, p, 3)).then_some(plan)
}

fn ready_unicorn(g: &Game, p: PlayerId) -> Ready {
    ready_with(uni_plan(g, p, false).is_some())
}

fn cast_next_uni(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(plan) = uni_plan(g, p, true) else { return Ok(false) };
    let Some(&c) = plan.casts.first() else { return Ok(true) };
    cast_named(g, p, c)
}

fn uni_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    [bf(g, p, DEREVI), bf(g, p, EMIEL), bf(g, p, TENDER)].into_iter().flatten().collect()
}

fn emiel_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, EMIEL)
}

fn derevi_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, DEREVI)
}

const S_FETCH_UNI: Step = step("Sisay fetches a missing piece", StepKind::Engine, fetch_next_uni);
const S_CAST_NEXT_UNI: Step = step("cast Derevi or Emiel", StepKind::Spell, cast_next_uni);

// ================================================================== Rograkh & Thrasios
/// how the table is won once the mana is infinite (Thrasios draws the library first)
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fin {
    Finale,
    Mastermind,
}

fn has_card(g: &Game, p: PlayerId, name: &str) -> bool {
    hand_card(g, p, &[name]).is_some() || lib_card(g, p, name).is_some()
}

/// Thrasios can draw the library: on the battlefield, or in the command zone (or the hand) on p's own main phase,
/// where the unbounded mana casts it whatever the tax
fn thrasios_reach(g: &Game, p: PlayerId) -> bool {
    if bf(g, p, THRASIOS).is_some() {
        return true;
    }
    let main = g.active == Some(p) && matches!(g.step, TurnStep::Start | TurnStep::Main1 | TurnStep::Main2);
    main && (thrasios_card(g, p).is_some())
}

/// Thrasios in the command zone or in hand
fn thrasios_card(g: &Game, p: PlayerId) -> Option<CardId> {
    g.player(p).cmds_in_zone().find(|&c| cname(g, c) == THRASIOS).or_else(|| hand_card(g, p, &[THRASIOS]))
}

fn finisher(g: &Game, p: PlayerId) -> Option<Fin> {
    // Finale in hand needs no draw: the unbounded mana casts it for X of 10 or more
    if hand_card(g, p, &[FINALE]).is_some() {
        return Some(Fin::Finale);
    }
    if !thrasios_reach(g, p) {
        return None;
    }
    if has_card(g, p, FINALE) {
        return Some(Fin::Finale);
    }
    ((bf(g, p, MASTERMIND).is_some() || has_card(g, p, MASTERMIND)) && has_card(g, p, GSZ)).then_some(Fin::Mastermind)
}

fn move_to_hand(g: &mut Game, p: PlayerId, name: &str) {
    if hand_card(g, p, &[name]).is_some() {
        return;
    }
    if let Some(c) = lib_card(g, p, name) {
        let lib = &mut g.player_mut(p).library;
        if let Some(i) = lib.iter().position(|&x| x == c) {
            lib.remove(i);
        }
        g.player_mut(p).hand.push(c);
    }
}

fn thrasios_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, THRASIOS)
}

/// Thrasios's {4} until the library is empty: the finisher is in hand (the infinite mana pays for it)
fn thrasios_draw(g: &mut Game, p: PlayerId) -> Res<bool> {
    match finisher(g, p) {
        Some(Fin::Finale) => {
            g.player_mut(p).grudge.insert(intern("combo_finisher"), 1.0);
            move_to_hand(g, p, FINALE);
        }
        Some(Fin::Mastermind) => {
            g.player_mut(p).grudge.insert(intern("combo_finisher"), 2.0);
            if bf(g, p, MASTERMIND).is_none() {
                move_to_hand(g, p, MASTERMIND);
            }
            move_to_hand(g, p, GSZ);
        }
        None => return Ok(false),
    }
    Ok(true)
}

/// the finishing spell: Finale of Devastation with X of 10 or more in the first main phase (or a Mastermind cast)
fn finish_spell(g: &mut Game, p: PlayerId) -> Res<bool> {
    match finisher(g, p) {
        Some(Fin::Finale) => {
            let Some(c) = hand_card(g, p, &[FINALE]) else { return Ok(false) };
            cast_abstract(g, p, c, true)
        }
        Some(Fin::Mastermind) => {
            if bf(g, p, MASTERMIND).is_some() {
                return Ok(true);
            }
            let Some(c) = hand_card(g, p, &[MASTERMIND]) else { return Ok(false) };
            cast_named(g, p, c)
        }
        None => Ok(false),
    }
}

/// Mastermind's "each player draws a card", activated until the opponents are decked (Finale needs no ability)
fn finish_ability(g: &mut Game, p: PlayerId) -> Res<bool> {
    if g.player(p).grudge.get(&intern("combo_finisher")).copied() != Some(2.0) {
        return Ok(true);
    }
    let Some(m) = bf(g, p, MASTERMIND) else { return Ok(false) };
    ability_window(g, p, Some(m), "each player draws a card (again and again)", Some(6.0), None)
}

/// Thrasios is cast from the command zone (or hand) when the finish needs its draws and it is not out: the unbounded
/// mana pays the cost and the tax (floating mana added for exactly that cast), and the cast is a normal spell the
/// table may counter (a countered commander goes back to the command zone)
fn cast_thrasios(g: &mut Game, p: PlayerId) -> Res<bool> {
    if bf(g, p, THRASIOS).is_some() || hand_card(g, p, &[FINALE]).is_some() {
        return Ok(true);
    }
    let Some(c) = thrasios_card(g, p) else { return Ok(false) };
    let (gn, pips) = cost_of(g, p, c);
    let before = g.player(p).floating.any;
    g.player_mut(p).floating.any += gn + pips.chars().count() as u32;
    crate::ai::brain::do_cast(g, p, c, None)?;
    let f = &mut g.player_mut(p).floating;
    f.any = f.any.min(before);
    Ok(bf(g, p, THRASIOS).is_some())
}

const S_FIN_THRASIOS: Step = step("cast Thrasios (the unbounded mana pays the tax)", StepKind::Spell, cast_thrasios);

const S_FIN_DRAW: Step = Step {
    label: "Thrasios draws the library for the finisher",
    kind: StepKind::Ability,
    src: thrasios_src,
    pay: (0, ""),
    run: thrasios_draw,
    keys: no_keys,
};
const S_FIN_SPELL: Step = step("cast Finale of Devastation (or Faerie Mastermind)", StepKind::Spell, finish_spell);
const S_FIN_ABILITY: Step = step("Faerie Mastermind's draw, again and again", StepKind::Engine, finish_ability);

// --- Plan A: Oboro Breezecaller + Gaea's Cradle + Talon Gates of Madara
fn gates_in_reach(g: &Game, p: PlayerId) -> bool {
    land_on_bf(g, p, GATES).is_some() || hand_card(g, p, &[GATES]).is_some()
}

// --- t6 rograkh-thrasios: the creature tutors as a step of the line. Oracle: Green Sun's Zenith {X}{G} (a green
// creature, MV X or less, onto the battlefield; GSZ shuffles itself back), Nature's Rhythm {X}{G}{G} and Chord of
// Calling {X}{G}{G}{G} (any creature), Invasion of Ikoria {X}{G}{G} (a non-Human creature). Finale is left out: it is
// the finisher.
const NATURES_RHYTHM: &str = "Nature's Rhythm";
const CHORD: &str = "Chord of Calling";
const INVASION: &str = "Invasion of Ikoria // Zilortha, Apex of Ikoria";
/// (tutor, finds a non-green creature, finds a Human)
const X_TUTORS: [(&str, bool, bool); 4] =
    [(GSZ, false, true), (NATURES_RHYTHM, true, true), (CHORD, true, true), (INVASION, true, false)];

/// a tutor in hand that puts `piece` (in the library) onto the battlefield, and its whole cost with X = the piece's
/// mana value: (tutor, generic, pips)
fn x_tutor(g: &Game, p: PlayerId, piece: &str) -> Option<(CardId, u32, String)> {
    let c = lib_card(g, p, piece)?;
    let d = g.db.get(c);
    let (green, human) = (d.pips.contains('G'), d.has_subtype("human"));
    X_TUTORS.iter().find_map(|&(t, any, humans)| {
        if (!any && !green) || (!humans && human) {
            return None;
        }
        let tc = hand_card(g, p, &[t])?;
        if !castable(g, p, tc, "hand") {
            return None;
        }
        let (gn, pips) = cost_of(g, p, tc);
        Some((tc, gn + d.cmc, pips))
    })
}

/// cast the tutor for `piece` (X paid, a normal counter window) and put the piece onto the battlefield
fn tutor_onto_bf(g: &mut Game, p: PlayerId, piece: &str) -> Res<bool> {
    let Some((t, gn, pips)) = x_tutor(g, p, piece) else { return Ok(false) };
    if !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    let x = gn - cost_of(g, p, t).0;
    pay(g, p, x, "", false)?;
    if !cast_abstract(g, p, t, false)? {
        return Ok(false);
    }
    if cname(g, t) == GSZ {
        let pl = g.player_mut(p);
        pl.gy.retain(|&c| c != t);
        pl.library.push(t);
        crate::engine::tutors::shuffle_library(g, p);
    }
    let Some(c) = lib_card(g, p, piece) else { return Ok(false) };
    g.player_mut(p).library.retain(|&x| x != c);
    crate::glog!(g, "    {} puts {} onto the battlefield", cname(g, t), piece);
    crate::engine::zones::enter(g, p, c, crate::engine::zones::Enter::default())?;
    Ok(bf(g, p, piece).is_some())
}

/// how line A gets Oboro: out, cast from hand, or put in by a tutor
enum Piece {
    Out,
    Cast(CardId),
    Tutor(u32, String),
}

/// Oboro and the mana for the loop's first turn: (how Oboro comes, the extra mana besides a cast from hand)
fn plan_a(g: &Game, p: PlayerId) -> Option<(Piece, Vec<CardId>)> {
    land_on_bf(g, p, CRADLE)?;
    if !gates_in_reach(g, p) || finisher(g, p).is_none() {
        return None;
    }
    let oboro = if bf(g, p, OBORO).is_some() {
        Piece::Out
    } else if let Some(c) = hand_card(g, p, &[OBORO]) {
        Piece::Cast(c)
    } else {
        let (_, gn, pips) = x_tutor(g, p, OBORO)?;
        Piece::Tutor(gn, pips)
    };
    // Cradle at 6: 6 + Gates 1 in, {2} (Oboro) + {4} (Gates back from hand) out; Training Grounds or Biomancer's
    // Familiar make Oboro's ability {1}, so Cradle at 5 is enough
    let need = super::t6_green1::creature_ability_cost(g, p, 2) as usize + 4;
    let casts: Vec<CardId> = if let Piece::Cast(c) = oboro { vec![c] } else { vec![] };
    let coming = usize::from(!matches!(oboro, Piece::Out));
    let ok = match &oboro {
        Piece::Tutor(gn, pips) => can_pay(g, p, gn + need as u32, pips, false),
        _ => affordable(g, p, &casts, need as u32),
    };
    if creatures(g, p) + coming < need || !ok {
        return None;
    }
    Some((oboro, casts))
}

fn ready_plan_a(g: &Game, p: PlayerId) -> Ready {
    ready_with(plan_a(g, p).is_some())
}

fn cast_oboro(g: &mut Game, p: PlayerId) -> Res<bool> {
    match plan_a(g, p) {
        None => Ok(false),
        Some((Piece::Out, _)) => Ok(true),
        Some((Piece::Cast(c), _)) => cast_named(g, p, c),
        Some((Piece::Tutor(..), _)) => tutor_onto_bf(g, p, OBORO),
    }
}

fn a_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    let mut v: Vec<PermId> = bf(g, p, OBORO).into_iter().collect();
    if creatures(g, p) <= 6 {
        // at exactly 6 every creature is a key piece: one removal and Cradle makes too little
        v.extend(g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)));
    }
    v
}

fn oboro_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, OBORO)
}

/// Gates comes back from hand (the {4} ability), and every entry phases out the opposing creatures: the loop phases out
/// the whole table's creatures. The enter trigger is an ability the opponents may answer.
fn gates_loop(g: &mut Game, p: PlayerId) -> Res<bool> {
    let l = match land_on_bf(g, p, GATES) {
        Some(l) => l,
        None => {
            let Some(c) = hand_card(g, p, &[GATES]) else { return Ok(false) };
            let hand = &mut g.player_mut(p).hand;
            if let Some(i) = hand.iter().position(|&x| x == c) {
                hand.remove(i);
            }
            g.add_land(p, c, false)
        }
    };
    if !ability_window_card(g, p, g.perm(l).card(), "target creature phases out (every time)", None, None)? {
        return Ok(false);
    }
    let victims: Vec<PermId> =
        g.opps(p).flat_map(|q| g.player(q).perms.clone()).filter(|&m| live(g, m) && g.is_creature(m)).collect();
    for m in victims {
        g.perm_mut(m).phased = true;
    }
    g.bf_ver += 1;
    Ok(true)
}

// --- Eternal Witness + Snap + Gaea's Cradle
fn gy_card(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).gy.iter().copied().find(|&c| cname(g, c) == name)
}

/// the loop starts with Witness in hand, on the battlefield (Snap it first, recast it) or in the library with a
/// creature tutor in hand (Green Sun's Zenith, Nature's Rhythm, Chord of Calling put it in), and Snap in hand or in
/// the graveyard (Witness's enter trigger returns it; not with Witness already out). The cards still to cast from
/// hand are returned.
fn plan_b(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    land_on_bf(g, p, CRADLE)?;
    let (w, s) = (hand_card(g, p, &[WITNESS]), hand_card(g, p, &[SNAP]));
    let w_out = bf(g, p, WITNESS).and_then(|m| g.perm(m).cd);
    let tutor = if w.is_none() && w_out.is_none() { Some(x_tutor(g, p, WITNESS)?) } else { None };
    if s.is_none() && (w_out.is_some() || gy_card(g, p, SNAP).is_none()) {
        return None;
    }
    let sc = s.or_else(|| gy_card(g, p, SNAP))?;
    // Cradle at 5 + a {U} land in, Witness {1}{G}{G} + Snap {1}{U} out, Snap untaps two lands: +1 a cycle. A Witness
    // already out is bounced by the loop, so it does not count for Cradle. A tutored Witness: the tutor and Snap are
    // paid first, Snap's untap recasts Witness
    let feeders = creatures(g, p) - usize::from(w_out.is_some());
    if feeders < 5 || finisher(g, p).is_none() {
        return None;
    }
    let ok = match &tutor {
        Some((_, gn, pips)) => {
            let (sg, sp) = cost_of(g, p, sc);
            can_pay(g, p, gn + sg, &format!("{pips}{sp}"), false)
        }
        None => affordable(g, p, &[w.or(w_out)?, sc], 0),
    };
    ok.then(|| w.into_iter().chain(s).collect())
}

fn ready_plan_b(g: &Game, p: PlayerId) -> Ready {
    ready_with(plan_b(g, p).is_some())
}

/// Witness is cast, or put in by a tutor (unless it is already out and Snap is in hand: Snap bounces it first);
/// with Snap in the graveyard, the enter trigger's one target is Snap (whatever card the engine's own pick returned
/// goes back)
fn cast_witness(g: &mut Game, p: PlayerId) -> Res<bool> {
    if bf(g, p, WITNESS).is_some() {
        return Ok(hand_card(g, p, &[SNAP]).is_some());
    }
    let snap_gy = if hand_card(g, p, &[SNAP]).is_none() { gy_card(g, p, SNAP) } else { None };
    let (hand0, gy0) = (g.player(p).hand.clone(), g.player(p).gy.clone());
    let entered = match hand_card(g, p, &[WITNESS]) {
        Some(w) => cast_named(g, p, w)?,
        None => tutor_onto_bf(g, p, WITNESS)?,
    };
    if !entered {
        return Ok(false);
    }
    if let Some(s) = snap_gy
        && g.player(p).gy.contains(&s)
    {
        let back: Vec<CardId> =
            g.player(p).hand.iter().copied().filter(|c| !hand0.contains(c) && gy0.contains(c)).collect();
        let pl = g.player_mut(p);
        for c in back {
            pl.hand.retain(|&x| x != c);
            pl.gy.push(c);
        }
        pl.gy.retain(|&x| x != s);
        pl.hand.push(s);
        crate::glog!(g, "    Eternal Witness returns Snap");
    }
    Ok(hand_card(g, p, &[SNAP]).is_some())
}

fn snap_witness(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(s) = hand_card(g, p, &[SNAP]) else { return Ok(false) };
    cast_abstract(g, p, s, false)
}

fn witness_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, WITNESS)
}

fn b_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    if creatures(g, p) <= 6 {
        g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
    } else {
        vec![]
    }
}

// --- Lines C and D: Badgermole Cub (earthbend) + a clone
// Oracle: Badgermole Cub {1}{G}: "When this creature enters, earthbend 1" (a land becomes a 0/0 haste creature, +1/+1
// counter, still a land) and "Whenever you tap a creature for mana, add an additional {G}". The engine has no
// earthbend, so these two lines are counted from the cards' text and the sim never plays the creature-lands.
//
// Line C (Cub + Mockingbird or Phantasmal Image + Oboro + Cradle): a clone enters as a copy of the earthbent land L1
// (mana value 0, so Mockingbird with X=0 costs {U}); the copy is a plain land (the earthbend is not copiable). Loop:
// tap Cradle (N {G}) and the copy ({U}), pay Oboro {2}, return the copy to hand, untap Cradle, recast the clone with
// the {U}. Net per loop: N - 2 with Mockingbird ({U}), N - 3 with Phantasmal Image ({1}{U}). N counts Cub, L1 and
// Oboro, so N >= 3 (Mockingbird) or 4 (Image) is unbounded; no Gates. Needs one {U} land to earthbend. Clever
// Impersonator can't copy a land and Flesh Duplicate needs {U}{U} (only one {U} comes back): both left out of C.
//
// Line D (Cub + a clone of it + Eternal Witness + Snap, no Cradle): two Cubs, two earthbent lands L1 and L2. Each
// creature-land tap makes 1 + 2 = 3 mana: 6 for both. Witness {1}{G}{G} + Snap {1}{U} = 5; Snap untaps both lands and
// returns Witness, Witness returns Snap: +1 a cycle (L1 gives the {U}). Any clone that can copy Cub works.
//
// Line G (Springheart Nantuko on Witness + Vibrance with the Oboro/Gates loop) is unbounded once line A is: each Gates
// entry renews one {R} (the Gates filter, {1},{T}), so plain loops bank {R}{R} and the mana for the next Witness copy
// (Oboro 2 + Gates 4 + landfall 2 + evoke 2), and 3 damage a copy follows. But it needs line A running, and line A
// already wins through Thrasios and Finale (or Mastermind + GSZ); G only matters when neither finisher can be reached
// (Thrasios exiled and Finale gone), which the traces do not show. Not modelled.  // --- t6 rograkh-thrasios
const CUB: &str = "Badgermole Cub";
const MOCKINGBIRD: &str = "Mockingbird";
const IMAGE: &str = "Phantasmal Image";
const IMPERSONATOR: &str = "Clever Impersonator";
const FLESH: &str = "Flesh Duplicate";
const CLONES_C: &[&str] = &[MOCKINGBIRD, IMAGE];
const CLONES_D: &[&str] = &[IMAGE, FLESH, MOCKINGBIRD, IMPERSONATOR];
const ALL_CLONES: &[&str] = &[MOCKINGBIRD, IMAGE, FLESH, IMPERSONATOR];

/// a land other than Cradle and Gates that taps for {U} (the one Cub earthbends)
fn blue_land(g: &Game, p: PlayerId) -> bool {
    g.player(p).lands.iter().any(|&l| {
        g.perm(l).on_bf
            && !matches!(cname(g, g.perm(l).card()), CRADLE | GATES | super::t6_green2::RITES)
            && crate::engine::mana::land_colors_now(g, p, l).has('U')
    })
}

/// the cheapest clone in hand among these names
fn clone_in_hand(g: &Game, p: PlayerId, names: &[&str]) -> Option<CardId> {
    names.iter().find_map(|n| hand_card(g, p, &[n]))
}

/// Cub: Some(None) it is out, Some(Some(card)) it is in hand, None neither
fn cub_card(g: &Game, p: PlayerId) -> Option<Option<CardId>> {
    if bf(g, p, CUB).is_some() { Some(None) } else { hand_card(g, p, &[CUB]).map(Some) }
}

/// the spells line C needs besides the clone: (Cub, Oboro, clone)
fn plan_c(g: &Game, p: PlayerId) -> Option<()> {
    land_on_bf(g, p, CRADLE)?;
    if finisher(g, p).is_none() || !blue_land(g, p) {
        return None;
    }
    let cub = cub_card(g, p)?;
    let oboro = if bf(g, p, OBORO).is_some() { None } else { Some(hand_card(g, p, &[OBORO])?) };
    let clone = clone_in_hand(g, p, CLONES_C)?;
    // Cub, the earthbent land and Oboro feed Cradle: 3 for Mockingbird ({U}), 4 for Phantasmal Image ({1}{U})
    let need = if cname(g, clone) == MOCKINGBIRD { 3 } else { 4 };
    let n = creatures(g, p) + usize::from(cub.is_some()) + usize::from(oboro.is_some()) + 1;
    let mut casts: Vec<CardId> = cub.into_iter().chain(oboro).collect();
    casts.push(clone);
    // 2 spare: the first Oboro activation before Cradle is tapped
    (n >= need && affordable(g, p, &casts, 2)).then_some(())
}

fn ready_plan_c(g: &Game, p: PlayerId) -> Ready {
    ready_with(plan_c(g, p).is_some())
}

fn cast_cub(g: &mut Game, p: PlayerId) -> Res<bool> {
    if bf(g, p, CUB).is_some() {
        return Ok(true);
    }
    let Some(c) = hand_card(g, p, &[CUB]) else { return Ok(false) };
    cast_named(g, p, c)
}

fn cast_oboro_c(g: &mut Game, p: PlayerId) -> Res<bool> {
    if bf(g, p, OBORO).is_some() {
        return Ok(true);
    }
    let Some(c) = hand_card(g, p, &[OBORO]) else { return Ok(false) };
    cast_named(g, p, c)
}

/// a clone is a copy, so it is not found by its own name: it was cast if it left the hand without being countered
fn cast_clone_of(g: &mut Game, p: PlayerId, names: &[&str]) -> Res<bool> {
    let Some(c) = clone_in_hand(g, p, names) else { return Ok(false) };
    crate::ai::brain::do_cast(g, p, c, None)?;
    Ok(!g.player(p).hand.contains(&c) && !g.player(p).gy.contains(&c))
}

fn cast_clone_c(g: &mut Game, p: PlayerId) -> Res<bool> {
    cast_clone_of(g, p, CLONES_C)
}

fn cast_clone_d(g: &mut Game, p: PlayerId) -> Res<bool> {
    cast_clone_of(g, p, CLONES_D)
}

fn c_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    let mut v: Vec<PermId> = bf_any(g, p, &[OBORO, CUB]).into_iter().collect();
    if creatures(g, p) <= 4 {
        // at the minimum every creature is a key piece
        v.extend(g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)));
    }
    v
}

fn plan_d(g: &Game, p: PlayerId) -> Option<()> {
    if finisher(g, p).is_none() || !blue_land(g, p) {
        return None;
    }
    let cub = cub_card(g, p)?;
    let clone = clone_in_hand(g, p, CLONES_D)?;
    hand_card(g, p, &[WITNESS])?;
    hand_card(g, p, &[SNAP])?;
    let mut casts: Vec<CardId> = cub.into_iter().collect();
    casts.push(clone);
    // two lands stay untapped to be earthbent: they pay for Witness and Snap (6 in, 5 out)
    affordable(g, p, &casts, 2).then_some(())
}

fn ready_plan_d(g: &Game, p: PlayerId) -> Ready {
    ready_with(plan_d(g, p).is_some())
}

/// the two creature-lands pay for Witness and Snap (the sim has no earthbend), so these casts are the shortcut's, free
fn cast_witness_d(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(w) = hand_card(g, p, &[WITNESS]) else { return Ok(false) };
    cast_abstract(g, p, w, true)
}

fn snap_witness_d(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(s) = hand_card(g, p, &[SNAP]) else { return Ok(false) };
    cast_abstract(g, p, s, true)
}

fn d_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| cname(g, c) == CUB))
        .collect()
}

// ================================================================== the combos
/// the Tier 6 combos of this module (chained after the Godo ones by `combos::all`)
pub static COMBOS: [Combo; 9] = [
    combo(
        "Preston + Felidar Guardian + Radiant Solar (flicker spell)",
        &[QUIET, &[PRESTON], &[FELIDAR], &[SOLAR], FLICKERS],
        ready_dack_spell,
        &[S_DACK_PIECES, S_DACK_FLICKER, S_DACK_LOOP],
    ),
    combo(
        "Preston + Felidar Guardian + Radiant Solar (loop running)",
        &[QUIET, &[PRESTON], &[FELIDAR], &[SOLAR]],
        ready_dack_running,
        &[S_DACK_PIECES, S_DACK_LOOP],
    ),
    combo(
        "Dack Fayden: the first flip wins the turn (35 piles)",
        &[QUIET, &[DACK]],
        ready_dack_flip,
        &[
            S_FLIP_PIECES,
            S_FLIP_SPELL,
            S_FLIP_FETCH,
            S_FLIP_LOOP,
            S_FLIP_PIECES,
            S_FLIP_DACK2,
            S_FLIP_PRESTON,
            S_FLIP_SOLAR,
            S_FLIP_PIECES,
            S_FLIP_LOOP,
        ],
    ),
    combo(
        "Ioreth + Marvin + a legend that taps for mana (DDR)",
        &[QUIET, &[IORETH], &[MARVIN], &[SELVALA, SHANG, "Sisay, Weatherlight Captain"], &[DOOM]],
        ready_ddr,
        &[
            S_FETCH_DDR,
            S_FETCH_DDR,
            S_FETCH_DDR,
            S_FETCH_DDR,
            S_CAST_NEXT_DDR,
            S_CAST_NEXT_DDR,
            S_CAST_NEXT_DDR,
            S_CAST_NEXT_DDR,
            S_CAST_NEXT_DDR,
            pieces("Ioreth, Marvin and the legend survive", ddr_keys),
            ability("Ioreth untaps Marvin and the legend (again and again)", ioreth_src, (0, "")),
            S_DOOM_IN,
            S_DOOM_PING,
        ],
    ),
    combo(
        "Derevi + Emiel + Gaea's Cradle / Bloom Tender (Flash Unicorn)",
        &[QUIET, &[DEREVI], &[EMIEL], &[CRADLE, TENDER], &[DOOM]],
        ready_unicorn,
        &[
            S_FETCH_UNI,
            S_FETCH_UNI,
            S_FETCH_UNI,
            S_CAST_NEXT_UNI,
            S_CAST_NEXT_UNI,
            pieces("Derevi, Emiel and the mana source survive", uni_keys),
            ability("Emiel blinks Derevi ({3})", emiel_src, (3, "")),
            trigger("Derevi's enter trigger untaps the mana source (again and again)", derevi_src),
            S_DOOM_IN,
            S_DOOM_PING,
        ],
    ),
    combo(
        "Oboro Breezecaller + Gaea's Cradle + Talon Gates of Madara",
        &[QUIET, &[OBORO], &[CRADLE], &[GATES], &[THRASIOS]],
        ready_plan_a,
        &[
            step("cast Oboro Breezecaller", StepKind::Spell, cast_oboro),
            pieces("Oboro and the creatures that feed Cradle survive", a_keys),
            Step {
                label: "Oboro returns Talon Gates to hand: untap Cradle (again and again)",
                kind: StepKind::Ability,
                src: oboro_src,
                pay: (2, ""),
                run: done,
                keys: no_keys,
            },
            step("Talon Gates comes back for {4}: the table's creatures phase out", StepKind::Engine, gates_loop),
            S_FIN_THRASIOS,
            S_FIN_DRAW,
            S_FIN_SPELL,
            S_FIN_ABILITY,
        ],
    ),
    combo(
        "Eternal Witness + Snap + Gaea's Cradle",
        &[QUIET, &[WITNESS], &[SNAP], &[CRADLE], &[THRASIOS]],
        ready_plan_b,
        &[
            step("cast Eternal Witness", StepKind::Spell, cast_witness),
            step("Snap on Eternal Witness (untap two lands)", StepKind::Spell, snap_witness),
            trigger("Eternal Witness returns Snap (again and again)", witness_src),
            pieces("the creatures that feed Cradle survive", b_keys),
            S_FIN_THRASIOS,
            S_FIN_DRAW,
            S_FIN_SPELL,
            S_FIN_ABILITY,
        ],
    ),
    combo(
        "Badgermole Cub + clone + Oboro Breezecaller + Gaea's Cradle",
        &[QUIET, &[CUB], ALL_CLONES, &[OBORO], &[CRADLE], &[THRASIOS]],
        ready_plan_c,
        &[
            step("cast Badgermole Cub (earthbend 1)", StepKind::Spell, cast_cub),
            step("cast Oboro Breezecaller", StepKind::Spell, cast_oboro_c),
            pieces("Cub, Oboro and the creatures that feed Cradle survive", c_keys),
            step("cast Mockingbird / Phantasmal Image as a copy of the earthbent land", StepKind::Spell, cast_clone_c),
            Step {
                label: "Oboro returns the copy to hand: untap Cradle (again and again)",
                kind: StepKind::Ability,
                src: oboro_src,
                pay: (2, ""),
                run: done,
                keys: no_keys,
            },
            S_FIN_THRASIOS,
            S_FIN_DRAW,
            S_FIN_SPELL,
            S_FIN_ABILITY,
        ],
    ),
    combo(
        "Badgermole Cub + clone + Eternal Witness + Snap",
        &[QUIET, &[CUB], ALL_CLONES, &[WITNESS], &[SNAP], &[THRASIOS]],
        ready_plan_d,
        &[
            step("cast Badgermole Cub (earthbend 1)", StepKind::Spell, cast_cub),
            step("cast a clone as a second Badgermole Cub (earthbend 1)", StepKind::Spell, cast_clone_d),
            pieces("both Badgermole Cubs survive", d_keys),
            step("cast Eternal Witness", StepKind::Spell, cast_witness_d),
            step("Snap on Eternal Witness (untap two creature-lands)", StepKind::Spell, snap_witness_d),
            trigger("Eternal Witness returns Snap (again and again)", witness_src),
            S_FIN_THRASIOS,
            S_FIN_DRAW,
            S_FIN_SPELL,
            S_FIN_ABILITY,
        ],
    ),
];
