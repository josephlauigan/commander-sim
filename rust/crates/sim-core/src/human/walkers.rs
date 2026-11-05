//! Practice mode: the loyalty abilities of eight planeswalkers whose choices the AI's code used to make for a person
//! (walker-fixes task). A person's walker comes from a `cards::walker(...)` entry, which replaces the walker's whole
//! generic list, so each entry here lists every loyalty ability the card has (ultimates included). Each asks what the
//! Oracle text lets the person choose BEFORE the loyalty is paid (a cancelled choice costs nothing), then carries the
//! effect out after the ability resolves, reusing the AI's resolution code where it is right.
//!
//! - **Karn Liberated**: +4 the player (any, you included) who exiles a card from their hand, -3 the permanent to exile,
//!   -14 restart (the AI's, read as a win).
//! - **Ugin, the Ineffable**: +1 (the AI's; the person is shown the card exiled face down), -3 the coloured permanent.
//! - **Karn, Scion of Urza**: +1 (the AI's: an opponent, an AI seat, chooses; the log names both cards), -1 which card with
//!   a silver counter, -2 the Construct.
//! - **Karn, the Great Creator**: +1 the noncreature artifact that becomes a creature until your next turn (or none),
//!   -2 the artifact card in exile to take.
//! - **Teferi, Time Raveler**: +1 (no effect here), -3 the artifact, creature or enchantment to bounce (or none), then draw.
//! - **Wrenn and Six**: +1 the land card to take back (or none), -1 the target of the damage, -7 the emblem.
//! - **Venser, the Sojourner**: +2 the permanent you own to exile until the next end step, -1 (the AI's creatures-can't-be-
//!   blocked flag, which a person never got), -8 the emblem.
//!
//! Never reached for an AI seat: `cards::abilities` is only asked for a person's permanent.

use super::abilities::PermAbility;
use super::cards::{Ready, go, pick, pick_perm, pname, walker};
use super::{Target, choices, legal};
use crate::cards::Types;
use crate::engine::chars::card_types;
use crate::engine::values::{colors_of, untargetable};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};

/// the abilities of p's walker m named `nm` that this file provides, or None
pub fn abilities(nm: &str) -> Option<Vec<PermAbility>> {
    Some(match nm {
        "Karn Liberated" => walker(&[
            (4, "target player exiles a card from their hand", karn_lib_plus),
            (-3, "exile target permanent", karn_lib_minus3),
            (
                -14,
                "restart the game, leaving in exile the cards exiled with Karn; put them onto the battlefield",
                karn_lib_ult,
            ),
        ]),
        "Ugin, the Ineffable" => walker(&[
            (1, "exile the top card of your library face down and look at it; create a 2/2 Spirit", ugin_plus),
            (-3, "destroy target permanent that's one or more colors", ugin_minus3),
        ]),
        "Karn, Scion of Urza" => walker(&[
            (
                1,
                "reveal the top two cards; an opponent chooses one for your hand, the other is exiled with a silver counter",
                karn_su_plus,
            ),
            (-1, "put a card you own with a silver counter on it from exile into your hand", karn_su_minus1),
            (
                -2,
                "create a 0/0 Construct artifact creature token (+1/+1 for each artifact you control)",
                karn_su_minus2,
            ),
        ]),
        "Karn, the Great Creator" => walker(&[
            (
                1,
                "until your next turn, up to one target noncreature artifact becomes an artifact creature",
                karn_gc_plus,
            ),
            (-2, "choose a face-up artifact card you own in exile; put it into your hand", karn_gc_minus2),
        ]),
        "Teferi, Time Raveler" => walker(&[
            (1, "until your next turn, you may cast sorcery spells as though they had flash (not modeled)", ttr_plus),
            (
                -3,
                "return up to one target artifact, creature, or enchantment to its owner's hand; draw a card",
                ttr_minus3,
            ),
        ]),
        "Wrenn and Six" => walker(&[
            (1, "return up to one target land card from your graveyard to your hand", w6_plus),
            (-1, "1 damage to any target", w6_minus1),
            (-7, "emblem: instant and sorcery cards in your graveyard have retrace", w6_minus7),
        ]),
        "Venser, the Sojourner" => walker(&[
            (2, "exile target permanent you own; it returns at the beginning of the next end step", venser_plus2),
            (-1, "creatures can't be blocked this turn", venser_minus1),
            (-8, "emblem: whenever you cast a spell, exile target permanent", venser_minus8),
        ]),
        _ => return None,
    })
}

