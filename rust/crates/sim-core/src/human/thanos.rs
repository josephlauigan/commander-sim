//! JD's Thanos deck for the person's seat: its finishing lines played by hand.
//!
//! * Thassa's Oracle + Demonic Consultation / Tainted Pact: the person casts either one in response to the Oracle's
//!   enter trigger (`oracle_window` gives them priority on their own trigger), names a card or takes cards by choice,
//!   and the trigger resolves last (`oracle_etb`).
//! * Underworld Breach + Lion's Eye Diamond + Brain Freeze / Grapeshot: LED is an ability (`led`), storm copies ask for
//!   their targets as the spell is cast (`storm_asks`), and the whole loop is a table shortcut (`breach_abilities`).
//! * Ad Nauseam: the person decides after every card whether to reveal another (`ad_nauseam`).
//!
//! Never reached for an AI seat: every entry point is behind `is_human`.

use super::abilities::{Why, ability};
use super::{Target, choices, legal};
use crate::engine::{life, stack, zones};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game, StackItem, StackKind, Step};

const ORACLE: &str = "Thassa's Oracle";
const CONSULT: &str = "Demonic Consultation";
const PACT: &str = "Tainted Pact";

fn pname(g: &Game, p: PlayerId) -> String {
    g.player(p).name.to_string()
}

fn cname(g: &Game, c: CardId) -> String {
    g.db.get(c).name.to_string()
}

// ------------------------------------------------------------------ Thassa's Oracle
/// Does the person get priority on this trigger of theirs? Only the Oracle's enter trigger, and only when they hold
/// Demonic Consultation or Tainted Pact and could cast it: a trigger of your own is otherwise passed over unless
/// auto-pass is on 'all'.
pub fn oracle_window(g: &Game, q: PlayerId, top: &StackItem) -> bool {
    top.kind == StackKind::Trigger
        && top.controller == q
        && top.name.starts_with(ORACLE)
        && g.player(q).hand.iter().any(|&c| matches!(&*g.db.get(c).name, CONSULT | PACT))
        && super::human::can_respond(g, q)
}

/// Thassa's Oracle enters under the person's control: its trigger goes on the stack and they (and the table) get
/// priority first. When it resolves: look at the top X cards (X is your devotion to blue), put up to one of them on
/// top (your choice) and the rest on the bottom in a random order; if X is at least the number of cards in your
/// library, you win. X and the library are read now, so a Consultation or Pact cast in response counts.
pub fn oracle_etb(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    if !stack::trigger_window(
        g,
        p,
        Some(src),
        "look at the top cards of your library, then win if your devotion to blue is at least their number",
        Some(9.0),
    )? {
        return Ok(());
    }
    let x = crate::impls::combos::oracle_devotion(g, p);
    let n = g.player(p).library.len();
    if x >= n {
        crate::glog!(g, "    Thassa's Oracle: devotion to blue {x}, library {n}: {} wins", pname(g, p));
        return crate::ai::win(g, p, "combo", Some(false));
    }
    let k = x.min(n);
    if k == 0 {
        crate::glog!(g, "    Thassa's Oracle: devotion to blue 0, library {n}: nothing to look at");
        return Ok(());
    }
    let mut top: Vec<CardId> = (0..k).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
    let labels: Vec<String> = top.iter().map(|&c| choices::card_label(g, c)).collect();
    let prompt = format!(
        "Thassa's Oracle: the top {k} of your library (devotion {x}, library {n}). Put which one on top? The rest go to the bottom in a random order"
    );
    let best = choices::choose(g, p, "choose", &prompt, &labels, Some("none of them on top"), None)?;
    let keep = best.map(|i| top.remove(i));
    g.rng.shuffle(&mut top);
    let lib = &mut g.player_mut(p).library;
    lib.splice(0..0, top);
    if let Some(c) = keep {
        lib.push(c);
    }
    crate::glog!(g, "    Thassa's Oracle: {k} looked at, {} put on the bottom", if keep.is_some() { k - 1 } else { k });
    Ok(())
}

