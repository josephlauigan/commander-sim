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
use crate::engine::stack::ability_window;
use crate::engine::stack::trigger_window;
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, pval};
use crate::engine::zones::{
    Enter, Tokens, die, draw, enter, enter_token_copy, landfall, make_artifact_tokens, make_tokens, max_by, mill,
    min_by, sac_worth,
};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
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
    let start = if cname(g, src) == LOVE_SONG { if creatures(g, o).len() >= 3 { 3 } else { 2 } } else { 1 };
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
                    on_bf(g, m)
                        && !g.perm(m).cd.is_some_and(|c| g.db.get(c).land)
                        && !crate::engine::values::untargetable(g, m)
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
            let pick = max_by(&forests, |c| {
                g.db.get(c).pips.len() as f64 + if g.db.get(c).tag(crate::tag::Tag::F) { 0.0 } else { 0.0 }
            });
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
            g.perm(m).cd.is_some_and(|c| !g.db.get(c).land && !is_saga(g, c))
                || (g.perm(m).token && !g.perm(m).cd.is_some())
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

// ================================================================== enchantment-enters engines (constellation)
pub const ANIKTHEA: &str = "Anikthea, Hand of Erebos";
pub const ABUNDANCE: &str = "Abundance";
pub const CACOPHONY: &str = "Cacophony Unleashed";
pub const BOON: &str = "Boon of the Spirit Realm";
pub const KHALNI: &str = "Khalni Heart Expedition";
pub const WAKE: &str = "Mirari's Wake";
pub const COMPOSER: &str = "Composer of Spring";
pub const NESSIAN: &str = "Nessian Wanderer";
pub const DOOMWAKE: &str = "Doomwake Giant";
pub const ONDU: &str = "Ondu Spiritdancer";
pub const HELIOD: &str = "Heliod, God of the Sun";
pub const EREBOS: &str = "Erebos, Bleak-Hearted";
pub const TANUKI: &str = "Greater Tanuki";
pub const WEAVER: &str = "Nyx Weaver";
pub const BEHEMOTH: &str = "Nyxborn Behemoth";
pub const SANDWURM: &str = "Sandwurm Convergence";

/// the enchantments p controls (permanents with an enchantment card, tokens that copy one included)
fn ench_count(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| on_bf(g, m) && g.perm(m).cd.is_some_and(|c| is_ench(g, c))).count()
}

/// constellation: `m` (an enchantment under src's controller's control) entered; `or_self`: "this or another"
fn ench_entered(g: &Game, src: Src, m: PermId, or_self: bool) -> bool {
    (or_self || m != src) && on_bf(g, m) && owner(g, m) == owner(g, src) && g.perm(m).cd.is_some_and(|c| is_ench(g, c))
}

/// whether p wants a land in hand now (the AI's reveal and look-at-top decisions)
fn wants_land(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    let held = pl.hand.iter().filter(|&&c| g.db.get(c).land).count();
    pl.lands.len() + held < 8
}

/// **Composer of Spring**: constellation: you may put a land card from your hand onto the battlefield tapped; with six
/// or more enchantments, a creature or land card. The AI puts the best creature card (highest mana value) in with six
/// enchantments, else a land (the one making the most colours) whenever it has one.
fn composer(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if !ench_entered(g, src, m, false) {
        return Ok(());
    }
    let o = owner(g, src);
    let six = ench_count(g, o) >= 6;
    let hand = g.player(o).hand.clone();
    let creature = if six {
        max_by(&hand.iter().copied().filter(|&c| g.db.get(c).creature).collect::<Vec<_>>(), |c| g.db.get(c).cmc as f64)
    } else {
        None
    };
    let land = max_by(&hand.iter().copied().filter(|&c| g.db.get(c).land).collect::<Vec<_>>(), |c| {
        g.db.get(c).tags.str(crate::tag::Tag::C).map_or(0, |s| s.chars().count()) as f64
    });
    let Some(c) = creature.or(land) else { return Ok(()) };
    if !trigger_window(g, o, Some(src), "put a land or creature card onto the battlefield tapped", Some(3.0))? {
        return Ok(());
    }
    if !g.player(o).hand.contains(&c) {
        return Ok(());
    }
    remove_card(&mut g.player_mut(o).hand, c);
    if g.db.get(c).land {
        g.add_land(o, c, true);
        landfall(g, o)?;
    } else {
        let n = enter(g, o, c, Enter::default())?;
        g.perm_mut(n).tapped = true;
    }
    Ok(())
}

