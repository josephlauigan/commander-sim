//! Kefka, Court Mage's deck (Grixis discard, key `kefka`): the cards the engine's tags and compiled abilities don't
//! cover, the madness / mayhem / graveyard-casting mechanics, and the deck's AI plan.
//!
//! - Mechanics: **madness** (`MADNESS`) and **mayhem** (`MAYHEM`) are tables by card name that the engine's discard
//!   paths consult (`discarded`, called from `zones::discard_cards` / `discard_index`); any card can be added. Casting
//!   a card from the graveyard (mayhem, Demonic Embrace, flashback with a discard) goes through `cast_card` with the
//!   zone `escape` (or `gy`: flashback exiles), which also reports the card leaving the graveyard (`gy_left`: Garrison
//!   Excavator).
//! - Cards: see the doc comment of each function; the audit verdicts are in data/card_overrides.toml.
//! - The AI plan: `kefka_prio` (cast priorities), `discard_choice` (what to bin to Kefka's trigger and the looters),
//!   `flip_utility` (when to pay {8}), `loop_safe` (the Psychosis Crawler loop), Decorum Dissertation's target.

use super::common::{AURA, AuraSpec, aura};
use super::lands::sac_land;
use super::partials::remove_card;
use super::yshtola::take_card;
use crate::ai::pool::generic_prio;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, copy_spell};
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{may_window, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{etgh, pval, untargetable};
use crate::engine::zones::{
    Enter, Tokens, bounce, die, discard_cards, draw, enter, exile_perm, make_tokens, max_by, min_by,
};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

// ------------------------------------------------------------------ names
const ICHOR_SLICK: &str = "Ichor Slick";
const SCARLET_SPIDER: &str = "Scarlet Spider, Kaine";
const DEMONIC_EMBRACE: &str = "Demonic Embrace";
const TWINNED_VISION: &str = "Twinned Vision";
const POSTMORTEM: &str = "Postmortem Professor";
const VOIDBORN: &str = "Susurian Voidborn";
const DECORUM: &str = "Decorum Dissertation";
const EXCAVATOR: &str = "Garrison Excavator";
const CRAWLER: &str = "Psychosis Crawler";
const NIV: &str = "Niv-Mizzet, the Firemind";
const KEFKA: &str = "Kefka, Court Mage // Kefka, Ruler of Ruin";

/// **Madness**: cards that may be cast for this cost (generic, pips) when they are discarded.
pub const MADNESS: [(&str, u32, &str); 1] = [(ICHOR_SLICK, 3, "B")];
/// **Mayhem**: cards that may be cast from the graveyard for this cost (generic, pips) the turn they were discarded.
pub const MAYHEM: [(&str, u32, &str); 1] = [(SCARLET_SPIDER, 0, "B")];
/// cards that are better in the graveyard than in hand (they have a way back): the AI discards them first
const GY_VALUE: [&str; 5] = [DEMONIC_EMBRACE, TWINNED_VISION, POSTMORTEM, ICHOR_SLICK, SCARLET_SPIDER];

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn is_kefka(g: &Game, p: PlayerId) -> bool {
    g.player(p).key == "kefka"
}

// ------------------------------------------------------------------ kefka's state
/// Kefka, Ruler of Ruin is on p's battlefield
pub fn flipped(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        x.on_bf && !x.phased && x.data.truthy(DataKey::Ruin) && name_of(g, m) == KEFKA
    })
}

fn kefka_out(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && !g.perm(m).phased && name_of(g, m) == KEFKA)
}

fn crawler_out(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && !g.perm(m).phased && name_of(g, m) == CRAWLER)
}

fn niv_out(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && !g.perm(m).phased && name_of(g, m) == NIV)
}

fn alive_opps(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    g.opps(p).filter(|&q| g.player(q).alive).collect()
}

/// **The Psychosis Crawler loop is safe to start**: every card drawn makes each opponent lose 1, and each loss draws
/// a card with the flipped Kefka, so the loop runs until the table is dead or the library is. It kills the table only
/// when the library holds more cards than the most healthy opponent has life (one draw takes 1 from every opponent,
/// so the last one dies after that many draws). The brief says "the lowest opponent's life"; the highest is the safe
/// reading, since a loop that kills some opponents and then draws the library out loses the game.
pub fn loop_safe(g: &Game, p: PlayerId) -> bool {
    let opps = alive_opps(g, p);
    let Some(most) = opps.iter().map(|&q| g.player(q).life).max() else { return true };
    g.player(p).library.len() as i32 > most + 1
}

/// **Niv-Mizzet, the Firemind with the flipped Kefka**: each card drawn is 1 damage to one target, each life lost on
/// your turn draws a card, so pinging a face runs until the table is dead or the library is. One ping takes 1 life from
/// one opponent, so the loop kills the table only when the library holds more cards than all their life together.
pub fn niv_safe(g: &Game, p: PlayerId) -> bool {
    let total: i32 = alive_opps(g, p).iter().map(|&q| g.player(q).life.max(0)).sum();
    g.player(p).library.len() as i32 > total + 1
}

/// **Hold the face damage**: Kefka is flipped, a draw-trigger engine (the Crawler, Niv-Mizzet) is out and its mandatory
/// loop would deck us. Damage that comes off a draw trigger then goes at a creature (or finishes a player) rather than
/// at a player's face. Used by the engine's 'any target' damage for the kefka deck.
pub fn loop_guard(g: &Game, p: PlayerId) -> bool {
    is_kefka(g, p) && flipped(g, p) && ((crawler_out(g, p) && !loop_safe(g, p)) || (niv_out(g, p) && !niv_safe(g, p)))
}

// ------------------------------------------------------------------ the discard choice
fn type_bits(t: Types) -> u8 {
    let mut bits = 0u8;
    for (i, k) in [
        Types::LAND,
        Types::CREATURE,
        Types::INSTANT,
        Types::SORCERY,
        Types::ARTIFACT,
        Types::ENCHANTMENT,
        Types::PLANESWALKER,
    ]
    .iter()
    .enumerate()
    {
        if t.has(*k) {
            bits |= 1 << i;
        }
    }
    bits
}

/// the card types of `c` as a bit set (land, creature, instant, sorcery, artifact, enchantment, planeswalker)
pub fn card_types(g: &Game, c: CardId) -> u8 {
    type_bits(g.db.get(c).types)
}

fn wants_gy(g: &Game, c: CardId) -> bool {
    GY_VALUE.contains(&cname(g, c))
}

/// **What to discard** (Kefka's trigger and the looters): cards that want to be in the graveyard (madness, mayhem,
/// graveyard-castable) first; otherwise the least useful card, preferring one whose types are not yet in the pile of
/// discarded cards (each new type is a card drawn). `pile`: the type bits already discarded this way.
pub fn discard_choice(g: &Game, p: PlayerId, hand: &[CardId], pile: u8) -> Option<CardId> {
    let score = |c: CardId| -> f64 {
        let mut v = card_worth(g, p, c, false);
        v += crate::ai::decks::breach_keep_bonus(g, p, c);
        if wants_gy(g, c) {
            v -= 100.0;
        }
        if cname(g, c) == "Deflecting Swat" {
            v += 40.0; // untagged, so card_worth gives 0: held for the response like any other protection (Prot is 40)
        }
        if cname(g, c) == CRAWLER || cname(g, c) == NIV {
            v += 60.0; // held for the loop, not worthless
        }
        if matches!(cname(g, c), "Underworld Breach" | "Yawgmoth's Will") && storm_card_left(g, p) {
            v += 70.0; // the Breach line still has its storm half: card_worth reads 0 for both, so they were binned
        }
        if is_protection(g, c) && !flipped(g, p) {
            v += 45.0; // Kefka is still the plan: protection beats a land (25), even after the instant type costs 30
        }
        let new = card_types(g, c) & !pile;
        v - 30.0 * new.count_ones() as f64 // a new type is a card drawn
    };
    let cands: Vec<CardId> = hand.iter().copied().filter(|&c| !g.player(p).stolen.contains_key(&c)).collect();
    min_by(&cands, score)
}

/// engine.discard_worst for the Kefka deck: n discards by `discard_choice`
pub fn discard_n(g: &mut Game, p: PlayerId, n: u32) -> Res {
    for _ in 0..n {
        let hand = g.player(p).hand.clone();
        let Some(c) = discard_choice(g, p, &hand, 0) else { return Ok(()) };
        discard_cards(g, p, &[c])?;
    }
    Ok(())
}

