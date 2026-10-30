//! Tier 2 pool decks `kaalia-mardu-creature-cheat` and `sythis-selesnya-enchantress` (EDHREC's bracket-3 average
//! lists): the cards the ability language doesn't cover (rust/agents/t12-kaalia-sythis.md). Resurgent Belief's "return
//! all enchantment cards from your graveyard to the battlefield" is compiled (dsl_parse.rs, `// --- Tiers 1/2
//! (kaalia-sythis)`: the `all` reanimation); the rest is here.
//!
//! Kaalia:
//! - Angelic Arbiter: an opponent who cast a spell this turn can't attack (attack cap 0); an opponent who attacked
//!   this turn can't cast spells.
//! - Avacyn, Angel of Horror: she or another nontoken creature you control dies: that card returns to the battlefield
//!   under your control at the beginning of the next end step (those dying in the same wipe as her too).
//! - Drana and Linvala: activated abilities of creatures your opponents control can't be activated (their mana
//!   abilities included). Left out: Drana and Linvala doesn't gain those abilities.
//! - Kaalia, Zenith Seeker: enters: look at the top six, an Angel, a Demon and a Dragon card from among them to hand
//!   (the best of each), the rest on the bottom in a random order.
//! - Liesa, Forgotten Archangel: another nontoken creature you control dies: that card returns to its owner's hand at
//!   the beginning of the next end step; a creature an opponent controls that would die is exiled instead.
//! - Lord of the Void: combat damage to a player: exile their top seven, the best creature card among them onto the
//!   battlefield under your control.
//! - Rakdos, Patron of Chaos: your end step: the most threatening opponent sacrifices its two least valuable nonland,
//!   nontoken permanents when they are worth less to it than your two cards, else you draw two.
//! - Serra's Emissary: chooses creature: you take no combat damage, and creatures can't block your creatures. Left
//!   out: the other card types, and damage from creatures to your creatures (they block unharmed only in the rules).
//! - Valgavoth, Terror Eater: ward (sacrifice three nonland permanents; the AI pays when they are worth less than
//!   Valgavoth, else its spell is countered); cards going to an opponent's graveyard are exiled instead (creatures
//!   dying, and every other card as state-based sweep); during your turn, cast those cards for life equal to their
//!   mana value. Left out: lands among them are not played.
//! - Vilis, Broker of Blood: {B}, pay 2 life: target creature gets -1/-1 until end of turn (the AI kills an X/1 or
//!   uses it to draw while it has life to spare); whenever you lose life, draw that many cards.
//!
//! Sythis:
//! - Strength of the Harvest // Haven of the Harvest: enchanted creature gets +1/+1 for each creature and/or
//!   enchantment you control; or played as a land that enters tapped and taps for {G} or {W}.
//! - Branchloft Pathway // Boulderloft Pathway: t45_prosper's Pathways ({G} or {W}, chosen as it is played).
//! - Calix, Guided by Fate: constellation: a +1/+1 counter on your best creature; Calix or an enchanted creature of
//!   yours deals combat damage to a player: once each turn, a token copy of your best nonlegendary enchantment.
//! - Resurgent Belief: suspend 2 for {1}{W} (from hand, with enchantments in the graveyard worth returning); never
//!   hard-cast (no mana cost).
//! - Weaver of Harmony: (anthem compiled) {G},{T}: copy an enchantment source's draw trigger: when you cast an
//!   enchantment spell while you control Enchantress's Presence, Sythis or Eidolon of Blossoms, a card more. Left out:
//!   copies of other triggered and activated abilities.

use crate::engine::repl;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, casts_this_turn};
use crate::engine::life::{check_state, lose_life};
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{epow, etgh, pval, threat};
use crate::engine::zones::{Enter, die, draw, enter, enter_token_copy, max_by, sac_worth};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{AURA, AuraSpec, aura};
use crate::impls::partials::{at_once, remove_card};
use crate::impls::t3::{ability, plan};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};