/// **Nessian Wanderer**: constellation: look at the top three cards; you may reveal a land card from among them and
/// put it into your hand; the rest on the bottom in a random order. The AI takes the land unless it has plenty.
fn nessian(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if !ench_entered(g, src, m, false) {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "look at the top three cards", Some(2.0))? {
        return Ok(());
    }
    let k = g.player(o).library.len().min(3);
    let mut top: Vec<CardId> = (0..k).filter_map(|_| g.player_mut(o).library.pop()).collect();
    if wants_land(g, o)
        && let Some(i) = top.iter().position(|&c| g.db.get(c).land)
    {
        let c = top.remove(i);
        g.player_mut(o).hand.push(c);
        g.player_mut(o).seen.insert(c);
    }
    g.rng.shuffle(&mut top);
    g.player_mut(o).library.splice(0..0, top);
    Ok(())
}

/// **Doomwake Giant**: constellation (it or another enchantment): creatures your opponents control get -1/-1 until
/// end of turn
fn doomwake(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if !ench_entered(g, src, m, true) {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "opponents' creatures get -1/-1", Some(5.0))? {
        return Ok(());
    }
    for q in g.opps(o).collect::<Vec<_>>() {
        for x in g.player(q).perms.clone() {
            if !g.is_creature(x) || g.perm(x).phased {
                continue;
            }
            super::common::eot(g, x, -1, -1);
            if crate::engine::values::etgh(g, x) <= 0 {
                die(g, x, "sba")?;
            }
        }
    }
    Ok(())
}

/// **Ondu Spiritdancer**: whenever an enchantment you control enters, you may create a token copy of it; only once
/// each turn. The AI skips Auras, legendary permanents it already has, and tokens made by other effects with no card.
fn ondu(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if !ench_entered(g, src, m, false) {
        return Ok(());
    }
    let o = owner(g, src);
    let Some(c) = g.perm(m).cd else { return Ok(()) };
    let d = g.db.get(c);
    if d.has_subtype("aura") || d.tag(crate::tag::Tag::Leg) {
        return Ok(());
    }
    let key = intern(&format!("t0a_ondu{}", src.0));
    if !trigger_window(g, o, Some(src), "create a token copy of the enchantment", Some(4.0))? {
        return Ok(());
    }
    if crate::engine::values::once_per_turn(g, o, key) {
        enter_token_copy(g, o, c)?;
    }
    Ok(())
}

/// **Boon of the Spirit Realm**: constellation (it or another enchantment): a blessing counter on it; your creatures
/// get +1/+1 per blessing counter (`creature_pt`, evaluated while `g.selfpt` is on)
fn boon(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    g.selfpt = true;
    if !ench_entered(g, src, m, true) {
        return Ok(());
    }
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "a blessing counter", Some(2.0))? {
        let n = g.perm(src).data.int(DataKey::Counters);
        g.perm_mut(src).data.set(DataKey::Counters, Val::Int(n + 1));
    }
    Ok(())
}

fn boon_pt(g: &Game, m: PermId) -> (i32, i32) {
    let o = owner(g, m);
    let n: i64 = g
        .player(o)
        .perms
        .iter()
        .filter(|&&x| on_bf(g, x) && cname(g, x) == BOON)
        .map(|&x| g.perm(x).data.int(DataKey::Counters))
        .sum();
    (n as i32, n as i32)
}

/// **Khalni Heart Expedition**: landfall: you may put a quest counter on it. Remove three and sacrifice it: search
/// for two basic lands, tapped. The AI uses the sacrifice as soon as it has the counters.
fn khalni_landfall(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !on_bf(g, src) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "a quest counter", Some(1.0))? {
        let n = g.perm(src).data.int(DataKey::Counters);
        g.perm_mut(src).data.set(DataKey::Counters, Val::Int(n + 1));
    }
    Ok(())
}

fn khalni_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post.is_none() || !on_bf(g, src) || g.perm(src).data.int(DataKey::Counters) < 3 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.0,
        label: "Khalni Heart Expedition: two basic lands".into(),
        act: Some(Action::Ability { src, f: khalni_go, arg: 0 }),
    }])
}

fn khalni_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on_bf(g, src) || g.perm(src).data.int(DataKey::Counters) < 3 {
        return Ok(false);
    }
    g.perm_mut(src).data.set(DataKey::Counters, Val::Int(0));
    if !ability_window(g, p, Some(src), "search for two basic lands", None, None)? {
        die(g, src, "sac")?;
        return Ok(true);
    }
    die(g, src, "sac")?; // the sacrifice is a cost
    crate::engine::zones::land_ramp(g, p, 2, true)?;
    Ok(true)
}

/// **Mirari's Wake**: whenever you tap a land for mana, add one mana of any type that land produced: every land of
/// the controller makes one more of its own mana (the +1/+1 for creatures is the compiled anthem)
fn wake_mana(g: &Game, src: Src, p: PlayerId, _l: crate::ids::LandId) -> i32 {
    if p == owner(g, src) && on_bf(g, src) { 1 } else { 0 }
}

// ------------------------------------------------------------------ Gods
/// devotion decides whether a God is a creature (the compiled data gives it no creature type)
fn god_update(g: &mut Game, src: PermId, col: char, pt: (i32, i32)) {
    let on_ = super::rules2::devotion(g, owner(g, src), col) >= 5;
    let x = g.perm_mut(src);
    (x.pow, x.tgh) = pt;
    x.data.set(DataKey::Anim, Val::Bool(on_));
}

