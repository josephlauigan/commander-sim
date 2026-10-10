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

/// brain.chump_prob: how likely d chump-blocks, by how much of its life is coming at it
pub fn chump_prob(g: &Game, d: PlayerId, incoming: i32) -> f64 {
    let frac = incoming as f64 / g.player(d).life.max(1) as f64;
    sig((frac - 0.35) / 0.08)
}

/// PORT(M4): brain.choose_defender (threat, lethal, grudge, blockers, taxes, play style). Until then the fixed rule of
/// ais.choose_defender: a player the attack can nearly kill, else the biggest threat with a little noise.
pub fn choose_defender(g: &mut Game, p: PlayerId) -> PlayerId {
    use crate::engine::values::{shielded, threat};
    let mut opps: Vec<PlayerId> = g.opps(p).filter(|&q| !shielded(g, q)).collect();
    if opps.is_empty() {
        opps = g.opps(p).collect();
    }
    let my: i32 = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.tapped && !x.noatk && !x.sick
        })
        .map(|&m| epow(g, m))
        .sum();
    let lethal: Vec<PlayerId> = opps.iter().copied().filter(|&q| g.player(q).life as f64 <= my as f64 * 0.6).collect();
    if let Some(q) = crate::engine::zones::min_by(&lethal, |q| g.player(q).life as f64) {
        return q;
    }
    let mut best: Option<(PlayerId, f64)> = None;
    for q in opps {
        let v = threat(g, p, q) + g.rng.random() * 3.0;
        if best.is_none_or(|b| v > b.1) {
            best = Some((q, v));
        }
    }
    best.unwrap().0
}

/// PORT(M4): brain.filter_attackers (keep ground creatures home when the table threatens a lot of damage)
pub fn filter_attackers(_g: &mut Game, _p: PlayerId, atk: Vec<PermId>) -> Vec<PermId> {
    atk
}

/// PORT(M4): pool_ai.attack_filter (an outside deck's own attack rules)
pub fn attack_filter(_g: &mut Game, _p: PlayerId, atk: Vec<PermId>, _d: PlayerId) -> Vec<PermId> {
    atk
}

/// PORT(M4): search.choose_attack, the look-ahead's attack plan: (defender, 'filtered' | 'all' | 'none')
pub fn choose_attack(_g: &mut Game, _p: PlayerId) -> Res<Option<(PlayerId, Sym)>> {
    Ok(None)
}

/// PORT(M4): brain.attack_response (instant removal on a dangerous attacker)
pub fn attack_response(_g: &mut Game, _d: PlayerId, _p: PlayerId, _atk: &[PermId]) -> Res<bool> {
    Ok(false)
}

/// PORT(M4): brain.end_of_turn_window (instant-speed draw, flash creatures and removal before your turn)
pub fn end_of_turn_window(_g: &mut Game, _p: PlayerId) -> Res {
    Ok(())
}

/// PORT(M4): brain.main, the heuristic AI's main phase (with the look-ahead's choices). Until then: cast the most
/// expensive permanent or untargeted spell that's affordable, the commander included, until nothing fits.
pub fn main(g: &mut Game, p: PlayerId, _post: bool) -> Res {
    use crate::engine::{cast, mana};
    use crate::tag::Tag;
    for _ in 0..18 {
        if g.over || !g.player(p).alive {
            return Ok(());
        }
        g.tick()?;
        let pl = g.player(p);
        let mut cands: Vec<(CardId, Sym)> = pl
            .hand
            .iter()
            .copied()
            .filter(|&c| {
                let d = g.db.get(c);
                !d.land && ![Tag::Rem, Tag::Wipe, Tag::Ctr, Tag::Rean, Tag::Fill].iter().any(|&t| d.tag(t))
            })
            .map(|c| (c, "hand"))
            .collect();
        if pl.cmd_in_zone {
            cands.push((pl.cmd, "cmd"));
        }
        let mut best: Option<(CardId, Sym, u32, String)> = None;
        for (c, zone) in cands {
            let (gn, pips) = mana::cost_of(g, p, c);
            if !cast::castable(g, p, c, zone) || !mana::can_pay(g, p, gn, &pips, false) {
                continue;
            }
            if best.as_ref().is_none_or(|b| g.db.get(c).cmc > g.db.get(b.0).cmc) {
                best = Some((c, zone, gn, pips));
            }
        }
        let Some((c, zone, gn, pips)) = best else { return Ok(()) };
        g.pay_for = Some(c);
        let paid = mana::pay(g, p, gn, &pips, false);
        g.pay_for = None;
        if !paid? {
            return Ok(());
        }
        cast::cast_card(g, p, c, zone, crate::state::Ctx::default())?;
    }
    Ok(())
}

/// PORT(M5): ais.combo_interrupted (an opponent stops a combo with a counter or removal)
pub fn combo_interrupted(_g: &mut Game, _p: PlayerId, _which: &str, _key: &[PermId]) -> Res<bool> {
    Ok(false)
}

/// PORT(M5): ais.seph_bval (Sephiroth's reanimation target value). Until then: the card's bomb rating.
pub fn seph_bval(g: &Game, _p: PlayerId, c: CardId) -> f64 {
    g.db.get(c).bomb as f64
}

/// PORT(M5): ais.note_bomb (Sephiroth's reports)
pub fn note_bomb(_g: &mut Game, _p: PlayerId, _c: CardId, _was_removed: bool) {}

/// PORT(M5): ais.seph_dredge (Sephiroth dredges instead of drawing)
pub fn seph_dredge(_g: &mut Game, _p: PlayerId) -> Res<bool> {
    Ok(false)
}

/// PORT(M5): ais.engine_payoff (Veyran's engine is online)
pub fn engine_payoff(_g: &Game, _p: PlayerId) -> bool {
    false
}

/// PORT(M5): ais.end_step's Sephiroth milestones (reports)
pub fn seph_end_milestones(_g: &mut Game, _p: PlayerId) {}
