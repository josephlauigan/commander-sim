//! Tier 6 combo lines, part 3 (plan row 18): the declared steps of two base combos, the cEDH lines the specs name that
//! were missing, and the Godo leftovers combo1 did not build. Framework: `combos::Combo` / `combos::Step` (each step
//! has the answer window that fits it; an answered step stops the combo, the pieces already played stay).
//!
//! * Base combos with steps (their `Combo` entries in `combos::BASE` point at `ORACLE_STEPS` / `KINNAN_BASALT_STEPS`):
//!   Thassa's Oracle + Demonic Consultation / Tainted Pact (cast Oracle, cast the exile spell in response, the
//!   Oracle trigger is a trigger window), Kinnan + Basalt Monolith (the loop's pieces must survive, then the outlet).
//! * Kinnan + Basalt with an outlet when no {U}{G} can be paid for Kinnan's own ability: Thrasios (draw the deck) or
//!   Mirage Mirror (the loop's colourless becomes {U}{G}, then Kinnan).
//! * Underworld Breach + Lion's Eye Diamond + Brain Freeze: the loop is played (real escapes, real storm mills, each
//!   cast counterable), finishing with an escaped Thassa's Oracle on an empty library or Brain Freeze on the table.
//! * Ad Nauseam: a real Ad Nauseam, then whichever line the cards drawn made ready (it needs the mana left).
//! * Hullbreaker Horror with Kinnan and two rocks that cast between each other (Mox Amber / Lotus Petal ...).
//! * Godo: Goblin Welder + Goblin Engineer, the rebuild with Godo already out (Cursed Mirror / Sculpting Steel / Jaxis /
//!   a token copy spell), and Basalt Monolith with a copied opposing Kinnan.

use super::combos::{self, Combo, QUIET, Ready, Step, StepKind, done, no_keys, no_src};
use super::t6_combos_godo as gd;
use super::t6_combos_godo::{Cost, affordable, bf, cost, in_hand, in_lib, nm, ok_ready};
use super::t6_godo::welder_go;
use crate::engine::cast::{cast_card, castable};
use crate::engine::mana::{can_pay, pay};
use crate::engine::zones::die;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};

const ORACLE: &str = "Thassa's Oracle";
const CONSULT: &str = "Demonic Consultation";
const PACT: &str = "Tainted Pact";
const KINNAN: &str = "Kinnan, Bonder Prodigy";
const BASALT: &str = "Basalt Monolith";
const THRASIOS: &str = "Thrasios, Triton Hero";
const MIRAGE: &str = "Mirage Mirror";
const BREACH: &str = "Underworld Breach";
const LED: &str = "Lion's Eye Diamond";
const BRAIN_FREEZE: &str = "Brain Freeze";
const AD: &str = "Ad Nauseam";
const HORROR: &str = "Hullbreaker Horror";
const WELDER: &str = "Goblin Welder";
const ENGINEER: &str = "Goblin Engineer";
const SHIMMER: &str = "Heat Shimmer";

// ------------------------------------------------------------------ helpers
fn no() -> Ready {
    (false, vec![], vec![])
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).gy.iter().copied().find(|&c| cname(g, c) == name)
}

fn in_hand_or_lib(g: &Game, p: PlayerId, name: &str) -> bool {
    in_hand(g, p, name).is_some() || in_lib(g, p, name)
}

/// pay for a card of this name in hand and cast it (the normal counter window); false if it can't be cast or was
/// countered
fn cast_hand(g: &mut Game, p: PlayerId, name: &str) -> Res<bool> {
    let Some(c) = in_hand(g, p, name) else { return Ok(false) };
    if !castable(g, p, c, "hand") {
        return Ok(false);
    }
    let (gn, pips) = cost(g, p, c);
    if !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    cast_card(g, p, c, "hand", Ctx::default())
}

/// a step that casts this card unless it is already on the battlefield
fn cast_unless_out(g: &mut Game, p: PlayerId, name: &str) -> Res<bool> {
    if bf(g, p, name).is_some() {
        return Ok(true);
    }
    Ok(cast_hand(g, p, name)? && bf(g, p, name).is_some())
}