/// a plain message to the person at p (no-op for an AI seat)
fn tell_info(g: &Game, p: PlayerId, text: &str) {
    if let Some(h) = super::human_of(g, p) {
        h.tell(p, crate::sym::intern("info"), text);
    }
}

/// every permanent p may target (lands included): on the table, not phased, an opponent's hexproof and shroud respected,
/// and `keep`
fn permanents(g: &Game, p: PlayerId, keep: impl Fn(&Game, PermId) -> bool) -> Vec<PermId> {
    let mut out = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in q.perms.iter().chain(q.lands.iter()) {
            let x = g.perm(m);
            if !x.on_bf || x.phased || (q.id != p && untargetable(g, m)) || !keep(g, m) {
                continue;
            }
            out.push(m);
        }
    }
    out
}

fn is_artifact(g: &Game, m: PermId) -> bool {
    card_types(g, m).has(Types::ARTIFACT)
}

/// the permanent the person picks from `cands`; None: they cancelled (`optional`: "no target" is an answer too)
fn pick_one(g: &Game, p: PlayerId, prompt: &str, cands: &[PermId], optional: bool) -> Res<Option<PermId>> {
    pick_perm(g, p, "target", prompt, cands, Some(if optional { "no target" } else { "cancel" }))
}

fn on_table(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && g.players.iter().any(|q| q.perms.contains(&m) || q.lands.contains(&m))
}

// ------------------------------------------------------------------ Karn Liberated
fn karn_lib_plus(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let ps: Vec<PlayerId> = g.players.iter().map(|q| q.id).filter(|&q| legal::player_targetable(g, q)).collect();
    let labels: Vec<String> = ps
        .iter()
        .map(|&q| {
            format!(
                "{}{} ({} card(s) in hand)",
                pname(g, q),
                if q == p { " (you)" } else { "" },
                crate::engine::values::hand_count(g, q)
            )
        })
        .collect();
    let Some(k) = choices::choose(
        g,
        p,
        "target",
        "Karn Liberated +4: which player exiles a card from their hand?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(Ready::Cancel);
    };
    let q = ps[k];
    go(move |g| {
        let hand = g.player(q).hand.clone();
        if hand.is_empty() {
            crate::glog!(g, "    {} has no card in hand to exile", pname(g, q));
            return Ok(());
        }
        // the target player chooses: a person picks, an AI seat gives up the card it wants least
        let c = if super::is_human(g, q) {
            choices::pick_cards(g, q, &hand, 1, "Karn Liberated: exile which card from your hand?", "choose")?[0]
        } else {
            crate::engine::zones::min_by(&hand, |c| crate::engine::tutors::card_worth(g, q, c, false))
                .unwrap_or(hand[0])
        };
        let pl = g.player_mut(q);
        let Some(i) = pl.hand.iter().position(|&x| x == c) else { return Ok(()) };
        pl.hand.remove(i);
        if pl.is_commander(c) {
            pl.set_cmd_zone(c, true); // a commander may go to the command zone instead
        } else {
            pl.exile.push(c);
        }
        crate::glog!(g, "    Karn Liberated: {} exiles {} from their hand", pname(g, q), g.db.get(c).name);
        Ok(())
    })
}

fn karn_lib_minus3(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let cands = permanents(g, p, |_, _| true);
    let Some(t) = pick_one(g, p, "Karn Liberated -3: exile which permanent?", &cands, false)? else {
        return Ok(Ready::Cancel);
    };
    go(move |g| {
        if on_table(g, t) {
            crate::engine::removal::apply_removal(g, Some(p), t, "exile", None)?;
        }
        Ok(())
    })
}

fn karn_lib_ult(_g: &mut Game, p: PlayerId, m: PermId) -> Res<Ready> {
    go(move |g| (crate::impls::rules::KARN_ULT.eff)(g, p, m))
}

// ------------------------------------------------------------------ Ugin, the Ineffable
fn ugin_plus(_g: &mut Game, p: PlayerId, m: PermId) -> Res<Ready> {
    go(move |g| {
        if let Some(&c) = g.player(p).library.last() {
            tell_info(g, p, &format!("Ugin, the Ineffable: you look at {}, exiled face down.", g.db.get(c).name));
        }
        crate::impls::karn_cards::ugin_plus(g, p, m)
    })
}

fn ugin_minus3(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let cands = permanents(g, p, |g, m| !colors_of(g, m).is_empty());
    if cands.is_empty() {
        return Ok(Ready::Why("There is no permanent of one or more colors to target.".into()));
    }
    let Some(t) = pick_one(g, p, "Ugin, the Ineffable -3: destroy which permanent?", &cands, false)? else {
        return Ok(Ready::Cancel);
    };
    go(move |g| {
        if on_table(g, t) {
            crate::engine::removal::apply_removal(g, Some(p), t, "destroy", None)?;
        }
        Ok(())
    })
}