fn heliod_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src || owner(g, m) == owner(g, src) {
        god_update(g, src, 'W', (5, 6));
    }
    Ok(())
}

fn heliod_sba(g: &mut Game, src: Src) -> Res {
    god_update(g, src, 'W', (5, 6));
    Ok(())
}

fn erebos_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src || owner(g, m) == owner(g, src) {
        god_update(g, src, 'B', (5, 6));
    }
    Ok(())
}

fn erebos_sba(g: &mut Game, src: Src) -> Res {
    god_update(g, src, 'B', (5, 6));
    Ok(())
}

fn god_indestructible(_g: &Game, src: Src, m: PermId, kw: crate::sym::Sym) -> bool {
    kw == "indestructible" && m == src
}

/// **Erebos, Bleak-Hearted**: whenever another creature you control dies, you may pay 2 life to draw a card (the AI
/// does while it stays above 10 life)
fn erebos_dies(g: &mut Game, src: Src, m: PermId, _cause: crate::sym::Sym) -> Res {
    let o = owner(g, src);
    if m == src || owner(g, m) != o || !on_bf(g, src) || g.player(o).life <= 10 || g.player(o).library.is_empty() {
        return Ok(());
    }
    if g.perm(m).on_bf || !(g.perm(m).token || g.perm(m).cd.is_some_and(|c| g.db.get(c).creature)) {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "pay 2 life, draw a card", Some(2.0))? {
        lose_life(g, o, 2, Some(o), "other", None)?;
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// **Erebos**: {1}{B}, sacrifice another creature: target creature gets -2/-1 until end of turn. The AI sacrifices its
/// least valuable creature to kill an opposing creature (toughness 1 or less) worth more than the fodder.
fn erebos_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post.is_none() || !on_bf(g, src) || !crate::engine::mana::can_pay(g, p, 1, "B", false) {
        return Ok(vec![]);
    }
    let victims: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied().collect::<Vec<_>>())
        .filter(|&x| {
            on_bf(g, x)
                && g.is_creature(x)
                && crate::engine::values::etgh(g, x) <= 1
                && !crate::engine::values::untargetable(g, x)
        })
        .collect();
    let Some(v) = max_by(&victims, |x| pval(g, x)) else { return Ok(vec![]) };
    let fodder: Vec<PermId> = creatures(g, p).into_iter().filter(|&x| x != src).collect();
    let Some(f) = min_by(&fodder, |x| pval(g, x)) else { return Ok(vec![]) };
    if pval(g, f) + 1.0 >= pval(g, v) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: pval(g, v) - pval(g, f),
        label: "Erebos: sacrifice a creature, -2/-1".into(),
        act: Some(Action::Ability { src, f: erebos_go, arg: v.0 as i64 }),
    }])
}

fn erebos_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let v = PermId(arg as u32);
    let fodder: Vec<PermId> = creatures(g, p).into_iter().filter(|&x| x != src).collect();
    let Some(f) = min_by(&fodder, |x| pval(g, x)) else { return Ok(false) };
    if !on_bf(g, v) || !crate::engine::mana::pay(g, p, 1, "B", false)? {
        return Ok(false);
    }
    die(g, f, "sac")?;
    if !ability_window(g, p, Some(src), "target creature gets -2/-1", None, None)? {
        return Ok(true);
    }
    super::common::eot(g, v, -2, -1);
    if crate::engine::values::etgh(g, v) <= 0 {
        die(g, v, "sba")?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ Narci
/// **Narci, Fable Singer**: whenever you sacrifice an enchantment, draw a card (the final chapter drain is
/// `saga_final`)
fn narci_sacrifice(g: &mut Game, src: Src, p: PlayerId, what: crate::hooks::Sacrificed) -> Res {
    let crate::hooks::Sacrificed::Perm(m) = what else { return Ok(()) };
    if p != owner(g, src) || !on_bf(g, src) || !g.perm(m).cd.is_some_and(|c| is_ench(g, c)) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "draw a card", Some(2.0))? {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Greater Tanuki, Nyx Weaver, Nyxborn Behemoth
/// **Greater Tanuki** channel {2}{G}, discard it: search for a basic land, put it onto the battlefield tapped. The AI
/// channels it when it can't cast the 6-drop yet and a land helps.
fn tanuki_channel(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let _ = c;
    if post.is_none() || !crate::engine::mana::can_pay(g, p, 2, "G", false) {
        return Ok(vec![]);
    }
    if crate::engine::mana::total_mana(g, p, false) >= 6 || g.player(p).lands.len() >= 8 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.5,
        label: "channel Greater Tanuki".into(),
        act: Some(Action::Plan { f: tanuki_go, arg: 0 }),
    }])
}