const fn step(label: &'static str, kind: StepKind, run: fn(&mut Game, PlayerId) -> Res<bool>) -> Step {
    Step { label, kind, src: no_src, pay: (0, ""), run, keys: no_keys }
}

const fn trig(
    label: &'static str,
    kind: StepKind,
    src: fn(&Game, PlayerId) -> Option<PermId>,
    run: fn(&mut Game, PlayerId) -> Res<bool>,
) -> Step {
    Step { label, kind, src, pay: (0, ""), run, keys: no_keys }
}

const fn held(label: &'static str, keys: fn(&Game, PlayerId) -> Vec<PermId>) -> Step {
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

fn oracle_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, ORACLE)
}

fn kinnan_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, KINNAN)
}

fn horror_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, HORROR)
}

fn win_alt(g: &mut Game, p: PlayerId) -> Res {
    crate::ai::win(g, p, "combo", Some(false))
}

// ================================================================== Thassa's Oracle + Consultation / Pact
fn cast_oracle(g: &mut Game, p: PlayerId) -> Res<bool> {
    g.combo_spell = true; // its own enter effect waits: the exile spell comes next
    let r = cast_hand(g, p, ORACLE);
    g.combo_spell = false;
    Ok(r? && bf(g, p, ORACLE).is_some())
}

/// Demonic Consultation (naming a card not in the library) or Tainted Pact (taking nothing): the library is exiled
fn cast_exiler(g: &mut Game, p: PlayerId) -> Res<bool> {
    let name = if in_hand(g, p, CONSULT).is_some() { CONSULT } else { PACT };
    if !cast_hand(g, p, name)? {
        return Ok(false);
    }
    let lib = std::mem::take(&mut g.player_mut(p).library);
    g.player_mut(p).exile.extend(lib);
    crate::glog!(g, "    {} exiles its library", g.player(p).name);
    Ok(true)
}

pub const ORACLE_STEPS: [Step; 3] = [
    step("cast Thassa's Oracle", StepKind::Spell, cast_oracle),
    step("Demonic Consultation / Tainted Pact in response exiles the library", StepKind::Spell, cast_exiler),
    trig("Thassa's Oracle's trigger: X is at least the library size", StepKind::Trigger, oracle_src, done),
];

/// the Oracle trigger resolved with an empty library: an alternate win
pub fn oracle_finish(g: &mut Game, p: PlayerId) -> Res {
    win_alt(g, p)
}

// ================================================================== Kinnan + Basalt Monolith
fn kinnan_basalt_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    [bf(g, p, KINNAN), bf(g, p, BASALT)].into_iter().flatten().collect()
}

pub const KINNAN_BASALT_STEPS: [Step; 2] = [
    held("Kinnan and Basalt Monolith stay in play through the loop", kinnan_basalt_keys),
    trig("Kinnan's ability puts a creature onto the battlefield (the outlet)", StepKind::Ability, kinnan_src, done),
];

/// can p pay {U}{G} (what Kinnan's own ability needs besides the loop's colourless)?
pub fn can_pay_ug(g: &Game, p: PlayerId) -> bool {
    can_pay(g, p, 0, "UG", false)
}

// ------------------------------------------------------------------ the other outlets
fn outlet_ready(g: &Game, p: PlayerId, outlet: &str) -> Ready {
    let (Some(k), Some(b), Some(o)) = (bf(g, p, KINNAN), bf(g, p, BASALT), bf(g, p, outlet)) else { return no() };
    if can_pay_ug(g, p) {
        return no(); // Kinnan's own ability is the outlet then (the base combo)
    }
    (true, vec![k, b, o], vec![])
}

fn ready_thrasios(g: &Game, p: PlayerId) -> Ready {
    outlet_ready(g, p, THRASIOS)
}

fn ready_mirage(g: &Game, p: PlayerId) -> Ready {
    outlet_ready(g, p, MIRAGE)
}

fn thrasios_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, THRASIOS)
}

fn mirage_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, MIRAGE)
}

fn thrasios_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    [bf(g, p, KINNAN), bf(g, p, BASALT), bf(g, p, THRASIOS)].into_iter().flatten().collect()
}

fn mirage_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    [bf(g, p, KINNAN), bf(g, p, BASALT), bf(g, p, MIRAGE)].into_iter().flatten().collect()
}

