//! Python's play/cards.py. Ported by the practice-cards task.
//! The per-card choices of the people's decks (targets, modes, tutors, pay-or-not, ordering), and the helpers the
//! `HUMAN(phase 9)` sites in `impls/*` share.

use super::{Target, choices, legal};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

/// `hc.choose(g, p, kind, prompt, [legal.describe_target(g, p, m) for m in cands], cancel=...)`: the permanent the
/// person picks from `cands`, None if they cancel (`cancel: None` is a forced choice: the first when no answer fits).
pub fn pick_perm(
    g: &Game,
    p: PlayerId,
    kind: &str,
    prompt: &str,
    cands: &[PermId],
    cancel: Option<&str>,
) -> Res<Option<PermId>> {
    let labels: Vec<String> = cands.iter().map(|&m| legal::describe_target(g, p, Target::Perm(m))).collect();
    Ok(choices::choose(g, p, kind, prompt, &labels, cancel, None)?.map(|k| cands[k]))
}

/// `hc.choose(g, p, kind, prompt, [NAME(q) for q in opps], cancel=None)`: the opponent the person picks.
pub fn pick_player(g: &Game, p: PlayerId, kind: &str, prompt: &str, opps: &[PlayerId]) -> Res<PlayerId> {
    let labels: Vec<String> = opps.iter().map(|&q| g.player(q).name.to_string()).collect();
    Ok(opps[choices::choose_one(g, p, kind, prompt, &labels)?])
}

/// cards._creatures: creatures p's spell or ability can target (opponents' hexproof and shroud respected)
pub fn creatures(g: &Game, p: PlayerId, opp_only: bool) -> Vec<PermId> {
    let mut out = vec![];
    for q in g.players.iter().filter(|q| q.alive && !(opp_only && q.id == p)) {
        for &m in &q.perms {
            if g.is_creature(m)
                && !g.perm(m).phased
                && !(g.perm(m).owner != p && crate::engine::values::untargetable(g, m))
            {
                out.push(m);
            }
        }
    }
    out
}

/// cards.pick_creature: the creature the person picks (None: no target, or they decline an `optional` one)
pub fn pick_creature(
    g: &Game,
    p: PlayerId,
    prompt: &str,
    opp_only: bool,
    optional: bool,
    keep: Option<&dyn Fn(&Game, PlayerId, PermId) -> bool>,
) -> Res<Option<PermId>> {
    let tg: Vec<PermId> = creatures(g, p, opp_only).into_iter().filter(|&m| keep.is_none_or(|f| f(g, p, m))).collect();
    if tg.is_empty() {
        return Ok(None);
    }
    pick_perm(g, p, "target", prompt, &tg, if optional { Some("no target") } else { None })
}

/// cards._blink: a token is exiled for good; else t2.blink
pub fn blink(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if g.perm(m).token {
        crate::engine::zones::leave(g, m)?;
        crate::glog!(g, "    {} (a token) is exiled for good", g.perm(m).name);
        return Ok(());
    }
    crate::impls::t2::blink(g, p, m)?;
    Ok(())
}

/// cards.ephemerate_rebound: your upkeep: cast Ephemerate from exile for free, on a creature you choose; not cast, it
/// stays in exile
pub fn ephemerate_rebound(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let cre: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
    if cre.is_empty() {
        return Ok(());
    }
    let Some(m) = pick_perm(
        g,
        p,
        "target",
        "Rebound: cast Ephemerate again? Exile and return which creature",
        &cre,
        Some("don't cast it"),
    )?
    else {
        return Ok(());
    };
    let pl = g.player_mut(p);
    if let Some(i) = pl.exile.iter().position(|&x| x == c) {
        pl.exile.remove(i);
    }
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::engine::cast::on_cast(g, p, c)?;
    crate::glog!(g, "  {} casts Ephemerate from exile (rebound) on {}", g.player(p).name, g.perm(m).name);
    if !g.over && crate::engine::stack::counter_window(g, p, c, 3.0, vec![])? && g.player(p).perms.contains(&m) {
        blink(g, p, m)?;
    }
    g.player_mut(p).gy.push(c);
    Ok(())
}


/// cards.pick_exile: 'you may exile n cards from your graveyard': no, the suggested n, or n picked one at a time. The
/// cards, or []
pub fn pick_exile(
    g: &Game,
    p: PlayerId,
    pool: &[CardId],
    n: usize,
    prompt: &str,
    default: &[CardId],
) -> Res<Vec<CardId>> {
    let names: Vec<&str> = default.iter().map(|&c| &*g.db.get(c).name).collect();
    let labels = vec![format!("yes: {}", names.join(", ")), "yes, and I pick the cards".to_string()];
    let Some(k) = choices::choose(g, p, "choose", prompt, &labels, Some("no"), None)? else { return Ok(vec![]) };
    if k == 0 {
        return Ok(default.to_vec());
    }
    let mut picked: Vec<CardId> = vec![];
    while picked.len() < n {
        let left: Vec<CardId> = pool.iter().copied().filter(|c| !picked.contains(c)).collect();
        let labels: Vec<String> = left.iter().map(|&c| g.db.get(c).name.to_string()).collect();
        let prompt = format!("Exile which card? ({} of {n})", picked.len() + 1);
        let j = choices::choose_one(g, p, "choose", &prompt, &labels)?;
        picked.push(left[j]);
    }
    Ok(picked)
}