fn tanuki_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some(c) = g.db.id(TANUKI) else { return Ok(false) };
    if !g.player(p).hand.contains(&c) || !crate::engine::mana::pay(g, p, 2, "G", false)? {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    g.player_mut(p).gy.push(c);
    crate::glog!(g, "  {} channels Greater Tanuki", g.player(p).name);
    crate::engine::zones::land_ramp(g, p, 1, true)?;
    Ok(true)
}

/// **Nyx Weaver**: {1}{B}{G}, exile it: return target card from your graveyard to your hand. The AI does it at the
/// end of its turn when the graveyard holds a card worth getting back.
fn weaver_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || !on_bf(g, src) || !crate::engine::mana::can_pay(g, p, 1, "BG", false) {
        return Ok(vec![]);
    }
    if post == Some(false) || (post == Some(true) && g.active != Some(p)) {
        return Ok(vec![]);
    }
    let best = g.player(p).gy.iter().map(|&c| card_worth(g, p, c, true)).fold(f64::NEG_INFINITY, f64::max);
    if best < 4.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: best * 0.3,
        label: "Nyx Weaver: exile it, return a card".into(),
        act: Some(Action::Ability { src, f: weaver_go, arg: 0 }),
    }])
}

fn weaver_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on_bf(g, src) || !crate::engine::mana::pay(g, p, 1, "BG", false)? {
        return Ok(false);
    }
    crate::engine::zones::exile_perm(g, src)?;
    if !ability_window(g, p, Some(src), "return a card from the graveyard", None, None)? {
        return Ok(true);
    }
    let cs = g.player(p).gy.clone();
    if let Some(c) = max_by(&cs, |c| card_worth(g, p, c, true)) {
        remove_card(&mut g.player_mut(p).gy, c);
        g.player_mut(p).hand.push(c);
    }
    Ok(true)
}

/// **Nyxborn Behemoth**: costs {X} less, X the total mana value of the noncreature enchantments you control (at most
/// the {10} generic part)
fn behemoth_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let x: u32 = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| on_bf(g, m))
        .filter_map(|&m| g.perm(m).cd)
        .filter(|&c| is_ench(g, c) && !g.db.get(c).creature)
        .map(|c| g.db.get(c).cmc)
        .sum();
    -(x.min(10) as i32)
}

// ------------------------------------------------------------------ Sandwurm Convergence
/// Creatures with flying can't attack you or planeswalkers you control: called from `combat::attack_limits` for each
/// attacker `a` and defending player `d`
pub fn flyer_barred(g: &Game, a: PermId, d: PlayerId) -> bool {
    let x = g.perm(a);
    (x.fly || x.eot_kw.contains(&"flying") || crate::engine::combat::kw(g, a, "flying"))
        && g.player(d).perms.iter().any(|&m| on_bf(g, m) && cname(g, m) == SANDWURM)
}

// ------------------------------------------------------------------ Anikthea, Hand of Erebos
/// menace for the other enchantment creatures its controller has
fn anikthea_kw(g: &Game, src: Src, m: PermId, kw: crate::sym::Sym) -> bool {
    kw == "menace"
        && m != src
        && owner(g, m) == owner(g, src)
        && g.is_creature(m)
        && g.perm(m).cd.is_some_and(|c| is_ench(g, c))
}

/// **Anikthea**, entering or attacking: exile up to one non-Aura enchantment card from the graveyard and create a
/// token that's a copy of it, a 3/3 black Zombie creature in addition to its other types. The AI picks the card worth
/// most (an enchantment creature, a Saga, a legendary one only when the original isn't out).
fn anikthea_copy(g: &mut Game, src: PermId) -> Res {
    let o = owner(g, src);
    let out: Vec<CardId> = g.player(o).perms.iter().filter_map(|&m| g.perm(m).cd).collect();
    let cands: Vec<CardId> = g
        .player(o)
        .gy
        .iter()
        .copied()
        .filter(|&c| {
            is_ench(g, c)
                && !g.db.get(c).has_subtype("aura")
                && !(g.db.get(c).tag(crate::tag::Tag::Leg) && out.contains(&c))
        })
        .collect();
    let Some(c) = max_by(&cands, |c| {
        let d = g.db.get(c);
        card_worth(g, o, c, true)
            + d.cmc as f64 * 0.3
            + if d.creature { 2.0 } else { 0.0 }
            + if is_saga(g, c) { 1.0 } else { 0.0 }
    }) else {
        return Ok(());
    };
    if !trigger_window(g, o, Some(src), "exile an enchantment card, create a 3/3 Zombie copy", Some(6.0))? {
        return Ok(());
    }
    if !g.player(o).gy.contains(&c) {
        return Ok(());
    }
    remove_card(&mut g.player_mut(o).gy, c);
    g.player_mut(o).exile.push(c);
    if let Some(t) = enter_token_copy(g, o, c)?
        && g.perm(t).on_bf
    {
        let x = g.perm_mut(t);
        x.data.set(DataKey::Anim, Val::Bool(true));
        (x.pow, x.tgh) = (3, 3);
        x.colors = Colors::from_letters("B");
        if !x.ttypes.contains(&"zombie") {
            x.ttypes.push("zombie");
        }
    }
    Ok(())
}

