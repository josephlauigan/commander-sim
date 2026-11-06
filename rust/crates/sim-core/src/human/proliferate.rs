//! Practice mode (model-gaps-a4, item 6): a person chooses what they proliferate. Rule 701.27: "choose any number of
//! permanents and/or players that have a counter, then give each another counter of each kind that permanent or
//! player already has." The AI's proliferate (`impls::common::proliferate`) picks by itself and is unchanged.
//!
//! `common::proliferate` hands every proliferate a person controls to `once` (one proliferate action; Tekuthal's
//! "twice" and every source run it: Karn's Bastion, Vraska's 0, Cankerbloom, High Perfect Morcant, Marvin using
//! those, Atraxa and the rest, and the compiled "proliferate" effects). The person is shown each permanent and
//! player with a counter the engine models, the ones the AI would pick first (marked), and picks one at a time until
//! they are done, as for any "any number of" choice.
//!
//! The counters: +1/+1 and -1/-1 (a creature's `plus`), loyalty, the engine's other counters on a permanent (charge,
//! a land's kept counters, fate, oil, bloodline, influence, stun, time, defense, revival, hour, gold and quest
//! counters), lore counters on a Saga that has card code to run its chapters; poison, energy and experience counters on
//! a player, and the Incubator's and The Ozolith's counters (the engine keeps those on the player).

use super::{Target, choices, legal};
use crate::engine::repl::{self, Ctr, On, Path};
use crate::flow::Res;
use crate::ids::{PermId, PlayerId};
use crate::state::{DataKey, Game, Val};

/// the counters kept in an Int under a data key, with the name to show
const KEYS: &[(DataKey, &str)] = &[
    (DataKey::Charge, "charge"),
    (DataKey::Counters, "other"),
    (DataKey::Fate, "fate"),
    (DataKey::Oil, "oil"),
    (DataKey::Bloodline, "bloodline"),
    (DataKey::Influence, "influence"),
    (DataKey::Stun, "stun"),
    (DataKey::Vanish, "time"),
    (DataKey::Defense, "defense"),
    (DataKey::Revival, "revival"),
    (DataKey::T0Stella, "hour"),
    (DataKey::T45Dragon, "gold"),
    (DataKey::T12Isshin, "quest"),
    (DataKey::T12, "time"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Perm(PermId),
    Player(PlayerId),
    /// the Incubator token's +1/+1 counters the engine keeps on the player
    Incubator(PlayerId),
    /// The Ozolith's counters
    Ozolith(PlayerId),
}

fn is_vivid(g: &Game, m: PermId) -> bool {
    crate::impls::common::VIVID.contains(&&*g.db.get(g.perm(m).card()).name)
}

/// the Int counters under `k` on m (a Vivid land has two charge counters until its data says otherwise)
fn key_count(g: &Game, m: PermId, k: DataKey) -> i64 {
    match g.perm(m).data.get(k) {
        Some(Val::Int(n)) => *n,
        None if k == DataKey::Charge && g.perm(m).cd.is_some() && is_vivid(g, m) => 2,
        _ => 0,
    }
}

fn is_walker(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::PLANESWALKER))
}

/// Urza's Saga (a land): its lore counters, which the engine keeps on its controller
fn urza_lore(g: &Game, m: PermId) -> Option<u32> {
    g.player(g.perm(m).owner).sagas.iter().find(|x| x.0 == m).map(|x| x.1).filter(|&n| n > 0)
}

fn saga_hook(g: &Game, m: PermId) -> Option<(crate::hooks::SagaWantsFn, crate::hooks::SagaLoreFn)> {
    if g.perm(m).data.get(DataKey::Lore).is_none() {
        return None;
    }
    g.perm(m).cd.and_then(|c| g.registry.get(c)).and_then(|i| i.saga)
}

/// what a permanent's counters are, for the list
fn counter_text(g: &Game, m: PermId) -> Vec<String> {
    let x = g.perm(m);
    let mut out = vec![];
    if x.plus > 0 {
        out.push(format!("{} +1/+1", x.plus));
    } else if x.plus < 0 {
        out.push(format!("{} -1/-1", -x.plus));
    }
    if let Some(l) = x.loyalty
        && l > 0
        && is_walker(g, m)
    {
        out.push(format!("{l} loyalty"));
    }
    for &(k, name) in KEYS {
        let n = key_count(g, m, k);
        if n > 0 {
            out.push(format!("{n} {name}"));
        }
    }
    if saga_hook(g, m).is_some() {
        out.push(format!("{} lore", x.data.int(DataKey::Lore)));
    }
    if let Some(n) = urza_lore(g, m) {
        out.push(format!("{n} lore"));
    }
    out
}