/// cards.pick_blue: an alternative cost that exiles a blue card from your hand (Force of Will, Force of Negation):
/// which one
pub fn pick_blue(g: &Game, p: PlayerId, blues: &[CardId], what: &str) -> Res<CardId> {
    if blues.len() == 1 {
        return Ok(blues[0]);
    }
    let labels: Vec<String> = blues.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    let k = choices::choose_one(g, p, "choose", &format!("{what}: exile which blue card from your hand?"), &labels)?;
    Ok(blues[k])
}

/// cards._look: the top n cards of the library, taken off it (the top first)
fn look(g: &mut Game, p: PlayerId, n: usize) -> Vec<CardId> {
    let k = n.min(g.player(p).library.len());
    (0..k).map(|_| g.player_mut(p).library.pop().unwrap()).collect()
}

/// cards._bottom: cards to the bottom of the library (index 0)
fn bottom(g: &mut Game, p: PlayerId, cards: Vec<CardId>) {
    g.player_mut(p).library.splice(0..0, cards);
}

/// cards.expressive_iteration: top three: one into your hand, one exiled (you may play it this turn), the last on the
/// bottom
pub fn expressive_iteration(g: &mut Game, p: PlayerId) -> Res {
    let mut top = look(g, p, 3);
    if top.is_empty() {
        return Ok(());
    }
    let keep = choices::pick_cards(g, p, &top, 1, "Expressive Iteration: put which card into your hand?", "choose")?[0];
    remove_one(&mut top, keep);
    g.player_mut(p).hand.push(keep);
    if !top.is_empty() {
        let prompt = "Expressive Iteration: exile which card (you may play it this turn)?";
        let ex = choices::pick_cards(g, p, &top, 1, prompt, "choose")?[0];
        remove_one(&mut top, ex);
        let pl = g.player_mut(p);
        pl.hand.push(ex);
        pl.impulse.push(ex);
    }
    let n = top.len();
    bottom(g, p, top);
    crate::glog!(g, "    {} keeps one card, exiles one to play this turn, puts {} on the bottom", g.player(p).name, n);
    Ok(())
}

/// cards.look_and_take: look at the top n, put k into your hand, the rest on the bottom (Flow State, Stock Up)
pub fn look_and_take(g: &mut Game, p: PlayerId, n: usize, k: usize, name: &str) -> Res {
    let mut top = look(g, p, n);
    let picked = choices::pick_cards(g, p, &top, k, &format!("{name}: put which card into your hand?"), "choose")?;
    for &x in &picked {
        remove_one(&mut top, x);
        g.player_mut(p).hand.push(x);
    }
    let left = top.len();
    bottom(g, p, top);
    crate::glog!(g, "    {} takes {}, puts {} on the bottom", g.player(p).name, picked.len(), left);
    Ok(())
}

/// Python's `list.remove`: the first element equal to x
fn remove_one(v: &mut Vec<CardId>, x: CardId) {
    if let Some(i) = v.iter().position(|&y| y == x) {
        v.remove(i);
    }
}

/// cards._damage: n damage from source to x, a player or permanent chosen already (the same rules as
/// choices.deal_damage)
pub fn damage(g: &mut Game, p: PlayerId, x: Target, n: i32, source: &str, kind: &str) -> Res {
    use crate::engine::{life, removal, zones};
    let m = match x {
        Target::Player(q) => {
            if g.player(q).alive {
                life::lose_life(g, q, n, Some(p), crate::sym::intern(kind), None)?;
            }
            return life::check_state(g);
        }
        Target::Perm(m) => m,
    };
    if !g.perm(m).on_bf {
        return Ok(());
    }
    let name = g.perm(m).name;
    if g.is_creature(m) && crate::engine::values::etgh(g, m) <= n {
        removal::apply_removal(g, Some(p), m, &format!("dmg{n}"), None)?;
    } else if g.is_creature(m) {
        crate::glog!(g, "    {} deals {} damage to {} (it survives)", source, n, name);
    }
    let x = g.perm(m);
    let pw = x.cd.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::PLANESWALKER));
    if x.on_bf && pw && x.loyalty.is_some() {
        let l = x.loyalty.unwrap() - n;
        g.perm_mut(m).loyalty = Some(l);
        if l <= 0 {
            zones::leave(g, m)?;
            zones::to_zone_card(g, m, zones::Zone::Gy);
        }
    }
    life::check_state(g)
}