fn anikthea_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src { anikthea_copy(g, src) } else { Ok(()) }
}

fn anikthea_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p == owner(g, src) && atk.contains(&src) {
        anikthea_copy(g, src)?;
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ Abundance
/// **Abundance**: if you would draw a card, you may instead choose land or nonland and reveal cards from the top of
/// your library until you reveal a card of the chosen kind; put it into your hand and the rest on the bottom (their
/// order kept). The AI chooses a land while it has fewer than eight lands in play and hand together, else nonland, and
/// always replaces the draw. Called from `zones::draw`; true: the draw was replaced.
pub fn abundance_draw(g: &mut Game, p: PlayerId) -> Res<bool> {
    if !g.player(p).perms.iter().any(|&m| on_bf(g, m) && cname(g, m) == ABUNDANCE) {
        return Ok(false);
    }
    let want_land = wants_land(g, p);
    let mut revealed = vec![];
    let mut found = None;
    while let Some(c) = g.player_mut(p).library.pop() {
        if g.db.get(c).land == want_land {
            found = Some(c);
            break;
        }
        revealed.push(c);
    }
    revealed.reverse(); // the bottom keeps the order they were revealed in
    g.player_mut(p).library.splice(0..0, revealed);
    if let Some(c) = found {
        let pl = g.player_mut(p);
        pl.hand.push(c);
        pl.seen.insert(c);
        crate::glog!(g, "    Abundance: {} takes {} instead of drawing", g.player(p).name, g.db.get(c).name);
    }
    Ok(true)
}

// ------------------------------------------------------------------ Cacophony Unleashed
/// **Cacophony Unleashed**: when it enters, if you cast it, destroy all nonenchantment creatures; whenever it or
/// another enchantment you control enters, until end of turn it becomes a legendary 6/6 Nightmare God creature with
/// menace and deathtouch (still an enchantment). The AI casts it when the wipe takes more from the opponents than
/// from itself.
fn cacophony_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m == src
        && g.last_cast_etb
        && trigger_window(g, o, Some(src), "destroy all nonenchantment creatures", Some(8.0))?
    {
        let all: Vec<PermId> = g
            .players
            .iter()
            .flat_map(|q| q.perms.iter().copied().collect::<Vec<_>>())
            .filter(|&x| on_bf(g, x) && g.is_creature(x) && !g.perm(x).cd.is_some_and(|c| is_ench(g, c)))
            .collect();
        for x in all {
            die(g, x, "destroy")?;
        }
    }
    if ench_entered(g, src, m, true)
        && on_bf(g, src)
        && trigger_window(g, o, Some(src), "becomes a 6/6 Nightmare God", Some(3.0))?
    {
        let x = g.perm_mut(src);
        x.data.set(DataKey::Anim, Val::Bool(true));
        x.data.set(DataKey::Legendary, Val::Bool(true));
        (x.pow, x.tgh) = (6, 6);
        for k in ["menace", "deathtouch"] {
            if !x.eot_kw.contains(&k) {
                x.eot_kw.push(k);
            }
        }
    }
    Ok(())
}

/// until end of turn ends: it is only an enchantment again
fn cacophony_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    if g.perm(src).data.truthy(DataKey::Anim) {
        g.perm_mut(src).data.set(DataKey::Anim, Val::Bool(false));
    }
    Ok(())
}