const ARBITER: &str = "Angelic Arbiter";
const AVACYN: &str = "Avacyn, Angel of Horror";
const DRANA: &str = "Drana and Linvala";
const SEEKER: &str = "Kaalia, Zenith Seeker";
const LIESA: &str = "Liesa, Forgotten Archangel";
const LORD: &str = "Lord of the Void";
const RAKDOS: &str = "Rakdos, Patron of Chaos";
const EMISSARY: &str = "Serra's Emissary";
const VALGAVOTH: &str = "Valgavoth, Terror Eater";
const VILIS: &str = "Vilis, Broker of Blood";
const HARVEST: &str = "Strength of the Harvest // Haven of the Harvest";
const CALIX: &str = "Calix, Guided by Fate";
const BELIEF: &str = "Resurgent Belief";
const WEAVER: &str = "Weaver of Harmony";
/// enchantment sources whose triggers draw when you cast an enchantment or one enters (Weaver of Harmony copies one)
const DRAW_ENGINES: [&str; 3] = ["Enchantress's Presence", "Sythis, Harvest's Hand", "Eidolon of Blossoms"];

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

/// p's live permanent with this name
fn mine(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && named(g, m, name))
}

/// the live permanent with this name and its controller, anywhere
fn anyone(g: &Game, name: &str) -> Option<(PlayerId, PermId)> {
    g.players.iter().filter(|q| q.alive).find_map(|q| mine(g, q.id, name).map(|m| (q.id, m)))
}

fn is_ench(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ENCHANTMENT))
}

/// the card a dying permanent puts in the graveyard (a clone's own card), with its owner; None for a token
fn dying_card(g: &Game, m: PermId) -> Option<(CardId, PlayerId)> {
    let x = g.perm(m);
    if x.token {
        return None;
    }
    x.phys.or(x.cd).map(|c| (c, x.orig))
}

// ================================================================== Angelic Arbiter
/// Each opponent who cast a spell this turn can't attack with creatures.
fn arbiter_cap(g: &Game, src: Src, p: PlayerId, _d: PlayerId) -> Option<i32> {
    (live(g, src) && p != owner(g, src) && casts_this_turn(g, p) > 0).then_some(0)
}

/// Each opponent who attacked with a creature this turn can't cast spells.
fn arbiter_cast(g: &Game, src: Src, caster: PlayerId, _c: CardId, _zone: Sym) -> bool {
    !(live(g, src) && caster != owner(g, src) && g.player(caster).attacking == Some(g.turn_stamp()))
}

// ================================================================== Avacyn, Angel of Horror / Liesa, Forgotten Archangel
/// queue the card of creature m (controlled by p) to return at the beginning of the next end step: to the battlefield
/// under p's control (Avacyn), or to its owner's hand (Liesa)
fn queue_return(g: &mut Game, p: PlayerId, m: PermId, to_hand: bool) {
    let Some((cd, own)) = dying_card(g, m) else { return };
    g.t12_due.push((p, cd, own, to_hand));
    let how = if to_hand { "to its owner's hand" } else { "to the battlefield" };
    crate::glog!(g, "    {} returns {how} at the beginning of the next end step", cname(g, cd));
}

/// a nontoken creature m of src's controller died: the trigger (Avacyn: another one; Liesa: another one)
fn return_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, src);
    if m == src || owner(g, m) != o || !g.is_creature(m) || g.perm(m).token {
        return Ok(());
    }
    let to_hand = named(g, src, LIESA);
    let label =
        if to_hand { "return it to its owner's hand at the next end step" } else { "return it at the next end step" };
    if trigger_window(g, o, Some(src), label, Some(3.0))? {
        queue_return(g, o, m, to_hand);
    }
    Ok(())
}

/// Avacyn herself dies: she returns too, and so do the creatures of hers dying in the same wipe after her (her
/// ability looks back in time: `batch_dies`)
fn avacyn_self_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    note_batch(g, o, false);
    if trigger_window(g, o, Some(src), "return Avacyn at the next end step", Some(4.0))? {
        queue_return(g, o, m, false);
    }
    Ok(())
}

/// Liesa dies in a wipe: the creatures of hers dying with her still return to hand (`batch_dies`)
fn liesa_self_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m == src {
        note_batch(g, owner(g, src), true);
    }
    Ok(())
}

