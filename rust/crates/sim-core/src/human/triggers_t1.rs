//! Practice mode: triggers later in a game whose choice the AI's card code made for the person (the T1 audit,
//! rust/agents/practice-followups.md). Each function is the person's branch of one card's trigger; the AI path stays in
//! the card's own file, and every function here is only reached for a seat with a person.
//!
//! - **Cityscape Leveler**: the target of "destroy up to one target nonland permanent" (cast and attack).
//! - **Iron Man, Armored Avenger**: the creature that gets the +1/+1 counter when you draw.
//! - **Korvold, Fae-Cursed King**: the permanent sacrificed when it enters or attacks.
//! - **Kura, the Boundless Sky**: the mode of its dies trigger, and the (up to three) lands it fetches.
//! - **Mishra's Self-Replicator**: the "you may pay {1}".
//! - **Myr Battlesphere**: X, the Myr tapped.
//! - **Portal to Phyrexia**: the creature card put onto the battlefield at upkeep.
//! - **Scarlet Witch**: the target and costs of the spell cast for free.
//! - **Susurian Voidborn**: the target opponent of the drain.
//! - **Sylvan Library**: the optional draw, the two cards chosen, and pay 4 life or put on top.
//! - **The Ozolith**: the creature that gets the counters (or none).
//! - **Vision, Synthezoid Avenger**: the mode chosen when a spell is cast outside its caster's turn.

use super::{Target, cards, choices, legal};
use crate::engine::hooks::fire_trigger;
use crate::engine::repl;
use crate::engine::values::untargetable;
use crate::engine::zones::{Tokens, die, make_tokens};
use crate::flow::Res;
use crate::hooks::{Call, Event, Sacrificed};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::intern;

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

// ------------------------------------------------------------------ Cityscape Leveler
/// the nonland permanents Cityscape Leveler's "up to one target" may take: anyone's, hexproof respected on an
/// opponent's
pub fn leveler_cands(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| {
            live(g, m)
                && !g.perm(m).cd.is_some_and(|c| g.db.get(c).land)
                && !(g.perm(m).owner != p && untargetable(g, m))
        })
        .collect()
}

/// the person picks the permanent Leveler destroys, or none ("up to one")
pub fn leveler_pick(g: &Game, p: PlayerId, when: &str) -> Res<Option<PermId>> {
    let cands = leveler_cands(g, p);
    if cands.is_empty() {
        return Ok(None);
    }
    let prompt = format!("Cityscape Leveler ({when}): destroy up to one target nonland permanent");
    cards::pick_perm(g, p, "target", &prompt, &cands, Some("no target"))
}

// ------------------------------------------------------------------ Iron Man, Armored Avenger
/// "Whenever you draw a card, put a +1/+1 counter on target creature": the creature the person picks (the AI always
/// feeds its Army)
pub fn ironman_counter(g: &mut Game, p: PlayerId) -> Res {
    let cre = cards::creatures(g, p, false);
    if cre.is_empty() {
        return Ok(());
    }
    let Some(t) = cards::pick_perm(g, p, "target", "Iron Man: put a +1/+1 counter on which creature?", &cre, None)?
    else {
        return Ok(());
    };
    crate::glog!(g, "    Iron Man puts a +1/+1 counter on {}", g.perm(t).name);
    repl::put_plus(g, t, 1);
    Ok(())
}

// ------------------------------------------------------------------ Korvold, Fae-Cursed King
/// "sacrifice another permanent": the person picks it, a Food, Clue or Treasure token or a land among the options
pub fn korvold_sacrifice(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    #[derive(Clone, Copy)]
    enum Pick {
        Token(&'static str),
        Perm(PermId),
        Land(PermId),
    }
    let mut opts: Vec<(Pick, String)> = vec![];
    for (kind, have) in [("Food", g.player(p).foods), ("Clue", g.player(p).clues), ("Treasure", g.player(p).treasures)]
    {
        if have > 0 {
            opts.push((Pick::Token(kind), format!("a {kind} token ({have})")));
        }
    }
    for &m in &g.player(p).perms {
        if m != src && live(g, m) {
            opts.push((Pick::Perm(m), legal::describe_target(g, p, Target::Perm(m))));
        }
    }
    for &l in &g.player(p).lands {
        if live(g, l) {
            opts.push((Pick::Land(l), legal::describe_target(g, p, Target::Perm(l))));
        }
    }
    if opts.is_empty() {
        return Ok(());
    }
    let labels: Vec<String> = opts.iter().map(|o| o.1.clone()).collect();
    let k = choices::choose_one(g, p, "choose", "Korvold: sacrifice which other permanent?", &labels)?;
    match opts[k].0 {
        Pick::Perm(m) => die(g, m, "sac"),
        Pick::Token(kind) => {
            match kind {
                "Food" => {
                    crate::impls::common::sac_food(g, p, 1)?;
                    return Ok(());
                }
                "Clue" => g.player_mut(p).clues -= 1,
                _ => g.player_mut(p).treasures -= 1,
            }
            if !g.hooks.is_empty() {
                fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what: Sacrificed::Token(kind) })?;
            }
            Ok(())
        }
        Pick::Land(l) => {
            let cd = g.perm(l).card();
            crate::engine::turn::remove_land(g, p, l);
            g.player_mut(p).gy.push(cd);
            if !g.hooks.is_empty() {
                fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what: Sacrificed::Card(cd) })?;
                fire_trigger(g, Event::LandGy, Call::Cards { p, cards: vec![cd] })?;
            }
            Ok(())
        }
    }
}