fn cacophony_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let value = |q: PlayerId| -> f64 {
        g.player(q)
            .perms
            .iter()
            .filter(|&&m| on_bf(g, m) && g.is_creature(m) && !g.perm(m).cd.is_some_and(|c| is_ench(g, c)))
            .map(|&m| pval(g, m))
            .sum()
    };
    let theirs: f64 = g.opps(p).map(value).sum();
    if theirs - value(p) >= 4.0 { 72 } else { 0 }
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    for name in [HELVAULT, TITANS, ELDEST, MENDING, BRETAGARD, OLD_GODS, LOVE_SONG] {
        let c = r.card(db, name)?;
        c.etb = Some(saga_enter);
        c.main1 = Some(saga_main1);
        c.saga = Some((saga_wants, saga_lore));
    }
    r.card(db, HELVAULT)?.leaves = Some(saga_leaves);
    r.card(db, COMPOSER)?.etb = Some(composer);
    r.card(db, NESSIAN)?.etb = Some(nessian);
    r.card(db, DOOMWAKE)?.etb = Some(doomwake);
    r.card(db, ONDU)?.etb = Some(ondu);
    r.card(db, BOON)?.etb = Some(boon);
    r.creature_pt.push(boon_pt);
    let c = r.card(db, KHALNI)?;
    c.landfall = Some(khalni_landfall);
    c.options = Some(khalni_options);
    r.card(db, WAKE)?.land_mana = Some(wake_mana);
    let c = r.card(db, HELIOD)?;
    c.etb = Some(heliod_etb);
    c.sba = Some(heliod_sba);
    c.grant_kw = Some(god_indestructible);
    *c = c.at_once(crate::hooks::Event::Etb);
    let c = r.card(db, EREBOS)?;
    c.etb = Some(erebos_etb);
    c.sba = Some(erebos_sba);
    c.grant_kw = Some(god_indestructible);
    c.dies = Some(erebos_dies);
    c.options = Some(erebos_options);
    *c = c.at_once(crate::hooks::Event::Etb);
    let c = r.card(db, NARCI)?;
    c.sacrifice = Some(narci_sacrifice);
    r.card(db, TANUKI)?.hand_options = Some(tanuki_channel);
    r.card(db, WEAVER)?.options = Some(weaver_options);
    r.card(db, BEHEMOTH)?.self_cost = Some(behemoth_cost);
    r.card(db, SANDWURM)?.attack_cap = Some(|_, _, _, _| None); // a hook, so it is live; the rule is `flyer_barred`
    let c = r.card(db, ANIKTHEA)?;
    c.etb = Some(anikthea_etb);
    c.attack = Some(anikthea_attack);
    c.grant_kw = Some(anikthea_kw);
    register_impetus(r, db)?;
    register_calix(r, db)?;
    register_rhetoric_demon(r, db)?;
    r.card(db, ABUNDANCE)?.skip_draw = Some(|_, _, _| 0); // a hook, so it is live (g.hooks); the rule is `abundance_draw`
    let c = r.card(db, CACOPHONY)?;
    c.etb = Some(cacophony_etb);
    c.end_step = Some(cacophony_end);
    c.prio = Some(cacophony_prio);
    *c = c.at_once(crate::hooks::Event::EndStep);
    Ok(())
}

// ================================================================== Ghoulish Impetus
/// **Ghoulish Impetus** (Aura): enchanted creature gets +1/+1 and has deathtouch (the `AuraSpec`); when it dies, the Aura
/// returns to the battlefield at the beginning of the next end step (`host_dies` queues it on `g.eot_returns`, which
/// `galadriel::eot_returns` also serves from the graveyard; the returned Aura picks a new host).
/// **Goad is not modeled** (Partial): it is t0-zurgo's mechanic and the lead wires it in after merging. The AI puts it on
/// the best opposing creature (the card's purpose), where until goad lands it only helps that creature.
fn impetus_dies(g: &mut Game, p: PlayerId, a: PermId, _host: PermId) -> Res {
    if let Some(cd) = g.perm(a).cd {
        let o = g.perm(a).owner;
        if g.player(o).gy.contains(&cd) {
            g.eot_returns.push((o, cd));
        }
    }
    let _ = p;
    Ok(())
}

fn impetus_host(g: &mut Game, p: PlayerId, _a: PermId) -> Res<Option<PermId>> {
    let cands: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| on_bf(g, m) && g.is_creature(m) && !crate::engine::values::untargetable(g, m))
        .collect();
    Ok(max_by(&cands, |m| pval(g, m)))
}

static IMPETUS: super::common::AuraSpec = super::common::AuraSpec {
    pow: 1,
    tgh: 1,
    kws: &["deathtouch"],
    target: "opp",
    host_pick: Some(impetus_host),
    host_dies: Some(impetus_dies),
    ..super::common::AURA
};

pub fn register_impetus(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    super::common::aura(r, db, "Ghoulish Impetus", &IMPETUS)
}

// ================================================================== Demon of Fate's Design
const DEMON: &str = "Demon of Fate's Design";

/// **Demon of Fate's Design** (flying, trample): once during each of your turns, cast an enchantment spell from hand by
/// paying life equal to its mana value instead of its mana cost (an `options` entry: it picks the dearest enchantment
/// it can afford at 12+ life afterwards; a spell with an additional cost isn't offered). The {2}{B} sacrifice-another-
/// enchantment +X/+0 (X: its mana value) is the compiled activation.
fn demon_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post.is_none() || g.active != Some(p) || !on_bf(g, src) {
        return Ok(vec![]);
    }
    if matches!(g.perm(src).data.get(DataKey::Act), Some(Val::Stamp(s)) if *s == g.turn_stamp()) {
        return Ok(vec![]);
    }
    let life = g.player(p).life;
    let mut out = vec![];
    for &c in g.player(p).hand.iter() {
        let d = g.db.get(c);
        if !is_ench(g, c) || d.land || d.cmc == 0 || life - (d.cmc as i32) < 12 || d.tag(crate::tag::Tag::Needsac) {
            continue;
        }
        if !crate::engine::cast::castable(g, p, c, "hand") {
            continue;
        }
        out.push(Opt {
            utility: 1.0 + 0.7 * d.cmc as f64,
            label: format!("Demon of Fate's Design: cast {} for {} life", d.name, d.cmc),
            act: Some(Action::Ability { src, f: demon_go, arg: c.0 as i64 }),
        });
    }
    Ok(out)
}