fn note_batch(g: &mut Game, p: PlayerId, to_hand: bool) {
    let Some(b) = g.cur_batch else { return };
    g.t12_batch.retain(|e| e.1 == b);
    g.t12_batch.push((p, b, to_hand));
}

/// engine/zones.rs `die_triggers`: p's nontoken creature m died in the wipe that killed p's Avacyn or Liesa before it
pub fn batch_dies(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let Some(b) = g.cur_batch else { return Ok(()) };
    let hits: Vec<bool> = g.t12_batch.iter().filter(|e| e.0 == p && e.1 == b).map(|e| e.2).collect();
    for to_hand in hits {
        let label = if to_hand { "Liesa: return it to hand at the next end step" } else { "Avacyn: return it" };
        if trigger_window(g, p, None, label, Some(3.0))? {
            queue_return(g, p, m, to_hand);
        }
    }
    Ok(())
}

/// cardcode::delayed_end_step: the cards Avacyn and Liesa are waiting on come back (if still in the graveyard)
pub fn end_returns(g: &mut Game) -> Res {
    let due = std::mem::take(&mut g.t12_due);
    for (p, cd, own, to_hand) in due {
        if g.over || !g.player(p).alive || !g.player(own).alive {
            continue;
        }
        if !to_hand && crate::impls::t45_winota::jailed(g) {
            continue; // Soulless Jailer: permanent cards in graveyards can't enter
        }
        if !remove_card(&mut g.player_mut(own).gy, cd) {
            continue; // exiled, reanimated or gone to the command zone meanwhile
        }
        if to_hand {
            g.player_mut(own).hand.push(cd);
            crate::glog!(g, "  Liesa returns {} to {}'s hand", cname(g, cd), pname(g, own));
        } else {
            enter(g, p, cd, Enter { orig: Some(own), ..Enter::default() })?;
            crate::glog!(g, "  Avacyn returns {} to the battlefield ({})", cname(g, cd), pname(g, p));
        }
    }
    check_state(g)
}

/// Liesa: if a creature an opponent controls would die, exile it instead (engine/zones.rs `die` reads it)
fn liesa_no_gy(g: &Game, src: Src, p: PlayerId) -> i32 {
    (live(g, src) && p != owner(g, src)) as i32
}

// ================================================================== Drana and Linvala
/// Activated abilities of creatures your opponents control can't be activated (impls/rules.rs `ability_locked`); their
/// mana abilities neither.
fn drana_mana(g: &Game, src: Src, p: PlayerId) -> i32 {
    (live(g, src) && p != owner(g, src)) as i32
}

// ================================================================== Kaalia, Zenith Seeker
/// When Kaalia enters, look at the top six; an Angel, a Demon and/or a Dragon card to hand, the rest to the bottom in
/// a random order.
fn seeker_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m != src || g.player(o).library.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "look at the top six for an Angel, a Demon and a Dragon", Some(3.0))? {
        return Ok(());
    }
    let mut top = crate::impls::t0_captain::pop_top(g, o, 6);
    for t in ["angel", "demon", "dragon"] {
        let fits: Vec<CardId> = top.iter().copied().filter(|&c| g.db.get(c).has_subtype(t)).collect();
        if let Some(c) = max_by(&fits, |c| card_worth(g, o, c, false) + g.db.get(c).cmc as f64) {
            remove_card(&mut top, c);
            g.player_mut(o).hand.push(c);
            crate::glog!(g, "    Kaalia, Zenith Seeker finds {}", cname(g, c));
        }
    }
    crate::impls::t0_captain::bottom_random(g, o, top);
    Ok(())
}

// ================================================================== Lord of the Void
/// Whenever it deals combat damage to a player, exile the top seven cards of that player's library, then put a
/// creature card from among them onto the battlefield under your control.
fn lord_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if a != src || g.player(d).library.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "exile their top seven, a creature onto the battlefield", Some(4.0))? {
        return Ok(());
    }
    let top = crate::impls::t0_captain::pop_top(g, d, 7);
    g.player_mut(d).exile.extend(&top);
    let cr: Vec<CardId> = top.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    crate::glog!(g, "    Lord of the Void exiles {} cards of {}'s", top.len(), pname(g, d));
    let Some(c) = max_by(&cr, |c| g.db.get(c).bomb as f64 * 2.0 + g.db.get(c).pow as f64 + g.db.get(c).cmc as f64)
    else {
        return Ok(());
    };
    if crate::impls::t45_winota::jailed(g) {
        return Ok(());
    }
    remove_card(&mut g.player_mut(d).exile, c);
    enter(g, p, c, Enter { orig: Some(d), ..Enter::default() })?;
    crate::glog!(g, "    Lord of the Void puts {} onto the battlefield", cname(g, c));
    Ok(())
}

