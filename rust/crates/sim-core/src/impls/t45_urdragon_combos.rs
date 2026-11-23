//! The Ur-Dragon deck's three infinite combos (`data/spellbook/urdragon.json`, `included`), as `combos::Combo`s with
//! declared steps (`combos::Step`, the pattern of `t6_combos_godo.rs`): each step has the answer window that fits it,
//! an answered step stops the combo (the pieces already played stay), and once the last answerable step passes the
//! loop is not played out but resolved by the combo's `finish`. The steps pay their own mana (so `mana` is empty);
//! `ready` checks the zones, the timing and that the mana for every step is there, and is never true earlier.
//!
//! **1. Astral Dragon + Cursed Mirror** (Spellbook 703-5291; Cursed Mirror on the battlefield, not copying a creature,
//! Astral Dragon in hand, {6}{U}{U}). Cast Astral Dragon; Project Image makes two token copies of the Mirror, 3/3
//! flying Dragon creatures in addition to being artifacts. Each token enters as a copy of Astral Dragon (the Mirror's
//! "as this enters" replacement, with haste) and so has its own Project Image: two more Mirror tokens, each entering
//! as an Astral Dragon again. Infinite hasty Dragons (and infinite copies of the Mirror). Steps: the Astral Dragon spell
//! (counter window), Project Image's trigger (the ability-answer path), then the Dragon and the Mirror must survive
//! (instant-speed removal or bounce on either: the tokens copy Astral Dragon, the trigger targets the Mirror). Finish:
//! as Helm of the Host + Combat Celebrant and the Godo lines, the player wins (unbounded hasty attackers), so the line
//! needs a combat still to come.
//!
//! **2. Miirym + Astral Dragon + Doubling Season** (Spellbook 380-703-4772; Miirym and Doubling Season on the
//! battlefield, Astral Dragon in hand, {6}{U}{U}). Cast Astral Dragon: Miirym copies it (a token Astral Dragon, doubled
//! by Doubling Season), and each Astral Dragon's Project Image aims at Doubling Season, so every batch of token copies
//! adds Doubling Seasons and the next batch is twice again as big: a board that grows without bound until the stack
//! runs out (the Spellbook: "near-infinite creature tokens"). The Miirym copy is the engine's own trigger (a step with
//! its own window); Project Image is the combo's trigger step. The tokens are summoning sick (no haste), so the finish
//! does not win outright: it fills the battlefield with 3/3 flying Dragon tokens up to `zones::TOKEN_CAP`, and the
//! next combat is the kill (a sweeper still answers it).
//!
//! **3. Garth One-Eye + Twinflame + Spark Double + Doubling Season** (Spellbook 1171-1235-3908-4772; Garth, Spark
//! Double (entered as a copy of Garth) and Doubling Season on the battlefield, Twinflame in hand). Twinflame on Spark
//! Double's Garth makes hasty token Garths that are not legendary (Spark Double's copy "isn't legendary", and a token
//! copy copies that), twice as many with Doubling Season. A fresh Garth has all six names open: it chooses Regrowth
//! and returns Twinflame, which is cast again on the new Garths; the others choose Black Lotus (three mana each, more
//! than the loop spends) and Braingeyser (cards). Infinite cards, mana and hasty Garths. Steps: Twinflame (counter
//! window), a token Garth's ability (the ability-answer path), the Regrowth copy (counter window), then the Garths and
//! Doubling Season must survive. Finish: as above, the player wins by the unbounded hasty attackers, so it needs a
//! combat still to come. Mana: one Twinflame on one Garth gives two token Garths (Doubling Season), one for Regrowth
//! ({1}{G}) and one for Black Lotus (+3): against Twinflame's {1}{R} that loses a mana a round. The loop only pays for
//! itself from the second Twinflame, which strives onto the new Garths ({2}{R} per extra target, each extra Garth
//! nets a Lotus). So `ready` asks for {1}{R} + {1}{G} + {2}{R}: seven mana, the Spellbook's total ({4}{R}{R}{R}).
//!
//! Not modelled for a bot: the Lotus and Braingeyser rounds themselves (the loop is resolved by `finish`; a person's
//! choice about the newest Garths is `human::urdragon::garth_spare`), Garth Lotus mana
//! paying for the first Twinflame (the mana must be open), the Regrowth copy's cast triggers.