// ------------------------------------------------------------------ Demonic Consultation, Tainted Pact
/// The person's Demonic Consultation / Tainted Pact (or a Grapeshot with the copies' targets they chose) as it
/// resolves. True if handled here (the caller puts the card away).
pub fn resolve_spell(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<bool> {
    match &*g.db.get(c).name {
        CONSULT => consultation(g, p)?,
        PACT => pact(g, p)?,
        "Grapeshot" if !ctx.perms.is_empty() || !ctx.faces.is_empty() => grapeshot(g, p, ctx)?,
        _ => return Ok(false),
    }
    Ok(true)
}

/// Demonic Consultation: name a card; exile the top six cards of your library, then reveal cards from the top until
/// the named card shows up: it goes to your hand and everything else revealed is exiled. A card that is not in the
/// library exiles all of it.
fn consultation(g: &mut Game, p: PlayerId) -> Res {
    let mut names: Vec<String> = g.player(p).library.iter().map(|&c| cname(g, c)).collect();
    names.sort();
    names.dedup();
    let mut labels = vec!["a card that is not in your library (your whole library is exiled)".to_string()];
    labels.extend(names.iter().cloned());
    let k = choices::choose_one(
        g,
        p,
        "choose",
        "Demonic Consultation: name a card (the top six are exiled, then it exiles cards until it finds the one you name)",
        &labels,
    )?;
    let named = (k > 0).then(|| names[k - 1].clone());
    crate::glog!(g, "    {} names {}", pname(g, p), named.as_deref().unwrap_or("a card that is not in the library"));
    let mut exiled = 0;
    for _ in 0..6 {
        let Some(c) = g.player_mut(p).library.pop() else { break };
        g.player_mut(p).exile.push(c);
        exiled += 1;
    }
    let mut found = None;
    while let Some(c) = g.player_mut(p).library.pop() {
        if named.as_deref() == Some(&*cname(g, c)) {
            found = Some(c);
            break;
        }
        g.player_mut(p).exile.push(c);
        exiled += 1;
    }
    match found {
        Some(c) => {
            g.player_mut(p).hand.push(c);
            crate::glog!(
                g,
                "    Demonic Consultation: {exiled} cards exiled, {} into {}'s hand",
                cname(g, c),
                pname(g, p)
            );
        }
        None => crate::glog!(g, "    Demonic Consultation: {exiled} cards exiled, the library is empty"),
    }
    Ok(())
}

/// Tainted Pact: exile the top card; you may put it into your hand unless it has the same name as another card
/// exiled this way. Repeat until you put a card into your hand or exile two cards with the same name.
fn pact(g: &mut Game, p: PlayerId) -> Res {
    let mut seen: Vec<String> = vec![];
    let mut rest_declined = false;
    loop {
        let Some(c) = g.player_mut(p).library.pop() else {
            crate::glog!(g, "    Tainted Pact: {} has no cards left to exile", pname(g, p));
            return Ok(());
        };
        g.player_mut(p).exile.push(c);
        let name = cname(g, c);
        if seen.contains(&name) {
            crate::glog!(g, "    Tainted Pact exiles a second {name}: it ends ({} cards exiled)", seen.len() + 1);
            return Ok(());
        }
        seen.push(name.clone());
        if rest_declined {
            continue;
        }
        let labels = vec![
            "put it into your hand (this ends the Pact)".to_string(),
            "leave it exiled and exile the next card".to_string(),
            "leave it and every card after it exiled (the Pact runs until two share a name or the library is empty)"
                .to_string(),
        ];
        let prompt = format!(
            "Tainted Pact: {name} is exiled ({} so far). Put it into your hand?{}",
            seen.len(),
            if seen.len() > 1 {
                format!(" Already exiled: {}", seen[..seen.len() - 1].join(", "))
            } else {
                String::new()
            }
        );
        match choices::choose_one(g, p, "choose", &prompt, &labels)? {
            0 => {
                let pl = g.player_mut(p);
                if let Some(i) = pl.exile.iter().rposition(|&x| x == c) {
                    pl.exile.remove(i);
                }
                pl.hand.push(c);
                crate::glog!(g, "    Tainted Pact: {} puts {name} into hand", pname(g, p));
                return Ok(());
            }
            1 => {}
            _ => rest_declined = true,
        }
    }
}

// ------------------------------------------------------------------ Ad Nauseam
/// Ad Nauseam: reveal the top card, put it into your hand, lose life equal to its mana value; you may repeat this any
/// number of times. The person is asked after every card, with their life and the cards so far.
pub fn ad_nauseam(g: &mut Game, p: PlayerId) -> Res {
    let mut got: Vec<(String, i32)> = vec![];
    let labels = vec!["reveal the top card".to_string(), "stop".to_string()];
    loop {
        if g.over || !g.player(p).alive || g.player(p).library.is_empty() {
            break;
        }
        let so_far = if got.is_empty() {
            "nothing yet".to_string()
        } else {
            let lost: i32 = got.iter().map(|x| x.1).sum();
            let names: Vec<String> = got.iter().map(|(n, v)| format!("{n} ({v})")).collect();
            format!("{} cards for {lost} life: {}", got.len(), names.join(", "))
        };
        let prompt = format!(
            "Ad Nauseam: you are at {} life, {} cards in your library. Revealed so far: {so_far}. Reveal another?",
            g.player(p).life,
            g.player(p).library.len()
        );
        if choices::choose_one(g, p, "choose", &prompt, &labels)? != 0 {
            break;
        }
        let c = g.player_mut(p).library.pop().unwrap();
        let (name, mv) = (cname(g, c), g.db.get(c).cmc as i32);
        let pl = g.player_mut(p);
        pl.hand.push(c);
        pl.seen.insert(c);
        crate::glog!(g, "    Ad Nauseam reveals {name}: {mv} life");
        got.push((name, mv));
        life::lose_life(g, p, mv, Some(p), "other", None)?;
        life::check_state(g)?;
    }
    g.player_mut(p).stat("adnaus_cards", got.len() as i64);
    crate::glog!(g, "    Ad Nauseam: {} cards, life now {}", got.len(), g.player(p).life);
    Ok(())
}

// ------------------------------------------------------------------ storm: Brain Freeze, Grapeshot
/// How many copies a storm spell cast now would get: the spells cast this turn so far, by every player
fn storm_prior(g: &Game) -> i32 {
    g.players.iter().map(|q| crate::engine::cast::casts_this_turn(g, q.id)).sum()
}

fn player_targets(g: &Game) -> Vec<PlayerId> {
    g.players.iter().filter(|q| q.alive && legal::player_targetable(g, q.id)).map(|q| q.id).collect()
}

/// The targets of a storm spell and its copies, asked as it is cast (a storm trigger copies it then, and each copy
/// may have a new target): the spell itself, then all the copies at the same target or one question per copy.
/// Brain Freeze fills `ctx.mill` (one player per copy, the original first), Grapeshot `ctx.faces` / `ctx.perms`.
/// Returns false if the person cancelled.
pub fn storm_asks(g: &Game, p: PlayerId, c: CardId, ctx: &mut Ctx) -> Res<bool> {
    let name = cname(g, c);
    let brain = name == "Brain Freeze";
    let prior = storm_prior(g);
    ctx.storm = Some(prior);
    let copies = prior.max(0) as usize;
    let tg: Vec<Target> = if brain {
        player_targets(g).into_iter().map(Target::Player).collect()
    } else {
        choices::damage_targets(g, p, crate::cards::Colors::from_letters("R"))
    };
    if tg.is_empty() {
        return Ok(true);
    }
    let labels: Vec<String> = tg.iter().map(|&t| legal::describe_target(g, p, t)).collect();
    let what = if brain { "target player" } else { "any target" };
    let prompt = format!(
        "{name}: choose a target ({what}). Storm: {prior} cop{} will be made",
        if prior == 1 { "y" } else { "ies" }
    );
    let Some(first) = choices::choose(g, p, "target", &prompt, &labels, Some("cancel"), None)? else {
        return Ok(false);
    };
    let mut picked = vec![tg[first]];
    if copies > 0 {
        let how = vec![
            format!("all {copies} copies at the same target ({})", labels[first]),
            "choose a target for each copy".to_string(),
        ];
        let pr = format!("{name}: storm made {copies} cop{}. Where do they go?", if copies == 1 { "y" } else { "ies" });
        let each = choices::choose_one(g, p, "choose", &pr, &how)? == 1;
        for i in 0..copies {
            if each {
                let pr = format!("{name}: target for copy {} of {copies}", i + 1);
                picked.push(tg[choices::choose_one(g, p, "target", &pr, &labels)?]);
            } else {
                picked.push(tg[first]);
            }
        }
    }
    for t in picked {
        match t {
            Target::Player(q) if brain => ctx.mill.push(q),
            Target::Player(q) => ctx.faces.push(q),
            Target::Perm(m) => ctx.perms.push(m),
        }
    }
    Ok(true)
}

/// Grapeshot as it resolves: 1 damage from the spell and from each copy, at the targets chosen as it was cast
fn grapeshot(g: &mut Game, p: PlayerId, ctx: &Ctx) -> Res {
    let n = ctx.perms.len() + ctx.faces.len();
    for &q in &ctx.faces {
        if g.player(q).alive {
            choices::apply_damage(g, p, Target::Player(q), 1, "Grapeshot", "burn")?;
        }
    }
    for &m in &ctx.perms {
        if g.perm(m).on_bf {
            choices::apply_damage(g, p, Target::Perm(m), 1, "Grapeshot", "burn")?;
        }
    }
    crate::glog!(g, "    Grapeshot: {n} damage");
    Ok(())
}

// ------------------------------------------------------------------ Lion's Eye Diamond and the Breach loop
/// Lion's Eye Diamond: discard your hand, sacrifice it: add three mana of any one colour. A mana ability, so nothing
/// answers it; it can be used at instant speed, the mana staying in your pool.
pub fn led(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if !g.perm(m).on_bf {
        return Ok(Some("Lion's Eye Diamond is gone.".into()));
    }
    let labels: Vec<String> =
        ["white", "blue", "black", "red", "green"].iter().map(|c| format!("three {c} mana")).collect();
    let Some(k) = choices::choose(
        g,
        p,
        "choose",
        "Lion's Eye Diamond: discard your hand and sacrifice it. Add which three mana?",
        &labels,
        Some("cancel"),
        None,
    )?
    else {
        return Ok(None);
    };
    let colour = ['W', 'U', 'B', 'R', 'G'][k];
    let hand = g.player(p).hand.clone();
    zones::discard_cards(g, p, &hand)?;
    zones::die(g, m, "sac")?;
    g.player_mut(p).pool.add(colour, 3);
    crate::glog!(
        g,
        "  {} cracks Lion's Eye Diamond for {{{colour}}}{{{colour}}}{{{colour}}} ({} cards discarded)",
        pname(g, p),
        hand.len()
    );
    g.player_mut(p).stat("led_used", 1);
    Ok(None)
}

/// The abilities of Underworld Breach for the person: the table shortcut for the loop, when it is really assembled
pub fn breach_abilities(g: &Game, p: PlayerId) -> Option<Vec<super::abilities::PermAbility>> {
    if !crate::impls::t6_combos_extra::human_loop_ready(g, p) || legal::sorcery_timing(g, p).is_some() {
        return None;
    }
    if g.step != Step::Main1 && g.step != Step::Main2 {
        return None;
    }
    Some(vec![ability(
        "loop Underworld Breach + Lion's Eye Diamond + Brain Freeze at yourself until your library is empty, then escape Thassa's Oracle (every escape is cast for real, so the table can answer; your hand is discarded)",
        |g: &mut Game, p: PlayerId, _m: PermId| {
            if !crate::impls::t6_combos_extra::human_loop_ready(g, p) {
                return Ok(Some("The loop isn't assembled any more.".into()));
            }
            crate::impls::t6_combos_extra::human_loop(g, p)?;
            Ok(None)
        },
    )])
}