// ================================================================== Rakdos, Patron of Chaos
/// q's two least valuable nonland, nontoken permanents (none when it has fewer than two)
fn rakdos_fodder(g: &Game, q: PlayerId) -> Vec<PermId> {
    let mut v: Vec<PermId> = g
        .player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && !g.perm(m).token && g.perm(m).cd.is_some_and(|c| !g.db.get(c).land))
        .collect();
    if v.len() < 2 {
        return vec![];
    }
    v.sort_by(|&a, &b| sac_worth(g, a).total_cmp(&sac_worth(g, b)));
    v.truncate(2);
    v
}

/// At the beginning of your end step, target opponent may sacrifice two nonland, nontoken permanents of their choice.
/// If they don't, you draw two cards. (The target: the most threatening opponent; it sacrifices when the two are worth
/// less to it than two cards to you.)
fn rakdos_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if p != o {
        return Ok(());
    }
    let opps: Vec<PlayerId> = g.opps(o).filter(|&q| !crate::engine::hooks::any_player_hexproof(g, q)).collect();
    let Some(q) = max_by(&opps, |q| threat(g, o, q)) else { return Ok(()) };
    if !trigger_window(g, o, Some(src), "target opponent sacrifices two permanents or you draw two", Some(3.0))? {
        return Ok(());
    }
    let two = rakdos_fodder(g, q);
    let cost: f64 = two.iter().map(|&m| sac_worth(g, m)).sum();
    if two.len() == 2 && cost < 3.0 {
        for m in two {
            crate::glog!(g, "    {} sacrifices {} (Rakdos)", pname(g, q), g.perm(m).name);
            die(g, m, "sac")?;
        }
        return Ok(());
    }
    crate::glog!(g, "    {} declines: {} draws two (Rakdos)", pname(g, q), pname(g, o));
    draw(g, o, 2, false)
}

// ================================================================== Serra's Emissary
/// As this creature enters, choose a card type: creature (the type the decks it meets attack with).
fn emissary_as_enters(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    g.perm_mut(m).data.set(DataKey::Ctype, Val::Str("creature"));
    g.t12_emissary = true;
    crate::glog!(g, "    Serra's Emissary chooses creature");
    Ok(())
}

/// p has protection from creatures (a Serra's Emissary of p's chose creature)
fn creature_prot(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        live(g, m) && named(g, m, EMISSARY) && crate::impls::galadriel::ctype(g, m) == Some(intern("creature"))
    })
}

/// engine/combat.rs: combat damage from creatures to p is prevented (you have protection from creatures)
pub fn emissary_shields(g: &Game, p: PlayerId) -> bool {
    g.t12_emissary && creature_prot(g, p)
}

/// cardcode::evasion_blocked: creatures can't block a creature whose controller's Emissary chose creature
pub fn evasion_blocked(g: &Game, b: PermId, a: PermId) -> bool {
    g.t12_emissary && g.is_creature(b) && creature_prot(g, owner(g, a))
}

// ================================================================== Valgavoth, Terror Eater
/// engine/removal.rs `legal_targets`: p can't pay Valgavoth's ward (fewer than three nonland permanents), so it
/// doesn't aim removal at it
pub fn ward_blocks(g: &Game, p: PlayerId, m: PermId) -> bool {
    named(g, m, VALGAVOTH) && ward_fodder(g, p, m).len() < 3
}

/// actor's three least valuable nonland permanents (fewer if it has fewer)
fn ward_fodder(g: &Game, actor: PlayerId, m: PermId) -> Vec<PermId> {
    let mut v: Vec<PermId> = g
        .player(actor)
        .perms
        .iter()
        .copied()
        .filter(|&x| x != m && live(g, x) && g.perm(x).cd.is_none_or(|c| !g.db.get(c).land))
        .collect();
    v.sort_by(|&a, &b| sac_worth(g, a).total_cmp(&sac_worth(g, b)));
    v.truncate(3);
    v
}

