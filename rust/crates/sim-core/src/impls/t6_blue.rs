//! Tier 6 batch `blue` (rust/agents/t6-batches.md): the blue and small utility cards of the Kraum & Tymna, Rograkh &
//! Silas and Kinnan decks. Cards only: no engine file is touched; what needs the engine is listed in
//! rust/agents/engine-needs-blue.md.
//!
//! - Borne Upon a Wind: draw, and the one-turn "cast spells as though they had flash" flag (`t6_lands::FLASH_FLAG`).
//! - Mnemonic Betrayal: opponents' graveyards are exiled and their nonland cards held in hand (any mana type) for the turn.
//! - Timetwister: every player's hand and graveyard shuffled into their library, then seven cards drawn.
//! - Demonic Counsel: a Demon search, or any card with delirium (four card types in the graveyard).
//! - Rain of Filth: sets the one-turn flag (`rain_of_filth_active`); the sacrifice-a-land mana source is an engine need.
//! - Sevinne's Reclamation: the best permanent card of mana value 3 or less returns; from the graveyard it is copied.
//! - Exhibition Tidecaller: Opus on the cast hook; the old compiled mill-3 trigger is removed (dsl = []).

use crate::cards::CardDb;
use crate::engine::tutors::{card_worth, shuffle_library, tutor};
use crate::engine::stack::trigger_window;
use crate::engine::zones::{Enter, agent_for, agent_take, draw, enter, max_by, mill, searchable};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{CardId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::Sym;

fn set_turn_flag(g: &mut Game, p: PlayerId, key: Sym) {
    let stamp = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(key, stamp);
}

/// does p hold the one-turn grant "you may cast spells this turn as though they had flash"? (Borne Upon a Wind;
/// shared with Emergence Zone). The engine's cast-timing check and the AI's end-of-turn window should read this.
pub fn flash_turn_grant(g: &Game, p: PlayerId) -> bool {
    g.player(p).flag_turn.get(crate::impls::t6_lands::FLASH_FLAG) == Some(&g.turn_stamp())
}

// ================================================================== Borne Upon a Wind
/// Borne Upon a Wind: you may cast spells this turn as though they had flash; draw a card
fn borne_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    set_turn_flag(g, p, crate::impls::t6_lands::FLASH_FLAG);
    draw(g, p, 1, false)?;
    Ok("gy")
}

// ================================================================== Mnemonic Betrayal
/// Mnemonic Betrayal: exile every opponent's graveyard; p may cast spells from among those cards this turn with mana
/// of any type (the cards are held in p's hand like Opposition Agent's, `yshtola::take_card`, and listed in
/// `impulse` so the end step clears what was not cast); Betrayal itself is exiled.
/// Missing: what remains goes back to its owner's graveyard (the end-step cleanup exiles it with p's own impulse cards).
fn betrayal_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in g.opps(p).collect::<Vec<_>>() {
        let gy = std::mem::take(&mut g.player_mut(q).gy);
        let n = gy.len();
        for c in gy {
            if g.db.get(c).land {
                g.player_mut(q).exile.push(c); // a land can't be cast
            } else {
                crate::impls::yshtola::take_card(g, p, c, q, "Mnemonic Betrayal");
                g.player_mut(p).impulse.push(c);
            }
        }
        crate::glog!(g, "    Mnemonic Betrayal exiles {} card(s) from {}'s graveyard", n, g.player(q).name);
    }
    Ok("exile")
}

// ================================================================== Timetwister
/// Timetwister: each player shuffles their hand and graveyard into their library, then draws seven cards; Timetwister
/// itself goes to the graveyard afterwards (it is not shuffled in). A card held from an opponent (Opposition Agent
/// style) goes into its owner's library.
fn timetwister_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut order = vec![p];
    order.extend(g.after(p).filter(|&q| g.player(q).alive));
    for &q in &order {
        let hand = std::mem::take(&mut g.player_mut(q).hand);
        let gy = std::mem::take(&mut g.player_mut(q).gy);
        for c in hand.into_iter().chain(gy) {
            let owner = g.player_mut(q).stolen.swap_remove(&c).unwrap_or(q);
            g.player_mut(q).agent_ids.retain(|&x| x != c);
            g.player_mut(q).impulse.retain(|&x| x != c);
            g.player_mut(owner).library.push(c);
        }
    }
    for &q in &order {
        shuffle_library(g, q);
    }
    for &q in &order {
        draw(g, q, 7, false)?;
        if g.over {
            break;
        }
    }
    Ok("gy")
}


// ================================================================== Demonic Counsel
/// Demonic Counsel: search for a Demon card (any card with delirium: four or more card types in your graveyard),
/// reveal it, put it into your hand, then shuffle. Delirium searches use the deck's own tutor pick; a plain Demon
/// search takes the Demon the deck would value most (a card taken by Opposition Agent follows its rule).
fn counsel_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if crate::impls::t6_green2::delirium(g, p) {
        tutor(g, p, "any")?;
        return Ok("gy");
    }
    let is_demon = |g: &Game, c: CardId| g.db.get(c).has_subtype("demon");
    if crate::human::is_human(g, p) {
        let found = crate::human::choices::search(g, p, &|g: &Game, c: CardId| is_demon(g, c), 1, "Search your library for a Demon", true)?;
        for c in found {
            let pl = g.player_mut(p);
            pl.hand.push(c);
            pl.stat("tutored", 1);
            pl.seen.insert(c);
        }
        return Ok("gy");
    }
    let cands: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| is_demon(g, c)).collect();
    if let Some(c) = max_by(&cands, |c| card_worth(g, p, c, false)) {
        let lib = &mut g.player_mut(p).library;
        let i = lib.iter().position(|&x| x == c).unwrap();
        lib.remove(i);
        if let Some(a) = agent_for(g, p) {
            agent_take(g, a, p, c);
        } else {
            let pl = g.player_mut(p);
            pl.hand.push(c);
            pl.stat("tutored", 1);
            pl.seen.insert(c);
            crate::glog!(g, "    {} tutors {} (Demonic Counsel)", g.player(p).name, g.db.get(c).name);
        }
    }
    shuffle_library(g, p);
    Ok("gy")
}