const THRASIOS_STEPS: [Step; 2] = [
    held("Kinnan, Basalt Monolith and Thrasios stay in play", thrasios_keys),
    trig("Thrasios: {4}, scry and reveal, again and again (draw the deck)", StepKind::Ability, thrasios_src, done),
];

const MIRAGE_STEPS: [Step; 3] = [
    held("Kinnan, Basalt Monolith and Mirage Mirror stay in play", mirage_keys),
    trig(
        "Mirage Mirror copies a dual land, then Basalt Monolith: infinite {U}{G}",
        StepKind::Ability,
        mirage_src,
        done,
    ),
    trig("Kinnan's ability puts every creature onto the battlefield", StepKind::Ability, kinnan_src, done),
];

// ================================================================== Underworld Breach + LED + Brain Freeze
const LINE: [&str; 4] = [BREACH, LED, BRAIN_FREEZE, ORACLE];

/// cards in hand and graveyard that can pay an escape cost (not the line's own cards)
fn fuel(g: &Game, p: PlayerId) -> usize {
    let pl = g.player(p);
    pl.hand.iter().chain(pl.gy.iter()).filter(|&&c| !LINE.contains(&cname(g, c))).count()
}

fn gy_castable(g: &Game, p: PlayerId, name: &str) -> bool {
    in_gy(g, p, name).is_some_and(|c| castable(g, p, c, "gy"))
}

/// where the Brain Freeze is: in hand (LED discards it into the graveyard) or already in the graveyard
fn bf_available(g: &Game, p: PlayerId) -> bool {
    in_hand(g, p, BRAIN_FREEZE).is_some() || gy_castable(g, p, BRAIN_FREEZE)
}

fn oracle_available(g: &Game, p: PlayerId) -> bool {
    in_hand(g, p, ORACLE).is_some() || gy_castable(g, p, ORACLE)
}

fn breach_ready(g: &Game, p: PlayerId, with_oracle: bool) -> Ready {
    let breach = bf(g, p, BREACH).is_some()
        || in_hand(g, p, BREACH).is_some_and(|c| castable(g, p, c, "hand") && can_pay(g, p, 1, "R", false));
    let led = bf(g, p, LED).is_some() || in_hand(g, p, LED).is_some_and(|c| castable(g, p, c, "hand"));
    if !breach || !led || !bf_available(g, p) || fuel(g, p) < 3 || oracle_available(g, p) != with_oracle {
        return no();
    }
    ok_ready()
}

fn ready_breach_oracle(g: &Game, p: PlayerId) -> Ready {
    breach_ready(g, p, true)
}

fn ready_breach_table(g: &Game, p: PlayerId) -> Ready {
    breach_ready(g, p, false)
}

/// escape a card of this name from the graveyard with Underworld Breach: exile three other cards (the line's own
/// cards last), then the real cast (counter window, effects); the mana comes from Lion's Eye Diamond
fn escape(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> Res<bool> {
    let Some(c) = in_gy(g, p, name) else { return Ok(false) };
    if !castable(g, p, c, "gy") {
        return Ok(false);
    }
    let mut others: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&x| x != c).collect();
    others.sort_by_key(|&x| LINE.contains(&cname(g, x)));
    if others.len() < 3 {
        return Ok(false);
    }
    for &x in others.iter().take(3) {
        let pl = g.player_mut(p);
        if let Some(i) = pl.gy.iter().position(|&y| y == x) {
            pl.gy.remove(i);
            pl.exile.push(x);
        }
    }
    crate::glog!(g, "  {} escapes {} with Underworld Breach", g.player(p).name, name);
    cast_card(g, p, c, "escape", ctx)
}

/// sacrifice Lion's Eye Diamond (a mana ability: nothing answers it): the hand is discarded, UUU is made
fn crack_led(g: &mut Game, p: PlayerId) -> Res {
    if let Some(m) = bf(g, p, LED) {
        die(g, m, "sac")?;
    }
    let hand = std::mem::take(&mut g.player_mut(p).hand);
    g.player_mut(p).gy.extend(hand);
    Ok(())
}