/// engine/removal.rs `apply_removal`: Ward—Sacrifice three nonland permanents. False: actor doesn't pay (or can't), and
/// its spell is countered.
pub fn sac_ward(g: &mut Game, actor: PlayerId, m: PermId) -> Res<bool> {
    if !named(g, m, VALGAVOTH) || actor == owner(g, m) {
        return Ok(true);
    }
    let fod = ward_fodder(g, actor, m);
    let cost: f64 = fod.iter().map(|&x| sac_worth(g, x)).sum();
    if fod.len() < 3 || cost >= pval(g, m) {
        return Ok(false);
    }
    for x in fod {
        crate::glog!(g, "    {} sacrifices {} for ward", pname(g, actor), g.perm(x).name);
        die(g, x, "sac")?;
    }
    Ok(true)
}

/// If a card you didn't control would be put into an opponent's graveyard from anywhere, exile it instead (creatures
/// dying: engine/zones.rs `die` through this; other cards: `valg_sweep`).
fn valg_no_gy(g: &Game, src: Src, p: PlayerId) -> i32 {
    (live(g, src) && p != owner(g, src)) as i32
}

/// engine/zones.rs `die`: m was exiled instead of dying (a graveyard replacement): Valgavoth keeps it if it was a card
/// of an opponent of its controller's
pub fn exiled_instead(g: &mut Game, m: PermId) {
    let Some((cd, own)) = dying_card(g, m) else { return };
    let Some((v, _)) = anyone(g, VALGAVOTH) else { return };
    if own != v && owner(g, m) != v {
        g.t12_void.push((cd, own));
    }
}

/// the cards in q's graveyard, as (card, q) entries
fn gy_entries(g: &Game, q: PlayerId) -> Vec<(CardId, PlayerId)> {
    g.player(q).gy.iter().map(|&c| (c, q)).collect()
}

/// state-based sweep: the cards that reached an opponent's graveyard since the last look are exiled with Valgavoth
/// (those that were there before it entered stay; so do the spells cast through it, which its controller controlled)
fn valg_sweep(g: &mut Game, src: Src) -> Res {
    let o = owner(g, src);
    let opps: Vec<PlayerId> = g.opps(o).collect();
    if g.t12_valg != Some(src) {
        g.t12_valg = Some(src);
        g.t12_void.clear();
        g.t12_pass.clear();
        g.t12_kept = opps.iter().flat_map(|&q| gy_entries(g, q)).collect();
        return Ok(());
    }
    for q in opps {
        let gy = g.player(q).gy.clone();
        let mut kept: Vec<(CardId, PlayerId)> = g.t12_kept.iter().copied().filter(|e| e.1 == q).collect();
        let mut out = vec![];
        for &c in &gy {
            if let Some(i) = kept.iter().position(|&e| e == (c, q)) {
                kept.swap_remove(i);
            } else if let Some(i) = g.t12_pass.iter().position(|&e| e == (c, q)) {
                g.t12_pass.swap_remove(i);
            } else {
                out.push(c);
            }
        }
        for c in out {
            remove_card(&mut g.player_mut(q).gy, c);
            g.player_mut(q).exile.push(c);
            g.t12_void.push((c, q));
            crate::glog!(g, "    Valgavoth exiles {} ({})", cname(g, c), pname(g, q));
        }
        g.t12_kept.retain(|e| e.1 != q);
        let now = gy_entries(g, q);
        g.t12_kept.extend(now);
    }
    Ok(())
}

/// the cards exiled with Valgavoth that are still in exile and can be cast (nonland)
fn valg_cards(g: &Game) -> Vec<(CardId, PlayerId)> {
    g.t12_void.iter().copied().filter(|&(c, q)| g.player(q).exile.contains(&c) && !g.db.get(c).land).collect()
}