// ------------------------------------------------------------------ Karn, Scion of Urza
/// +1: the top two cards are revealed and an opponent (an AI seat: it gives you the card you want less) chooses which one
/// you get; the person is told both cards before it is done, and the log names the one each way
fn karn_su_plus(_g: &mut Game, p: PlayerId, m: PermId) -> Res<Ready> {
    go(move |g| {
        let names: Vec<String> =
            g.player(p).library.iter().rev().take(2).map(|&c| g.db.get(c).name.to_string()).collect();
        if !names.is_empty() {
            tell_info(g, p, &format!("Karn, Scion of Urza: you reveal {}.", names.join(" and ")));
        }
        crate::impls::karn_engines::karn_plus(g, p, m)
    })
}

fn karn_su_minus1(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let cs = g.player(p).exile.silver();
    if cs.is_empty() {
        return Ok(Ready::Why("You own no card with a silver counter on it in exile.".into()));
    }
    let labels: Vec<String> = cs.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) =
        pick(g, p, "target", "Karn, Scion of Urza -1: put which card with a silver counter into your hand?", &labels)?
    else {
        return Ok(Ready::Cancel);
    };
    let c = cs[k];
    go(move |g| {
        // the object with the counter (not an older copy of the same card)
        let at = g.player(p).exile.entries().position(|(x, o)| x == c && o.silver);
        if let Some(i) = at {
            let pl = g.player_mut(p);
            pl.exile.remove(i);
            pl.hand.push(c);
            pl.seen.insert(c);
            crate::glog!(g, "    Karn returns {} from exile", g.db.get(c).name);
        }
        Ok(())
    })
}

fn karn_su_minus2(_g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    go(move |g| crate::impls::karn_engines::make_construct(g, p))
}

// ------------------------------------------------------------------ Karn, the Great Creator
fn karn_gc_plus(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let cands = permanents(g, p, |g, m| is_artifact(g, m) && !g.is_creature(m));
    let t = if cands.is_empty() {
        None
    } else {
        pick_one(
            g,
            p,
            "Karn, the Great Creator +1: which noncreature artifact becomes a creature until your next turn?",
            &cands,
            true,
        )?
    };
    go(move |g| {
        let Some(t) = t.filter(|&t| on_table(g, t) && !g.is_creature(t)) else { return Ok(()) };
        let mv = g.perm(t).cd.map_or(0, |c| g.db.get(c).cmc as i32);
        let x = g.perm_mut(t);
        let (pw, tg) = (x.pow, x.tgh);
        x.data.set(DataKey::Anim, Val::Bool(true));
        x.pow = mv;
        x.tgh = mv;
        g.karn_anim.push((p, t, pw, tg));
        g.bf_ver += 1;
        crate::glog!(
            g,
            "    {} becomes a {mv}/{mv} artifact creature until {}'s next turn",
            g.perm(t).name,
            pname(g, p)
        );
        crate::engine::life::check_state(g)
    })
}

/// the start of p's turn: what p's Karn, the Great Creator animated is an ordinary artifact again
pub fn karn_revert(g: &mut Game, p: PlayerId) {
    if g.karn_anim.is_empty() {
        return;
    }
    let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut g.karn_anim).into_iter().partition(|e| e.0 == p);
    g.karn_anim = rest;
    for (_, m, pw, tg) in mine {
        if g.perm(m).on_bf && g.perm(m).data.truthy(DataKey::Anim) {
            let x = g.perm_mut(m);
            x.data.set(DataKey::Anim, Val::Bool(false));
            x.pow = pw;
            x.tgh = tg;
        }
    }
    g.bf_ver += 1;
}

fn karn_gc_minus2(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    // "from outside the game" (the sideboard) isn't modeled: the face-up artifact cards you own in exile
    let cs: Vec<CardId> =
        g.player(p).exile.iter().copied().filter(|&c| g.db.get(c).types.has(Types::ARTIFACT)).collect();
    if cs.is_empty() {
        return Ok(Ready::Why("You own no face-up artifact card in exile (a sideboard isn't modeled).".into()));
    }
    let labels: Vec<String> = cs.iter().map(|&c| choices::card_label(g, c)).collect();
    let Some(k) = choices::choose(
        g,
        p,
        "target",
        "Karn, the Great Creator -2: put which artifact card from exile into your hand?",
        &labels,
        Some("none"),
        None,
    )?
    else {
        return go(|_| Ok(())); // "you may": the loyalty is paid, nothing is taken
    };
    let c = cs[k];
    go(move |g| {
        let pl = g.player_mut(p);
        if pl.exile.take(c) {
            pl.hand.push(c);
            pl.seen.insert(c);
            crate::glog!(g, "    Karn takes {} from exile", g.db.get(c).name);
        }
        Ok(())
    })
}