use super::combos::{Combo, Ready, Step, StepKind, done, no_keys, no_src};
use super::t6_combos_godo::{affordable, bf, bf_last, cname, cost, in_hand, in_lib, live, nm, no, ok_ready};
use crate::cards::Colors;
use crate::engine::cast::{cast_card, castable, on_cast};
use crate::engine::life::check_state;
use crate::engine::mana::{can_pay_spell, pay_spell};
use crate::engine::repl;
use crate::engine::stack::counter_window;
use crate::engine::zones::{TOKEN_CAP, Tokens, enter_token_copy_with, make_tokens};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{CopyExc, Ctx, Game, Step as TurnStep};

const ASTRAL: &str = "Astral Dragon";
const MIRROR: &str = "Cursed Mirror";
const MIIRYM: &str = "Miirym, Sentinel Wyrm";
const SEASON: &str = "Doubling Season";
const GARTH: &str = "Garth One-Eye";
const TWINFLAME: &str = "Twinflame";
const SPARK: &str = "Spark Double";
const REGROWTH: &str = "Regrowth";

// ------------------------------------------------------------------ helpers
/// p is The Ur-Dragon deck's seat: the raw key, not `ai::hand_key` (which is "" for it: the deck has no hand-written AI).
/// The first expression of every branch that exists for this deck's Garth line and free drop, so a game with no such
/// seat plays exactly as before.
pub fn urdragon_seat(g: &Game, p: PlayerId) -> bool {
    &*g.player(p).key == "urdragon"
}

/// p's turn, before combat: a line that wins by attacking with hasty tokens needs a combat still to come
fn precombat(g: &Game, p: PlayerId) -> bool {
    g.active == Some(p) && matches!(g.step, TurnStep::Start | TurnStep::Main1)
}

/// the Mirror as the artifact (a Mirror that became a creature copy is no longer one: the line needs the artifact)
fn mirror_artifact(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, MIRROR).filter(|&m| !g.is_creature(m) && g.perm(m).exc.revert_to.is_none())
}

/// Spark Double's Garth: a Garth that is a copy made by the Spark Double card
fn spark_garth(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .find(|&m| live(g, m) && nm(g, m) == GARTH && g.perm(m).phys.is_some_and(|c| cname(g, c) == SPARK))
}

/// the Garth that is not Spark Double's
fn plain_garth(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .find(|&m| live(g, m) && nm(g, m) == GARTH && !g.perm(m).phys.is_some_and(|c| cname(g, c) == SPARK))
}

/// the Garth line can still be played, as a pure read of p's zones (`twinflame` in impls/mine.rs holds Twinflame for it).
/// Garth, Spark Double and Doubling Season must each be somewhere p can still use them: in hand or library, or on p's
/// battlefield (phased out counts: it comes back) as Garth (a non-token one that is not Spark Double's copy), as
/// Spark Double's Garth, and as Doubling Season. A piece in the graveyard or exile, under another player's control,
/// on the battlefield as something else (Spark Double that copied another creature), or nowhere is dead.
pub fn garth_line_alive(g: &Game, p: PlayerId) -> bool {
    let held = |name: &str| in_hand(g, p, name).is_some() || in_lib(g, p, name);
    let spark_card = |m: PermId| g.perm(m).phys.is_some_and(|c| cname(g, c) == SPARK);
    let on = |name: &str, f: &dyn Fn(PermId) -> bool| {
        g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && nm(g, m) == name && f(m))
    };
    let garth = held(GARTH) || on(GARTH, &|m| !g.perm(m).token && !spark_card(m));
    let spark = held(SPARK) || on(GARTH, &|m| spark_card(m));
    let season = held(SEASON) || on(SEASON, &|_| true);
    garth && spark && season
}

/// Spark Double entering for an urdragon AI: the plain Garth to copy when the rest of the line is near, else None (the
/// caller keeps its best-creature pick). Near: Doubling Season on the battlefield or in hand and Twinflame in hand, or a
/// tutor in hand (Demonic or Vampiric; Worldly Tutor finds only creatures) that can fetch whichever of the two is still
/// in the library. Stricter than `garth_line_alive`: a Spark Double spent on Garth gives up the best creature's copy.
pub fn spark_garth_target(g: &Game, p: PlayerId) -> Option<PermId> {
    if !urdragon_seat(g, p) {
        return None;
    }
    let garth = card_garth(g, p)?;
    line_near(g, p).then_some(garth)
}