/// escape Lion's Eye Diamond again and crack it
fn led_round(g: &mut Game, p: PlayerId) -> Res<bool> {
    if !escape(g, p, LED, Ctx::default())? {
        return Ok(false);
    }
    crack_led(g, p)?;
    Ok(true)
}

fn brain_freeze_at(g: &mut Game, p: PlayerId, whom: Vec<PlayerId>) -> Res<bool> {
    escape(g, p, BRAIN_FREEZE, Ctx { mill: whom, ..Ctx::default() })
}

fn cast_breach(g: &mut Game, p: PlayerId) -> Res<bool> {
    cast_unless_out(g, p, BREACH)
}

fn cast_led(g: &mut Game, p: PlayerId) -> Res<bool> {
    cast_unless_out(g, p, LED)
}

fn breach_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    bf(g, p, BREACH).into_iter().collect()
}

/// crack LED, then the first Brain Freeze on yourself (storm: every copy mills three)
fn first_freeze(g: &mut Game, p: PlayerId) -> Res<bool> {
    crack_led(g, p)?;
    brain_freeze_at(g, p, vec![p; 200])
}

/// repeat LED, Brain Freeze at yourself until the library is empty (Oracle line) or the graveyard holds fuel for the
/// table (Brain Freeze line)
fn self_loop(g: &mut Game, p: PlayerId, to_empty: bool) -> Res<bool> {
    for _ in 0..10 {
        let done = if to_empty { g.player(p).library.is_empty() } else { g.player(p).gy.len() >= 30 };
        if done {
            return Ok(true);
        }
        if !led_round(g, p)? || !brain_freeze_at(g, p, vec![p; 200])? {
            return Ok(false);
        }
    }
    Ok(if to_empty { g.player(p).library.is_empty() } else { g.player(p).gy.len() >= 30 })
}

fn self_loop_empty(g: &mut Game, p: PlayerId) -> Res<bool> {
    self_loop(g, p, true)
}

fn self_loop_fuel(g: &mut Game, p: PlayerId) -> Res<bool> {
    self_loop(g, p, false)
}

/// escape Oracle on the empty library: its enter effect wins (a trigger the table can still answer)
fn escape_oracle(g: &mut Game, p: PlayerId) -> Res<bool> {
    if !led_round(g, p)? {
        return Ok(false);
    }
    Ok(escape(g, p, ORACLE, Ctx::default())? || g.over)
}

/// the Oracle's trigger resolved (the win ends the game); anything else means it was answered
fn oracle_won(g: &mut Game, _p: PlayerId) -> Res<bool> {
    Ok(g.over)
}

/// Brain Freeze on the table, round after round, until every opponent's library is empty
fn table_loop(g: &mut Game, p: PlayerId) -> Res<bool> {
    for _ in 0..12 {
        let libs: Vec<(PlayerId, usize)> =
            g.opps(p).filter(|&q| g.player(q).alive).map(|q| (q, g.player(q).library.len())).collect();
        if libs.iter().all(|x| x.1 == 0) {
            return Ok(true);
        }
        if !led_round(g, p)? {
            return Ok(false);
        }
        let mut whom = vec![];
        for (q, n) in libs {
            whom.extend(std::iter::repeat_n(q, n.div_ceil(3)));
        }
        if !brain_freeze_at(g, p, whom)? {
            return Ok(false);
        }
    }
    Ok(g.opps(p).all(|q| g.player(q).library.is_empty()))
}

/// the opponents' libraries are empty: they lose at their next draw
fn table_finish(g: &mut Game, p: PlayerId) -> Res {
    win_alt(g, p)
}

const BREACH_OPEN: [Step; 5] = [
    step("cast Underworld Breach", StepKind::Spell, cast_breach),
    held("Underworld Breach stays in play (enchantment removal before the first escape)", breach_keys),
    step("cast Lion's Eye Diamond", StepKind::Spell, cast_led),
    step("crack Lion's Eye Diamond, escape Brain Freeze on yourself", StepKind::Spell, first_freeze),
    step(
        "Lion's Eye Diamond and Brain Freeze escape again until the library is empty",
        StepKind::Spell,
        self_loop_empty,
    ),
];