// ------------------------------------------------------------------ madness and mayhem
fn alt_cost(table: &[(&str, u32, &'static str)], name: &str) -> Option<(u32, &'static str)> {
    table.iter().find(|x| x.0 == name).map(|x| (x.1, x.2))
}

/// the engine's discard paths report here once the cards are in the graveyard: mayhem remembers them for the turn,
/// madness offers the cast for the madness cost (a person is not asked: the AI casts when it has a use for the spell)
pub fn discarded(g: &mut Game, q: PlayerId, cards: &[CardId]) -> Res {
    let st = g.turn_stamp();
    for &c in cards {
        let name = cname(g, c);
        if alt_cost(&MAYHEM, name).is_some() {
            let pl = g.player_mut(q);
            match &mut pl.discard_log {
                Some((s, v)) if *s == st => v.push(c),
                other => *other = Some((st, vec![c])),
            }
        }
    }
    for &c in cards {
        if g.over || !g.player(q).alive {
            break;
        }
        let Some((gn, pips)) = alt_cost(&MADNESS, cname(g, c)) else { continue };
        if g.player(q).gy.contains(&c) {
            madness_cast(g, q, c, gn, pips)?;
        }
    }
    Ok(())
}

/// Ichor Slick's madness: -3/-3 on the best creature it helps with; no target worth it, no cast
fn madness_cast(g: &mut Game, q: PlayerId, c: CardId, gn: u32, pips: &str) -> Res {
    if !can_pay(g, q, gn, pips, false) {
        return Ok(());
    }
    let Some(t) = slick_target(g, q, c) else { return Ok(()) };
    pay(g, q, gn, pips, false)?;
    crate::glog!(g, "    {} casts {} for its madness cost", pname(g, q), cname(g, c));
    cast_card(g, q, c, "escape", Ctx { target: Some(t), ..Ctx::default() })?;
    Ok(())
}

/// the creature Ichor Slick should hit: an opposing creature it kills, the most valuable
fn slick_target(g: &Game, q: PlayerId, c: CardId) -> Option<PermId> {
    let tg = legal_targets(g, q, "shrink3", "c", false, Some(c));
    let kill: Vec<PermId> = tg.into_iter().filter(|&m| etgh(g, m) <= 3 && pval(g, m) >= 1.5).collect();
    max_by(&kill, |m| pval(g, m))
}

/// mayhem: was this card discarded by its owner this turn?
fn mayhem_ready(g: &Game, p: PlayerId, c: CardId) -> bool {
    matches!(&g.player(p).discard_log, Some((s, v)) if *s == g.turn_stamp() && v.contains(&c))
}

// ------------------------------------------------------------------ Garrison Excavator
/// **Garrison Excavator**: whenever one or more cards leave your graveyard, create a 2/2 red and white Spirit. Called
/// where this deck's cards leave a graveyard: casting from it (`cast_card`'s graveyard zones), Postmortem Professor,
/// Dark Ritual-style exile costs. An opponent exiling the graveyard is not seen (the engine has no single point for
/// it); `n` is the number of cards that left at once (one Spirit, as Oracle's "one or more").
pub fn gy_left(g: &mut Game, p: PlayerId, n: usize) -> Res {
    if n == 0 || !g.player(p).alive {
        return Ok(());
    }
    let ex: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && !g.perm(m).phased && name_of(g, m) == EXCAVATOR && !g.perm(m).neutered)
        .collect();
    for m in ex {
        if trigger_window(g, p, Some(m), "a 2/2 Spirit", Some(3.0))? {
            let spec = Tokens {
                tgh: Some(2),
                color: Some(Colors::from_letters("RW")),
                types: vec![intern("spirit")],
                ..Tokens::new(1, 2)
            };
            make_tokens(g, p, spec)?;
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ the spells
/// a hand-reveal discard: the opponent with the best card `ok` accepts loses it (you choose)
fn hand_attack(g: &mut Game, p: PlayerId, ok: fn(&Game, CardId) -> bool, what: &str) -> Res {
    let mut best: Option<(PlayerId, CardId, f64)> = None;
    for q in alive_opps(g, p) {
        for &c in &g.player(q).hand {
            if !ok(g, c) {
                continue;
            }
            let w = card_worth(g, q, c, false);
            if best.is_none_or(|b| w > b.2) {
                best = Some((q, c, w));
            }
        }
    }
    let Some((q, c, _)) = best else {
        crate::glog!(g, "    {what}: no card to take");
        return Ok(());
    };
    crate::glog!(g, "    {what}: {} discards {}", pname(g, q), cname(g, c));
    discard_cards(g, q, &[c])
}

fn noncreature_nonland(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    !d.creature && !d.land
}

fn nonland(g: &Game, c: CardId) -> bool {
    !g.db.get(c).land
}

/// **Duress**: target opponent reveals their hand; you choose a noncreature, nonland card, which they discard
fn duress(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    hand_attack(g, p, noncreature_nonland, "Duress")?;
    Ok("gy")
}

/// **Dark Inquiry**: target opponent reveals their hand; you choose a nonland card, which they discard
fn dark_inquiry(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    hand_attack(g, p, nonland, "Dark Inquiry")?;
    Ok("gy")
}

fn hand_attack_prio(g: &Game, p: PlayerId, c: CardId, ok: fn(&Game, CardId) -> bool) -> i32 {
    let n = alive_opps(g, p).iter().map(|&q| g.player(q).hand.iter().filter(|&&x| ok(g, x)).count()).max().unwrap_or(0);
    if n == 0 {
        return 0;
    }
    let cheap = g.db.get(c).cmc <= 1;
    (if cheap { 52 } else { 44 }) + (2 * n as i32).min(8)
}

fn duress_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    hand_attack_prio(g, p, c, noncreature_nonland)
}

fn inquiry_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    hand_attack_prio(g, p, c, nonland)
}

/// **Inner Fire**: add {R} for each card in your hand (the mana floats until the turn ends)
fn inner_fire(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = g.player(p).hand.len() as u32;
    g.player_mut(p).floating.r += n;
    crate::glog!(g, "    {} adds {n} red mana (Inner Fire)", pname(g, p));
    Ok("gy")
}

/// Inner Fire pays for itself with a big hand: the hand after casting it must hold enough to spend the mana on, or
/// to flip Kefka
fn inner_fire_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let pl = g.player(p);
    let hand = pl.hand.iter().filter(|&&x| x != c).count() as u32;
    let kefka_flip = kefka_out(g, p) && !flipped(g, p);
    let spare = total_mana(g, p, false).saturating_sub(g.db.get(c).cmc);
    if kefka_flip && spare + hand >= 8 && spare < 8 {
        return 72;
    }
    if hand >= 6 { 38 } else { 0 }
}

/// **Decorum Dissertation**: target player draws two cards and loses 2 life. **Paradigm**: it is exiled as it resolves,
/// and at the beginning of each of your first main phases you may cast a copy of it for free (`paradigm`). The target
/// is an opponent once Kefka, Ruler of Ruin is out (their loss draws you cards), yourself before.
fn decorum(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let t = decorum_target(g, p);
    crate::glog!(g, "    Decorum Dissertation: {} draws two and loses 2 life", pname(g, t));
    draw(g, t, 2, false)?;
    lose_life(g, t, 2, Some(p), "other", None)?;
    Ok("exile")
}

fn decorum_target(g: &Game, p: PlayerId) -> PlayerId {
    if flipped(g, p) {
        let opps = alive_opps(g, p);
        if let Some(q) = min_by(&opps, |q| g.player(q).life as f64) {
            return q;
        }
    }
    p
}

fn decorum_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if flipped(g, p) && !alive_opps(g, p).is_empty() {
        return 70;
    }
    if g.player(p).life > 12 && g.player(p).library.len() > 12 { 46 } else { 0 }
}

/// the Paradigm copy at the beginning of your precombat main phase (called from `turn.rs` once the phase starts)
pub fn paradigm(g: &mut Game, p: PlayerId) -> Res {
    let Some(c) = g.player(p).exile.iter().copied().find(|&c| cname(g, c) == DECORUM) else { return Ok(()) };
    if !g.player(p).alive || g.over {
        return Ok(());
    }
    let Some(t) = Some(decorum_target(g, p)) else { return Ok(()) };
    if t == p && (g.player(p).life <= 6 || g.player(p).library.len() < 8) {
        return Ok(()); // "may": not when it would hurt
    }
    crate::glog!(g, "  {} casts a copy of Decorum Dissertation (Paradigm)", pname(g, p));
    copy_spell(g, p, c, None, true)
}

/// **Arcane Omens**: converge: target player discards X cards, X the number of colours of mana spent to cast it
fn omens(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x.max(1) as usize;
    let opps = alive_opps(g, p);
    // the opponent hurt most: the one holding the most cards
    let Some(q) = max_by(&opps, |q| g.player(q).hand.len() as f64) else { return Ok("gy") };
    for _ in 0..x {
        let hand = g.player(q).hand.clone();
        let Some(c) = min_by(&hand, |c| card_worth(g, q, c, false)) else { break };
        crate::glog!(g, "    Arcane Omens: {} discards {}", pname(g, q), cname(g, c));
        discard_cards(g, q, &[c])?;
    }
    Ok("gy")
}