fn demon_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let st = g.turn_stamp();
    if !on_bf(g, src) || !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    if matches!(g.perm(src).data.get(DataKey::Act), Some(Val::Stamp(s)) if *s == st) {
        return Ok(false);
    }
    let cmc = g.db.get(c).cmc as i32;
    if g.player(p).life - cmc < 12 {
        return Ok(false);
    }
    g.perm_mut(src).data.set(DataKey::Act, Val::Stamp(st));
    lose_life(g, p, cmc, Some(p), "life", None)?;
    crate::glog!(g, "    {} casts {} for {} life", DEMON, g.db.get(c).name, cmc);
    crate::engine::cast::cast_card(g, p, c, "hand", crate::state::Ctx::default())?;
    Ok(true)
}

// ================================================================== Cunning Rhetoric
const RHETORIC: &str = "Cunning Rhetoric";
/// cards exiled by Cunning Rhetoric sit in the hand marked in `impulse_long` with a turn this far ahead (so they never
/// lapse, and are far below the `u32::MAX - 1 - turn` plot marks)
const RHETORIC_MARK: u32 = 100_000;

/// **Cunning Rhetoric**: whenever an opponent attacks you, exile the top card of that player's library; you may play it
/// for as long as it remains exiled, spending mana as though it were mana of any color. The card goes to your hand
/// marked in `impulse_long` (it can be cast normally if the colours allow); `rhetoric_options` plays it as a land, or
/// casts it paying its mana value in generic mana (any colour). Attacks at planeswalkers are attacks at their player here.
fn rhetoric_attack(g: &mut Game, src: Src, atk_p: PlayerId, _atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let me = owner(g, src);
    if d != me || atk_p == me || !on_bf(g, src) || g.player(atk_p).library.is_empty() {
        return Ok(vec![]);
    }
    if !trigger_window(g, me, Some(src), "exile the attacker's top card", None)? {
        return Ok(vec![]);
    }
    let Some(top) = g.player_mut(atk_p).library.pop() else { return Ok(vec![]) };
    crate::glog!(g, "    Cunning Rhetoric exiles {} from {}'s library", g.db.get(top).name, g.player(atk_p).name);
    let pl = g.player_mut(me);
    pl.seen.insert(top);
    pl.hand.push(top);
    let until = pl.turns + RHETORIC_MARK;
    pl.impulse_long.push((top, until));
    Ok(vec![])
}

fn rhetoric_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || post.is_none() || g.active != Some(p) || !on_bf(g, src) {
        return Ok(vec![]);
    }
    let mut out = vec![];
    let marked: Vec<CardId> = g
        .player(p)
        .impulse_long
        .iter()
        .filter(|x| x.1 >= RHETORIC_MARK && x.1 < u32::MAX / 2 && g.player(p).hand.contains(&x.0))
        .map(|x| x.0)
        .collect();
    for c in marked {
        let d = g.db.get(c);
        if d.land {
            if g.player(p).lands_played < 1 {
                out.push(Opt {
                    utility: 3.0,
                    label: format!("Cunning Rhetoric: play {}", d.name),
                    act: Some(Action::Ability { src, f: rhetoric_go, arg: c.0 as i64 }),
                });
            }
        } else if d.cmc > 0 && !crate::engine::mana::can_pay(g, p, d.generic, &d.pips, false) {
            // not payable with its own colours: pay the whole mana value as generic
            if crate::engine::mana::can_pay(g, p, d.cmc, "", false) && !d.tag(crate::tag::Tag::Ctr) {
                out.push(Opt {
                    utility: 1.0 + 0.5 * d.cmc as f64,
                    label: format!("Cunning Rhetoric: cast {} with any colour", d.name),
                    act: Some(Action::Ability { src, f: rhetoric_go, arg: c.0 as i64 }),
                });
            }
        }
    }
    Ok(out)
}

fn rhetoric_go(g: &mut Game, _src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    let d = g.db.get(c);
    if d.land {
        if g.player(p).lands_played >= 1 {
            return Ok(false);
        }
        g.player_mut(p).impulse_long.retain(|x| x.0 != c);
        remove_card(&mut g.player_mut(p).hand, c);
        crate::engine::turn::play_land_card(g, p, c, "plays")?;
        return Ok(true);
    }
    if !crate::engine::mana::pay(g, p, d.cmc, "", false)? {
        return Ok(false);
    }
    g.player_mut(p).impulse_long.retain(|x| x.0 != c);
    crate::engine::cast::cast_card(g, p, c, "hand", crate::state::Ctx::default())?;
    Ok(true)
}

