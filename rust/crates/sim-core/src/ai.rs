//! The AI's decisions that the engine calls inline (Python's `ais.py`, `ai/brain.py`, `ai/pool_ai.py`,
//! `ai/search.py`). Ported in M3 and M4; until then each is a `PORT(Mx)` placeholder that makes the plainest choice
//! (no response, no protection), so the engine can be built and tested on its own.

use crate::ids::PlayerId;
use crate::state::Game;
use crate::sym::Sym;

/// brain.note_damage: remember who hurt p (the defender choice hits back at a grudge, which fades by 0.7 a hit)
pub fn note_damage(g: &mut Game, p: PlayerId, src: Option<PlayerId>, n: i32) {
    let Some(s) = src.filter(|&s| s != p) else { return };
    let key: Sym = g.player(s).key;
    let gr = g.player_mut(p).grudge.entry(key).or_insert(0.0);
    *gr = *gr * 0.7 + n as f64;
}

use crate::engine::values::epow;
use crate::flow::Res;
use crate::ids::{CardId, PermId};
use crate::state::Ctx;

/// ais.win: p wins, every opponent loses. A win through life or damage (a drain, burn or combat loop: through_life,
/// the default for 'combo') is survived by an opponent whose life can't change (Teferi's Protection, cast now if
/// they hold it); alternate wins (Thassa's Oracle, Revel in Riches, mill) are not.
pub fn win(g: &mut Game, p: PlayerId, how: &'static str, through_life: Option<bool>) -> Res {
    let through_life = through_life.unwrap_or(how == "combo");
    let mut left = vec![];
    for q in g.opps(p).collect::<Vec<_>>() {
        if through_life && crate::engine::zones::last_chance(g, q)? {
            crate::glog!(g, "    {} survives: its life total can't change", g.player(q).name);
            left.push(q);
            continue;
        }
        let pl = g.player_mut(q);
        pl.last_src = Some(p);
        pl.last_kind = how;
        crate::engine::life::eliminate(g, q);
    }
    if !left.is_empty() && !g.goldfish {
        let names: Vec<&str> = left.iter().map(|&q| g.player(q).name).collect();
        crate::glog!(g, "*** {} goes off, but {} survive", g.player(p).name, names.join(", "));
        return crate::engine::life::check_state(g);
    }
    crate::glog!(g, "*** {} wins by {}", g.player(p).name, how);
    g.over = true;
    g.winner = Some(p);
    g.wintype = Some(how);
    Ok(())
}

/// ais.regrow (Eternal Witness, Mnemonic Wall): the first card of this list in the graveyard back to hand
pub fn regrow(g: &mut Game, p: PlayerId, only_is: bool) -> Res {
    const ORDER: [&str; 17] = [
        "Reanimate",
        "Animate Dead",
        "Demonic Tutor",
        "Necromancy",
        "Entomb",
        "Buried Alive",
        "Dread Return",
        "Diabolic Tutor",
        "Unburial Rites",
        "Persist",
        "Counterspell",
        "Swan Song",
        "Dovin's Veto",
        "Heroic Intervention",
        "Toxic Deluge",
        "Swords to Plowshares",
        "Assassin's Trophy",
    ];
    for n in ORDER {
        let Some(c) = g.db.id(n) else { continue };
        let d = g.db.get(c);
        if only_is && !(d.instant || d.sorcery) {
            continue;
        }
        let key = crate::engine::zones::KEYSPELL.iter().any(|&t| d.tag(t));
        let pl = g.player_mut(p);
        if let Some(i) = pl.gy.iter().position(|&x| x == c) {
            pl.gy.remove(i);
            pl.hand.push(c);
            if key {
                pl.stat("key_regrown", 1);
            }
            return Ok(());
        }
    }
    Ok(())
}

/// ais.necro_floor: the life to keep: the biggest board one opponent could swing at you plus 6, never under 10
pub fn necro_floor(g: &Game, p: PlayerId) -> i32 {
    let biggest = g
        .opps(p)
        .map(|q| {
            g.player(q).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).map(|&m| epow(g, m)).sum()
        })
        .max()
        .unwrap_or(0);
    10.max(biggest + 6)
}