/// During your turn, you may play cards exiled with Valgavoth; a spell cast this way costs life equal to its mana
/// value. The AI casts the best one it can afford with ten life to spare.
fn valg_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || owner(g, src) != p || !live(g, src) {
        return Ok(vec![]);
    }
    let life = g.player(p).life;
    let cs: Vec<(CardId, PlayerId)> = valg_cards(g)
        .into_iter()
        .filter(|&(c, _)| (g.db.get(c).cmc as i32) + 10 <= life && crate::engine::cast::castable(g, p, c, "lib"))
        .collect();
    let Some((c, q)) = max_by(&cs, |(c, _)| card_worth(g, p, c, false) + g.db.get(c).cmc as f64 * 5.0) else {
        return Ok(vec![]);
    };
    let label = format!("Valgavoth: cast {} for {} life", cname(g, c), g.db.get(c).cmc);
    let u = 2.0 + 0.5 * g.db.get(c).cmc as f64;
    Ok(vec![ability(u, label, src, valg_go, (c.0 as i64) | ((q.0 as i64) << 16))])
}

fn valg_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, q) = (CardId((arg & 0xffff) as u16), PlayerId((arg >> 16) as u8));
    if !live(g, src) || !valg_cards(g).contains(&(c, q)) {
        return Ok(false);
    }
    let mv = g.db.get(c).cmc as i32;
    remove_card(&mut g.player_mut(q).exile, c);
    if let Some(i) = g.t12_void.iter().position(|&e| e == (c, q)) {
        g.t12_void.remove(i);
    }
    g.t12_pass.push((c, q)); // its controller controlled it: back to q's graveyard for good
    crate::glog!(g, "  {} casts {} with Valgavoth, paying {} life", pname(g, p), cname(g, c), mv);
    lose_life(g, p, mv, Some(p), "life", Some(false))?;
    cast_card(g, p, c, "lib", Ctx { owner: Some(q), ..Ctx::default() })?;
    Ok(true)
}

// ================================================================== Vilis, Broker of Blood
/// Whenever you lose life, draw that many cards.
fn vilis_lose(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    let o = owner(g, src);
    if p != o || n <= 0 || !live(g, src) {
        return Ok(());
    }
    let label = format!("draw {n} card(s)");
    if trigger_window(g, o, Some(src), &label, Some(2.0 + n as f64 * 0.5))? {
        draw(g, o, n as u32, false)?;
    }
    Ok(())
}

/// {B}, Pay 2 life: target creature gets -1/-1 until end of turn. The AI kills the best opposing creature with
/// toughness 1, or (with 14 life or more and a library to draw from) shrinks the biggest one just to draw two.
fn vilis_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || !live(g, src) || g.player(p).life <= 4 || !can_pay(g, p, 0, "B", false) {
        return Ok(vec![]);
    }
    let theirs: Vec<PermId> =
        g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).filter(|&m| live(g, m) && g.is_creature(m)).collect();
    let theirs: Vec<PermId> = theirs.into_iter().filter(|&m| !crate::engine::values::untargetable(g, m)).collect();
    let ones: Vec<PermId> = theirs.iter().copied().filter(|&m| etgh(g, m) <= 1).collect();
    if let Some(t) = max_by(&ones, |m| pval(g, m)) {
        let label = format!("Vilis: -1/-1 on {}", g.perm(t).name);
        return Ok(vec![ability(2.5 + pval(g, t) * 0.3, label, src, vilis_go, t.0 as i64)]);
    }
    if g.player(p).life < 14 || g.player(p).library.len() < 10 {
        return Ok(vec![]);
    }
    let t = max_by(&theirs, |m| epow(g, m) as f64).unwrap_or(src);
    let label = format!("Vilis: -1/-1 on {} (draw two)", g.perm(t).name);
    Ok(vec![ability(2.2, label, src, vilis_go, t.0 as i64)])
}

fn vilis_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !live(g, src) || !can_pay(g, p, 0, "B", false) || g.player(p).life <= 2 {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    lose_life(g, p, 2, Some(p), "life", Some(false))?;
    if ability_window(g, p, Some(src), "target creature gets -1/-1", None, Some(t))? && live(g, t) {
        let x = g.perm_mut(t);
        x.eot_pt = (x.eot_pt.0 - 1, x.eot_pt.1 - 1);
        if g.is_creature(t) && etgh(g, t) <= 0 {
            die(g, t, "sba")?;
        }
    }
    Ok(true)
}