const BREACH_ORACLE_STEPS: [Step; 7] = [
    BREACH_OPEN[0],
    BREACH_OPEN[1],
    BREACH_OPEN[2],
    BREACH_OPEN[3],
    BREACH_OPEN[4],
    step("escape Lion's Eye Diamond and Thassa's Oracle", StepKind::Spell, escape_oracle),
    step("Thassa's Oracle's trigger on an empty library", StepKind::Engine, oracle_won),
];

const BREACH_TABLE_STEPS: [Step; 6] = [
    BREACH_OPEN[0],
    BREACH_OPEN[1],
    BREACH_OPEN[2],
    BREACH_OPEN[3],
    step("fuel the graveyard, then Brain Freeze at the table again and again", StepKind::Spell, self_loop_fuel),
    step("Brain Freeze mills every opponent's library", StepKind::Spell, table_loop),
];

// ================================================================== Ad Nauseam into a win
fn ad_ready(g: &Game, p: PlayerId) -> Ready {
    let Some(ad) = in_hand(g, p, AD) else { return no() };
    if g.player(p).life < 26 || !castable(g, p, ad, "hand") {
        return no();
    }
    let oracle_line = in_hand_or_lib(g, p, ORACLE)
        && (in_hand_or_lib(g, p, CONSULT) || in_hand_or_lib(g, p, PACT))
        && can_pay(g, p, 3, "BBUUB", false);
    let breach_line = in_hand_or_lib(g, p, BREACH)
        && in_hand_or_lib(g, p, LED)
        && in_hand_or_lib(g, p, BRAIN_FREEZE)
        && can_pay(g, p, 4, "BBR", false);
    if oracle_line || breach_line { ok_ready() } else { no() }
}

fn cast_ad(g: &mut Game, p: PlayerId) -> Res<bool> {
    cast_hand(g, p, AD)
}

/// the cards Ad Nauseam drew made a line ready (with the mana that is left): go for it
fn found_a_line(g: &mut Game, p: PlayerId) -> Res<bool> {
    for part in ["Thassa's Oracle + Demonic", "Underworld Breach + Lion's Eye Diamond + Brain Freeze (Thassa"] {
        if let Some(i) = combos::combo_index(part)
            && combos::ready(g, p, i)
        {
            if combos::attempt(g, p, i as i64)? {
                return Ok(true);
            }
        }
    }
    crate::glog!(g, "    Ad Nauseam found no line it can pay for");
    Ok(false)
}

const AD_STEPS: [Step; 2] = [
    step("cast Ad Nauseam", StepKind::Spell, cast_ad),
    step("play the line the cards drawn made ready", StepKind::Engine, found_a_line),
];

/// the line found (if any) ran its own steps and finish; nothing more to do
fn ad_finish(_g: &mut Game, _p: PlayerId) -> Res {
    Ok(())
}

// ================================================================== Hullbreaker Horror + two rocks
/// rocks that make mana when cast between each other with the Horror out (the NBC primer's list)
const LOOP_A: &[&str] = &["Mox Opal", "Mox Amber", "Sol Ring", "Grim Monolith", "Mana Vault"];
const LOOP_B: &[&str] =
    &["Basalt Monolith", "Chrome Mox", "Lotus Petal", "Fellwar Stone", "Arcane Signet", "Talisman of Curiosity"];

fn loop_pair(a: &str, b: &str) -> bool {
    a != b
        && ((LOOP_A.contains(&a) && (LOOP_A.contains(&b) || LOOP_B.contains(&b)))
            || (LOOP_A.contains(&b) && LOOP_B.contains(&a)))
}

/// (the rock on the battlefield, the rock in hand that is cast and bounces it)
fn rock_pair(g: &Game, p: PlayerId) -> Option<(PermId, CardId)> {
    for &m in &g.player(p).perms {
        if !live(g, m) {
            continue;
        }
        for &c in &g.player(p).hand {
            if loop_pair(nm(g, m), cname(g, c)) && castable(g, p, c, "hand") {
                return Some((m, c));
            }
        }
    }
    None
}