/// the colours of mana p would spend on {4}{B} from its untapped sources: the converge count of Arcane Omens, and the
/// plan to pay it that way (each source pays one colour, distinct colours first)
fn converge_plan(
    g: &Game,
    p: PlayerId,
    generic: u32,
    pip: char,
) -> Option<(Vec<(crate::engine::mana::Unit, u32, char)>, u32)> {
    use crate::engine::mana::{Source, mana_units};
    let units = mana_units(g, p, false);
    let mut used: Vec<(crate::engine::mana::Unit, u32, char)> = vec![];
    let mut got = Colors::NONE;
    // the coloured pip first, from a source that makes it (the one with the fewest other colours)
    let mut taken = vec![false; units.len()];
    let i = (0..units.len())
        .filter(|&i| units[i].cols.has(pip))
        .min_by_key(|&i| (units[i].src == Source::Treasure, units[i].cols.count()))?;
    taken[i] = true;
    got = got.union(Colors::from_letters(&pip.to_string()));
    used.push((units[i], 1, pip));
    let mut need = generic;
    if units[i].amt > 1 {
        need = need.saturating_sub(units[i].amt - 1);
        used[0].1 = units[i].amt;
    }
    while need > 0 {
        // a source adding a new colour; else any source (the one wasting least)
        let mut best: Option<(usize, (i32, u32, i64))> = None;
        for (j, w) in units.iter().enumerate() {
            if taken[j] {
                continue;
            }
            let new = "WUBRG".chars().filter(|&c| w.cols.has(c) && !got.has(c)).count() as i32;
            let key = (-(new.min(1)), w.cols.count().min(1) as u32 * 0 + (w.amt.saturating_sub(need)), -(w.amt as i64));
            if best.is_none_or(|b| key < b.1) {
                best = Some((j, key));
            }
        }
        let (j, _) = best?;
        taken[j] = true;
        let w = units[j];
        let col = "WUBRG".chars().find(|&c| w.cols.has(c) && !got.has(c));
        let take = w.amt.min(need);
        if let Some(c) = col {
            got = got.union(Colors::from_letters(&c.to_string()));
        }
        used.push((w, take, col.unwrap_or('-')));
        need -= take;
    }
    Some((used, got.count() as u32))
}

fn omens_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || crate::human::is_human(g, p) || alive_opps(g, p).iter().all(|&q| g.player(q).hand.is_empty()) {
        return Ok(vec![]);
    }
    let Some((_, x)) = converge_plan(g, p, 4, 'B') else { return Ok(vec![]) };
    if !can_pay(g, p, 4, "B", false) {
        return Ok(vec![]);
    }
    let best = alive_opps(g, p).iter().map(|&q| g.player(q).hand.len()).max().unwrap_or(0) as u32;
    let u = 1.5 + x.min(best) as f64 * 1.2;
    Ok(vec![Opt {
        utility: u,
        label: format!("Arcane Omens (X={x})"),
        act: Some(Action::Plan { f: omens_go, arg: c.0 as i64 }),
    }])
}

fn omens_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 4, "B", false) {
        return Ok(false);
    }
    let Some((plan, x)) = converge_plan(g, p, 4, 'B') else { return Ok(false) };
    g.pay_for = Some(c);
    for (w, n, col) in plan {
        crate::engine::mana::spend_unit(g, p, w, n, &if col == 'B' { "B".to_string() } else { String::new() })?;
    }
    g.pay_for = None;
    cast_card(g, p, c, "hand", Ctx { x: x as i32, ..Ctx::default() })?;
    Ok(true)
}

/// **Ichor Slick**: target creature gets -3/-3 until end of turn (cycling {2}: `slick_cycle`; madness: `discarded`)
fn ichor_slick(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    if etgh(g, t) <= 3 {
        apply_removal(g, Some(p), t, "shrink3", Some(c))?;
    } else if !untargetable(g, t) {
        g.perm_mut(t).eot_pt.0 -= 3;
        g.perm_mut(t).eot_pt.1 -= 3;
        crate::glog!(g, "    {} gets -3/-3 until end of turn", g.perm(t).name);
    }
    Ok("gy")
}

fn slick_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    // cycling {2}: worth it when the Slick has no target and the hand is short of action, or at the end of the turn
    if !can_pay(g, p, 2, "", false) || g.player(p).hand.len() > 6 {
        return Ok(vec![]);
    }
    let live = slick_target(g, p, c).is_some();
    if live && post.is_some() {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: if post.is_none() { 1.8 } else { 0.4 },
        label: "Ichor Slick (cycling)".into(),
        act: Some(Action::Plan { f: slick_cycle, arg: c.0 as i64 }),
    }])
}

/// cycling {2}: discard the card (madness may let you cast it), draw a card
fn slick_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    crate::glog!(g, "  {} cycles Ichor Slick", pname(g, p));
    discard_cards(g, p, &[c])?;
    draw(g, p, 1, false)?;
    Ok(true)
}

/// the kefka deck's prio for Ichor Slick: a removal spell, but a discard outlet's best friend: kept for madness
fn slick_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    if slick_target(g, p, c).is_some() { 60 } else { 0 }
}

/// **Extinguish the Light**: destroy target creature or planeswalker; if its mana value was 3 or less, you gain 3 life
fn extinguish(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    let mv = g.perm(t).cd.map_or(0, |cd| g.db.get(cd).cmc);
    apply_removal(g, Some(p), t, "destroy", Some(c))?;
    if !g.perm(t).on_bf {
        if mv <= 3 {
            gain(g, p, 3)?;
        }
    }
    Ok("gy")
}

/// **Grab the Prize**: as an additional cost, discard a card; draw two cards; if the discarded card wasn't a land card,
/// 2 damage to each opponent (`ctx.x` is 1 when it wasn't a land)
fn grab_the_prize(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    draw(g, p, 2, false)?;
    if ctx.x == 1 {
        for q in alive_opps(g, p) {
            lose_life(g, q, 2, Some(p), "burn", Some(true))?;
        }
    }
    Ok("gy")
}

fn grab_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !can_pay(g, p, 1, "R", false) || g.player(p).library.len() < 6 {
        return Ok(vec![]);
    }
    let hand: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&x| x != c).collect();
    let Some(d) = discard_choice(g, p, &hand, 0) else { return Ok(vec![]) };
    let nonland = !g.db.get(d).land;
    let mut u = 2.0 + if nonland { 1.0 } else { 0.0 };
    if wants_gy(g, d) {
        u += 1.5;
    }
    if flipped(g, p) && nonland {
        u += 2.0 * alive_opps(g, p).len() as f64; // 2 damage to each opponent is cards
    }
    Ok(vec![Opt {
        utility: u,
        label: "Grab the Prize".into(),
        act: Some(Action::Plan { f: grab_go, arg: c.0 as i64 }),
    }])
}

fn grab_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "R", false) {
        return Ok(false);
    }
    let hand: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&x| x != c).collect();
    let Some(d) = discard_choice(g, p, &hand, 0) else { return Ok(false) };
    let nonland = !g.db.get(d).land;
    pay(g, p, 1, "R", false)?;
    discard_cards(g, p, &[d])?;
    cast_card(g, p, c, "hand", Ctx { x: nonland as i32, ..Ctx::default() })?;
    Ok(true)
}

/// **Twinned Vision**: draw a card; if it wasn't cast from your hand, draw two instead (`ctx.x` is 1 for the
/// flashback). Flashback—{1}{U/R}{U/R}, Discard a card (`twinned_gy`).
fn twinned_vision(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    draw(g, p, if ctx.x == 1 { 2 } else { 1 }, false)?;
    Ok("gy")
}

fn twinned_gy(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(pips) = ["UU", "UR", "RR"].into_iter().find(|x| can_pay(g, p, 1, x, false)) else { return Ok(vec![]) };
    let _ = pips;
    if g.player(p).hand.is_empty() || g.player(p).library.len() < 4 {
        return Ok(vec![]);
    }
    let u = if post.is_none() { 2.6 } else { 1.8 };
    Ok(vec![Opt {
        utility: u,
        label: "Twinned Vision (flashback)".into(),
        act: Some(Action::Plan { f: twinned_go, arg: c.0 as i64 }),
    }])
}

fn twinned_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) {
        return Ok(false);
    }
    let Some(pips) = ["UU", "UR", "RR"].into_iter().find(|x| can_pay(g, p, 1, x, false)) else { return Ok(false) };
    let hand = g.player(p).hand.clone();
    let Some(d) = discard_choice(g, p, &hand, 0) else { return Ok(false) };
    pay(g, p, 1, pips, false)?;
    discard_cards(g, p, &[d])?;
    cast_card(g, p, c, "gy", Ctx { x: 1, ..Ctx::default() })?;
    gy_left(g, p, 1)?;
    Ok(true)
}