/// cards.prismari_charm: choose one: surveil 2, then draw a card; 1 damage to each of one or two targets; return
/// target nonland permanent to its owner's hand
pub fn prismari_charm(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let modes: Vec<String> = [
        "surveil 2, then draw a card",
        "1 damage to each of one or two targets",
        "return target nonland permanent to its owner's hand",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let k = choices::choose_one(g, p, "choose", "Prismari Charm: choose one", &modes)?;
    if k == 0 {
        choices::scry(g, p, 2, true)?;
        crate::engine::zones::draw(g, p, 1, false)?;
        return Ok(());
    }
    if k == 1 {
        let mut tg = choices::damage_targets(g, p, crate::cards::Colors::from_letters("UR"));
        if tg.is_empty() {
            return Ok(());
        }
        let labels: Vec<String> = tg.iter().map(|&x| legal::describe_target(g, p, x)).collect();
        let a = choices::choose_one(g, p, "target", "Prismari Charm: 1 damage to which target?", &labels)?;
        let first = tg.remove(a);
        let b = if tg.is_empty() {
            None
        } else {
            let labels: Vec<String> = tg.iter().map(|&x| legal::describe_target(g, p, x)).collect();
            choices::choose(g, p, "target", "Prismari Charm: a second target?", &labels, Some("no second target"), None)?
        };
        let mut hit = vec![first];
        if let Some(b) = b {
            hit.push(tg[b]);
        }
        for x in hit {
            damage(g, p, x, 1, "Prismari Charm", "burn")?;
        }
        return Ok(());
    }
    let tg: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| !g.perm(m).phased && !(g.perm(m).owner != p && crate::engine::values::untargetable(g, m)))
        .collect();
    if tg.is_empty() {
        return Ok(());
    }
    let Some(t) = pick_perm(g, p, "target", "Prismari Charm: return which permanent?", &tg, None)? else {
        return Ok(());
    };
    crate::engine::removal::apply_removal(g, Some(p), t, "bounce", Some(c))
}

/// What card c needs before the person can cast it that `legal` doesn't know (Python's `cards.needs`): the reason
/// it can't be cast, or None. PORT(practice-cards): stub, never blocks.
pub fn needs(_g: &Game, _p: PlayerId, _c: CardId) -> Option<String> {
    None
}

/// Does p's Flashback (the card) give graveyard card c flashback (Python's `cards.granted_flashback`).
/// PORT(practice-cards): stub.
pub fn granted_flashback(_g: &Game, _p: PlayerId, _c: CardId) -> bool {
    false
}

// ---- stand-ins for Python's per-card tables (play/cards.py): PORT(practice-cards) replaces each body ----
use crate::flow::Res;
use crate::ids::PermId;

pub type PermFn = super::abilities::PermFn;

/// `cards.abilities(g, p, m)`: the abilities card code offers for permanent m (None: the card has no card code of
/// its own; Some: by the rules, replacing the AI's hook options). PORT(practice-cards): stub.
pub fn abilities(_g: &Game, _p: PlayerId, _m: PermId) -> Option<Vec<(String, PermFn)>> {
    None
}

/// `cards.copy_card(c)`: Return the Favor, Dualcaster Mage (cast to copy a spell on the stack). PORT(practice-cards).
pub fn copy_card(_g: &Game, _c: CardId) -> bool {
    false
}

/// `cards.copy_in_response(g, p, c, spell, caster)`: None or why not. PORT(practice-cards): stub.
pub fn copy_in_response(
    _g: &mut Game,
    _p: PlayerId,
    _c: CardId,
    _spell: Option<CardId>,
    _caster: PlayerId,
) -> Res<Option<String>> {
    Ok(None)
}

/// `c.name in cards.HAND`: a hand card whose casting card code handles. PORT(practice-cards): stub.
pub fn hand_override(_g: &Game, _c: CardId) -> bool {
    false
}

/// `cards.cast_from_hand(g, p, c)`. PORT(practice-cards): stub.
pub fn cast_from_hand(_g: &mut Game, _p: PlayerId, _c: CardId) -> Res<Option<String>> {
    Ok(None)
}

/// `c.name in cards.GY`: a graveyard card whose casting card code handles. PORT(practice-cards): stub.
pub fn gy_override(_g: &Game, _c: CardId) -> bool {
    false
}

/// `cards.GY[c.name](g, p, c)`. PORT(practice-cards): stub.
pub fn cast_from_gy(_g: &mut Game, _p: PlayerId, _c: CardId) -> Res<Option<String>> {
    Ok(None)
}

/// The result of `cards.CAST_TARGET` (a tagless spell with a creature target, picked with `pick_creature`).
pub enum CastTarget {
    /// the card has no such target
    NotNeeded,
    /// the person cancelled
    Cancelled,
    Picked(PermId),
}

/// PORT(practice-cards): stub.
pub fn cast_target(_g: &Game, _p: PlayerId, _c: CardId) -> Res<CastTarget> {
    Ok(CastTarget::NotNeeded)
}

/// `cards.mastery_target(g, p, c)`: Mizzix's Mastery's one target; false = cancelled. PORT(practice-cards): stub
/// (a person cannot cast Mastery for one target until this is ported).
pub fn mastery_target(_g: &mut Game, _p: PlayerId, _c: CardId) -> Res<bool> {
    Ok(false)
}