fn ready_horror(g: &Game, p: PlayerId) -> Ready {
    let Some(k) = bf(g, p, KINNAN) else { return no() };
    let Some((_, rock)) = rock_pair(g, p) else { return no() };
    let mut parts: Vec<Cost> = vec![cost(g, p, rock)];
    let horror = bf(g, p, HORROR);
    if horror.is_none() {
        let Some(h) = in_hand(g, p, HORROR) else { return no() };
        if !castable(g, p, h, "hand") {
            return no();
        }
        parts.push(cost(g, p, h));
    }
    if !affordable(g, p, &parts) {
        return no();
    }
    (true, [Some(k), horror].into_iter().flatten().collect(), vec![])
}

fn horror_in(g: &mut Game, p: PlayerId) -> Res<bool> {
    cast_unless_out(g, p, HORROR)
}

fn horror_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    [bf(g, p, KINNAN), bf(g, p, HORROR)].into_iter().flatten().collect()
}

/// cast the rock in hand; Hullbreaker Horror's trigger bounces the other one
fn cast_loop_rock(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some((_, c)) = rock_pair(g, p) else { return Ok(false) };
    let name = cname(g, c).to_string();
    cast_hand(g, p, &name)
}

const HORROR_STEPS: [Step; 5] = [
    step("cast Hullbreaker Horror (flash, can't be countered)", StepKind::Spell, horror_in),
    held("Kinnan and Hullbreaker Horror stay in play", horror_keys),
    step("cast the first rock (a counter on any loop spell ends the loop)", StepKind::Spell, cast_loop_rock),
    trig(
        "Hullbreaker Horror's trigger returns the other rock to its owner's hand",
        StepKind::Trigger,
        horror_src,
        done,
    ),
    trig("Kinnan's ability puts the winner onto the battlefield (infinite mana)", StepKind::Ability, kinnan_src, done),
];

// ================================================================== Godo: Goblin Welder + Goblin Engineer
fn is_art(g: &Game, m: PermId) -> bool {
    super::t5::is_artifact(g, m)
}

/// an artifact the Welder can give up that is not one of the two Equipment
fn spare_artifact(g: &Game, p: PlayerId) -> bool {
    g.player(p)
        .perms
        .iter()
        .any(|&m| live(g, m) && is_art(g, m) && !matches!(nm(g, m), "Helm of the Host" | "Hammer of Nazahn"))
}

fn welder_ready_piece(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, WELDER).filter(|&m| !g.perm(m).tapped && !g.perm(m).sick)
}

fn ready_welder(g: &Game, p: PlayerId) -> Ready {
    let Some(godo) = gd::godo_base(g, p) else { return no() };
    let Some(eng) = in_hand(g, p, ENGINEER) else { return no() };
    if welder_ready_piece(g, p).is_none()
        || !spare_artifact(g, p)
        || !in_lib(g, p, gd::HAMMER)
        || bf(g, p, gd::HAMMER).is_some()
        || !castable(g, p, eng, "hand")
        || !affordable(g, p, &[godo, cost(g, p, eng)])
    {
        return no();
    }
    ok_ready()
}

fn cast_engineer(g: &mut Game, p: PlayerId) -> Res<bool> {
    Ok(cast_hand(g, p, ENGINEER)? && bf(g, p, ENGINEER).is_some())
}

/// the Engineer's enter trigger (the engine ran it, with its answer window) put Helm into the graveyard
fn helm_in_gy(g: &mut Game, p: PlayerId) -> Res<bool> {
    Ok(in_gy(g, p, gd::HELM).is_some() && bf(g, p, WELDER).is_some())
}

/// the Welder swaps a spent rock for the Helm in the graveyard (the engine's own ability and its window); Hammer's
/// trigger attaches the Helm as it enters
fn welder_swap(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(w) = bf(g, p, WELDER) else { return Ok(false) };
    welder_go(g, w, p, 0)?;
    Ok(bf(g, p, gd::HELM).is_some() && gd::helm_on_godo(g, p))
}

const WELDER_STEPS: [Step; 7] = [
    step("cast Goblin Engineer", StepKind::Spell, cast_engineer),
    step("Goblin Engineer's enter trigger puts Helm of the Host into the graveyard", StepKind::Engine, helm_in_gy),
    gd::S_CAST,
    gd::S_FETCH_HAMMER,
    step("Goblin Welder swaps a spent rock for Helm of the Host; Hammer attaches it", StepKind::Engine, welder_swap),
    gd::S_COMBAT,
    gd::S_LOOP,
];