/// pool_ai.counter_threshold: the importance a spell needs before q counters it, lowered for decks built around
/// counterspells
pub fn counter_threshold(g: &Game, q: PlayerId, base: f64) -> f64 {
    let n = g.player(q).deck_names.iter().filter(|&&c| g.db.get(c).tag(crate::tag::Tag::Ctr)).count() as f64;
    base - (0.3 * n).min(2.5)
}

fn sig(x: f64) -> f64 {
    1.0 / (1.0 + (-x.clamp(-30.0, 30.0)).exp())
}

/// brain.wants_counter: the probability q spends a counter on a spell of importance val (to q)
pub fn wants_counter(val: f64, thr: f64, ncounters: usize) -> f64 {
    let scarcity = 0.65 + 0.35 * (ncounters as f64 / 2.0).min(1.0);
    sig((val - thr + 0.5) / 0.8) * scarcity
}

/// ais.land_enters_tapped. PORT(M3): the Oracle-text rules (check lands, fast lands ...: enters_rule) and Thalia,
/// Heretic Cathar / Archon of Emeria
pub fn land_enters_tapped(g: &Game, _p: PlayerId, c: CardId) -> bool {
    let t = &g.db.get(c).tags;
    t.has(crate::tag::Tag::F) || t.has(crate::tag::Tag::T)
}

/// PORT(M4): ais.flute_pick, the card name Disruptor Flute names
pub fn flute_pick(_g: &Game, _p: PlayerId) -> Option<CardId> {
    None
}

/// PORT(M4): ais.tutor_pick, the card a tutor of this kind fetches
pub fn tutor_pick(_g: &Game, _p: PlayerId, _kind: &str) -> Option<CardId> {
    None
}

/// PORT(M4): ais.tutor_value
pub fn tutor_value(_g: &Game, _p: PlayerId, _c: CardId) -> f64 {
    0.0
}

/// PORT(M4): ais.gy_worth (a reanimation target's value in the graveyard)
pub fn gy_worth(_g: &Game, _p: PlayerId, _c: CardId) -> f64 {
    0.0
}

/// PORT(M4): ais.deck_prio, the deck's cast priority for a card (0-90)
pub fn deck_prio(_g: &Game, _p: PlayerId, _c: CardId) -> f64 {
    0.0
}

/// PORT(M4): CI.card_etb_value
pub fn card_etb_value(_g: &Game, _p: PlayerId, _c: CardId) -> f64 {
    0.0
}

/// PORT(M4): CI.combo_imp, a combo piece's importance (9: completes a combo, 7: one short)
pub fn combo_imp(_g: &Game, _p: PlayerId, _c: CardId) -> f64 {
    0.0
}

/// PORT(M4): CI.spell_importance, how much opponents want to counter a pool deck's spell (0-9)
pub fn spell_importance(_g: &Game, _p: PlayerId, _c: CardId) -> f64 {
    0.0
}

/// PORT(M4): the cards an outside deck keeps when discarding (combo pieces and its wish list)
pub fn discard_keep(_g: &Game, _p: PlayerId) -> Vec<CardId> {
    vec![]
}

/// PORT(M4): ais.protect_response, owner protects m from removal (Heroic Intervention, Ephemerate ...): true if it did
pub fn protect_response(
    _g: &mut Game,
    _owner: PlayerId,
    _m: PermId,
    _kind: Sym,
    _actor: Option<PlayerId>,
    _spell: Option<CardId>,
) -> Res<bool> {
    Ok(false)
}

/// PORT(M4): ais.wipe_modes (Farewell, Austere Command's modes). Until then: creatures.
pub fn wipe_modes(_g: &Game, _p: PlayerId, _kind: Sym) -> Vec<Sym> {
    vec!["cre"]
}

/// PORT(M4): ais.wipe_response, q's answer to a wipe: 'all' (Teferi's Protection ...), 'indes', or none
pub fn wipe_response(_g: &mut Game, _q: PlayerId, _kind: Sym, _caster: PlayerId) -> Res<Option<Sym>> {
    Ok(None)
}

/// PORT(M5): ais.seph_fill_resolve (Entomb, Buried Alive ...: what goes into the graveyard)
pub fn seph_fill_resolve(_g: &mut Game, _p: PlayerId, _kind: Sym, _ctx: &Ctx) -> Res {
    Ok(())
}

/// PORT(M5): ais.seph_rean_resolve (a reanimation spell resolves)
pub fn seph_rean_resolve(_g: &mut Game, _p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res {
    Ok(())
}