// ================================================================== Strength of the Harvest // Haven of the Harvest
/// Enchanted creature gets +1/+1 for each creature and/or enchantment you control.
fn harvest_bonus(g: &Game, p: PlayerId, _a: PermId, _m: PermId) -> (i32, i32) {
    let k = g.player(p).perms.iter().filter(|&&x| live(g, x) && (g.is_creature(x) || is_ench(g, x))).count() as i32;
    (k, k)
}

static HARVEST_AURA: AuraSpec = AuraSpec { bonus: Some(harvest_bonus), ..AURA };

/// the land face, when the land drop has no other use (no other land in hand, fewer than five lands)
fn harvest_land_opt(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let pl = g.player(p);
    let allowed = 1 + crate::cardcode::extra_lands(g, p) + pl.extra_land_now;
    if post.is_none() || g.active != Some(p) || pl.lands_played >= allowed || !pl.hand.contains(&c) {
        return Ok(vec![]);
    }
    if pl.lands.len() >= 5 || pl.hand.iter().any(|&x| g.db.get(x).land) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.5,
        label: "play Haven of the Harvest as a land".into(),
        act: Some(Action::Plan { f: harvest_land_go, arg: c.0 as i64 }),
    }])
}

fn harvest_land_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let allowed = 1 + crate::cardcode::extra_lands(g, p) + g.player(p).extra_land_now;
    if g.player(p).lands_played >= allowed || !remove_card(&mut g.player_mut(p).hand, c) {
        return Ok(false);
    }
    crate::engine::turn::play_land_card(g, p, c, "plays")?;
    if let Some(&l) = g.player(p).lands.last().filter(|&&l| g.perm(l).card() == c) {
        g.perm_mut(l).tapped = true; // This land enters tapped.
    }
    Ok(true)
}

fn haven_cols(_g: &Game, _p: PlayerId, _l: PermId) -> Colors {
    Colors::from_letters("GW")
}

// ================================================================== Calix, Guided by Fate
/// Constellation — Whenever Calix or another enchantment you control enters, put a +1/+1 counter on target creature
/// (your best one, an evasive one first).
fn calix_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if owner(g, m) != o || !is_ench(g, m) {
        return Ok(());
    }
    let cr: Vec<PermId> = g.player(o).perms.iter().copied().filter(|&x| live(g, x) && g.is_creature(x)).collect();
    let Some(t) = max_by(&cr, |x| pval(g, x) + if g.perm(x).fly { 2.0 } else { 0.0 }) else { return Ok(()) };
    if trigger_window(g, o, Some(src), "a +1/+1 counter on target creature", Some(1.5))? && live(g, t) {
        repl::put_plus(g, t, 1);
    }
    Ok(())
}

/// Whenever Calix or an enchanted creature you control deals combat damage to a player, you may create a token
/// that's a copy of a nonlegendary enchantment you control (the most valuable). Do this only once each turn.
fn calix_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    let o = owner(g, src);
    if p != o || !(a == src || crate::impls::common::auras_on(g, a).iter().any(|&x| owner(g, x) == o)) {
        return Ok(());
    }
    let key = intern(&format!("calix_{}", src.0));
    if g.player(o).flag_turn.get(key) == Some(&g.turn_stamp()) {
        return Ok(());
    }
    let es: Vec<PermId> = g
        .player(o)
        .perms
        .iter()
        .copied()
        .filter(|&x| {
            live(g, x) && is_ench(g, x) && g.perm(x).cd.is_some_and(|c| !g.db.get(c).tag(crate::tag::Tag::Leg))
        })
        .collect();
    let Some(e) = max_by(&es, |x| pval(g, x) + 1.0) else { return Ok(()) };
    let cd = g.perm(e).cd.unwrap();
    crate::engine::values::once_per_turn(g, o, key);
    if trigger_window(g, o, Some(src), "a token copy of a nonlegendary enchantment", Some(3.0))? {
        crate::glog!(g, "    Calix copies {}", cname(g, cd));
        enter_token_copy(g, o, cd)?;
    }
    Ok(())
}