// ------------------------------------------------------------------ Kura, the Boundless Sky
/// "When Kura dies, choose one": up to three land cards from the library to the hand, or an X/X green Spirit, X = the
/// lands you control
pub fn kura_dies(g: &mut Game, p: PlayerId) -> Res {
    let labels = vec![
        "search your library for up to three land cards, reveal them, put them into your hand".to_string(),
        "create an X/X green Spirit creature token, where X is the number of lands you control".to_string(),
    ];
    let k = choices::choose_one(g, p, "choose", "Kura, the Boundless Sky: choose one", &labels)?;
    if k == 0 {
        for i in 0..3 {
            // the library may hold several copies of a land: the pool is rebuilt after each pick, one entry per name
            let mut pool: Vec<CardId> = vec![];
            for c in crate::engine::zones::searchable(g, p) {
                if g.db.get(c).land && !pool.iter().any(|&x| g.db.get(x).name == g.db.get(c).name) {
                    pool.push(c);
                }
            }
            pool.sort_by(|&a, &b| g.db.get(a).name.cmp(&g.db.get(b).name));
            if pool.is_empty() {
                break;
            }
            let labels: Vec<String> = pool.iter().map(|&c| choices::card_label(g, c)).collect();
            let prompt = format!("Kura, the Boundless Sky: search for a land card ({} of 3)", i + 1);
            let Some(j) = choices::choose(g, p, "search", &prompt, &labels, Some("find nothing more"), None)? else {
                break;
            };
            let c = pool[j];
            let lib = &mut g.player_mut(p).library;
            if let Some(at) = lib.iter().position(|&x| x == c) {
                lib.remove(at);
            }
            g.player_mut(p).hand.push(c);
        }
        choices::shuffle_library(g, p);
    } else {
        let x = g.player(p).lands.len() as i32;
        let mut spec = Tokens::new(1, x);
        spec.tgh = Some(x);
        spec.color = Some(crate::cards::Colors::from_letters("G"));
        spec.types = vec![intern("spirit")];
        make_tokens(g, p, spec)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Myr Battlesphere
/// "you may tap X untapped Myr you control": the Myr the person taps (none: X = 0)
pub fn battlesphere_tap(g: &Game, p: PlayerId, myr: &[PermId]) -> Res<Vec<PermId>> {
    let n = myr.len();
    let labels: Vec<String> = (0..=n).map(|x| format!("X = {x}")).collect();
    let prompt =
        "Myr Battlesphere: tap how many untapped Myr? (it gets +X/+0 and deals X damage to the player it's attacking)";
    let x = choices::choose_one(g, p, "choose", prompt, &labels)?;
    if x == 0 || x == n {
        return Ok(myr[..x].to_vec());
    }
    if myr.iter().all(|&m| g.perm(m).name == g.perm(myr[0]).name) {
        return Ok(myr[..x].to_vec()); // all alike: no choice to make
    }
    let mut left = myr.to_vec();
    let mut tapped = vec![];
    for i in 0..x {
        let prompt = format!("Myr Battlesphere: tap which Myr? ({} of {x})", i + 1);
        let m = cards::pick_perm(g, p, "choose", &prompt, &left, None)?.unwrap_or(left[0]);
        left.retain(|&y| y != m);
        tapped.push(m);
    }
    Ok(tapped)
}

// ------------------------------------------------------------------ Portal to Phyrexia
/// "put target creature card from a graveyard onto the battlefield under your control": the card (and whose graveyard)
pub fn portal_pick(g: &Game, p: PlayerId) -> Res<Option<(CardId, PlayerId)>> {
    let mut cands: Vec<(CardId, PlayerId)> = vec![];
    for q in g.players.iter().filter(|q| q.alive || q.id == p) {
        for &c in &q.gy {
            if g.db.get(c).creature {
                cands.push((c, q.id));
            }
        }
    }
    if cands.is_empty() {
        return Ok(None);
    }
    let labels: Vec<String> = cands
        .iter()
        .map(|&(c, q)| {
            let whose = if q == p { "your graveyard".to_string() } else { format!("{}'s graveyard", g.player(q).name) };
            format!("{} ({whose})", choices::card_label(g, c))
        })
        .collect();
    let k = choices::choose_one(
        g,
        p,
        "target",
        "Portal to Phyrexia: put which creature card onto the battlefield?",
        &labels,
    )?;
    Ok(Some(cands[k]))
}

// ------------------------------------------------------------------ Scarlet Witch
/// a spell cast without paying its mana cost from Scarlet Witch's exiled cards: the person's target (and any other
/// choice the cast asks before paying). None: they cancelled, so the card stays exiled with her. The additional costs
/// the choices made are returned to pay.
pub fn free_cast(g: &Game, p: PlayerId, c: CardId) -> Res<Option<(Ctx, super::specs::Costs)>> {
    let name = g.db.get(c).name.to_string();
    let mut ctx = Ctx::default();
    if let Some(tgts) = legal::spell_targets(g, p, c) {
        if tgts.is_empty() {
            return Ok(None); // no legal target: it can't be cast now
        }
        let labels: Vec<String> = tgts.iter().map(|&x| legal::describe_target(g, p, x)).collect();
        let prompt = format!("{name}: choose a target");
        let Some(k) = choices::choose(g, p, "target", &prompt, &labels, Some("cancel"), None)? else {
            return Ok(None);
        };
        match tgts[k] {
            Target::Player(q) => ctx.face = Some(q),
            Target::Perm(m) => ctx.target = Some(m),
        }
    }
    match super::specs::choose_pre(g, p, c, &mut ctx)? {
        super::specs::Pre::Ok(costs) => Ok(Some((ctx, costs))),
        _ => Ok(None),
    }
}

// ------------------------------------------------------------------ Susurian Voidborn
/// "target opponent loses 1 life": with more than one opponent the person picks
pub fn voidborn_target(g: &Game, p: PlayerId, opps: &[PlayerId]) -> Res<PlayerId> {
    if opps.len() == 1 {
        return Ok(opps[0]);
    }
    cards::pick_player(g, p, "target", "Susurian Voidborn: target which opponent loses 1 life?", opps)
}

// ------------------------------------------------------------------ Sylvan Library
/// "you may draw two additional cards. If you do, choose two cards in your hand drawn this turn. For each of those
/// cards, pay 4 life or put the card on top of your library." The hand's last `draw_n` cards are the ones drawn this
/// turn (the library hook runs at the first draw of the turn, so the first card is already in the hand).
pub fn sylvan_library(g: &mut Game, p: PlayerId) -> Res {
    if !choices::yes_no(g, p, "Sylvan Library: draw two additional cards?")? {
        return Ok(());
    }
    crate::engine::zones::draw(g, p, 2, false)?;
    let hand = g.player(p).hand.clone();
    let n = (g.player(p).draw_n as usize).min(hand.len());
    let drawn: Vec<CardId> = hand[hand.len() - n..].to_vec();
    let picked = choices::pick_cards(
        g,
        p,
        &drawn,
        2,
        "Sylvan Library: choose two cards in your hand drawn this turn (pay 4 life or put each on top of your library)",
        "choose",
    )?;
    for c in picked {
        let name = choices::card_label(g, c);
        let can_pay = g.player(p).life >= 4;
        let mut labels = vec!["put it on top of your library".to_string()];
        if can_pay {
            labels.push("pay 4 life and keep it".to_string());
        }
        let k = choices::choose_one(g, p, "choose", &format!("Sylvan Library: {name}"), &labels)?;
        if k == 1 {
            crate::engine::life::lose_life(g, p, 4, Some(p), "other", None)?;
            crate::t6stats::flow_life(g, p, "Sylvan Library", 4);
            crate::glog!(g, "    {} pays 4 life for a Sylvan Library card", g.player(p).name);
        } else {
            let pl = g.player_mut(p);
            if let Some(i) = pl.hand.iter().position(|&x| x == c) {
                pl.hand.remove(i);
                pl.library.push(c);
                crate::t6stats::flow_undraw(g, p, "Sylvan Library", 1);
                crate::glog!(g, "    {} puts a card on top of the library (Sylvan Library)", g.player(p).name);
            }
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ The Ozolith
/// "at the beginning of combat on your turn, you may move all counters from The Ozolith onto target creature"
pub fn ozolith_move(g: &mut Game, p: PlayerId) -> Res {
    let k = g.player(p).ozolith_counters;
    if k == 0 || !crate::engine::values::has(g, p, crate::tag::Tag::Ozolith) {
        return Ok(());
    }
    let cre = cards::creatures(g, p, false);
    if cre.is_empty() {
        return Ok(());
    }
    let prompt = format!("The Ozolith: move its {k} counter(s) onto which creature?");
    let Some(t) = cards::pick_perm(g, p, "target", &prompt, &cre, Some("don't move them"))? else {
        return Ok(());
    };
    repl::put_plus(g, t, k);
    g.player_mut(p).ozolith_counters = 0;
    g.player_mut(p).stat("ozolith_moves", 1);
    crate::glog!(g, "  The Ozolith moves {} counters onto {}", k, g.perm(t).name);
    Ok(())
}

// ------------------------------------------------------------------ Vision, Synthezoid Avenger
/// "choose one: put a +1/+1 counter on Vision; or Vision phases out": true for phasing out
pub fn vision_phases_out(g: &Game, p: PlayerId) -> Res<bool> {
    let labels = vec!["put a +1/+1 counter on Vision".to_string(), "Vision phases out".to_string()];
    Ok(choices::choose_one(g, p, "choose", "Vision, Synthezoid Avenger: choose one", &labels)? == 1)
}