/// the rest of the Garth line is near (Garth aside): Doubling Season on the battlefield or in hand and Twinflame in
/// hand, or a tutor in hand (Demonic or Vampiric) for whichever of the two is still in the library. The one test of
/// `spark_garth_target` (what Spark Double copies) and `spark_hold_shape` (when it waits for Garth).
fn line_near(g: &Game, p: PlayerId) -> bool {
    let season_out = bf(g, p, SEASON).is_some() || in_hand(g, p, SEASON).is_some();
    let twin_hand = in_hand(g, p, TWINFLAME).is_some();
    let tutor = in_hand(g, p, "Demonic Tutor").is_some() || in_hand(g, p, "Vampiric Tutor").is_some();
    (season_out && twin_hand)
        || (tutor && ((season_out && in_lib(g, p, TWINFLAME)) || (twin_hand && in_lib(g, p, SEASON))))
}

/// the Garth Spark Double can copy for the Garth line: on the battlefield, not phased out, not a token and not Spark
/// Double's own copy
fn card_garth(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        live(g, m) && nm(g, m) == GARTH && !g.perm(m).token && !g.perm(m).phys.is_some_and(|c| cname(g, c) == SPARK)
    })
}

/// incoming damage over life from which the Spark Double hold is off (the Twinflame hold's number, `mine::twinflame`)
pub const SPARK_HOLD_DANGER: f64 = 1.0;

/// Everything about the Spark Double hold except the table's danger (`spark_hold`): the Garth line is near
/// (`line_near`, the test Spark Double's copy uses) with Garth still to come, so a Spark Double now would copy
/// something else and cost the line its non-legendary Garth. Garth must be in hand (castable by next turn,
/// `next_turn_mana`) or in the library with a tutor of its own, never on the battlefield (then `spark_garth_target`
/// decides, with the same near test, so the hold never outlives Garth's arrival).
pub fn spark_hold_shape(g: &Game, p: PlayerId) -> bool {
    if !urdragon_seat(g, p) || card_garth(g, p).is_some() || !line_near(g, p) {
        return false;
    }
    let garth_hand = in_hand(g, p, GARTH);
    if garth_hand.is_none() {
        // Garth must still be fetchable: in the library, with a tutor in hand that p can cast by next turn and that the
        // line's other piece does not use (Demonic or Vampiric Tutor fetch anything, Worldly Tutor only Garth)
        let tutor = |name: &str| in_hand(g, p, name).is_some_and(|c| payable_next_turn(g, p, c));
        let any = u32::from(tutor("Demonic Tutor")) + u32::from(tutor("Vampiric Tutor"));
        let worldly = u32::from(tutor("Worldly Tutor"));
        let other_missing = u32::from(in_hand(g, p, TWINFLAME).is_none())
            + u32::from(bf(g, p, SEASON).is_none() && in_hand(g, p, SEASON).is_none());
        // The plan's wish rows send a tutor to Garth only once Doubling Season is out (`out:`, data/plans/urdragon.toml;
        // the Terror and Goldspan row comes before the line), so with Season only in hand the tutor fetches a Dragon.
        if !in_lib(g, p, GARTH) || bf(g, p, SEASON).is_none() || other_missing + 1 > any + worldly {
            return false; // Garth is dead (graveyard, exile, phased out, elsewhere) or no castable tutor is left for it
        }
    }
    if let Some(c) = garth_hand
        && !payable_next_turn(g, p, c)
    {
        return false; // Garth cannot be cast by next turn: the hold waits for nothing
    }
    true
}

