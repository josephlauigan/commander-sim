//! Practice mode: a person's Mystic Confluence ({3}{U}{U} instant: "Choose three. You may choose the same mode more
//! than once. • Counter target spell unless its controller pays {3}. • Return target creature to its owner's hand.
//! • Draw a card."). The AI plays it as a soft counter (and draws three at an end step) with its own code; a person
//! picks the three modes as it is cast, in response to a spell (`human::cast_counterspell`) or with nothing to
//! counter (`cast_plain`). The modes ride on `Game::confluence` until it resolves (`stack::resolve_counter` for a
//! counter, `resolve_rest` otherwise).

use super::{Target, choices, legal, mana};
use crate::engine::{self, zones};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};

pub const NAME: &str = "Mystic Confluence";

/// the three modes: (counter modes, creatures to return, cards to draw), or None if the person cancels
pub fn modes(g: &Game, p: PlayerId, counter: bool) -> Res<Option<(u32, Vec<PermId>, u32)>> {
    let (mut n, mut bounce, mut draws) = (0u32, vec![], 0u32);
    for i in 1..=3 {
        let cre: Vec<PermId> = g
            .players
            .iter()
            .filter(|q| q.alive)
            .flat_map(|q| q.perms.iter().copied())
            .filter(|&m| {
                let x = g.perm(m);
                x.on_bf && !x.phased && g.is_creature(m) && (x.owner == p || !engine::values::untargetable(g, m))
            })
            .collect();
        let mut kinds: Vec<&str> = vec![];
        let mut labels: Vec<String> = vec![];
        if counter {
            kinds.push("counter");
            labels.push("counter target spell unless its controller pays {3}".into());
        }
        if !cre.is_empty() {
            kinds.push("bounce");
            labels.push("return target creature to its owner's hand".into());
        }
        kinds.push("draw");
        labels.push("draw a card".into());
        let prompt = format!("{NAME}: choose mode {i} of 3 (the same mode may be chosen again)");
        let Some(k) = choices::choose(g, p, "choose", &prompt, &labels, Some("cancel"), None)? else { return Ok(None) };
        match kinds[k] {
            "counter" => n += 1,
            "bounce" => {
                let t: Vec<String> = cre.iter().map(|&m| legal::describe_target(g, p, Target::Perm(m))).collect();
                let pr = format!("{NAME}: return which creature to its owner's hand?");
                let Some(j) = choices::choose(g, p, "target", &pr, &t, Some("cancel"), None)? else { return Ok(None) };
                bounce.push(cre[j]);
            }
            _ => draws += 1,
        }
    }
    Ok(Some((n, bounce, draws)))
}

/// the modes other than the counter, as it resolves: return the creatures still there, then draw
pub fn resolve_rest(g: &mut Game, p: PlayerId, bounce: &[PermId], draws: u32) -> Res {
    for &m in bounce {
        if g.perm(m).on_bf && !g.perm(m).phased {
            crate::glog!(g, "    {NAME} returns {} to its owner's hand", g.perm(m).name);
            zones::bounce(g, m)?;
        }
    }
    if draws > 0 {
        zones::draw(g, p, draws, false)?;
    }
    Ok(())
}

/// a person casts Mystic Confluence with no counter mode (nothing to counter, or they chose not to): the modes, its
/// mana from the pool, then it goes on the stack like any spell. None, or why it can't be cast
pub fn cast_plain(g: &mut Game, p: PlayerId, c: CardId) -> Res<Option<String>> {
    let (gn, pips) = engine::mana::cost_of(g, p, c);
    if let Some(why) = mana::cost_problem(g, p, gn, &pips) {
        return Ok(Some(format!("Can't cast {NAME}. {why}")));
    }
    let Some(m) = modes(g, p, false)? else { return Ok(None) };
    cast_with(g, p, c, m)
}

/// cast it with the modes chosen (no counter mode): pay from the pool, then the stack
pub fn cast_with(g: &mut Game, p: PlayerId, c: CardId, m: (u32, Vec<PermId>, u32)) -> Res<Option<String>> {
    let (gn, pips) = engine::mana::cost_of(g, p, c);
    if let Some(why) = mana::pay_from_pool(g, p, gn, &pips) {
        return Ok(Some(format!("Can't cast {NAME}. {why}")));
    }
    g.confluence = Some(m);
    let r = engine::cast::cast_card(g, p, c, "hand", Ctx::default());
    g.confluence = None; // (resolved, or countered)
    r?;
    engine::life::check_state(g)?;
    Ok(None)
}