// ================================================================== Godo: the rebuild with Godo already out
/// Godo is on the battlefield with Hammer of Nazahn, Helm still in the library (his first trigger was answered): a copy
/// of Godo has an enter trigger of its own that fetches Helm
fn rebuild_base(g: &Game, p: PlayerId) -> bool {
    bf(g, p, gd::GODO).is_some()
        && bf(g, p, gd::HAMMER).is_some()
        && bf(g, p, gd::HELM).is_none()
        && in_lib(g, p, gd::HELM)
}

fn ready_rebuild_mirror(g: &Game, p: PlayerId) -> Ready {
    let (true, Some(c)) = (rebuild_base(g, p), gd::mirror_card(g, p)) else { return no() };
    if affordable(g, p, &[cost(g, p, c)]) { ok_ready() } else { no() }
}

fn ready_rebuild_jaxis(g: &Game, p: PlayerId) -> Ready {
    let (true, Some(j)) = (rebuild_base(g, p), gd::jaxis_plan(g, p)) else { return no() };
    if affordable(g, p, &[j]) { ok_ready() } else { no() }
}

fn ready_rebuild_spell(g: &Game, p: PlayerId) -> Ready {
    let (true, Some(c)) = (rebuild_base(g, p), gd::token_copier(g, p)) else { return no() };
    if affordable(g, p, &[cost(g, p, c)]) { ok_ready() } else { no() }
}

// ================================================================== Godo: Basalt Monolith + a copied Kinnan
fn opp_kinnan(g: &Game, p: PlayerId) -> Option<PermId> {
    g.opps(p).find_map(|q| bf(g, q, KINNAN))
}

/// Heat Shimmer (a token copy of any creature) or Cursed Mirror (becomes a copy)
fn kinnan_copier(g: &Game, p: PlayerId) -> Option<CardId> {
    in_hand(g, p, SHIMMER).or_else(|| in_hand(g, p, gd::MIRROR))
}

fn ready_basalt_kinnan(g: &Game, p: PlayerId) -> Ready {
    let Some(godo) = gd::godo_base(g, p) else { return no() };
    let (Some(_), Some(_), Some(c)) = (bf(g, p, BASALT), opp_kinnan(g, p), kinnan_copier(g, p)) else { return no() };
    // the copier is paid from lands; Godo's cost is paid from the loop (its {R} too)
    let _ = godo;
    if !castable(g, p, c, "hand") || !affordable(g, p, &[cost(g, p, c)]) {
        return no();
    }
    ok_ready()
}

fn copy_kinnan(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(k) = opp_kinnan(g, p) else { return Ok(false) };
    if let Some(c) = in_hand(g, p, SHIMMER) {
        return Ok(gd::cast_copy_spell(g, p, c, k, true)? && bf(g, p, KINNAN).is_some());
    }
    Ok(cast_hand(g, p, gd::MIRROR)? && bf(g, p, gd::MIRROR).is_some())
}

fn copied_kinnan_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    [bf(g, p, KINNAN).or_else(|| bf(g, p, gd::MIRROR)), bf(g, p, BASALT)].into_iter().flatten().collect()
}

/// tap Basalt for {C}{C}{C}, Kinnan's copy adds {C}{C}{C}, pay {3} to untap: unlimited colourless mana
fn infinite_colourless(g: &mut Game, p: PlayerId) -> Res<bool> {
    g.player_mut(p).floating.any += 60;
    crate::glog!(g, "    Basalt Monolith + Kinnan: unlimited mana");
    Ok(true)
}

const S_EQUIP_LOOP: Step = Step {
    label: "equip Helm of the Host to Godo (paid from the loop)",
    kind: StepKind::Ability,
    src: gd::helm_src,
    pay: (0, ""),
    run: gd::equip_helm,
    keys: no_keys,
};

const BASALT_KINNAN_STEPS: [Step; 8] = [
    step("a copy of the opposing Kinnan (Heat Shimmer / Cursed Mirror)", StepKind::Spell, copy_kinnan),
    held("the Kinnan copy and Basalt Monolith stay in play", copied_kinnan_keys),
    step("the loop makes unlimited colourless mana", StepKind::Engine, infinite_colourless),
    gd::S_CAST,
    gd::S_FETCH_HELM,
    S_EQUIP_LOOP,
    gd::S_COMBAT,
    gd::S_LOOP,
];

