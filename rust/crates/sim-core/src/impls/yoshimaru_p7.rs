//! The Yoshimaru / Tymna deck, part 7a: the person paths and models that were missing.
//!
//! # The Wandering Emperor
//! {2}{W}{W}, flash, loyalty 3. "As long as The Wandering Emperor entered this turn, you may activate her loyalty
//! abilities any time you could cast an instant" (`human/cards.rs` `loyalty`: a person's timing check).
//! - +1: put a +1/+1 counter on up to one target creature (any player's); it gains first strike until end of turn
//!   (`emp_plus`, shared by the person's walker in `human/walkers.rs`).
//! - -1: a 2/2 white Samurai creature token with vigilance (`emp_samurai`).
//! - -2: exile target tapped creature (any player's, your own included); you gain 2 life (`human/walkers.rs`).
//!
//! # Takenuma, Abandoned Mire
//! Channel {3}{B}, discard it: mill three cards, then return a creature or planeswalker card from your graveyard to your
//! hand; {1} less for each legendary creature you control. The return is mandatory when there is such a card, and a
//! card just milled counts (`takenuma_effect`, shared). The AI's `hand_options` (`takenuma_options`) channels it in
//! its first main phase with six lands to play when a creature or planeswalker card worth 3 or more mana is already
//! in the graveyard (the mill is not counted on) and the library will not run dry; it takes the card the AI wants
//! most. A person's is `human/cards.rs` (`takenuma`): they pick the card when there is a choice.
//!
//! # Eiganjo, Seat of the Empire
//! Channel {2}{W}, discard it: 4 damage to target attacking or blocking creature; {1} less for each legendary creature
//! you control. The AI's channel is `t6_lands.rs` (`hand_defend` / `hand_blocks`); a person's is `human/cards.rs`
//! (`eiganjo`: any player's attacking or blocking creature, at instant speed, Pithing Needle respected). This file
//! lists the creatures it may aim at (`combatants`) and when the person's declare-blockers stop is worth it
//! (`eiganjo_ready`).

use super::t3::plan;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::mana::{can_pay, pay};
use crate::engine::repl;
use crate::engine::tutors::card_worth;
use crate::engine::values::{colors_of, protected_from};
use crate::engine::zones::{Tokens, make_tokens, max_by, mill};
use crate::flow::Res;
use crate::hooks::{Opt, Registry};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

const EMPEROR: &str = "The Wandering Emperor";

// ------------------------------------------------------------------ targets of an enters trigger (part 7b)
/// every permanent, lands included, that a trigger of o's from `src` may target and `keep` allows: on the battlefield,
/// not phased out, shroud, hexproof and protection from the source's colours (and hexproof from a colour) respected
pub fn targets_from(g: &Game, o: PlayerId, src: PermId, keep: impl Fn(&Game, PermId) -> bool) -> Vec<PermId> {
    let cols = colors_of(g, src);
    g.players
        .iter()
        .filter(|q| q.alive)
        // (a land that is also a creature token is listed in both: once)
        .flat_map(|q| q.perms.iter().chain(q.lands.iter().filter(|l| !q.perms.contains(l))).copied())
        .filter(|&m| {
            super::vondam_elspeth::may_target(g, o, m)
                && !protected_from(g, m, cols)
                && !super::tymna_cards::hexproof_from(g, m, cols)
                && keep(g, m)
        })
        .collect()
}

/// may p's The Wandering Emperor (`src`) target creature t (white: protection from white stops it)
pub fn emp_legal(g: &Game, p: PlayerId, src: PermId, t: PermId) -> bool {
    g.is_creature(t) && super::vondam_elspeth::may_target(g, p, t) && !protected_from(g, t, colors_of(g, src))
}

/// +1: a +1/+1 counter on t, put by p (Hardened Scales and Doubling Season look at the one who puts it), and first
/// strike until end of turn
pub fn emp_plus(g: &mut Game, p: PlayerId, t: PermId) -> Res {
    if !g.perm(t).on_bf {
        return Ok(());
    }
    repl::put_counters(g, p, t, 1);
    super::partials::eot_kw(g, t, "first strike");
    crate::glog!(g, "    {EMPEROR}: a +1/+1 counter on {} and first strike until end of turn", g.perm(t).name);
    Ok(())
}

/// -1: a 2/2 white Samurai creature token with vigilance
pub fn emp_samurai(g: &mut Game, p: PlayerId) -> Res<Vec<PermId>> {
    let spec = Tokens {
        tgh: Some(2),
        color: Some(crate::cards::Colors::from_letters("W")),
        types: vec!["samurai"],
        ..Tokens::new(1, 2)
    };
    let toks = make_tokens(g, p, spec)?;
    for &t in &toks {
        g.perm_mut(t).vig = true;
    }
    Ok(toks)
}