// ------------------------------------------------------------------ Teferi, Time Raveler
fn ttr_plus(_g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    go(move |g| {
        super::auto_note(
            g,
            p,
            "Teferi, Time Raveler +1: casting sorceries at instant speed isn't modeled; the loyalty is all it does.",
        );
        Ok(())
    })
}

fn ttr_minus3(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let cands = permanents(g, p, |g, m| {
        let t = card_types(g, m);
        g.is_creature(m) || t.has(Types::ARTIFACT) || t.has(Types::ENCHANTMENT)
    });
    let t = if cands.is_empty() {
        None
    } else {
        pick_one(
            g,
            p,
            "Teferi, Time Raveler -3: return which artifact, creature or enchantment to its owner's hand?",
            &cands,
            true,
        )?
    };
    go(move |g| {
        if let Some(t) = t.filter(|&t| on_table(g, t)) {
            crate::engine::removal::apply_removal(g, Some(p), t, "bounce", None)?;
        }
        crate::engine::zones::draw(g, p, 1, false)
    })
}

// ------------------------------------------------------------------ Wrenn and Six
fn w6_plus(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let ls: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).land).collect();
    let c = if ls.is_empty() {
        None
    } else {
        let labels: Vec<String> = ls.iter().map(|&c| choices::card_label(g, c)).collect();
        choices::choose(
            g,
            p,
            "target",
            "Wrenn and Six +1: return which land card from your graveyard to your hand?",
            &labels,
            Some("no target"),
            None,
        )?
        .map(|k| ls[k])
    };
    go(move |g| {
        if let Some(c) = c {
            let pl = g.player_mut(p);
            if let Some(i) = pl.gy.iter().position(|&x| x == c) {
                pl.gy.remove(i);
                pl.hand.push(c);
                crate::glog!(g, "    Wrenn and Six returns {} to hand", g.db.get(c).name);
            }
        }
        Ok(())
    })
}

fn w6_minus1(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let tg = choices::damage_targets(g, p, Default::default());
    if tg.is_empty() {
        return Ok(Ready::Why("There is no target for the damage.".into()));
    }
    let labels: Vec<String> = tg.iter().map(|&x| legal::describe_target(g, p, x)).collect();
    let Some(k) = pick(g, p, "target", "Wrenn and Six -1: 1 damage to which target?", &labels)? else {
        return Ok(Ready::Cancel);
    };
    let t = tg[k];
    go(move |g| {
        if let Target::Perm(m) = t
            && !on_table(g, m)
        {
            return Ok(()); // gone before the ability resolved
        }
        choices::apply_damage(g, p, t, 1, "Wrenn and Six", "burn")
    })
}

fn w6_minus7(_g: &mut Game, p: PlayerId, m: PermId) -> Res<Ready> {
    go(move |g| crate::impls::legends::w6_ult(g, p, m))
}

// ------------------------------------------------------------------ Venser, the Sojourner
fn venser_plus2(g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    let cands = permanents(g, p, |g, m| g.perm(m).owner == p);
    let Some(t) = pick_one(g, p, "Venser, the Sojourner +2: exile which permanent you own?", &cands, false)? else {
        return Ok(Ready::Cancel);
    };
    go(move |g| {
        if !on_table(g, t) {
            return Ok(());
        }
        let (cd, token, name) = (g.perm(t).cd, g.perm(t).token, g.perm(t).name);
        crate::engine::zones::exile_perm(g, t)?;
        match cd {
            Some(cd) if !token => {
                // back under its owner's control at the beginning of the next end step
                g.eot_returns.push((p, cd));
                crate::glog!(g, "    Venser exiles {name} until the beginning of the next end step");
            }
            _ => crate::glog!(g, "    Venser exiles {name} (a token: it is gone for good)"),
        }
        Ok(())
    })
}

fn venser_minus1(_g: &mut Game, p: PlayerId, _m: PermId) -> Res<Ready> {
    go(move |g| {
        g.player_mut(p).unbl_all = Some(g.turn_stamp());
        crate::glog!(g, "    creatures can't be blocked this turn (Venser)");
        Ok(())
    })
}

fn venser_minus8(_g: &mut Game, p: PlayerId, m: PermId) -> Res<Ready> {
    go(move |g| crate::impls::alela::venser_emblem(g, p, m))
}