// ================================================================== Resurgent Belief
/// Suspend 2—{1}{W}: from hand in p's main phase, with two or more enchantment cards (or one worth 4) in the
/// graveyard.
fn belief_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let pl = g.player(p);
    if post.is_none() || g.active != Some(p) || !pl.hand.contains(&c) || !can_pay(g, p, 1, "W", false) {
        return Ok(vec![]);
    }
    let ench: Vec<CardId> =
        pl.gy.iter().copied().filter(|&x| g.db.get(x).types.has(Types::ENCHANTMENT) && !g.db.get(x).land).collect();
    let best = ench.iter().map(|&x| g.db.get(x).cmc).max().unwrap_or(0);
    if ench.len() < 2 && best < 4 {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.8 + 0.3 * ench.len() as f64, "suspend Resurgent Belief".into(), belief_go, c.0 as i64)])
}

fn belief_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "W", false) {
        return Ok(false);
    }
    pay(g, p, 1, "W", false)?;
    let pl = g.player_mut(p);
    remove_card(&mut pl.hand, c);
    pl.exile.push(c);
    pl.suspended.push((c, 2));
    crate::glog!(g, "  {} suspends Resurgent Belief", pname(g, p));
    Ok(true)
}

// ================================================================== Weaver of Harmony
/// {G}, {T}: copy target triggered ability you control from an enchantment source: when you cast an enchantment
/// spell while you control an enchantment that draws for it, Weaver copies that draw (the AI pays {G} when it can).
fn weaver_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    let x = g.perm(src);
    if caster != o || !live(g, src) || x.tapped || x.sick || !g.db.get(c).types.has(Types::ENCHANTMENT) {
        return Ok(());
    }
    if !DRAW_ENGINES.iter().any(|n| mine(g, o, n).is_some_and(|m| is_ench(g, m))) || !can_pay(g, o, 0, "G", false) {
        return Ok(());
    }
    if crate::cardcode::ability_locked(g, src, o) {
        return Ok(());
    }
    pay(g, o, 0, "G", false)?;
    g.perm_mut(src).tapped = true;
    if ability_window(g, o, Some(src), "copy an enchantment's draw trigger", None, None)? {
        crate::glog!(g, "    Weaver of Harmony copies the draw trigger");
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, ARBITER)?;
    c.attack_cap = Some(arbiter_cap);
    c.can_cast = Some(arbiter_cast);
    let c = r.card(db, AVACYN)?;
    c.dies = Some(return_dies);
    c.self_dies = Some(avacyn_self_dies);
    let c = r.card(db, LIESA)?;
    c.dies = Some(return_dies);
    c.self_dies = Some(liesa_self_dies);
    at_once(c, Event::SelfDies, true);
    c.no_graveyard = Some(liesa_no_gy);
    r.card(db, DRANA)?.no_creature_mana = Some(drana_mana);
    r.card(db, SEEKER)?.etb = Some(seeker_etb);
    r.card(db, LORD)?.combat_damage = Some(lord_damage);
    r.card(db, RAKDOS)?.end_step = Some(rakdos_end);
    r.card(db, EMISSARY)?.as_enters = Some(emissary_as_enters);
    let c = r.card(db, VALGAVOTH)?;
    c.no_graveyard = Some(valg_no_gy);
    c.sba = Some(valg_sweep);
    c.options = Some(valg_options);
    let c = r.card(db, VILIS)?;
    c.lose_life = Some(vilis_lose);
    c.options = Some(vilis_options);
    aura(r, db, HARVEST, &HARVEST_AURA)?;
    let c = r.card(db, HARVEST)?;
    c.hand_options = Some(harvest_land_opt);
    c.land_cols = Some(haven_cols);
    let c = r.card(db, CALIX)?;
    c.etb = Some(calix_etb);
    c.combat_damage = Some(calix_damage);
    let c = r.card(db, BELIEF)?;
    c.hand_options = Some(belief_options);
    c.prio = Some(|_, _, _| 0); // never hard-cast (no mana cost): suspended from hand
    r.card(db, WEAVER)?.cast = Some(weaver_cast);
    Ok(())
}