// ------------------------------------------------------------------ lands that are exiled and come back (part 7b)
// A land is listed in `Player::lands`, not among the permanents, and `zones::enter` puts a card into `perms`: so a land
// that is exiled and returns (Flickerwisp, Felidar Guardian) goes out by `remove_land` and comes in by `land_enters`
// (`t2::blink_inner` does it for a blink).

/// is m listed among its controller's lands (a land creature, Dryad Arbor, is: it is not among the permanents)
pub fn in_lands(g: &Game, m: PermId) -> bool {
    g.player(g.perm(m).owner).lands.contains(&m)
}

/// m leaves the battlefield without a zone: a land by `remove_land`, anything else by `zones::leave`
pub fn leave_battlefield(g: &mut Game, m: PermId) -> Res {
    if in_lands(g, m) {
        let ctrl = g.perm(m).owner;
        crate::engine::turn::remove_land(g, ctrl, m, None);
        Ok(())
    } else {
        crate::engine::zones::leave(g, m)
    }
}

/// land m leaves the battlefield for its owner's exile
pub fn exile_land(g: &mut Game, m: PermId) -> Res {
    let ctrl = g.perm(m).owner;
    crate::engine::turn::remove_land(g, ctrl, m, Some(crate::engine::zones::Zone::Exile));
    crate::engine::zones::exiled_from_bf(g, m, None)
}

/// o's spell or ability (from a card `source`) exiles m, a creature or other permanent that may be a land too: a land
/// goes through `exile_land` (ward paid for an opponent's), anything else through the removal
pub fn exile_target(g: &mut Game, o: PlayerId, m: PermId, source: Option<CardId>) -> Res {
    if in_lands(g, m) {
        if g.perm(m).owner != o && !crate::engine::ward::pay_ward(g, o, m)? {
            return Ok(());
        }
        return exile_land(g, m);
    }
    crate::engine::removal::apply_removal(g, Some(o), m, "exile", source)
}

/// land card cd enters the battlefield under p's control without being played (no land drop): untapped or tapped as
/// the card says (a shock land: p may pay its life, a person is asked, the AI decides as when it plays one), its enters
/// ability and landfall as for any land that arrives
pub fn land_enters(g: &mut Game, p: PlayerId, cd: CardId) -> Res<PermId> {
    let tapped = match crate::engine::turn::shock_cost(g, p, cd) {
        Some(n) => {
            let pay = if crate::human::is_human(g, p) {
                g.player(p).life >= n
                    && crate::human::choices::yes_no(
                        g,
                        p,
                        &format!("{}: pay {n} life so it enters untapped?", g.db.get(cd).name),
                    )?
            } else {
                crate::engine::turn::pays_for_land(g, p, cd, n)
            };
            if pay {
                crate::engine::life::lose_life(g, p, n, Some(p), "other", None)?;
            }
            !pay
        }
        None => crate::engine::turn::land_enters_tapped(g, p, cd),
    };
    let l = g.add_land(p, cd, tapped);
    crate::cardcode::land_etb(g, p, l)?;
    crate::engine::turn::land_arrives(g, p, l, cd)?;
    Ok(l)
}

/// the beginning of the end step: the cards waiting in `g.eot_returns` (exiled until then) come back under their
/// owners' control, a land card as a land and the rest through `galadriel::eot_returns`. What is put on the list as
/// they arrive (a land's landfall may exile something until the next end step) waits for the next end step.
pub fn end_step_returns(g: &mut Game) -> Res {
    if g.eot_returns.is_empty() {
        return Ok(());
    }
    let (lands, rest): (Vec<_>, Vec<_>) =
        std::mem::take(&mut g.eot_returns).into_iter().partition(|&(_, c)| g.db.get(c).land);
    for (p, cd) in lands {
        if g.player(p).alive && g.player_mut(p).exile.take(cd) {
            land_enters(g, p, cd)?;
            crate::glog!(g, "    {} returns to the battlefield", g.db.get(cd).name);
        }
    }
    let later = std::mem::replace(&mut g.eot_returns, rest);
    crate::impls::galadriel::eot_returns(g)?;
    g.eot_returns.extend(later);
    Ok(())
}

// ------------------------------------------------------------------ Eiganjo, Seat of the Empire
/// the attacking and blocking creatures an ability of p's (from a source of colours `source`) may target: on the
/// battlefield, hexproof and shroud respected, protection from the source's colours (from colorless: a colorless
/// source) too
pub fn combatants(g: &Game, p: PlayerId, source: Colors) -> Vec<PermId> {
    let mut fighting = g.in_combat.clone();
    for &x in &g.blocking {
        if !fighting.contains(&x) {
            fighting.push(x);
        }
    }
    fighting
        .into_iter()
        .filter(|&x| {
            g.perm(x).on_bf
                && g.player(g.perm(x).owner).perms.contains(&x)
                && g.is_creature(x)
                && super::vondam_elspeth::may_target(g, p, x)
                && !protected_from(g, x, source)
        })
        .collect()
}