/// Can p pay for card `c` on its next turn? Its price now (`cost_of`, taxes included), the count of mana
/// (`next_turn_mana`: lands, mana permanents, one land drop) and the colours: the mana units now (untapped lands and
/// mana permanents, `mana_units`) plus every tapped land (it untaps) as `land_card_unit` reads it, plus at most one
/// land from the hand, must pay it (`units_pay_with_rituals`, so rituals in hand count as they do for the combos). The
/// mana permanents `mana_units` leaves out now (tapped, or a summoning-sick dork) come from `next_turn_perm_units`;
/// the few with special rules are left out, so the answer can only err towards "not payable".
fn payable_next_turn(g: &Game, p: PlayerId, c: CardId) -> bool {
    use crate::engine::mana::{cost_of, land_card_unit, mana_units, next_turn_mana, units_pay_with_rituals};
    let (gn, pips) = cost_of(g, p, c);
    if next_turn_mana(g, p) < gn + pips.len() as u32 {
        return false;
    }
    let pl = g.player(p);
    let mut u = mana_units(g, p, false);
    for &l in &pl.lands {
        if g.perm(l).on_bf && g.perm(l).tapped {
            u.push(land_card_unit(g, p, g.perm(l).card()));
        }
    }
    u.extend(crate::engine::mana::next_turn_perm_units(g, p));
    let skip = [c];
    if units_pay_with_rituals(g, p, &u, gn, &pips, &skip) {
        return true;
    }
    pl.hand.iter().copied().filter(|&h| g.db.get(h).land).any(|h| {
        let mut v = u.clone();
        v.push(land_card_unit(g, p, h));
        units_pay_with_rituals(g, p, &v, gn, &pips, &skip)
    })
}

/// The Ur-Dragon seat holds Spark Double (cast or free-dropped) while `spark_hold_shape` says the Garth line is near
/// and Garth is still to come, unless the table can kill p (`SPARK_HOLD_DANGER`).
pub fn spark_hold(g: &Game, p: PlayerId) -> bool {
    spark_hold_shape(g, p) && crate::ai::brain::Situation::new(g, p).danger < SPARK_HOLD_DANGER
}

fn present(v: &[Option<PermId>]) -> Vec<PermId> {
    v.iter().copied().flatten().collect()
}

/// pay for a card of this name in hand and cast it (the normal counter window), its own enter trigger left to the
/// combo's steps (`Game::combo_spell`, read by `t45_miirym::astral_etb`)
fn cast_from_hand(g: &mut Game, p: PlayerId, name: &str) -> Res<bool> {
    let Some(c) = in_hand(g, p, name) else { return Ok(false) };
    if !castable(g, p, c, "hand") {
        return Ok(false);
    }
    let (gn, pips) = cost(g, p, c);
    if !can_pay_spell(g, p, c, gn, &pips, false) {
        return Ok(false);
    }
    pay_spell(g, p, c, gn, &pips, false)?;
    g.combo_spell = true;
    let r = cast_card(g, p, c, "hand", Ctx::default());
    g.combo_spell = false;
    r
}

// ------------------------------------------------------------------ ready checks
/// Astral Dragon in hand and castable with its mana (the pieces of lines 1 and 2)
fn astral_in_hand(g: &Game, p: PlayerId) -> bool {
    let Some(c) = in_hand(g, p, ASTRAL) else { return false };
    castable(g, p, c, "hand") && affordable(g, p, &[cost(g, p, c)])
}

fn ready_mirror(g: &Game, p: PlayerId) -> Ready {
    if !precombat(g, p) || mirror_artifact(g, p).is_none() || !astral_in_hand(g, p) {
        return no();
    }
    ok_ready()
}

fn ready_miirym(g: &Game, p: PlayerId) -> Ready {
    if bf(g, p, MIIRYM).is_none() || bf(g, p, SEASON).is_none() || !astral_in_hand(g, p) {
        return no();
    }
    ok_ready()
}

/// Twinflame ({1}{R}) and the Regrowth copy ({1}{G}) from open mana, plus the second Twinflame's strive ({2}{R}): the
/// loop gains mana only once Twinflame copies two or more Garths (see the module comment)
pub fn ready_garth(g: &Game, p: PlayerId) -> Ready {
    if !precombat(g, p) || bf(g, p, SEASON).is_none() || plain_garth(g, p).is_none() || spark_garth(g, p).is_none() {
        return no();
    }
    let (Some(t), Some(rg)) = (in_hand(g, p, TWINFLAME), g.db.id(REGROWTH)) else { return no() };
    let strive = super::t6_combos_godo::extra(2, "R");
    if !castable(g, p, t, "hand") || !affordable(g, p, &[cost(g, p, t), cost(g, p, rg), strive]) {
        return no();
    }
    ok_ready()
}