// ------------------------------------------------------------------ the table
/// the combos of this module (chained after the others by `combos::all`)
pub static COMBOS: [Combo; 11] = [
    combo(
        "Kinnan + Basalt Monolith + Thrasios (draw the deck)",
        &[QUIET, &[KINNAN], &[BASALT], &[THRASIOS]],
        ready_thrasios,
        &THRASIOS_STEPS,
    ),
    combo(
        "Kinnan + Basalt Monolith + Mirage Mirror ({U}{G} outlet)",
        &[QUIET, &[KINNAN], &[BASALT], &[MIRAGE]],
        ready_mirage,
        &MIRAGE_STEPS,
    ),
    combo(
        "Underworld Breach + Lion's Eye Diamond + Brain Freeze (Thassa's Oracle finish)",
        &[QUIET, &[BREACH], &[LED], &[BRAIN_FREEZE], &[ORACLE]],
        ready_breach_oracle,
        &BREACH_ORACLE_STEPS,
    ),
    Combo {
        name: "Underworld Breach + Lion's Eye Diamond + Brain Freeze (at the table)",
        groups: &[QUIET, &[BREACH], &[LED], &[BRAIN_FREEZE], &[ORACLE]],
        ready: ready_breach_table,
        mana: (0, ""),
        finish: Some(table_finish),
        xcost: &[],
        steps: &BREACH_TABLE_STEPS,
    },
    Combo {
        name: "Ad Nauseam into a win",
        groups: &[QUIET, &[AD], &[ORACLE]],
        ready: ad_ready,
        mana: (0, ""),
        finish: Some(ad_finish),
        xcost: &[],
        steps: &AD_STEPS,
    },
    combo(
        "Hullbreaker Horror + Kinnan + two rocks cast between each other",
        &[QUIET, &[KINNAN], &[HORROR], &["Mox Amber", "Sol Ring", "Mox Opal", "Grim Monolith", "Mana Vault"]],
        ready_horror,
        &HORROR_STEPS,
    ),
    combo(
        "Goblin Welder + Goblin Engineer, then Godo",
        &[QUIET, &[gd::GODO], &[gd::HELM], &[gd::HAMMER], &[WELDER], &[ENGINEER]],
        ready_welder,
        &WELDER_STEPS,
    ),
    combo(
        "Godo already out, Hammer out: Cursed Mirror / Sculpting Steel as a second Godo",
        &[QUIET, &[gd::GODO], &[gd::HELM], &[gd::HAMMER], &[gd::MIRROR, gd::STEEL]],
        ready_rebuild_mirror,
        &REBUILD_MIRROR,
    ),
    combo(
        "Godo already out, Hammer out: Jaxis blitzes a copy of Godo",
        &[QUIET, &[gd::GODO], &[gd::HELM], &[gd::HAMMER], &[gd::JAXIS]],
        ready_rebuild_jaxis,
        &REBUILD_JAXIS,
    ),
    combo(
        "Godo already out, Hammer out: a token copy spell on Godo",
        &[QUIET, &[gd::GODO], &[gd::HELM], &[gd::HAMMER], gd::TOKEN_COPIERS],
        ready_rebuild_spell,
        &REBUILD_SPELL,
    ),
    combo(
        "Godo + Basalt Monolith + a copied Kinnan",
        &[QUIET, &[gd::GODO], &[gd::HELM], &[BASALT], &[SHIMMER, gd::MIRROR]],
        ready_basalt_kinnan,
        &BASALT_KINNAN_STEPS,
    ),
];

const REBUILD_MIRROR: [Step; 5] = [gd::S_MIRROR, gd::S_MIRROR_TRIGGER, gd::S_WORN, gd::S_COMBAT, gd::S_LOOP];
const REBUILD_JAXIS: [Step; 5] = [gd::S_JAXIS_IN, gd::S_JAXIS_COPY, gd::S_WORN, gd::S_COMBAT, gd::S_LOOP];
const REBUILD_SPELL: [Step; 4] = [gd::S_COPY_SPELL, gd::S_WORN, gd::S_COMBAT, gd::S_LOOP];