/// **Stargaze**: look at twice X cards from the top of your library; put X of them into your hand and the rest into
/// your graveyard; you lose X life
fn stargaze(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x.max(0) as usize;
    let n = (2 * x).min(g.player(p).library.len());
    let top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
    let mut left = top;
    let mut hand_cards = vec![];
    for _ in 0..x.min(left.len()) {
        let k = (0..left.len())
            .max_by(|&a, &b| card_worth(g, p, left[a], false).total_cmp(&card_worth(g, p, left[b], false)))
            .unwrap();
        hand_cards.push(left.remove(k));
    }
    crate::glog!(
        g,
        "    Stargaze: {} takes {} card(s), {} to the graveyard",
        pname(g, p),
        hand_cards.len(),
        left.len()
    );
    g.player_mut(p).hand.extend(hand_cards);
    g.player_mut(p).gy.extend(left);
    lose_life(g, p, x as i32, Some(p), "other", None)?;
    Ok("gy")
}

fn stargaze_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() {
        return Ok(vec![]);
    }
    let x = stargaze_x(g, p, c);
    if x < 1 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.2 + 0.7 * x as f64 + if g.player(p).hand.len() <= 2 { 1.2 } else { 0.0 },
        label: format!("Stargaze (X={x})"),
        act: Some(Action::Plan { f: stargaze_go, arg: pack_cx(c, x) }),
    }])
}

fn pack_cx(c: CardId, x: u32) -> i64 {
    ((x as i64) << 32) | c.0 as i64
}

fn unpack_cx(a: i64) -> (CardId, u32) {
    (CardId((a & 0xffff_ffff) as u16), (a >> 32) as u32)
}

/// X for Stargaze: the spare mana, at most 5, leaving the life and the library healthy
fn stargaze_x(g: &Game, p: PlayerId, c: CardId) -> u32 {
    let pl = g.player(p);
    let mana = total_mana(g, p, false);
    let spare = mana.saturating_sub(2);
    if !can_pay(g, p, 0, "BB", false) {
        return 0;
    }
    let by_life = (pl.life - 10).max(0) as u32;
    let by_lib = (pl.library.len() / 2) as u32;
    let _ = c;
    spare.min(5).min(by_life).min(by_lib)
}

fn stargaze_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, x) = unpack_cx(arg);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, x, "BB", false) {
        return Ok(false);
    }
    pay(g, p, x, "BB", false)?;
    cast_card(g, p, c, "hand", Ctx { x: x as i32, ..Ctx::default() })?;
    Ok(true)
}

/// **Disintegrate**: X damage to any target; a creature it damages can't be regenerated, and is exiled if it would
/// die this turn (the card's `noregen` and `exiledie` tags). X is the spare mana: the lethal amount for a creature
/// worth killing, or for an opponent's life.
fn disintegrate_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !can_pay(g, p, 0, "R", false) {
        return Ok(vec![]);
    }
    let mana = total_mana(g, p, false);
    if mana < 2 {
        return Ok(vec![]);
    }
    let maxx = mana - 1;
    let mut out = vec![];
    for q in alive_opps(g, p) {
        let life = g.player(q).life;
        if life as u32 <= maxx && !crate::engine::values::player_hexproof(g, q) {
            out.push(Opt {
                utility: 9.0,
                label: format!("Disintegrate the face of {}", g.player(q).name),
                act: Some(Action::Plan { f: disintegrate_go, arg: dis_arg(c, life as u32, None, Some(q)) }),
            });
        }
    }
    let tg = legal_targets(g, p, &format!("dmg{maxx}"), "c", false, Some(c));
    if let Some(m) = max_by(&tg, |m| pval(g, m).min(if etgh(g, m) as u32 <= maxx { 99.0 } else { 0.0 })) {
        let need = etgh(g, m).max(1) as u32;
        if need <= maxx && pval(g, m) >= 3.0 {
            out.push(Opt {
                utility: pval(g, m) - 2.5 - 0.15 * need as f64,
                label: format!("Disintegrate {}", g.perm(m).name),
                act: Some(Action::Plan { f: disintegrate_go, arg: dis_arg(c, need, Some(m), None) }),
            });
        }
    }
    Ok(out)
}

fn dis_arg(c: CardId, x: u32, m: Option<PermId>, q: Option<PlayerId>) -> i64 {
    let who = match (m, q) {
        (Some(m), _) => (m.0 as i64) + 1,
        (None, Some(q)) => 0x10000 + q.0 as i64,
        _ => 0,
    };
    (c.0 as i64) | ((x as i64) << 16) | (who << 32)
}

fn disintegrate_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff) as u16);
    let x = ((arg >> 16) & 0xffff) as u32;
    let who = arg >> 32;
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, x, "R", false) {
        return Ok(false);
    }
    let mut ctx = Ctx { x: x as i32, rem_kind: Some(intern(&format!("dmg{x}"))), ..Ctx::default() };
    if who >= 0x10000 {
        let q = PlayerId((who - 0x10000) as u8);
        if !g.player(q).alive {
            return Ok(false);
        }
        ctx.face = Some(q);
    } else {
        let m = PermId((who - 1) as _);
        if !g.perm(m).on_bf {
            return Ok(false);
        }
        ctx.target = Some(m);
    }
    pay(g, p, x, "R", false)?;
    cast_card(g, p, c, "hand", ctx)?;
    Ok(true)
}

/// **Splatter Technique**: choose one: draw four cards; or 4 damage to each creature and planeswalker
fn splatter(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if ctx.mode == Some("draw") {
        draw(g, p, 4, false)?;
        return Ok("gy");
    }
    crate::glog!(g, "    Splatter Technique: 4 damage to each creature and planeswalker");
    let all: Vec<PermId> =
        g.players.iter().filter(|q| q.alive).flat_map(|q| q.perms.iter().copied().collect::<Vec<_>>()).collect();
    for m in all {
        if !g.perm(m).on_bf || g.perm(m).phased {
            continue;
        }
        if g.is_creature(m) {
            if etgh(g, m) <= 4 {
                apply_removal(g, Some(p), m, "dmg4", Some(c))?;
            }
        } else if g.perm(m).cd.is_some_and(|cd| g.db.get(cd).types.has(Types::PLANESWALKER)) {
            let l = g.perm(m).loyalty.unwrap_or(0) - 4;
            g.perm_mut(m).loyalty = Some(l.max(0));
            if l <= 0 {
                die(g, m, "sba")?;
            }
        }
    }
    check_state(g)?;
    Ok("gy")
}

fn splatter_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !can_pay(g, p, 1, "UURR", false) {
        return Ok(vec![]);
    }
    let mut out = vec![];
    if g.player(p).library.len() > 8 {
        let hand = g.player(p).hand.len();
        out.push(Opt {
            utility: 3.0 + if hand <= 2 { 1.5 } else { 0.0 } + if flipped(g, p) { 1.0 } else { 0.0 },
            label: "Splatter Technique (draw four)".into(),
            act: Some(Action::Plan { f: splatter_draw, arg: c.0 as i64 }),
        });
    }
    let (mut ol, mut ml) = (0.0, 0.0);
    for q in g.players.iter().filter(|q| q.alive).map(|q| q.id).collect::<Vec<_>>() {
        for &m in &g.player(q).perms {
            if g.perm(m).phased || !g.is_creature(m) || etgh(g, m) > 4 {
                continue;
            }
            if q == p {
                ml += pval(g, m);
            } else {
                ol += pval(g, m);
            }
        }
    }
    if ol > 0.0 {
        out.push(Opt {
            utility: (ol - 1.2 * ml) / 2.2 - 2.5 + 1.0,
            label: "Splatter Technique (4 damage to each creature)".into(),
            act: Some(Action::Plan { f: splatter_wipe, arg: c.0 as i64 }),
        });
    }
    Ok(out)
}

fn splatter_cast(g: &mut Game, p: PlayerId, arg: i64, mode: &'static str) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "UURR", false) {
        return Ok(false);
    }
    pay(g, p, 1, "UURR", false)?;
    cast_card(g, p, c, "hand", Ctx { mode: Some(intern(mode)), ..Ctx::default() })?;
    Ok(true)
}

fn splatter_draw(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    splatter_cast(g, p, arg, "draw")
}

fn splatter_wipe(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    splatter_cast(g, p, arg, "wipe")
}

// ------------------------------------------------------------------ Demonic Embrace, mayhem, Postmortem Professor
static EMBRACE: AuraSpec = AuraSpec { pow: 3, tgh: 1, kws: &["flying"], ..AURA };