/// card c in q's hand is Eiganjo and q could channel it now (`have` is the mana q has): a reason to stop at the declare
/// blockers step, as an instant would be
pub fn eiganjo_ready(g: &Game, q: PlayerId, c: CardId, have: u32) -> bool {
    let name = &*g.db.get(c).name;
    name == "Eiganjo, Seat of the Empire"
        && !super::t6_stax::needle_stops(g, name)
        && have > super::t6_lands::eiganjo_cost(g, q)
        && !combatants(g, q, Colors::NONE).is_empty()
}

// ------------------------------------------------------------------ Takenuma, Abandoned Mire
const TAKENUMA: &str = "Takenuma, Abandoned Mire";

/// legendary creatures p controls (phased-out ones do not count)
fn legendary_creatures(g: &Game, p: PlayerId) -> u32 {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| g.is_creature(m) && !g.perm(m).phased && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Leg)))
        .count() as u32
}

/// the generic part of Takenuma's channel for p: {3} less {1} for each legendary creature (the {B} is always paid)
pub fn takenuma_cost(g: &Game, p: PlayerId) -> u32 {
    3u32.saturating_sub(legendary_creatures(g, p))
}

/// the creature and planeswalker cards in p's graveyard
fn takenuma_cards(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            d.creature || d.types.has(Types::PLANESWALKER)
        })
        .collect()
}

/// the effect: mill three cards, then return a creature or planeswalker card from the graveyard to the hand (a card
/// just milled counts; with none, nothing is returned). The AI takes the one it wants most, a person picks (asked only
/// when there is more than one).
fn takenuma_effect(g: &mut Game, p: PlayerId) -> Res {
    mill(g, p, 3)?;
    let cs = takenuma_cards(g, p);
    let pick = match cs.len() {
        0 => None,
        1 => Some(cs[0]),
        _ if crate::human::is_human(g, p) => {
            let labels: Vec<String> = cs.iter().map(|&c| crate::human::choices::card_label(g, c)).collect();
            let prompt = "Takenuma: return which creature or planeswalker card to your hand?";
            crate::human::choices::choose(g, p, "choose", prompt, &labels, None, None)?.map(|k| cs[k])
        }
        _ => max_by(&cs, |c| card_worth(g, p, c, false)),
    };
    if let Some(c) = pick
        && let Some(i) = g.player(p).gy.iter().position(|&x| x == c)
    {
        g.player_mut(p).gy.remove(i);
        g.player_mut(p).hand.push(c);
        g.player_mut(p).seen.insert(c);
        crate::glog!(g, "    {} returns {} to hand (Takenuma)", g.player(p).name, g.db.get(c).name);
    }
    Ok(())
}

/// p channels Takenuma c, its cost paid: discarded as a cost, then the ability on the stack
pub fn takenuma_channel(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let pl = g.player_mut(p);
    if let Some(i) = pl.hand.iter().position(|&x| x == c) {
        pl.hand.remove(i);
    }
    pl.gy.push(c);
    crate::glog!(
        g,
        "  {} channels Takenuma: mill three, then return a creature or planeswalker card",
        g.player(p).name
    );
    let what = "mill three cards, then return a creature or planeswalker card";
    crate::engine::stack::ability_run_w(g, p, None, Some(c), what, None, None, 0.0, |g, _| takenuma_effect(g, p))?;
    Ok(())
}

/// the AI's channel: in its first main phase, with six lands to play (as for Sokenzan: a land is worth more early),
/// when a creature or planeswalker card worth 3+ mana (or that the AI wants) is already in the graveyard
fn takenuma_options(g: &mut Game, _c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let cost = takenuma_cost(g, p);
    let pl = g.player(p);
    let lands = pl.lands.len() + pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
    if post != Some(false)
        || lands < 6
        || pl.library.len() <= 3
        || super::t6_stax::needle_stops(g, TAKENUMA)
        || !can_pay(g, p, cost, "B", false)
    {
        return Ok(vec![]);
    }
    let best = takenuma_cards(g, p)
        .into_iter()
        .map(|c| card_worth(g, p, c, false).max(10.0 * g.db.get(c).cmc as f64))
        .fold(0.0, f64::max);
    if best < 30.0 {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.0 + (best / 60.0).min(1.5), "channel Takenuma".into(), takenuma_go, cost as i64)])
}

fn takenuma_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let cost = arg as u32;
    let Some(c) = g.db.id(TAKENUMA) else { return Ok(false) };
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, cost, "B", false) {
        return Ok(false);
    }
    if !pay(g, p, cost, "B", false)? {
        return Ok(false);
    }
    takenuma_channel(g, p, c)?;
    Ok(true)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, TAKENUMA)?.hand_options = Some(takenuma_options);
    Ok(())
}