// ------------------------------------------------------------------ steps
fn cast_astral(g: &mut Game, p: PlayerId) -> Res<bool> {
    Ok(cast_from_hand(g, p, ASTRAL)? && bf(g, p, ASTRAL).is_some())
}

fn astral_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, ASTRAL)
}

/// Project Image aimed at the Mirror: the Mirror and the Dragon it copies are still there
fn image_on_mirror(g: &mut Game, p: PlayerId) -> Res<bool> {
    Ok(bf(g, p, ASTRAL).is_some() && mirror_artifact(g, p).is_some())
}

fn mirror_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    present(&[bf(g, p, ASTRAL), bf(g, p, MIRROR)])
}

/// Miirym's trigger (the engine's own, with its own windows) made a token copy of Astral Dragon
fn miirym_copied(g: &mut Game, p: PlayerId) -> Res<bool> {
    let n = g.player(p).perms.iter().filter(|&&m| live(g, m) && nm(g, m) == ASTRAL).count();
    Ok(n >= 2 && bf(g, p, MIIRYM).is_some())
}

/// Project Image aimed at Doubling Season
fn image_on_season(g: &mut Game, p: PlayerId) -> Res<bool> {
    Ok(bf(g, p, ASTRAL).is_some() && bf(g, p, SEASON).is_some())
}

fn miirym_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    present(&[bf(g, p, MIIRYM), bf(g, p, SEASON), bf(g, p, ASTRAL)])
}

/// Twinflame on Spark Double's Garth (strive adds nothing for one target): pay for it, the normal counter window, then
/// the hasty token Garths (twice as many with Doubling Season; not legendary, exiled at the end step)
fn cast_twinflame(g: &mut Game, p: PlayerId) -> Res<bool> {
    let (Some(c), Some(target)) = (in_hand(g, p, TWINFLAME), spark_garth(g, p)) else { return Ok(false) };
    let (gn, pips) = cost(g, p, c);
    if !castable(g, p, c, "hand") || !can_pay_spell(g, p, c, gn, &pips, false) {
        return Ok(false);
    }
    pay_spell(g, p, c, gn, &pips, false)?;
    let hand = &mut g.player_mut(p).hand;
    if let Some(i) = hand.iter().position(|&x| x == c) {
        hand.remove(i);
    }
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts Twinflame -> {}", g.player(p).name, nm(g, target));
    on_cast(g, p, c)?;
    if g.over || !counter_window(g, p, c, 6.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(false);
    }
    let mut made = 0;
    if live(g, target) {
        let cd = g.perm(target).cd.unwrap();
        let exc = CopyExc { haste: true, not_legendary: true, ..CopyExc::default() };
        for _ in 0..repl::tokens(g, p, 1).min(2) {
            if let Some(t) = enter_token_copy_with(g, p, cd, Some(exc))?
                && g.perm(t).on_bf
            {
                let x = g.perm_mut(t);
                x.sick = false; // haste
                x.temp = true; // exiled at the next end step
                made += 1;
            }
        }
    }
    g.player_mut(p).gy.push(c);
    check_state(g)?;
    Ok(made > 0)
}

/// the newest Garth: a hasty token from Twinflame, untapped
fn token_garth(g: &Game, p: PlayerId) -> Option<PermId> {
    bf_last(g, p, GARTH).filter(|&m| !g.perm(m).tapped && g.perm(m).token)
}

fn token_garth_src(g: &Game, p: PlayerId) -> Option<PermId> {
    token_garth(g, p)
}

/// a token Garth taps and chooses Regrowth (the ability; the copy is cast in the next step)
fn garth_chooses_regrowth(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(t) = token_garth(g, p) else { return Ok(false) };
    g.perm_mut(t).tapped = true;
    crate::glog!(g, "  {} chooses Regrowth: a copy of it", g.perm(t).name);
    Ok(true)
}

/// the copy of Regrowth ({1}{G}, the normal counter window) returns Twinflame from the graveyard
fn cast_regrowth_copy(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(rg) = g.db.id(REGROWTH) else { return Ok(false) };
    if !g.player(p).gy.iter().any(|&c| cname(g, c) == TWINFLAME) {
        return Ok(false);
    }
    let (gn, pips) = cost(g, p, rg);
    if !can_pay_spell(g, p, rg, gn, &pips, false) {
        return Ok(false);
    }
    pay_spell(g, p, rg, gn, &pips, false)?;
    crate::glog!(g, "  {} casts a copy of Regrowth", g.player(p).name);
    if g.over || !counter_window(g, p, rg, 6.0, vec![])? {
        return Ok(false);
    }
    let Some(i) = g.player(p).gy.iter().position(|&c| cname(g, c) == TWINFLAME) else { return Ok(false) };
    let c = g.player_mut(p).gy.remove(i);
    g.player_mut(p).hand.push(c);
    crate::glog!(g, "    Regrowth returns {}", cname(g, c));
    Ok(true)
}