/// **Demonic Embrace**: enchanted creature gets +3/+1, has flying and is a Demon; you may cast this card from your
/// graveyard by paying 3 life and discarding a card in addition to its other costs
fn embrace_gy(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) && post != Some(true) {
        return Ok(vec![]);
    }
    if g.player(p).life <= 8 || g.player(p).hand.is_empty() || !can_pay(g, p, 1, "BB", false) {
        return Ok(vec![]);
    }
    if !g.player(p).perms.iter().any(|&m| g.is_creature(m) && !g.perm(m).phased) {
        return Ok(vec![]);
    }
    let hand = g.player(p).hand.clone();
    let Some(d) = discard_choice(g, p, &hand, 0) else { return Ok(vec![]) };
    if card_worth(g, p, d, false) > 40.0 && !wants_gy(g, d) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.4,
        label: "Demonic Embrace (graveyard)".into(),
        act: Some(Action::Plan { f: embrace_go, arg: c.0 as i64 }),
    }])
}

fn embrace_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || g.player(p).life <= 8 || !can_pay(g, p, 1, "BB", false) {
        return Ok(false);
    }
    let hand = g.player(p).hand.clone();
    let Some(d) = discard_choice(g, p, &hand, 0) else { return Ok(false) };
    pay(g, p, 1, "BB", false)?;
    lose_life(g, p, 3, Some(p), "life", None)?;
    discard_cards(g, p, &[d])?;
    cast_card(g, p, c, "escape", Ctx::default())?;
    gy_left(g, p, 1)?;
    Ok(true)
}

/// **Scarlet Spider, Kaine**: menace; when it enters, you may discard a card; if you do, a +1/+1 counter on him.
/// **Mayhem {B/R}**: cast from the graveyard for {B/R} if you discarded it this turn (`mayhem_gy`).
fn spider_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || p != o || g.player(o).hand.is_empty() {
        return Ok(());
    }
    if !may_window(g, o, Some(src), "may discard a card for a +1/+1 counter", Some(2.0))? {
        return Ok(());
    }
    if crate::human::is_human(g, o) {
        crate::human::choices::discard(g, o, 1, "Scarlet Spider, Kaine: discard a card (it gets a +1/+1 counter)")?;
        if g.perm(src).on_bf {
            crate::cardcode::put_plus(g, src, 1);
        }
        return Ok(());
    }
    let hand = g.player(o).hand.clone();
    // only when it costs nothing: a card that wants the graveyard, or a spare land
    let Some(d) = discard_choice(g, o, &hand, 0) else { return Ok(()) };
    let spare =
        wants_gy(g, d) || (g.db.get(d).land && g.player(o).lands.len() >= 5) || card_worth(g, o, d, false) <= 6.0;
    if !spare {
        return Ok(());
    }
    discard_cards(g, o, &[d])?;
    if g.perm(src).on_bf {
        crate::cardcode::put_plus(g, src, 1);
    }
    Ok(())
}

fn mayhem_gy(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some((gn, _)) = alt_cost(&MAYHEM, cname(g, c)) else { return Ok(vec![]) };
    if post.is_none() || !mayhem_ready(g, p, c) {
        return Ok(vec![]);
    }
    // {B/R}: either colour pays
    if !can_pay(g, p, gn, "B", false) && !can_pay(g, p, gn, "R", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 6.0,
        label: format!("{} (mayhem)", cname(g, c)),
        act: Some(Action::Plan { f: mayhem_go, arg: c.0 as i64 }),
    }])
}

fn mayhem_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !mayhem_ready(g, p, c) {
        return Ok(false);
    }
    let (gn, _) = alt_cost(&MAYHEM, cname(g, c)).unwrap();
    let pip = if can_pay(g, p, gn, "B", false) { "B" } else { "R" };
    if !can_pay(g, p, gn, pip, false) {
        return Ok(false);
    }
    pay(g, p, gn, pip, false)?;
    crate::glog!(g, "  {} casts {} from the graveyard (mayhem)", pname(g, p), cname(g, c));
    cast_card(g, p, c, "escape", Ctx::default())?;
    gy_left(g, p, 1)?;
    Ok(true)
}

/// **Postmortem Professor**: can't block; whenever it attacks, each opponent loses 1 life and you gain 1 life;
/// {1}{B}, exile an instant or sorcery card from your graveyard: return it from your graveyard to the battlefield
fn professor_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && g.perm(src).owner == p && trigger_window(g, p, Some(src), "drain 1", Some(3.0))? {
        let mut lost = false;
        for q in alive_opps(g, p) {
            let b = g.player(q).life;
            lose_life(g, q, 1, Some(p), "drain", None)?;
            lost |= g.player(q).life < b;
        }
        let _ = lost;
        gain(g, p, 1)?;
    }
    Ok(vec![])
}

fn professor_gy(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !can_pay(g, p, 1, "B", false) {
        return Ok(vec![]);
    }
    let fodder = professor_fodder(g, p);
    if fodder.is_none() {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.6,
        label: "Postmortem Professor (return)".into(),
        act: Some(Action::Plan { f: professor_go, arg: c.0 as i64 }),
    }])
}

/// the instant or sorcery to exile: the one worth least in the graveyard (nothing with a way back)
fn professor_fodder(g: &Game, p: PlayerId) -> Option<CardId> {
    let cs: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&x| {
            let d = g.db.get(x);
            (d.instant || d.sorcery) && d.pips.len() + d.generic as usize > 0 && !wants_gy(g, x)
        })
        .collect();
    min_by(&cs, |x| g.db.get(x).cmc as f64)
}

fn professor_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let Some(f) = professor_fodder(g, p) else { return Ok(false) };
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 1, "B", false) {
        return Ok(false);
    }
    pay(g, p, 1, "B", false)?;
    remove_card(&mut g.player_mut(p).gy, f);
    g.player_mut(p).exile.push(f);
    crate::glog!(g, "  {} exiles {} to return Postmortem Professor", pname(g, p), cname(g, f));
    if trigger_window(g, p, None, "Postmortem Professor returns", Some(3.0))? && remove_card(&mut g.player_mut(p).gy, c)
    {
        enter(g, p, c, Enter::default())?;
    }
    gy_left(g, p, 2)?;
    Ok(true)
}

// ------------------------------------------------------------------ the creatures
/// **Alpharael, Dreaming Acolyte**: when it enters, draw two cards, then discard two cards unless you discard an
/// artifact card. During your turn it has deathtouch.
fn alpharael_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || p != o || !trigger_window(g, o, Some(src), "draw two, then discard", Some(4.0))? {
        return Ok(());
    }
    draw(g, o, 2, false)?;
    let hand = g.player(o).hand.clone();
    let arts: Vec<CardId> = hand.iter().copied().filter(|&c| g.db.get(c).types.has(Types::ARTIFACT)).collect();
    let one_art = min_by(&arts, |c| card_worth(g, o, c, false) - if wants_gy(g, c) { 100.0 } else { 0.0 });
    let two = {
        let a = discard_choice(g, o, &hand, 0);
        a.and_then(|a| {
            let rest: Vec<CardId> = hand.iter().copied().filter(|&c| c != a).collect();
            discard_choice(g, o, &rest, card_types(g, a)).map(|b| (a, b))
        })
    };
    let cost = |cs: &[CardId]| -> f64 { cs.iter().map(|&c| (card_worth(g, o, c, false)).max(0.0)).sum() };
    match (one_art, two) {
        (Some(a), Some((x, y))) if cost(&[a]) <= cost(&[x, y]) + 4.0 => discard_cards(g, o, &[a])?,
        (Some(a), None) => discard_cards(g, o, &[a])?,
        (_, Some((x, y))) => discard_cards(g, o, &[x, y])?,
        (None, None) => {
            if let Some(&c) = hand.first() {
                discard_cards(g, o, &[c])?
            }
        }
    }
    Ok(())
}

fn alpharael_dt(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    m == src && kw == "deathtouch" && g.active == Some(g.perm(src).owner)
}

/// **Stadium Tidalmage**: whenever it enters or attacks, you may draw a card; if you do, discard a card
fn tidalmage_loot(g: &mut Game, o: PlayerId, src: PermId) -> Res {
    let human = crate::human::is_human(g, o);
    if (!human && g.player(o).library.len() < 3) || !may_window(g, o, Some(src), "may loot", Some(2.0))? {
        return Ok(());
    }
    draw(g, o, 1, false)?;
    if human {
        return crate::human::choices::discard(g, o, 1, "Stadium Tidalmage: discard a card");
    }
    let hand = g.player(o).hand.clone();
    if let Some(c) = discard_choice(g, o, &hand, 0) {
        discard_cards(g, o, &[c])?;
    }
    Ok(())
}

fn tidalmage_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && p == o {
        tidalmage_loot(g, o, src)?;
    }
    Ok(())
}

fn tidalmage_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && g.perm(src).owner == p {
        tidalmage_loot(g, p, src)?;
    }
    Ok(vec![])
}

