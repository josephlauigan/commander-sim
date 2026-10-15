//! Tier 0 pool deck card code: the cards of the anikthea precon (decklists/pool/t0-b2-precons/) that weren't modeled.
//! Filled in by the t0-anikthea agent (rust/agents/t0-anikthea.md). Each card's doc comment says how it is modeled;
//! the audit verdicts are in data/card_overrides.toml.
//!
//! - Sagas: one shared frame (`saga_enter`, `saga_main1`, `run_chapter`) with a chapter function per Saga.

use super::common::{oring_exile, oring_return};
use super::partials::remove_card;
use crate::cards::{CardDb, Colors};
use crate::engine::cast::discard_worst;
use crate::engine::life::{gain, lose_life};
use crate::engine::stack::trigger_window;
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, pval};
use crate::engine::zones::{
    Enter, Tokens, die, draw, enter, enter_token_copy, landfall, make_artifact_tokens, make_tokens, max_by, mill, min_by,
    sac_worth,
};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::intern;

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn cname(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn on_bf(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// p's creatures on the battlefield
fn creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| on_bf(g, m) && g.is_creature(m)).collect()
}

fn is_ench(g: &Game, c: CardId) -> bool {
    g.db.get(c).types.has(crate::cards::Types::ENCHANTMENT)
}

fn is_saga(g: &Game, c: CardId) -> bool {
    g.db.get(c).has_subtype("saga")
}

/// the opponent the AI feels least threatened by (lowest life): who it shares symmetric draws with
fn weakest_opp(g: &Game, p: PlayerId) -> Option<PlayerId> {
    let qs: Vec<PlayerId> = g.opps(p).collect();
    qs.into_iter().min_by_key(|&q| g.player(q).life)
}

// ================================================================== Sagas
pub const HELVAULT: &str = "Battle at the Helvault";
pub const TITANS: &str = "The Binding of the Titans";
pub const ELDEST: &str = "The Eldest Reborn";
pub const MENDING: &str = "The Mending of Dominaria";
pub const BRETAGARD: &str = "Battle for Bretagard";
pub const OLD_GODS: &str = "Binding the Old Gods";
pub const LOVE_SONG: &str = "Love Song of Night and Day";
pub const NARCI: &str = "Narci, Fable Singer";

fn lore(g: &Game, m: PermId) -> Option<i64> {
    g.perm(m).data.get(DataKey::Lore).map(Val::int)
}

/// the text a chapter goes on the stack with, and how much it matters
fn chapter_text(name: &str, n: i64) -> (&'static str, f64) {
    match (name, n) {
        (HELVAULT, 1 | 2) => ("exile a nonland permanent per player until this leaves", 7.0),
        (HELVAULT, _) => ("create Avacyn, an 8/8 Angel", 7.0),
        (TITANS, 1) => ("each player mills three", 3.0),
        (TITANS, 2) => ("exile up to two cards from graveyards", 3.0),
        (TITANS, _) => ("return a creature or land card to hand", 4.0),
        (ELDEST, 1) => ("each opponent sacrifices a creature or planeswalker", 7.0),
        (ELDEST, 2) => ("each opponent discards a card", 5.0),
        (ELDEST, _) => ("reanimate a creature or planeswalker from a graveyard", 7.0),
        (MENDING, 1 | 2) => ("mill two, return a creature card", 3.0),
        (MENDING, _) => ("return all lands from the graveyard", 6.0),
        (BRETAGARD, 1) => ("create a 1/1 Human Warrior", 2.0),
        (BRETAGARD, 2) => ("create a 1/1 Elf Warrior", 2.0),
        (BRETAGARD, _) => ("copy tokens with different names", 6.0),
        (OLD_GODS, 1) => ("destroy a nonland permanent", 7.0),
        (OLD_GODS, 2) => ("search for a Forest", 3.0),
        (OLD_GODS, _) => ("creatures gain deathtouch", 3.0),
        (LOVE_SONG, 1) => ("you and an opponent each draw two", 4.0),
        (LOVE_SONG, 2) => ("create a 1/1 Bird", 2.0),
        _ => ("+1/+1 counters on up to two creatures", 3.0),
    }
}

/// as a Saga enters it gets a lore counter, so chapter I triggers (cast, reanimated or blinked alike). Love Song has
/// read ahead: the AI starts it at chapter II (a Bird now, counters next turn) unless it is already wide enough for
/// III to be worth the whole card, or III alone has no use and I draws.
fn saga_enter(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let start = if cname(g, src) == LOVE_SONG {
        if creatures(g, o).len() >= 3 { 3 } else { 2 }
    } else {
        1
    };
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(start));
    run_chapter(g, o, src, start)
}