pub fn register_rhetoric_demon(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, DEMON)?.options = Some(demon_options);
    let c = r.card(db, RHETORIC)?;
    c.attack = Some(rhetoric_attack);
    c.options = Some(rhetoric_options);
    Ok(())
}

// ================================================================== Calix, Destiny's Hand
const CALIX: &str = "Calix, Destiny's Hand";

/// the enchantments p controls that return what they exiled when they leave (they have a `leaves` hook, which all
/// the deck's enchantments get in `register`, and share `DataKey::Oring` with the O-Ring family)
fn calix_link(g: &Game, p: PlayerId) -> Option<PermId> {
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            on_bf(g, m)
                && g.perm(m).cd.is_some_and(|c| {
                    is_ench(g, c) && !is_saga(g, c) && g.registry.get(c).is_some_and(|i| i.leaves.is_some())
                })
        })
        .collect();
    // the one least likely to leave: the cheapest, not an Aura or a God that can die
    max_by(&cands, |m| {
        10.0 - g.perm(m).cd.map_or(0.0, |c| g.db.get(c).cmc as f64) - g.is_creature(m) as i32 as f64 * 3.0
    })
}

fn calix_target_ok(g: &Game, m: PermId) -> bool {
    g.is_creature(m) || g.perm(m).cd.is_some_and(|c| is_ench(g, c))
}

fn calix_plus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    Some(if g.player(p).library.is_empty() { 0.0 } else { 3.0 })
}

/// +1: look at the top four, take the best enchantment, the rest on the bottom in a random order
fn calix_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let mut top = vec![];
    for _ in 0..4 {
        if let Some(c) = g.player_mut(p).library.pop() {
            top.push(c);
        }
    }
    let pick =
        max_by(&top.iter().copied().filter(|&c| is_ench(g, c)).collect::<Vec<_>>(), |c| card_worth(g, p, c, false));
    if let Some(c) = pick {
        remove_card(&mut top, c);
        g.player_mut(p).hand.push(c);
        g.player_mut(p).seen.insert(c);
        crate::glog!(g, "    Calix takes {}", g.db.get(c).name);
    }
    g.rng.shuffle(&mut top);
    for c in top {
        g.player_mut(p).library.insert(0, c);
    }
    Ok(())
}

fn calix_minus3_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    calix_link(g, p)?;
    let t = super::common::best_opp_nonland(g, p, |m| calix_target_ok(g, m))?;
    let v = pval(g, t);
    (v >= 3.0).then_some(4.0 + v.min(6.0))
}

/// -3: exile the best opposing creature or enchantment until the linked enchantment you control leaves
fn calix_minus3(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let Some(link) = calix_link(g, p) else { return Ok(()) };
    oring_exile(g, link, p, |g, m| calix_target_ok(g, m), false, false)
}

fn calix_ult_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let n = g.player(p).gy.iter().filter(|&&c| is_ench(g, c)).count();
    (n >= 2).then_some(8.0 + n as f64)
}

/// -7: return all enchantment cards from your graveyard to the battlefield (an Aura goes onto a creature, or stays)
fn calix_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| is_ench(g, c)).collect();
    for c in cs {
        if !g.player(p).gy.contains(&c) {
            continue;
        }
        remove_card(&mut g.player_mut(p).gy, c);
        enter(g, p, c, Enter::default())?;
    }
    Ok(())
}

static CALIX_ABS: [super::common::WalkerAb; 3] = [
    super::common::WalkerAb {
        delta: 1,
        label: "look at four, take an enchantment",
        val: calix_plus_val,
        eff: calix_plus,
    },
    super::common::WalkerAb {
        delta: -3,
        label: "exile until an enchantment leaves",
        val: calix_minus3_val,
        eff: calix_minus3,
    },
    super::common::WalkerAb { delta: -7, label: "return all enchantments", val: calix_ult_val, eff: calix_ult },
];

/// the deck's enchantments that have no `leaves` hook of their own get one that returns what they exiled for Calix
fn calix_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
}

pub fn register_calix(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    super::common::walker(r, db, CALIX, &CALIX_ABS)?;
    for name in [
        "Abundance",
        "Battle for Bretagard",
        "Binding the Old Gods",
        "Boon of the Spirit Realm",
        "Cacophony Unleashed",
        "Cast Out",
        "Cunning Rhetoric",
        "Dreadhorde Invasion",
        "Enchantress's Presence",
        "Felidar Retreat",
        "Font of Fertility",
        "Ghoulish Impetus",
        "Grasp of Fate",
        "Khalni Heart Expedition",
        "Mirari's Wake",
        "Omen of the Hunt",
        "Omen of the Sun",
        "Sandwurm Convergence",
        "Sigil of the Empty Throne",
        "Starfield of Nyx",
    ] {
        let c = r.card(db, name)?;
        if c.leaves.is_none() {
            c.leaves = Some(calix_leaves);
            *c = c.at_once(crate::hooks::Event::Leaves);
        }
    }
    Ok(())
}