/// **Screeching Soulbreaker**: flying; whenever it attacks, it deals 1 damage to each opponent and you gain 1 life
fn soulbreaker_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src)
        && g.perm(src).owner == p
        && trigger_window(g, p, Some(src), "1 damage to each opponent", Some(3.0))?
    {
        for q in alive_opps(g, p) {
            lose_life(g, q, 1, Some(p), "burn", Some(true))?;
        }
        gain(g, p, 1)?;
    }
    Ok(vec![])
}

/// **Sawblade Scamp**: haste; whenever you cast a noncreature spell, put an oil counter on it; {T}, remove an oil
/// counter: it deals 1 damage to each opponent
fn scamp_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    if caster == o && !g.db.get(c).creature && !g.db.get(c).land && g.perm(src).on_bf {
        let n = g.perm(src).data.int(DataKey::Counters);
        g.perm_mut(src).data.set(DataKey::Counters, Val::Int(n + 1));
    }
    Ok(())
}

fn scamp_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if post.is_none() || x.owner != p || x.tapped || x.phased || x.data.int(DataKey::Counters) < 1 {
        return Ok(vec![]);
    }
    if x.sick && !x.eot_kw.contains(&"haste") && !g.db.get(x.cd.unwrap()).tag(Tag::Haste) {
        return Ok(vec![]);
    }
    let opps = alive_opps(g, p).len() as f64;
    Ok(vec![Opt {
        utility: 2.0 + if flipped(g, p) { 2.5 * opps } else { 0.0 },
        label: "Sawblade Scamp: 1 damage to each opponent".into(),
        act: Some(Action::Ability { src, f: scamp_go, arg: 0 }),
    }])
}

fn scamp_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let n = g.perm(src).data.int(DataKey::Counters);
    if !g.perm(src).on_bf || g.perm(src).tapped || n < 1 {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    g.perm_mut(src).data.set(DataKey::Counters, Val::Int(n - 1));
    if crate::engine::stack::ability_window(g, p, Some(src), "1 damage to each opponent", Some(3.0), None)? {
        for q in alive_opps(g, p) {
            lose_life(g, q, 1, Some(p), "burn", Some(true))?;
        }
    }
    Ok(true)
}

/// **Susurian Voidborn**: whenever it or another creature or artifact you control dies, target opponent loses 1 life
/// and you gain 1 life. (Artifact tokens count: a Treasure, Clue or Food sacrificed is a dying artifact.) **Warp
/// {B}**: cast from your hand for {B}; exile it at the beginning of the next end step; you may cast it from exile on a
/// later turn (`voidborn_exile`).
fn voidborn_drain(g: &mut Game, src: PermId) -> Res {
    let o = g.perm(src).owner;
    if !g.player(o).alive || !trigger_window(g, o, Some(src), "target opponent loses 1 life", Some(3.0))? {
        return Ok(());
    }
    let opps = alive_opps(g, o);
    let Some(q) = min_by(&opps, |q| g.player(q).life as f64) else { return Ok(()) };
    lose_life(g, q, 1, Some(o), "drain", None)?;
    gain(g, o, 1)
}

fn voidborn_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(src).owner;
    let x = g.perm(m);
    let mine = x.owner == o;
    let is_art = x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT));
    if mine && (g.is_creature(m) || is_art) {
        voidborn_drain(g, src)?;
    }
    Ok(())
}

fn voidborn_sacrifice(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    // a Treasure, Clue or Food token is an artifact that dies (permanents sacrificed die through `dies`)
    if g.perm(src).owner == p && matches!(what, Sacrificed::Token(_)) {
        voidborn_drain(g, src)?;
    }
    Ok(())
}

fn voidborn_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    if !g.perm(src).on_bf || !g.perm(src).data.truthy(DataKey::Warp) {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let c = g.perm(src).cd.unwrap();
    g.perm_mut(src).data.remove(DataKey::Warp);
    crate::glog!(g, "    Susurian Voidborn (warp) is exiled");
    exile_perm_to_pile(g, src, o, c)
}

/// exile the warped creature: its card waits in the owner's exile for a later turn
fn exile_perm_to_pile(g: &mut Game, m: PermId, o: PlayerId, c: CardId) -> Res {
    exile_perm(g, m)?;
    // exile_perm puts the card in the exile pile; remember the turn so it can be cast on a later turn only
    let st = g.turn_stamp();
    g.player_mut(o).flag_turn.insert(intern("voidborn_warped"), st);
    if !g.player(o).exile.contains(&c) {
        g.player_mut(o).exile.push(c);
    }
    Ok(())
}

fn voidborn_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 0, "B", false) {
        return Ok(vec![]);
    }
    // warp pays when the full cost can't be paid this turn, or there is more to cast than mana
    let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
    if can_pay(g, p, cg, &cp, false) {
        return Ok(vec![]);
    }
    let _ = c;
    Ok(vec![Opt {
        utility: 2.2,
        label: "Susurian Voidborn (warp)".into(),
        act: Some(Action::Plan { f: voidborn_warp, arg: c.0 as i64 }),
    }])
}

fn voidborn_warp(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    cast_card(g, p, c, "hand", Ctx::default())?;
    let found = g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c));
    if let Some(m) = found {
        g.perm_mut(m).data.set(DataKey::Warp, Val::Bool(true));
    }
    Ok(true)
}

/// the warped Voidborn in exile: cast from there on a later turn, for its mana cost
pub fn kefka_options(g: &mut Game, p: PlayerId, post: bool) -> Res<Vec<Opt>> {
    let mut out = vec![];
    out.extend(crate::cardcode::breach_options(g, p, post)?);
    out.extend(will_options(g, p));
    if post.then_some(()).is_some() || true {
        let st = g.turn_stamp();
        let later = g.player(p).flag_turn.get(&intern("voidborn_warped")).is_none_or(|s| *s != st);
        if later && let Some(c) = g.player(p).exile.iter().copied().find(|&c| cname(g, c) == VOIDBORN) {
            let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
            if can_pay(g, p, cg, &cp, false) && g.active == Some(p) {
                out.push(Opt {
                    utility: 4.5,
                    label: "Susurian Voidborn (from exile)".into(),
                    act: Some(Action::Plan { f: voidborn_from_exile, arg: c.0 as i64 }),
                });
            }
        }
    }
    Ok(out)
}

fn voidborn_from_exile(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
    if !g.player(p).exile.contains(&c) || !can_pay(g, p, cg, &cp, false) {
        return Ok(false);
    }
    pay(g, p, cg, &cp, false)?;
    remove_card(&mut g.player_mut(p).exile, c);
    g.player_mut(p).flag_turn.shift_remove(&intern("voidborn_warped"));
    cast_card(g, p, c, "lib", Ctx::default())?;
    Ok(true)
}

/// **Corroding Dragonstorm**: when it enters, each opponent loses 2 life and you gain 2 life; surveil 2. When a Dragon
/// you control enters, return it to its owner's hand. (No Dragon is in the deck; the clause works for a Dragon token
/// or a Dragon taken from another deck.)
fn dragonstorm_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if p != o {
        return Ok(());
    }
    if m == src {
        if trigger_window(g, o, Some(src), "drain 2, surveil 2", Some(3.0))? {
            for q in alive_opps(g, o) {
                lose_life(g, q, 2, Some(o), "drain", None)?;
            }
            gain(g, o, 2)?;
            surveil(g, o, 2)?;
        }
    } else {
        let is_dragon = g.perm(m).cd.is_some_and(|c| g.db.get(c).has_subtype("dragon"))
            || g.perm(m).ttypes.iter().any(|t| &**t == "dragon");
        if is_dragon && g.perm(src).on_bf && trigger_window(g, o, Some(src), "return to hand", Some(1.0))? {
            bounce(g, src)?;
        }
    }
    Ok(())
}

/// surveil n: look at the top n cards; each goes to the graveyard or back on top (cards worth keeping stay)
pub fn surveil(g: &mut Game, p: PlayerId, n: usize) -> Res {
    let n = n.min(g.player(p).library.len());
    let top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
    let mut keep = vec![];
    for c in top {
        let bin = wants_gy(g, c) && !g.player(p).hand.iter().any(|&x| x == c) || card_worth(g, p, c, false) <= 6.0;
        if bin {
            g.player_mut(p).gy.push(c);
        } else {
            keep.push(c);
        }
    }
    // best card on top (the library's top is its end)
    keep.sort_by(|&a, &b| card_worth(g, p, a, false).total_cmp(&card_worth(g, p, b, false)));
    g.player_mut(p).library.extend(keep);
    Ok(())
}