/// after your draw step: a lore counter
fn saga_main1(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src) && on_bf(g, src) && lore(g, src).is_some() {
        saga_lore(g, p, src)?;
    }
    Ok(())
}

/// a lore counter (after the draw step, or proliferated): that chapter triggers
fn saga_lore(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let n = lore(g, src).unwrap_or(0) + 1;
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(n));
    run_chapter(g, p, src, n)
}

/// proliferate: another lore counter while the last chapter is still to come
fn saga_wants(g: &Game, p: PlayerId, m: PermId) -> bool {
    owner(g, m) == p && lore(g, m).is_some_and(|n| n < 3)
}

/// chapter n triggers; once the final chapter has resolved the Saga is sacrificed
fn run_chapter(g: &mut Game, p: PlayerId, src: PermId, n: i64) -> Res {
    if n <= 3 {
        let (text, imp) = chapter_text(&cname(g, src).to_string(), n);
        if trigger_window(g, p, Some(src), text, Some(imp))? {
            chapter(g, p, src, n)?;
            if n == 3 {
                saga_final(g, p, src)?;
            }
        }
    }
    if n >= 3 && on_bf(g, src) {
        die(g, src, "sac")?;
    }
    Ok(())
}

fn chapter(g: &mut Game, p: PlayerId, src: PermId, n: i64) -> Res {
    g.player_mut(p).stat(intern("t0a_chapters"), 1);
    match (cname(g, src).to_string().as_str(), n) {
        (HELVAULT, 1 | 2) => helvault_exile(g, p, src),
        (HELVAULT, _) => {
            let spec = Tokens {
                tgh: Some(8),
                fly: true,
                color: Some(Colors::from_letters("W")),
                types: vec!["angel"],
                sick: true,
                ..Tokens::new(1, 8)
            };
            for t in make_tokens(g, p, spec)? {
                let x = g.perm_mut(t);
                x.vig = true;
                x.data.set(DataKey::Indestr, Val::Bool(true));
                x.data.set(DataKey::Legendary, Val::Bool(true));
            }
            Ok(())
        }
        (TITANS, 1) => {
            for q in g.players.iter().filter(|q| q.alive).map(|q| q.id).collect::<Vec<_>>() {
                mill(g, q, 3)?;
            }
            Ok(())
        }
        (TITANS, 2) => titans_exile(g, p),
        (TITANS, _) => {
            let cands: Vec<CardId> =
                g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature || g.db.get(c).land).collect();
            // a creature card first (a land only when the hand has none to play and no creature is there)
            let pick = max_by(&cands, |c| card_worth(g, p, c, true) + if g.db.get(c).land { -5.0 } else { 0.0 });
            if let Some(c) = pick {
                remove_card(&mut g.player_mut(p).gy, c);
                g.player_mut(p).hand.push(c);
                crate::glog!(g, "    {} returns {} to hand", g.player(p).name, g.db.get(c).name);
            }
            Ok(())
        }
        (ELDEST, 1) => {
            for q in g.opps(p).collect::<Vec<_>>() {
                edict_creature_or_walker(g, q)?;
            }
            Ok(())
        }
        (ELDEST, 2) => {
            for q in g.opps(p).collect::<Vec<_>>() {
                discard_worst(g, q, 1)?;
            }
            Ok(())
        }
        (ELDEST, _) => eldest_reanimate(g, p),
        (MENDING, 1 | 2) => {
            mill(g, p, 2)?;
            let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
            if let Some(c) = max_by(&cs, |c| card_worth(g, p, c, true)) {
                remove_card(&mut g.player_mut(p).gy, c);
                g.player_mut(p).hand.push(c);
            }
            Ok(())
        }
        (MENDING, _) => {
            let ls: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).land).collect();
            for c in ls {
                remove_card(&mut g.player_mut(p).gy, c);
                g.add_land(p, c, false);
                landfall(g, p)?;
            }
            let gy = std::mem::take(&mut g.player_mut(p).gy);
            g.player_mut(p).library.extend(gy);
            shuffle_library(g, p);
            Ok(())
        }
        (BRETAGARD, 1) => {
            let spec = Tokens {
                tgh: Some(1),
                color: Some(Colors::from_letters("W")),
                types: vec!["human", "warrior"],
                ..Tokens::new(1, 1)
            };
            make_tokens(g, p, spec)?;
            Ok(())
        }
        (BRETAGARD, 2) => {
            let spec = Tokens {
                tgh: Some(1),
                color: Some(Colors::from_letters("G")),
                types: vec!["elf", "warrior"],
                ..Tokens::new(1, 1)
            };
            make_tokens(g, p, spec)?;
            Ok(())
        }
        (BRETAGARD, _) => bretagard_copy(g, p),
        (OLD_GODS, 1) => {
            let cands: Vec<PermId> = g
                .opps(p)
                .flat_map(|q| g.player(q).perms.iter().copied().collect::<Vec<_>>())
                .filter(|&m| {
                    on_bf(g, m) && !g.perm(m).cd.is_some_and(|c| g.db.get(c).land) && !crate::engine::values::untargetable(g, m)
                })
                .collect();
            if let Some(t) = max_by(&cands, |m| pval(g, m)) {
                crate::engine::removal::apply_removal(g, Some(p), t, "destroy", g.perm(src).cd)?;
            }
            Ok(())
        }
        (OLD_GODS, 2) => {
            let forests: Vec<CardId> = crate::engine::zones::searchable(g, p)
                .into_iter()
                .filter(|&c| g.db.get(c).land && g.db.get(c).has_subtype("forest"))
                .collect();
            // a basic Forest first (a dual with a Forest type fixes more, but a basic is what the AI keeps)
            let pick = max_by(&forests, |c| g.db.get(c).pips.len() as f64 + if g.db.get(c).tag(crate::tag::Tag::F) { 0.0 } else { 0.0 });
            if let Some(c) = pick {
                remove_card(&mut g.player_mut(p).library, c);
                g.add_land(p, c, true);
                landfall(g, p)?;
            }
            shuffle_library(g, p);
            Ok(())
        }
        (OLD_GODS, _) => {
            for m in creatures(g, p) {
                let x = g.perm_mut(m);
                if !x.eot_kw.contains(&"deathtouch") {
                    x.eot_kw.push("deathtouch");
                }
            }
            Ok(())
        }
        (LOVE_SONG, 1) => {
            draw(g, p, 2, false)?;
            if let Some(q) = weakest_opp(g, p) {
                draw(g, q, 2, false)?;
            }
            Ok(())
        }
        (LOVE_SONG, 2) => {
            let spec = Tokens {
                tgh: Some(1),
                fly: true,
                color: Some(Colors::from_letters("W")),
                types: vec!["bird"],
                ..Tokens::new(1, 1)
            };
            make_tokens(g, p, spec)?;
            Ok(())
        }
        (LOVE_SONG, _) => {
            let mut cs = creatures(g, p);
            cs.sort_by(|&a, &b| pval(g, b).partial_cmp(&pval(g, a)).unwrap_or(std::cmp::Ordering::Equal));
            for m in cs.into_iter().take(2) {
                crate::cardcode::put_plus(g, m, 1);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Narci: the final chapter ability of a Saga p controls resolved: each opponent loses X, p gains X (X = the Saga's
/// mana value)
fn saga_final(g: &mut Game, p: PlayerId, saga: PermId) -> Res {
    let x = g.perm(saga).cd.map_or(0, |c| g.db.get(c).cmc as i32);
    for n in g.player(p).perms.clone() {
        if !on_bf(g, n) || cname(g, n) != NARCI || owner(g, n) != p {
            continue;
        }
        if x > 0 && trigger_window(g, p, Some(n), "each opponent loses X life, you gain X", Some(5.0))? {
            for q in g.opps(p).collect::<Vec<_>>() {
                lose_life(g, q, x, Some(p), "triggers", None)?;
            }
            gain(g, p, x)?;
        }
    }
    Ok(())
}

/// Battle at the Helvault I and II: for each player, exile up to one target non-Saga, nonland permanent that player
/// controls until the Saga leaves. The AI exiles each opponent's best permanent worth taking and none of its own
/// (every opponent is a player, so this is one exile per opponent: `oring_exile` per opponent).
fn helvault_exile(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    oring_exile(
        g,
        src,
        p,
        |g, m| {
            g.perm(m).cd.is_some_and(|c| !g.db.get(c).land && !is_saga(g, c)) || (g.perm(m).token && !g.perm(m).cd.is_some())
        },
        true,
        false,
    )
}

/// the Saga left: what it exiled returns (an effect ending, not a trigger)
fn saga_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    if cname(g, src) == HELVAULT {
        oring_return(g, src)?;
    }
    Ok(())
}

/// The Binding of the Titans II: exile up to two target cards from graveyards; you gain 1 life per creature card.
/// The AI takes the best creature cards from opponents' graveyards (reanimation targets), then any card of an
/// opponent's, never one of its own.
fn titans_exile(g: &mut Game, p: PlayerId) -> Res {
    let mut cands: Vec<(PlayerId, CardId)> =
        g.opps(p).flat_map(|q| g.player(q).gy.iter().map(move |&c| (q, c)).collect::<Vec<_>>()).collect();
    cands.sort_by(|a, b| {
        let key = |&(_, c): &(PlayerId, CardId)| g.db.get(c).creature as i32 * 100 + g.db.get(c).cmc as i32;
        key(b).cmp(&key(a))
    });
    let mut life = 0;
    for (q, c) in cands.into_iter().take(2) {
        remove_card(&mut g.player_mut(q).gy, c);
        g.player_mut(q).exile.push(c);
        if g.db.get(c).creature {
            life += 1;
        }
    }
    if life > 0 {
        gain(g, p, life)?;
    }
    Ok(())
}

/// The Eldest Reborn I: q sacrifices the creature or planeswalker it values least
fn edict_creature_or_walker(g: &mut Game, q: PlayerId) -> Res {
    if crate::human::is_human(g, q) {
        return crate::engine::zones::edict(g, q, false);
    }
    let cands: Vec<PermId> = g
        .player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| on_bf(g, m) && (g.is_creature(m) || g.perm(m).loyalty.is_some()))
        .collect();
    if let Some(m) = min_by(&cands, |x| sac_worth(g, x)) {
        let name = g.perm(m).name.to_string();
        die(g, m, "sac")?;
        if !g.perm(m).on_bf {
            crate::glog!(g, "    {} sacrifices {} (The Eldest Reborn)", g.player(q).name, name);
        }
    }
    Ok(())
}

/// The Eldest Reborn III: the best creature or planeswalker card in any graveyard onto the battlefield under p's
/// control
fn eldest_reanimate(g: &mut Game, p: PlayerId) -> Res {
    let mut cands: Vec<(PlayerId, CardId)> = vec![];
    for q in g.players.iter().filter(|q| q.alive || q.id == p).map(|q| q.id).collect::<Vec<_>>() {
        for &c in &g.player(q).gy {
            let d = g.db.get(c);
            if d.creature || d.start_loyalty.is_some() {
                cands.push((q, c));
            }
        }
    }
    let Some((q, c)) = max_by(&cands, |(_, c)| card_worth(g, p, c, true) + g.db.get(c).cmc as f64 * 0.2) else {
        return Ok(());
    };
    remove_card(&mut g.player_mut(q).gy, c);
    crate::glog!(g, "    {} reanimates {}", g.player(p).name, g.db.get(c).name);
    enter(g, p, c, Enter::default())?;
    Ok(())
}

/// Battle for Bretagard III: for each creature token with a different name (types and colours) and each kind of
/// artifact token (Treasure, Food, Clue: counters in this engine) p controls, one more of it
fn bretagard_copy(g: &mut Game, p: PlayerId) -> Res {
    let mut seen: Vec<(Vec<crate::sym::Sym>, Colors)> = vec![];
    let mut originals = vec![];
    for m in creatures(g, p) {
        let x = g.perm(m);
        if !x.token || x.cd.is_some() {
            continue;
        }
        let key = (x.ttypes.clone(), x.colors);
        if !seen.contains(&key) {
            seen.push(key);
            originals.push(m);
        }
    }
    for m in originals {
        if !on_bf(g, m) {
            continue;
        }
        let x = g.perm(m).clone();
        let spec = Tokens {
            tgh: Some(x.tgh),
            fly: x.fly,
            dt: x.dt,
            lifelink: x.lifelink,
            types: x.ttypes.clone(),
            color: Some(x.colors),
            ..Tokens::new(1, x.pow)
        };
        make_tokens(g, p, spec)?;
    }
    let (t, f, c) = (g.player(p).treasures, g.player(p).foods, g.player(p).clues);
    for (n, kind) in [(t, "Treasure"), (f, "Food"), (c, "Clue")] {
        if n > 0 {
            make_artifact_tokens(g, p, kind, 1)?;
        }
    }
    let _ = enter_token_copy;
    let _ = (gain, epow);
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    for name in [HELVAULT, TITANS, ELDEST, MENDING, BRETAGARD, OLD_GODS, LOVE_SONG] {
        let c = r.card(db, name)?;
        c.etb = Some(saga_enter);
        c.main1 = Some(saga_main1);
        c.saga = Some((saga_wants, saga_lore));
    }
    r.card(db, HELVAULT)?.leaves = Some(saga_leaves);
    Ok(())
}