/// every permanent and player a person may choose, each with the text to show and whether the AI would pick it
/// (your counters that help you, your opponents' -1/-1 counters and poison), the AI's picks first
pub fn candidates(g: &Game, p: PlayerId) -> Vec<(Item, String, bool)> {
    let mut out: Vec<(Item, String, bool)> = vec![];
    for q in g.players.iter().filter(|q| q.alive).map(|q| q.id) {
        for (&m, land) in
            g.player(q).perms.iter().map(|m| (m, false)).chain(g.player(q).lands.iter().map(|m| (m, true)))
        {
            if g.perm(m).phased || (!land && !g.perm(m).on_bf) {
                continue;
            }
            let bits = counter_text(g, m);
            if bits.is_empty() {
                continue;
            }
            let x = g.perm(m);
            let mine = x.owner == p;
            let useful = if mine {
                x.plus > 0
                    || (x.loyalty.is_some_and(|l| l != 0) && is_walker(g, m))
                    || key_count(g, m, DataKey::Charge) > 0
                    || (land && key_count(g, m, DataKey::Counters) > 0)
                    || saga_hook(g, m).is_some_and(|s| (s.0)(g, p, m))
            } else {
                x.plus < 0 && g.is_creature(m)
            };
            let name = legal::describe_target(g, p, Target::Perm(m));
            out.push((
                Item::Perm(m),
                format!("{name}{}: {}", if land { " [land]" } else { "" }, bits.join(", ")),
                useful,
            ));
        }
        let pl = g.player(q);
        let mut bits = vec![];
        if pl.poison > 0 {
            bits.push(format!("{} poison", pl.poison));
        }
        if pl.energy > 0 {
            bits.push(format!("{} energy", pl.energy));
        }
        if pl.experience > 0 {
            bits.push(format!("{} experience", pl.experience));
        }
        if !bits.is_empty() {
            let you = if q == p { " (you)" } else { "" };
            out.push((
                Item::Player(q),
                format!("{} (player){you}: {}", pl.name, bits.join(", ")),
                q != p && pl.poison > 0,
            ));
        }
        if pl.incubator > 0 {
            out.push((Item::Incubator(q), format!("{}'s Incubator token: {} +1/+1", pl.name, pl.incubator), q == p));
        }
        if pl.ozolith_counters > 0 {
            out.push((
                Item::Ozolith(q),
                format!("{}'s The Ozolith: {} counters", pl.name, pl.ozolith_counters),
                q == p,
            ));
        }
    }
    out.sort_by_key(|x| !x.2); // the AI's picks first (stable)
    out
}

/// one more of each kind of counter on `m`
fn grow_perm(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if !g.perm(m).on_bf || g.perm(m).phased {
        return Ok(());
    }
    let plus = g.perm(m).plus;
    if plus != 0 {
        repl::put_counters(g, p, m, plus.signum()); // the counters step: Doubling Season, Hardened Scales, Vorinclex, Melira
    }
    if is_walker(g, m)
        && let Some(l) = g.perm(m).loyalty
        && l > 0
    {
        let k = repl::counters(g, p, On::Perm(m), Ctr::Loyalty, 1, Path::ALL);
        g.perm_mut(m).loyalty = Some(l + k);
    }
    for &(k, _) in KEYS {
        let n = key_count(g, m, k);
        if n > 0 {
            let add = repl::counters(g, p, On::Perm(m), Ctr::Other, 1, Path::ALL) as i64;
            g.perm_mut(m).data.set(k, Val::Int(n + add));
        }
    }
    if let Some((_, add_lore)) = saga_hook(g, m) {
        add_lore(g, p, m)?; // a lore counter: that chapter triggers
    }
    if urza_lore(g, m).is_some() {
        crate::impls::common::saga_lore_one(g, g.perm(m).owner, m)?; // chapter II or III
    }
    Ok(())
}

/// one proliferate action by the person p: asks what to proliferate and does it
pub fn once(g: &mut Game, p: PlayerId) -> Res {
    let mut left = candidates(g, p);
    let mut chosen: Vec<Item> = vec![];
    while !left.is_empty() {
        let labels: Vec<String> = left
            .iter()
            .map(|(_, t, useful)| if *useful { format!("{t}  (what the AI would pick)") } else { t.clone() })
            .collect();
        let pr = format!(
            "Proliferate: choose any number of permanents and players with counters ({} chosen); each gets another counter of each kind it has",
            chosen.len()
        );
        let done = if chosen.is_empty() { "proliferate nothing" } else { "done" };
        let Some(k) = choices::choose(g, p, "choose", &pr, &labels, Some(done), None)? else { break };
        chosen.push(left.remove(k).0);
    }
    if chosen.is_empty() {
        return Ok(());
    }
    crate::glog!(g, "  {} proliferates ({} chosen)", g.player(p).name, chosen.len());
    for it in chosen {
        match it {
            Item::Perm(m) => grow_perm(g, p, m)?,
            Item::Player(q) => {
                if g.player(q).poison > 0 {
                    repl::give_poison(g, p, q, 1, Path::POISON); // Melira, Vorinclex
                }
                for energy in [true, false] {
                    let n = if energy { g.player(q).energy } else { g.player(q).experience };
                    if n > 0 {
                        let k = repl::counters(g, p, On::Player(q), Ctr::Other, 1, Path::VORINCLEX);
                        let pl = g.player_mut(q);
                        if energy {
                            pl.energy += k;
                        } else {
                            pl.experience += k;
                        }
                    }
                }
            }
            Item::Incubator(q) => {
                let k = repl::counters(g, p, On::Kept(q), Ctr::Other, 1, Path::ALL); // Solemnity, Doubling Season
                if g.player(q).incubator > 0 {
                    g.player_mut(q).incubator += k;
                }
            }
            Item::Ozolith(q) => {
                let k = repl::counters(g, p, On::Kept(q), Ctr::Other, 1, Path::ALL);
                if g.player(q).ozolith_counters > 0 {
                    g.player_mut(q).ozolith_counters += k;
                }
            }
        }
        if g.over || !g.player(p).alive {
            return Ok(());
        }
    }
    {
        crate::engine::life::check_state(g)?; // a creature with 0 toughness, ten poison counters
    }
    Ok(())
}