/// **Intellect Devourer**: when it enters, each opponent exiles a card from their hand until it leaves the
/// battlefield; you may play lands and cast spells from among those cards, spending mana as though it were mana of
/// any color. The cards are held in your hand as Opposition Agent's are (`take_card`); if the Devourer leaves, those
/// you haven't cast go back to their owners' hands.
fn devourer_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || p != o || !trigger_window(g, o, Some(src), "each opponent exiles a card from hand", Some(4.0))? {
        return Ok(());
    }
    let mut held = vec![];
    for q in alive_opps(g, o) {
        let hand: Vec<CardId> = g.player(q).hand.iter().copied().filter(|&c| c != g.player(q).cmd).collect();
        // their choice: the card they miss least
        let Some(c) = min_by(&hand, |c| card_worth(g, q, c, false)) else { continue };
        remove_card(&mut g.player_mut(q).hand, c);
        take_card(g, o, c, q, "Intellect Devourer");
        held.push(Val::Card(c));
        held.push(Val::Player(q));
    }
    g.perm_mut(src).data.set(DataKey::Hostage, Val::List(held));
    Ok(())
}

fn devourer_leaves(g: &mut Game, _src: Src, m: PermId) -> Res {
    let Some(Val::List(v)) = g.perm(m).data.get(DataKey::Hostage).cloned() else { return Ok(()) };
    if name_of(g, m) != "Intellect Devourer" {
        return Ok(());
    }
    g.perm_mut(m).data.set(DataKey::Hostage, Val::None);
    let o = g.perm(m).owner;
    for pair in v.chunks(2) {
        if let [Val::Card(c), Val::Player(q)] = pair {
            let pl = g.player(o);
            if pl.hand.contains(c) && pl.stolen.get(c) == Some(q) {
                let pl = g.player_mut(o);
                remove_card(&mut pl.hand, *c);
                pl.stolen.shift_remove(c);
                pl.agent_ids.retain(|x| x != c);
                crate::glog!(g, "    {} returns to {}'s hand (Intellect Devourer left)", cname(g, *c), pname(g, *q));
                g.player_mut(*q).hand.push(*c);
            }
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ Gemstone Mine
/// **Gemstone Mine**: enters with three mining counters; {T}, remove a mining counter: add one mana of any color; if
/// there are no counters on it, sacrifice it. The counters are kept as the taps made (`Used`).
fn mine_tap(g: &mut Game, p: PlayerId, l: crate::ids::LandId, _n: u32) -> Res {
    let used = g.land(l).data.int(DataKey::Used) + 1;
    g.land_mut(l).data.set(DataKey::Used, Val::Int(used));
    if used >= 3 {
        crate::glog!(g, "    Gemstone Mine has no mining counters left: sacrificed");
        sac_land(g, p, l)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ cast priorities
fn zero(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

fn disintegrate_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

/// kefka's cast priority: the generic priorities (plus the cards' own `prio` slots), Psychosis Crawler held while
/// the loop with the flipped Kefka would deck us
pub fn kefka_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let name = cname(g, c);
    if name == CRAWLER || name == NIV {
        // every card drawn costs the table life: Kefka's discard and loot draws ping it even before the flip
        if flipped(g, p) {
            // the Crawler's loop is mandatory; Niv's pings are aimed (loop_guard), so he is never held
            return if name == NIV || loop_safe(g, p) { 80 } else { 0 };
        }
        return if kefka_out(g, p) { 84 } else { 70 };
    }
    if name == "Fiery Emancipation" {
        // triple damage: every Crawler / Niv ping is 3 to each opponent, and the flipped Kefka draws off all of it
        let pl = g.player(p);
        let pinger = |n: &str| {
            pl.perms.iter().any(|&m| crate::engine::values::card_name(g, m) == Some(n))
                || pl.hand.iter().any(|&x| cname(g, x) == n)
        };
        return if pinger(CRAWLER) || pinger(NIV) || flipped(g, p) { 78 } else { 45 };
    }
    if name == "Bribery" {
        // the best creature from an opponent's library: a bomb on Kefka's side (card code: galadriel.rs)
        return 62;
    }
    if let Some(v) = crate::ai::decks::breach_hold_prio(g, p, c) {
        return v; // the Breach line holds Breach, the storm cards and the rituals for itself
    }
    if name == "Underworld Breach" {
        // the graveyard Kefka fills (discards, looting) is the fuel: escape what was binned, once there is mana for it
        let k = crate::ai::decks::breach_candidates(g, p, false).len();
        return if k >= 2 && total_mana(g, p, false) >= 5 { 56 } else { 0 };
    }
    if name == "Triskelion" {
        // three pings: each one is a card once Kefka is flipped
        return if flipped(g, p) {
            74
        } else if kefka_out(g, p) {
            56
        } else {
            generic_prio(g, p, c)
        };
    }
    if let Some(v) = crate::impls::newcards_b::marvin_prio(g, p, c) {
        return v;
    }
    if name == "Rapturous Moment" {
        // draw three, discard two (Kefka's fodder) and five red mana back: a free filter whenever the hand can spend it
        let others = g.player(p).hand.iter().filter(|&&x| x != c && !g.db.get(x).land).count();
        return if others >= 2 { 58 } else { generic_prio(g, p, c) };
    }
    if (name == SCARLET_SPIDER || name == DEMONIC_EMBRACE) && kefka_out(g, p) && !flipped(g, p) {
        return 0; // kept in hand to be discarded to Kefka's trigger (mayhem / the graveyard cast)
    }
    generic_prio(g, p, c)
}

/// when to pay {8} for Kefka's flip (the utility of the option): after the plays that matter, unless holding up
/// interaction is clearly better; never starting the Crawler loop with a library that can't outlast the table
pub fn flip_utility(g: &Game, p: PlayerId) -> Option<f64> {
    if is_kefka(g, p) && crawler_out(g, p) && !loop_safe(g, p) {
        return None;
    }
    if !is_kefka(g, p) {
        return Some(3.0 + 0.5 * g.opps(p).count() as f64);
    }
    let mut u = 5.8 + 0.4 * g.opps(p).count() as f64;
    // holding interaction (a counterspell or instant removal) with the mana it needs, and danger on the table
    let held = g.player(p).hand.iter().any(|&c| {
        let d = g.db.get(c);
        d.instant && (d.tag(Tag::Ctr) || d.tag(Tag::Rem))
    });
    if held && total_mana(g, p, false) < 8 + 2 {
        u -= 1.2;
    }
    if crawler_out(g, p) {
        u += 2.0; // the flip turns the Crawler into the kill
    }
    Some(u)
}

// ------------------------------------------------------------------ registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, OPPRESSION)?;
    c.cast = Some(oppression);
    c.prio = Some(oppression_prio);
    for (name, f) in [
        ("Duress", duress as crate::hooks::ResolveFn),
        ("Dark Inquiry", dark_inquiry),
        ("Inner Fire", inner_fire),
        (DECORUM, decorum),
        ("Arcane Omens", omens),
        (ICHOR_SLICK, ichor_slick),
        ("Extinguish the Light", extinguish),
        ("Grab the Prize", grab_the_prize),
        (TWINNED_VISION, twinned_vision),
        ("Stargaze", stargaze),
        ("Disintegrate", disintegrate_resolve),
        ("Splatter Technique", splatter),
    ] {
        r.card(db, name)?.resolve = Some(f);
    }
    r.card(db, "Duress")?.prio = Some(duress_prio);
    r.card(db, "Dark Inquiry")?.prio = Some(inquiry_prio);
    r.card(db, "Inner Fire")?.prio = Some(inner_fire_prio);
    r.card(db, DECORUM)?.prio = Some(decorum_prio);
    r.card(db, ICHOR_SLICK)?.prio = Some(slick_prio);
    for name in ["Arcane Omens", "Grab the Prize", "Stargaze", "Disintegrate", "Splatter Technique"] {
        r.card(db, name)?.prio = Some(zero);
    }
    let _ = disintegrate_prio;
    r.card(db, "Arcane Omens")?.hand_options = Some(omens_hand);
    r.card(db, ICHOR_SLICK)?.hand_options = Some(slick_hand);
    r.card(db, "Grab the Prize")?.hand_options = Some(grab_hand);
    r.card(db, "Stargaze")?.hand_options = Some(stargaze_hand);
    r.card(db, "Disintegrate")?.hand_options = Some(disintegrate_hand);
    r.card(db, "Splatter Technique")?.hand_options = Some(splatter_hand);
    r.card(db, TWINNED_VISION)?.gy_options = Some(twinned_gy);
    r.card(db, DEMONIC_EMBRACE)?.gy_options = Some(embrace_gy);
    aura(r, db, DEMONIC_EMBRACE, &EMBRACE)?;
    let c = r.card(db, SCARLET_SPIDER)?;
    c.etb = Some(spider_etb);
    c.gy_options = Some(mayhem_gy);
    let c = r.card(db, POSTMORTEM)?;
    c.attack = Some(professor_attack);
    c.gy_options = Some(professor_gy);
    let c = r.card(db, "Alpharael, Dreaming Acolyte")?;
    c.etb = Some(alpharael_etb);
    c.grant_kw = Some(alpharael_dt);
    let c = r.card(db, "Stadium Tidalmage")?;
    c.etb = Some(tidalmage_etb);
    c.attack = Some(tidalmage_attack);
    r.card(db, "Screeching Soulbreaker")?.attack = Some(soulbreaker_attack);
    let c = r.card(db, "Sawblade Scamp")?;
    c.cast = Some(scamp_cast);
    c.options = Some(scamp_options);
    let c = r.card(db, VOIDBORN)?;
    c.dies = Some(voidborn_dies);
    c.sacrifice = Some(voidborn_sacrifice);
    c.end_step = Some(voidborn_end);
    c.hand_options = Some(voidborn_hand);
    r.card(db, "Corroding Dragonstorm")?.etb = Some(dragonstorm_etb);
    let c = r.card(db, "Intellect Devourer")?;
    c.etb = Some(devourer_etb);
    c.leaves = Some(devourer_leaves);
    r.card(db, "Gemstone Mine")?.on_tap_land = Some(mine_tap);
    Ok(())
}

/// **Disintegrate** resolves through the engine's burn path (`ctx.rem_kind` = `dmgX`, `ctx.face`): X damage to any
/// target, a creature exiled instead of dying (the card's `exiledie` tag) and not regenerating (`noregen`).
fn disintegrate_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let n: i32 = ctx.x;
    if let Some(q) = ctx.face {
        if !crate::engine::values::player_hexproof(g, q) {
            lose_life(g, q, n, Some(p), "burn", Some(true))?;
        }
    } else if let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) {
        if etgh(g, t) > n {
            return Ok("gy"); // damage that doesn't kill leaves nothing lasting
        }
        g.noregen = true;
        let r = apply_removal(g, Some(p), t, &format!("dmg{n}"), Some(c));
        g.noregen = false;
        r?;
    }
    Ok("gy")
}

// ======================================================== Yawgmoth's Will (the Kefka storm/Breach package)
const WILL: &str = "Yawgmoth's Will";

/// is the graveyard worth replaying? Returns the option's utility: a storm card (Brain Freeze, Grapeshot) with the
/// mana to cast it after the Will, or three cheap castable spells that fit in the mana left; never into an empty yard
fn will_value(g: &Game, p: PlayerId) -> Option<f64> {
    let pl = g.player(p);
    if pl.yawg || g.active != Some(p) || !can_pay(g, p, 2, "B", false) {
        return None;
    }
    let left = total_mana(g, p, false).saturating_sub(3);
    let cost = |c: CardId| {
        let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
        cg + cp.chars().count() as u32
    };
    let rituals = pl
        .gy
        .iter()
        .filter(|&&c| matches!(&*g.db.get(c).name, "Dark Ritual" | "Cabal Ritual" | "Lotus Petal" | "Jeska's Will"))
        .count() as u32;
    let storm = pl.gy.iter().any(|&c| matches!(&*g.db.get(c).name, "Brain Freeze" | "Grapeshot"));
    if storm && left + 2 * rituals >= 2 {
        return Some(7.0);
    }
    let mut costs: Vec<u32> = pl
        .gy
        .iter()
        .filter(|&&c| {
            let d = g.db.get(c);
            !d.land && !d.tag(Tag::Rean) && !d.tag(Tag::Fill) && !d.tag(Tag::Yawg)
        })
        .map(|&c| cost(c))
        .collect();
    costs.sort_unstable();
    (costs.len() >= 3 && costs.iter().take(3).sum::<u32>() <= left).then_some(5.0)
}

fn will_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some(w) = g.player(p).hand.iter().copied().find(|&c| cname(g, c) == WILL) else { return Ok(false) };
    if will_value(g, p).is_none() {
        return Ok(false);
    }
    pay(g, p, 2, "B", false)?;
    cast_card(g, p, w, "hand", Ctx::default())?;
    Ok(true)
}

/// Yawgmoth's Will for a deck that isn't built around it: only with a yard that pays for itself
pub fn will_options(g: &mut Game, p: PlayerId) -> Vec<Opt> {
    if !g.player(p).hand.iter().any(|&c| cname(g, c) == WILL) {
        return vec![];
    }
    will_value(g, p)
        .map(|u| Opt {
            utility: u,
            label: "Yawgmoth's Will (replay the graveyard)".into(),
            act: Some(Action::Plan { f: will_go, arg: 0 }),
        })
        .into_iter()
        .collect()
}

/// Brain Freeze or Grapeshot is in hand, graveyard or library (so Breach / Yawgmoth's Will still has a finisher)
fn storm_card_left(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    pl.hand
        .iter()
        .chain(pl.gy.iter())
        .chain(pl.library.iter())
        .any(|&c| matches!(cname(g, c), "Brain Freeze" | "Grapeshot"))
}

/// a card that protects Kefka: Deflecting Swat (untagged) or a protection-tagged spell (Slip Out the Back)
fn is_protection(g: &Game, c: CardId) -> bool {
    cname(g, c) == "Deflecting Swat"
        || matches!(g.db.get(c).tags.str(Tag::Prot), Some("hi" | "phase" | "indes" | "blink"))
}

/// What the tutors fetch when the board wants the two-card win rather than the Breach line: Kefka is flipped, or is
/// out with {8} within a land drop, and neither Psychosis Crawler nor Niv-Mizzet is in hand or on the battlefield.
/// The Crawler first while the library outlasts the table's life (the loop is safe), else Niv-Mizzet.
pub fn tutor_win_piece(g: &Game, p: PlayerId) -> Option<[&'static str; 2]> {
    if !is_kefka(g, p) {
        return None;
    }
    let pl = g.player(p);
    let rocks = pl.perms.iter().filter(|&&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Rock))).count();
    let ready = flipped(g, p) || (kefka_out(g, p) && pl.lands.len() + rocks >= 7);
    let held = |n: &str| pl.hand.iter().any(|&c| cname(g, c) == n);
    if !ready || crawler_out(g, p) || niv_out(g, p) || held(CRAWLER) || held(NIV) {
        return None;
    }
    Some(if loop_safe(g, p) { [CRAWLER, NIV] } else { [NIV, CRAWLER] })
}