// ================================================================== Rain of Filth
const RAIN_FLAG: &str = "rain_of_filth";

/// have p's lands "Sacrifice this land: Add {B}" until end of turn (Rain of Filth)? The mana code (`total_mana`, the
/// unit list `pay` uses) should add one {B} unit per land p controls, tapped or not, paid by sacrificing the land.
pub fn rain_of_filth_active(g: &Game, p: PlayerId) -> bool {
    g.player(p).flag_turn.get(RAIN_FLAG) == Some(&g.turn_stamp())
}

/// Rain of Filth: until end of turn, lands you control gain "Sacrifice this land: Add {B}" (sets the turn flag; the
/// engine has no sacrifice-a-land mana source yet)
fn rain_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    set_turn_flag(g, p, RAIN_FLAG);
    Ok("gy")
}


// ================================================================== Sevinne's Reclamation
/// the permanent card with mana value 3 or less in p's graveyard the AI returns: creatures by power, then other
/// permanents by mana value; a land only when nothing else is there
fn reclaim_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let cs: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&c| (g.db.get(c).perm || g.db.get(c).land) && g.db.get(c).cmc <= 3).collect();
    let key = |c: CardId| {
        let d = g.db.get(c);
        (!d.land) as i32 as f64 * 10000.0 + d.creature as i32 as f64 * 1000.0 + d.pow as f64 * 10.0 + d.cmc as f64
    };
    max_by(&cs, key)
}

/// put the chosen permanent card from p's graveyard onto the battlefield (a land enters like a played one, without
/// using the land drop)
fn reclaim_return(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(x) = reclaim_pick(g, p) else { return Ok(false) };
    let gy = &mut g.player_mut(p).gy;
    let i = gy.iter().position(|&c| c == x).unwrap();
    gy.remove(i);
    if g.db.get(x).land {
        let tapped = crate::engine::turn::land_enters_tapped(g, p, x);
        let l = g.add_land(p, x, tapped);
        crate::cardcode::land_etb(g, p, l)?;
        crate::engine::zones::landfall(g, p)?;
    } else {
        enter(g, p, x, Enter::default())?;
    }
    crate::glog!(g, "    Sevinne's Reclamation returns {}", g.db.get(x).name);
    Ok(true)
}

/// Sevinne's Reclamation: return target permanent card with mana value 3 or less from your graveyard to the
/// battlefield; cast from a graveyard (flashback), the AI copies it and the copy returns a second card
fn reclamation_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    reclaim_return(g, p)?;
    if ctx.from_gy {
        reclaim_return(g, p)?;
    }
    Ok("gy")
}


// ================================================================== Exhibition Tidecaller
/// the mana spent on the spell being cast (c): its mana value plus X, or nothing when an alternative cost was paid.
/// The engine records no real payment total, so cost reductions are not seen (see engine-needs-blue.md).
fn mana_spent_on(g: &Game, c: CardId) -> i32 {
    match &g.cur_cast {
        Some((cc, ctx, _)) if *cc == c => {
            if ctx.paid_otherwise {
                0
            } else {
                g.db.get(c).cmc as i32 + ctx.x.max(0)
            }
        }
        _ => g.db.get(c).cmc as i32,
    }
}

/// Exhibition Tidecaller (Opus): whenever you cast an instant or sorcery spell, target player mills three cards; if
/// five or more mana was spent to cast it, that player mills ten instead. The AI aims it at the opposing player it
/// most wants to hurt (the DSL's "target player" rule: highest threat, not hexproof).
fn tidecaller_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != o || !(d.instant || d.sorcery) {
        return Ok(());
    }
    let n = if mana_spent_on(g, c) >= 5 { 10 } else { 3 };
    if !trigger_window(g, o, Some(src), &format!("mill {n}"), Some(2.0))? {
        return Ok(());
    }
    let Some(&t) = crate::dsl::players(g, o, Some("target_player"), &crate::dsl::DslCtx::default()).first() else {
        return Ok(());
    };
    crate::glog!(g, "    Exhibition Tidecaller: {} mills {}", g.player(t).name, n);
    mill(g, t, n)
}


// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Exhibition Tidecaller")?.cast = Some(tidecaller_cast);
    r.card(db, "Sevinne's Reclamation")?.resolve = Some(reclamation_resolve);
    r.card(db, "Rain of Filth")?.resolve = Some(rain_resolve);
    r.card(db, "Demonic Counsel")?.resolve = Some(counsel_resolve);
    r.card(db, "Timetwister")?.resolve = Some(timetwister_resolve);
    r.card(db, "Borne Upon a Wind")?.resolve = Some(borne_resolve);
    r.card(db, "Mnemonic Betrayal")?.resolve = Some(betrayal_resolve);
    Ok(())
}