fn garth_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    present(&[plain_garth(g, p), spark_garth(g, p), bf(g, p, SEASON)])
}

// ------------------------------------------------------------------ the finish of line 2
/// the board that grows without bound: Dragon tokens to the battlefield's token cap (summoning sick; the next combat
/// is the kill)
fn dragon_flood(g: &mut Game, p: PlayerId) -> Res {
    dragon_flood_n(g, p, TOKEN_CAP as u32)
}

/// `n` of those Dragons before any doubling (the person's shortcut, `human/urdragon.rs`, picks how many)
pub fn dragon_flood_n(g: &mut Game, p: PlayerId, n: u32) -> Res {
    let spec = Tokens {
        fly: true,
        tgh: Some(3),
        types: vec!["dragon"],
        color: Some(Colors::from_letters("U")),
        ..Tokens::new(n, 3)
    };
    let made = make_tokens(g, p, spec)?.len();
    crate::glog!(g, "    {} has {} more 3/3 flying Dragons", g.player(p).name, made);
    Ok(())
}

// ------------------------------------------------------------------ the step tables
const fn step(label: &'static str, kind: StepKind, run: fn(&mut Game, PlayerId) -> Res<bool>) -> Step {
    Step { label, kind, src: no_src, pay: (0, ""), run, keys: no_keys }
}

const fn trigger(
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

const MIRROR_STEPS: [Step; 3] = [
    step("cast Astral Dragon", StepKind::Spell, cast_astral),
    trigger(
        "Project Image: two Cursed Mirror tokens, each entering as another Astral Dragon",
        StepKind::Trigger,
        astral_src,
        image_on_mirror,
    ),
    held("Astral Dragon and Cursed Mirror stay: every new Dragon makes two more Mirrors", mirror_keys),
];

const MIIRYM_STEPS: [Step; 4] = [
    step("cast Astral Dragon", StepKind::Spell, cast_astral),
    step("Miirym's trigger: a token copy of Astral Dragon", StepKind::Engine, miirym_copied),
    trigger(
        "Project Image: token copies of Doubling Season, which double the next batch",
        StepKind::Trigger,
        astral_src,
        image_on_season,
    ),
    held("Miirym, Astral Dragon and Doubling Season stay: each batch is twice the last", miirym_keys),
];

const GARTH_STEPS: [Step; 4] = [
    step("cast Twinflame on Spark Double's Garth", StepKind::Spell, cast_twinflame),
    trigger(
        "a hasty token Garth taps and chooses Regrowth",
        StepKind::Ability,
        token_garth_src,
        garth_chooses_regrowth,
    ),
    step("cast the copy of Regrowth: Twinflame returns", StepKind::Spell, cast_regrowth_copy),
    held("the Garths and Doubling Season stay: Black Lotus, Braingeyser and Twinflame again", garth_keys),
];

/// the Ur-Dragon combos (chained after the others by `combos::all`), best first
pub static COMBOS: [Combo; 3] = [
    Combo {
        name: "Astral Dragon + Cursed Mirror",
        groups: &[&[ASTRAL], &[MIRROR]],
        ready: ready_mirror,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &MIRROR_STEPS,
    },
    Combo {
        name: "Miirym + Astral Dragon + Doubling Season",
        groups: &[&[MIIRYM], &[ASTRAL], &[SEASON]],
        ready: ready_miirym,
        mana: (0, ""),
        finish: Some(dragon_flood),
        xcost: &[],
        steps: &MIIRYM_STEPS,
    },
    Combo {
        name: "Garth One-Eye + Twinflame + Spark Double + Doubling Season",
        groups: &[&[GARTH], &[TWINFLAME], &[SPARK], &[SEASON]],
        ready: ready_garth,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &GARTH_STEPS,
    },
];