// ------------------------------------------------------------------ Oppression
const OPPRESSION: &str = "Oppression";

/// **Oppression**: "Whenever a player casts a spell, that player discards a card." Symmetric: Kefka's own spells
/// cost him a card too (the discard goes through `discard_cards`, so madness, mayhem and the graveyard fuel apply).
/// The caster chooses (`discard_worst`: Kefka bins his madness / mayhem cards first). Nothing happens on an empty hand.
fn oppression(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    let o = g.perm(src).owner;
    if g.player(caster).hand.is_empty() || !g.player(caster).alive {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "the caster discards a card", None)? {
        return Ok(());
    }
    if g.player(caster).hand.is_empty() {
        return Ok(());
    }
    crate::glog!(g, "    Oppression: {} discards a card", pname(g, caster));
    crate::engine::cast::discard_worst(g, caster, 1)
}

/// Kefka's cast priority for Oppression: a tax on every spell, his own included, that falls on whoever holds cards. He
/// dumps his hand (looting, discards to Kefka's trigger), the table does not, so the fewer cards he keeps the better
/// it is for him: top priority with a discard payoff in play (Kefka out, a flip, madness / mayhem cards in hand) while
/// the opponents hold cards; high when the opponents hold more cards than he does; low otherwise. Never while a combo
/// turn is in hand (the storm cards, Breach, Will: every spell of the turn would cost a card).
fn oppression_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let hand = &g.player(p).hand;
    // a combo turn in hand: a storm card or Breach / Will with mana to go off and spells to chain (Kefka sits on the
    // pieces for turns, so the mere presence of one is not enough)
    let piece = hand
        .iter()
        .any(|&x| matches!(cname(g, x), "Brain Freeze" | "Grapeshot" | "Underworld Breach" | "Yawgmoth's Will"));
    let chain = hand.iter().filter(|&&x| x != c && !g.db.get(x).land).count();
    if piece && chain >= 4 && total_mana(g, p, false) >= 6 {
        return 0;
    }
    let spare = hand.iter().filter(|&&x| x != c).count() as i32;
    let opp_cards: i32 = g.opps(p).map(|q| g.player(q).hand.len() as i32).sum();
    let gy_fuel = hand.iter().filter(|&&x| x != c && wants_gy(g, x)).count() as i32;
    let payoff = kefka_out(g, p) || flipped(g, p) || gy_fuel > 0;
    if payoff && opp_cards > 0 {
        66
    } else if opp_cards >= spare && opp_cards > 0 {
        58
    } else {
        25
    }
}
