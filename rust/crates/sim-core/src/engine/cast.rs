//! Casting: costs beyond mana, what may be cast, what happens on a cast (cast triggers, magecraft, copies), casting a
//! card through the stack, and resolving it (engine.py's "casting" section, cast_card, resolve).

use crate::ai;
use crate::cardcode;
use crate::cards::Types;
use crate::engine::hooks::{self, fire_trigger};
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::removal::{apply_removal, apply_wipe, legal_targets};
use crate::engine::stack::{counter_window, counterable, stack_window_keep, trigger_window};
use crate::engine::tutors::{ad_nauseam, card_worth, jeskas_will, pile_tutor, tutor, tutor_to_top};
use crate::engine::values::*;
use crate::engine::zones::*;
use crate::flow::Res;
use crate::hooks::{Call, Event};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, StackItem, StackKind, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

// ------------------------------------------------------------------ costs and permission
/// What p would sacrifice for a cost ('creature', 'artifact', 'artifact or creature', 'green creature', 'permanent'):
/// the cheapest permanent, or a Treasure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fodder {
    Perm(PermId),
    Treasure,
}

pub fn sac_fodder(g: &Game, p: PlayerId, what: &str, exclude: Option<PermId>) -> Option<Fodder> {
    let art = what.contains("artifact") || what == "permanent";
    let cre = what.contains("creature") || what == "permanent";
    let mut cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            Some(m) != exclude
                && !x.phased
                && !x.is_cmd
                && !no_sac(g, m)
                && ((cre && g.is_creature(m))
                    || (art && x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)))
                    || what == "permanent")
        })
        .collect();
    if what.contains("green") {
        cands.retain(|&m| colors_of(g, m).has('G'));
    }
    let best = min_by(&cands, |m| sac_worth(g, m));
    if art && g.player(p).treasures > 0 && best.is_none_or(|b| sac_worth(g, b) > 0.6) {
        return Some(Fodder::Treasure);
    }
    best.map(Fodder::Perm)
}

/// "As an additional cost to cast this spell, sacrifice ... / discard a card" (compiled cards). dry: can it be paid?
/// Otherwise pay it. False when it can't be paid. Python's `additional_cost`.
pub fn additional_cost(g: &mut Game, p: PlayerId, c: CardId, dry: bool) -> Res<bool> {
    let texts: Vec<String> = match &g.db.get(c).dsl {
        Some(serde_json::Value::Array(abs)) => abs
            .iter()
            .filter(|a| a.get("type").and_then(|t| t.as_str()) == Some("additional_cost"))
            .filter_map(|a| a.get("text").and_then(|t| t.as_str()).map(String::from))
            .collect(),
        _ => vec![],
    };
    enum Choice {
        Land(crate::ids::LandId),
        Sac(Fodder),
        Discard,
    }
    for t in texts {
        let mut choice = None;
        if t.contains("sacrifice a land") {
            let lands = g.player(p).lands.clone();
            // a tapped basic first
            let key = |l: crate::ids::LandId| {
                let basic = BASIC_LANDS.contains(&&*g.db.get(g.land(l).cd).name);
                (!g.land(l).tapped as i32 as f64) * 2.0 + (!basic as i32 as f64)
            };
            if let Some(l) = min_by(&lands, key) {
                choice = Some(Choice::Land(l));
            }
        } else if let Some(what) = sac_phrase(&t)
            && let Some(f) = sac_fodder(g, p, &what, None)
            && (f == Fodder::Treasure || matches!(f, Fodder::Perm(m) if pval(g, m) < 4.0) || !t.contains("discard"))
        {
            choice = Some(Choice::Sac(f));
        }
        if choice.is_none() && t.contains("discard a card") && g.player(p).hand.iter().any(|&x| x != c) {
            choice = Some(Choice::Discard);
        }
        let Some(choice) = choice else { return Ok(false) };
        if dry {
            continue;
        }
        match choice {
            Choice::Land(l) => {
                let cd = g.land(l).cd;
                g.player_mut(p).lands.retain(|&x| x != l);
                g.land_mut(l).on_bf = false;
                g.player_mut(p).gy.push(cd);
                if !g.hooks.is_empty() {
                    fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what: crate::hooks::Sacrificed::Card(cd) })?;
                }
            }
            Choice::Sac(Fodder::Treasure) => {
                g.player_mut(p).treasures -= 1;
                if !g.hooks.is_empty() {
                    fire_trigger(
                        g,
                        Event::Sacrifice,
                        Call::Sacrifice { p, what: crate::hooks::Sacrificed::Token("Treasure") },
                    )?;
                }
            }
            Choice::Sac(Fodder::Perm(m)) => {
                g.sac_snapshot = Some((g.perm(m).cd.map_or(0, |c| g.db.get(c).cmc), etgh(g, m)));
                die(g, m, "sac")?;
            }
            Choice::Discard => {
                let others: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&x| x != c).collect();
                let lands: Vec<CardId> = others.iter().copied().filter(|&x| g.db.get(x).land).collect();
                let spells: Vec<CardId> = others.iter().copied().filter(|&x| !g.db.get(x).land).collect();
                let x = if lands.len() > 2 || spells.is_empty() {
                    lands[0]
                } else {
                    min_by(&spells, |y| card_worth(g, p, y, false)).unwrap()
                };
                discard_cards(g, p, &[x])?;
            }
        }
    }
    Ok(true)
}

/// the permanent kind an additional cost sacrifices ("sacrifice a creature" -> "creature")
fn sac_phrase(t: &str) -> Option<String> {
    let i = t.find("sacrifice ")? + "sacrifice ".len();
    let rest = &t[i..];
    let rest =
        rest.strip_prefix("an ").or_else(|| rest.strip_prefix("a ")).or_else(|| rest.strip_prefix("another "))?;
    for kind in [
        "green artifact or creature",
        "green creature or artifact",
        "green creature",
        "green artifact",
        "green permanent",
        "artifact or creature",
        "creature or artifact",
        "creature",
        "artifact",
        "permanent",
    ] {
        if rest.starts_with(kind) {
            return Some(kind.to_string());
        }
    }
    None
}

/// can p cast c right now as far as locks go (Rule of Law, Drannith Magistrate, Grand Abolisher ...)
pub fn castable(g: &Game, p: PlayerId, c: CardId, zone: &str) -> bool {
    let d = g.db.get(c);
    if !d.creature && !d.land {
        if let Some((q, t)) = g.player(p).hope_lock
            && g.player(q).alive
            && g.player(q).turns == t
        {
            return false;
        }
        if let Some((st, q)) = g.silence
            && st == g.turn_stamp()
            && q != p
        {
            return false;
        }
    }
    g.hooks.is_empty() || hooks::allowed(g, p, c, zone)
}

/// spells p has cast during the current turn (anyone's turn)
pub fn casts_this_turn(g: &Game, p: PlayerId) -> i32 {
    match &g.player(p).turn_casts {
        Some((st, cs)) if *st == g.turn_stamp() => cs.len() as i32,
        _ => 0,
    }
}

// ------------------------------------------------------------------ casting triggers
/// what happens when p casts c (cast triggers of tagged cards, magecraft, copies). Python's `on_cast`.
pub fn on_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let st = g.turn_stamp();
    if g.player(p).mistrise_next == Some(st) {
        g.player_mut(p).mistrise_next = None; // Mistrise Village: this spell can't be countered
        g.unc_cast = Some((c, st));
    }
    let d = g.db.get(c);
    let (instant_or_sorcery, creature, artifact) = (d.instant || d.sorcery, d.creature, d.types.has(Types::ARTIFACT));
    let cmc = d.cmc;
    if let Some(cost) = d.tags.str(Tag::Pactpay) {
        let pc = parse_cost(cost); // Slaughter Pact: pay at your next upkeep or lose the game
        g.player_mut(p).pact_debts.push(pc);
    }
    if !g.player(p).emblems.is_empty() {
        cardcode::emblem_cast(g, p, c)?;
    }
    if g.player(p).glimpse {
        cardcode::glimpse_draw(g, p, c)?;
    }
    let pl = g.player_mut(p);
    match &mut pl.turn_casts {
        Some((s, cs)) if *s == st => cs.push(c),
        tc => *tc = Some((st, vec![c])),
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        if has(g, q, Tag::Sauron) {
            let src = find(g, q, Tag::Sauron)[0];
            if trigger_window(g, q, Some(src), "amass 1", None)? {
                amass(g, q, 1)?;
            }
        }
        if has(g, q, Tag::Rhystic) {
            let src = find(g, q, Tag::Rhystic)[0];
            if trigger_window(g, q, Some(src), "draw unless they pay {1}", None)? && cardcode::rhystic_unpaid(g, p)? {
                draw(g, q, 1, false)?;
            }
        }
        if has(g, q, Tag::Kaervek) && cmc > 0 {
            let src = find(g, q, Tag::Kaervek)[0];
            if trigger_window(g, q, Some(src), "damage", None)? {
                cardcode::kaervek(g, q, p, c)?;
            }
        }
    }
    if instant_or_sorcery {
        count_is_cast(g, p);
        magecraft(g, p, Some(c), false)?;
    }
    // PORT(M3): DSLMOD.fire(g, 'cast', caster=p, spell=c)
    cardcode::self_cast(g, p, c)?; // "when you cast this spell" (cascade)
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::Cast, Call::Cast { caster: p, c })?;
    }
    if has(g, p, Tag::Jin) && (artifact || instant_or_sorcery) && once_per_turn(g, p, "jincopy") {
        let src = find(g, p, Tag::Jin)[0];
        let name = format!("copy {}", g.db.get(c).name);
        if trigger_window(g, p, Some(src), &name, Some(5.0))? {
            copy_spell(g, p, c, None, false)?; // Jin-Gitaxias copies your first artifact/instant/sorcery each turn
        }
    }
    if instant_or_sorcery && &*g.db.get(c).name != "Galvanic Iteration" && g.player(p).galvanic == Some(st) {
        g.player_mut(p).galvanic = None; // Galvanic Iteration: copy the next instant or sorcery
        let name = format!("Galvanic Iteration: copy {}", g.db.get(c).name);
        if trigger_window(g, p, None, &name, Some(4.0))? {
            copy_spell(g, p, c, None, false)?;
        }
    }
    if instant_or_sorcery && g.player(p).ral_copy == Some(st) {
        g.player_mut(p).ral_copy = None; // Ral, Storm Conduit -2: copy the next instant/sorcery
        let name = format!("Ral, Storm Conduit: copy {}", g.db.get(c).name);
        if trigger_window(g, p, None, &name, Some(5.0))? {
            copy_spell(g, p, c, None, false)?;
        }
    }
    if !creature {
        prowess(g, p);
    }
    if instant_or_sorcery {
        // HUMAN(phase 9): a person copies in their own window, not here
        cardcode::hand_cast(g, p, c)?; // Return the Favor copies your spell
        for q in g.opps(p).collect::<Vec<_>>() {
            cardcode::hand_opp_cast(g, q, c)?; // Dualcaster Mage copies anyone's
        }
    }
    if g.player(p).spells_this_turn == 3 {
        // Emeritus of Conflict: third spell each turn -> prepared
        for x in find(g, p, Tag::Conflict) {
            g.perm_mut(x).data.set(DataKey::Prepared, Val::Bool(true));
        }
    }
    if has(g, p, Tag::Eris) && g.player(p).spells_this_turn == 2 {
        let src = find(g, p, Tag::Eris)[0];
        if trigger_window(g, p, Some(src), "a 4/4 Dragon", None)? {
            let n = if has(g, p, Tag::Veyran) && instant_or_sorcery { 2 } else { 1 };
            for x in make_tokens(g, p, Tokens { fly: true, ..Tokens::new(n, 4) })? {
                g.perm_mut(x).data.set(DataKey::Prowess, Val::Bool(true)); // Eris: a 4/4 flying Dragon with prowess
            }
        }
    }
    if has(g, p, Tag::Prolifall) && creature {
        let src = find(g, p, Tag::Prolifall)[0];
        if trigger_window(g, p, Some(src), "counters on your Army", None)?
            && let Some(a) = army_of(g, p)
        {
            g.perm_mut(a).plus += find(g, p, Tag::Prolifall).len() as i32;
        }
    }
    if has(g, p, Tag::Birgi) {
        // Birgi: add R whenever you cast a spell; an instant/sorcery trigger is doubled by Veyran
        let extra = if has(g, p, Tag::Veyran) && instant_or_sorcery { 1 } else { 0 };
        g.player_mut(p).floating.r += 1 + extra;
    }
    if !creature && let Some(a) = army_of(g, p) {
        for x in find(g, p, Tag::Prolif) {
            if trigger_window(g, p, Some(x), "a counter on your Army", None)? && g.perm(a).on_bf {
                cardcode::add_counters(g, a, 1);
            }
        }
    }
    check_state(g)
}

/// prowess: +1/+1 until end of turn for each creature with it whenever you cast a noncreature spell
pub fn prowess(g: &mut Game, p: PlayerId) {
    for m in g.player(p).perms.clone() {
        let x = g.perm(m);
        if x.phased || !g.is_creature(m) {
            continue;
        }
        let has_it = x.cd.is_some_and(|c| g.db.get(c).has_kw("prowess") || cardcode::is_prowess_card(g, c))
            || x.data.truthy(DataKey::Prowess);
        if has_it {
            let y = g.perm_mut(m);
            y.eot_pt = (y.eot_pt.0 + 1, y.eot_pt.1 + 1);
        }
    }
}

pub const MAGECRAFT_TAGS: [Tag; 7] =
    [Tag::Ping, Tag::Dragoncaller, Tag::Mystic, Tag::Spelltok, Tag::Spelldraw, Tag::Kiln, Tag::Spellloot];

/// Each magecraft ability triggers once, +1 with Veyran, +1 more with Harmonic Prodigy if the source is a Shaman or
/// Wizard. copy: a copied spell (only "cast or copy" abilities trigger).
pub fn magecraft(g: &mut Game, p: PlayerId, c: Option<CardId>, copy: bool) -> Res {
    let vey = has(g, p, Tag::Veyran) as u32;
    let prod = has(g, p, Tag::Prodigy) as u32;
    let base_mult = 1 + vey;
    let thor = has(g, p, Tag::Thor) as i32;
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if opps.is_empty() {
        return Ok(());
    }
    for m in g.player(p).perms.clone() {
        let x = g.perm(m);
        let Some(cd) = x.cd else { continue };
        if x.phased {
            continue;
        }
        let d = g.db.get(cd);
        let t = d.tags.clone();
        let name = d.name.to_string();
        if copy && ![Tag::Spelldraw, Tag::Kiln, Tag::Ral].iter().any(|&k| t.has(k)) {
            continue; // copies only trigger "cast or copy" abilities (Archmage Emeritus, Storm-Kiln Artist, Ral)
        }
        let mult = base_mult + if t.has(Tag::Shaman) || t.has(Tag::Wizard) { prod } else { 0 };
        if MAGECRAFT_TAGS.iter().any(|&k| t.has(k)) && !trigger_window(g, p, Some(m), "magecraft", None)? {
            continue;
        }
        if let Some(mut base) = t.int(Tag::Ping) {
            if t.has(Tag::Opus3) && c.is_some_and(|c| g.db.get(c).cmc >= 5) {
                base = 3; // Thunderdrum Soloist, 5+ mana spell
            }
            let dmg = (base + thor) * mult as i32;
            if t.has(Tag::Ral) {
                // HUMAN(phase 9): the person's target (hc.deal_damage)
                let tgt = max_by(&opps, |o| threat(g, p, o)).unwrap();
                lose_life(g, tgt, dmg, Some(p), "burn", None)?;
            } else {
                for &q in &opps {
                    lose_life(g, q, dmg, Some(p), "burn", None)?;
                }
            }
        }
        if t.has(Tag::Dragoncaller) {
            make_tokens(
                g,
                p,
                Tokens { fly: true, color: Some(crate::cards::Colors::from_letters("R")), ..Tokens::new(mult, 5) },
            )?;
        }
        if t.has(Tag::Mystic) {
            make_tokens(
                g,
                p,
                Tokens { fly: true, color: Some(crate::cards::Colors::from_letters("U")), ..Tokens::new(mult, 1) },
            )?;
        }
        if let Some(pw) = t.int(Tag::Spelltok) {
            // Talrand, Young Pyromancer, Third Path Iconoclast
            let fly = t.has(Tag::Spelltokfly);
            let color = if fly {
                "U"
            } else if name.contains("Iconoclast") {
                ""
            } else {
                "R"
            };
            make_tokens(
                g,
                p,
                Tokens { fly, color: Some(crate::cards::Colors::from_letters(color)), ..Tokens::new(mult, pw) },
            )?;
        }
        if t.has(Tag::Spelldraw) {
            draw(g, p, mult, false)?;
        }
        if t.has(Tag::Kiln) {
            add_treasure(g, p, mult as i32)?; // Storm-Kiln Artist: a Treasure per trigger
        }
        if t.has(Tag::Spellloot) {
            // Muse Seeker: draw, then discard unless 5+ mana spent
            draw(g, p, mult, false)?;
            if !c.is_some_and(|c| g.db.get(c).cmc >= 5) {
                discard_worst(g, p, mult)?;
            }
        }
        if t.has(Tag::Veyran) {
            let y = g.perm_mut(m); // Veyran's own magecraft +1/+1 (it doubles itself)
            y.eot_pt = (y.eot_pt.0 + mult as i32, y.eot_pt.1 + mult as i32);
        }
        if t.has(Tag::Sanar) && !copy && !g.perm(m).tapped {
            g.perm_mut(m).tapped = true; // Sanar: tap for a Treasure once you've cast an instant or sorcery
            add_treasure(g, p, 1)?;
        }
    }
    if has(g, p, Tag::Aether) && !copy {
        let n = g.player(p).spells_this_turn * base_mult;
        gain(g, p, n as i32)?;
    }
    Ok(())
}

/// p discards its n least useful cards (outside decks keep combo pieces and wished cards)
pub fn discard_worst(g: &mut Game, p: PlayerId, n: u32) -> Res {
    // HUMAN(phase 9): the person picks (hc.discard)
    let keep = ai::discard_keep(g, p);
    for _ in 0..n {
        if g.player(p).hand.is_empty() {
            return Ok(());
        }
        let hand = g.player(p).hand.clone();
        let lands: Vec<CardId> = hand.iter().copied().filter(|&x| g.db.get(x).land).collect();
        let nonl: Vec<CardId> = hand.iter().copied().filter(|&x| !g.db.get(x).land).collect();
        let cmc = |x: CardId| g.db.get(x).cmc as f64;
        if !keep.is_empty() {
            let spare: Vec<CardId> = nonl.iter().copied().filter(|x| !keep.contains(x)).collect();
            if spare.len() + lands.len() > 0 {
                let x = if lands.len() > 2 || spare.is_empty() {
                    lands.first().copied().unwrap_or_else(|| min_by(&nonl, cmc).unwrap())
                } else {
                    max_by(&spare, cmc).unwrap()
                };
                discard_cards(g, p, &[x])?;
                continue;
            }
        }
        let x = if lands.len() > 2 || nonl.is_empty() { lands[0] } else { max_by(&nonl, cmc).unwrap() };
        discard_cards(g, p, &[x])?;
    }
    Ok(())
}

/// burn: kill a worthwhile creature if possible, otherwise go face on the weakest opponent
pub fn bolt_something(g: &mut Game, p: PlayerId, dmg: i32) -> Res {
    let tg: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !untargetable(g, m) && etgh(g, m) <= dmg)
        .collect();
    let best = max_by(&tg, |m| pval(g, m));
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if let Some(b) = best.filter(|&b| pval(g, b) >= 3.0) {
        apply_removal(g, Some(p), b, &format!("dmg{dmg}"), None)
    } else if let Some(q) = min_by(&opps, |o| g.player(o).life as f64) {
        lose_life(g, q, dmg, Some(p), "burn", None)
    } else {
        Ok(())
    }
}

/// how much a spell matters: (to everyone, to particular players). Python's `spell_imp`.
pub fn spell_imp(g: &Game, p: PlayerId, c: CardId, ctx: &Ctx) -> (f64, Vec<(PlayerId, f64)>) {
    let d = g.db.get(c);
    let t = &d.tags;
    if let Some(tg) = ctx.target {
        return (0.0, vec![(g.perm(tg).owner, pval(g, tg))]);
    }
    if let Some(w) = t.str(Tag::Wipe) {
        let aff = g
            .opps(p)
            .map(|q| {
                let v: f64 = g
                    .player(q)
                    .perms
                    .iter()
                    .filter(|&&m| g.is_creature(m) || matches!(w, "rift" | "rebuke"))
                    .map(|&m| pval(g, m))
                    .sum();
                (q, 0.8 * v)
            })
            .collect();
        return (0.0, aff);
    }
    if ctx.rean_target.is_some() {
        return (ctx.rean_value, vec![]);
    }
    let key = g.player(p).key;
    if c == g.player(p).cmd {
        let v = match key {
            "seph" => 8.0,
            "veyran" => 5.0,
            "sauron" | "najeela" => 6.0,
            "galadriel" | "yshtola" | "alela" | "jodah" => 7.0,
            _ => CMD_IMP,
        };
        return (v, vec![]);
    }
    if d.bomb != 0 && key == "seph" {
        return (d.bomb as f64, vec![]);
    }
    let pair = |other: Tag| if has(g, p, other) { 9.0 } else { 4.0 };
    if t.has(Tag::Vkitten) {
        return (pair(Tag::Vfire), vec![]);
    }
    if t.has(Tag::Vfire) {
        return (pair(Tag::Vkitten), vec![]);
    }
    if t.has(Tag::Sword) {
        return (pair(Tag::Assault), vec![]);
    }
    if t.has(Tag::Assault) {
        return (pair(Tag::Sword), vec![]);
    }
    if t.has(Tag::Skate) {
        let big = army_of(g, p).is_some_and(|a| g.perm(a).plus >= 6);
        return (if big { 7.0 } else { 3.0 }, vec![]);
    }
    for (k, v) in [
        (Tag::Aether, 6.0),
        (Tag::Crusade, 6.0),
        (Tag::Rhystic, 5.0),
        (Tag::Tithe, 5.0),
        (Tag::Mirror, 5.0),
        (Tag::Witchking, 5.0),
        (Tag::Dragoncaller, 5.0),
        (Tag::Normgc, 7.0),
        (Tag::Mother, 7.0),
        (Tag::Onering, 6.0),
        (Tag::Sphinx, 6.0),
        (Tag::Breach, 5.0),
        (Tag::Panoptic, 5.0),
    ] {
        if t.has(k) {
            return (v, vec![]);
        }
    }
    if t.has(Tag::Tokx) && ctx.x >= 4 {
        return (5.0, vec![]);
    }
    (ai::combo_imp(g, p, c).max(ai::spell_importance(g, p, c)), vec![])
}

/// importance of an outside deck's commander spell (counter decisions)
pub const CMD_IMP: f64 = 6.0;

pub fn parse_cost(s: &str) -> (u32, String) {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    (digits.parse().unwrap_or(0), s[digits.len()..].to_string())
}

pub fn is_casts(g: &Game, p: PlayerId) -> i32 {
    match g.player(p).is_cast_n {
        Some((st, n)) if st == g.turn_stamp() => n,
        _ => 0,
    }
}

pub fn count_is_cast(g: &mut Game, p: PlayerId) {
    let st = g.turn_stamp();
    let n = is_casts(g, p);
    g.player_mut(p).is_cast_n = Some((st, n + 1));
}

// ------------------------------------------------------------------ casting a card
/// the graveyard a spell goes to: its owner's (a card cast from an opponent's deck goes back to theirs)
fn gy_of(ctx: &Ctx, p: PlayerId) -> PlayerId {
    ctx.owner.unwrap_or(p)
}

/// p casts card c (already paid for) from zone ('hand', 'gy', 'cmd', 'escape', 'mgy', 'lib'). It goes on the stack;
/// true if it resolved. Python's `cast_card`.
pub fn cast_card(g: &mut Game, p: PlayerId, c: CardId, zone: Sym, ctx: Ctx) -> Res<bool> {
    g.tick()?;
    let mut ctx = ctx;
    match zone {
        "hand" => {
            let pl = g.player_mut(p);
            let Some(i) = pl.hand.iter().position(|&x| x == c) else { return Ok(false) }; // paying moved it
            pl.hand.remove(i);
        }
        "gy" | "escape" | "mgy" => {
            let pl = g.player_mut(p);
            if let Some(i) = pl.gy.iter().position(|&x| x == c) {
                pl.gy.remove(i);
            }
            if zone == "mgy"
                && let Some(t) = ctx.muld_type
            {
                cardcode::muld_mark(g, p, t)?; // cast from the graveyard by permission (Muldrotha)
            }
        }
        "cmd" => {
            if &*g.db.get(c).name == "Liesa, Shroud of Dusk" && g.player(p).tax > 0 {
                let tax = g.player(p).tax as i32;
                lose_life(g, p, tax, Some(p), "other", None)?;
            }
            let pl = g.player_mut(p);
            pl.cmd_in_zone = false;
            pl.tax += 2;
        }
        _ => {} // 'lib': Bolas's Citadel, already off the top of the library
    }
    if g.player(p).agent_ids.contains(&c) && !g.player(p).hand.contains(&c) {
        // the card taken from an opponent is cast: other copies cost as usual
        g.player_mut(p).agent_ids.retain(|&x| x != c);
        if let Some(owner) = g.player_mut(p).stolen.swap_remove(&c)
            && owner != p
        {
            ctx.owner = Some(owner); // Gonti, Hostage Taker: still theirs
        }
    }
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(
        g,
        "  {} casts {}{}",
        g.player(p).name,
        g.db.get(c).name,
        ctx.target.map_or(String::new(), |t| format!(" -> {} ({})", g.perm(t).name, g.player(g.perm(t).owner).name))
    );
    let prev = g.cur_cast.replace((c, ctx.clone(), zone)); // cast triggers see its targets and X
    let r = (|| -> Res<Option<(bool, Option<CardId>, Ctx)>> {
        on_cast(g, p, c)?;
        if g.over || !g.player(p).alive {
            return Ok(None);
        }
        let (imp, aff) = spell_imp(g, p, c, &ctx);
        let d = g.db.get(c);
        let jin_kind = d.types.has(Types::ARTIFACT) || d.instant || d.sorcery;
        let jin: Vec<PlayerId> = g.opps(p).filter(|&q| has(g, q, Tag::Jin)).collect();
        if jin_kind && !jin.is_empty() && once_per_turn(g, jin[0], "jincounter") {
            crate::glog!(g, "    Jin-Gitaxias counters {}", g.db.get(c).name);
            if zone == "gy" {
                g.player_mut(p).exile.push(c)
            } else {
                g.player_mut(p).gy.push(c)
            }
            return Ok(Some((false, None, ctx.clone())));
        }
        g.last_counter = None;
        let id = g.stack_pushes + 1;
        let item = StackItem {
            id,
            controller: p,
            card: Some(c),
            ctx: ctx.clone(),
            zone,
            imp,
            aff: aff.clone(),
            generic: true,
            kind: StackKind::Spell,
            name: g.db.get(c).name.to_string(),
            passed: vec![],
            countered: false,
            countered_by: None,
        };
        let human_near = g.humans.any(); // PORT(phase 9): _copy_window
        if !(imp > 0.0 || !aff.is_empty() || human_near) || !counterable(g, &item) {
            return Ok(Some((true, None, ctx.clone()))); // nobody would answer it
        }
        let (ok, by) = stack_window_keep(g, p, item)?;
        Ok(Some((ok, by, ctx.clone())))
    })();
    let ctx_now = g.cur_cast.take().map(|x| x.1);
    g.cur_cast = prev;
    let Some((ok, by, _)) = r? else { return Ok(false) };
    let ctx = ctx_now.unwrap_or(ctx);
    if !ok {
        if by.is_none() && g.last_counter.is_none() && !g.player(p).gy.contains(&c) && !g.player(p).exile.contains(&c) {
            // Jin-Gitaxias already moved it
        }
        let last = by;
        g.last_counter = last;
        let lt = last.map(|x| g.db.get(x));
        let land = g.db.get(c).land;
        if c == g.player(p).cmd {
            g.player_mut(p).cmd_in_zone = true;
        } else if zone == "gy" || ctx.exile_after {
            g.player_mut(p).exile.push(c);
        } else if lt.is_some_and(|x| x.tag(Tag::Lapse)) && !land {
            g.player_mut(p).library.push(c);
        } else if lt.is_some_and(|x| &*x.name == "Venser, Shaper Savant" || x.tag(Tag::Remand)) {
            g.player_mut(p).hand.push(c); // Fatehold Charm: back to its owner's hand
        } else if g.bounced_spell {
            g.bounced_spell = false;
            g.player_mut(p).hand.push(c);
        } else if let Some(q) = ctx.desert_to.filter(|&q| g.player(q).alive) {
            enter(g, q, c, Enter { orig: Some(p), ..Enter::default() })?; // Desertion
            crate::glog!(g, "    {} enters under {}'s control (Desertion)", g.db.get(c).name, g.player(q).name);
        } else if lt.is_some_and(|x| x.tag(Tag::Ctrexile)) && !land {
            g.player_mut(p).exile.push(c); // Dissipate
        } else if !land {
            let o = gy_of(&ctx, p);
            g.player_mut(o).gy.push(c);
        }
        return Ok(false);
    }
    resolve(g, p, c, &ctx, zone)?;
    check_state(g)?;
    Ok(true)
}

/// Flashback (the card): recast the best affordable instant/sorcery from your graveyard
pub fn flashback_grant(g: &mut Game, p: PlayerId) -> Res {
    // HUMAN(phase 9): play/cards.flashback_target
    let cs: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&x| {
            let d = g.db.get(x);
            (d.instant || d.sorcery)
                && ![Tag::Ctr, Tag::Fbgrant, Tag::Rem, Tag::Wipe].iter().any(|&k| d.tag(k))
                && can_pay(g, p, d.generic, &d.pips, false)
        })
        .collect();
    let Some(x) = max_by(&cs, |c| {
        let d = g.db.get(c);
        d.tags.int(Tag::Draw).unwrap_or(0) as f64 * 100.0 + d.cmc as f64
    }) else {
        return Ok(());
    };
    let d = g.db.get(x);
    let (gn, pips) = (d.generic, d.pips.to_string());
    pay(g, p, gn, &pips, false)?;
    cast_card(g, p, x, "gy", Ctx::default())?;
    Ok(())
}

/// spell c resolves; the abilities it triggers go on the stack once it has finished
pub fn resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx, zone: Sym) -> Res {
    g.resolving += 1;
    let r = resolve_inner(g, p, c, ctx, zone);
    g.resolving -= 1;
    r?;
    if g.resolving == 0 && !g.trig_queue.is_empty() {
        crate::engine::stack::flush_triggers(g)?;
    }
    Ok(())
}

/// where a resolved spell goes
fn to_gy_or_exile(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx, zone: Sym, exile: bool) {
    if zone == "copy" {
        return; // copies move no card
    }
    if zone == "gy" || ctx.exile_after || exile {
        g.player_mut(p).exile.push(c);
    } else {
        let o = gy_of(ctx, p);
        g.player_mut(o).gy.push(c);
    }
}

fn resolve_inner(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx, zone: Sym) -> Res {
    let d = g.db.get(c);
    let t = d.tags.clone();
    let (perm, has_dsl) = (d.perm, d.dsl.is_some());
    if !perm && let Some(f) = g.registry.get(c).and_then(|i| i.resolve) {
        // hand-written spell (returns where it goes)
        let dest = f(g, p, c, ctx)?;
        if dest != "handled" && zone != "copy" {
            to_gy_or_exile(g, p, c, ctx, zone, dest == "exile");
        }
        return Ok(());
    }
    if has_dsl && !perm {
        crate::dsl::resolve_spell(g, p, c, ctx)?; // interpreter-driven instant / sorcery
        to_gy_or_exile(g, p, c, ctx, zone, false);
        return Ok(());
    }
    if perm && zone == "copy" {
        enter_token_copy(g, p, c)?; // a copy of a permanent spell becomes a token
        return Ok(());
    }
    if perm {
        if t.has(Tag::Moxd) {
            // Mox Diamond: discard a land card instead, or it goes to the graveyard
            let lands: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&x| g.db.get(x).land).collect();
            if lands.is_empty() {
                g.player_mut(p).gy.push(c);
                crate::glog!(g, "    Mox Diamond goes to the graveyard (no land to discard)");
                return Ok(());
            }
            let x = min_by(&lands, |l| {
                let lt = &g.db.get(l).tags;
                lt.str(Tag::C).map_or(0, |s| s.chars().count()) as f64 * 2.0 - lt.has(Tag::T) as i32 as f64
            })
            .unwrap();
            let pl = g.player_mut(p);
            let i = pl.hand.iter().position(|&y| y == x).unwrap();
            pl.hand.remove(i);
            pl.gy.push(x);
        }
        let prev = std::mem::replace(&mut g.cast_target, ctx.target); // an Aura's target, chosen as cast
        let r = enter(g, p, c, Enter { was_cast: true, orig: ctx.owner, ..Enter::default() });
        g.cast_target = prev;
        let m = r?;
        if c == g.player(p).cmd {
            g.perm_mut(m).is_cmd = true;
        }
        if let Some(n) = t.int(Tag::Lr) {
            land_ramp(g, p, n.max(0) as u32, t.has(Tag::Lrt))?;
        }
        return Ok(());
    }
    if let Some(n) = t.int(Tag::Draw) {
        let n = if zone == "gy" { t.int(Tag::Fbdraw).unwrap_or(n) } else { n };
        draw(g, p, n.max(0) as u32, false)?;
    }
    if let Some(n) = t.int(Tag::Treas) {
        add_treasure(g, p, n)?;
    }
    if let Some(n) = t.int(Tag::Lr) {
        land_ramp(g, p, n.max(0) as u32, t.has(Tag::Lrt))?;
        if t.has(Tag::Lh) {
            land_to_hand(g, p)?;
        }
    }
    if t.has(Tag::Ringtempt) {
        cardcode::ring_tempt(g, p)?; // Ringsight: the Ring tempts you first
    }
    if let Some(k) = t.str(Tag::Tut) {
        if t.has(Tag::Top) {
            tutor_to_top(g, p, k, 0)?; // Mystical Tutor: on top of the library
        } else {
            tutor(g, p, k)?;
        }
    }
    if t.has(Tag::Gifts) {
        pile_tutor(g, p, 4, 2)?; // Gifts Ungiven: four cards, opponent puts two in the graveyard
    }
    if t.has(Tag::Intuition) {
        pile_tutor(g, p, 3, 1)?; // Intuition: three cards, opponent picks the one you keep
    }
    if t.has(Tag::Jeska) {
        jeskas_will(g, p)?;
    }
    if t.has(Tag::Explore) {
        g.player_mut(p).extra_land_now += 1;
    }
    if t.has(Tag::Loam) {
        let mut ls: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&x| g.db.get(x).land).collect();
        ls.sort_by_key(|&x| std::cmp::Reverse(g.db.get(x).tags.str(Tag::C).map_or(0, |s| s.chars().count())));
        for x in ls.into_iter().take(3) {
            let pl = g.player_mut(p);
            let i = pl.gy.iter().position(|&y| y == x).unwrap();
            pl.gy.remove(i);
            pl.hand.push(x);
        }
    }
    if t.has(Tag::Rishkar) {
        let cr: Vec<PermId> =
            g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
        let n = cr.iter().map(|&m| epow(g, m)).max().unwrap_or(0);
        draw(g, p, n.max(0) as u32, false)?;
        let free: Vec<CardId> = g
            .player(p)
            .hand
            .iter()
            .copied()
            .filter(|&x| {
                let d = g.db.get(x);
                !d.land && d.cmc <= 5 && ![Tag::Ctr, Tag::Rem, Tag::Wipe].iter().any(|&k| d.tag(k))
            })
            .collect();
        if let Some(x) = max_by(&free, |x| g.db.get(x).cmc as f64 * 1000.0 + card_worth(g, p, x, false)) {
            cast_card(g, p, x, "hand", Ctx::default())?;
        }
    }
    if t.has(Tag::Threedreams) {
        cardcode::tutor_auras(g, p, 3)?;
    }
    if t.has(Tag::Seal) {
        tutor_to_top(g, p, "any", 2)?; // Imperial Seal / Vampiric Tutor: card on top, lose 2 life
    }
    if t.has(Tag::Adnaus) {
        ad_nauseam(g, p, None)?;
    }
    if let Some(n) = t.int(Tag::Lose)
        && !ctx.paid_otherwise
    {
        lose_life(g, p, n, Some(p), "other", None)?;
    }
    if let Some(n) = t.int(Tag::Selfdmg) {
        lose_life(g, p, n, Some(p), "other", None)?;
    }
    if t.has(Tag::Discard1) {
        discard_worst(g, p, 1)?;
    }
    let rem = ctx.rem_kind.or_else(|| t.str(Tag::Rem).map(intern));
    if let Some(face) = ctx.face {
        if !player_hexproof(g, face) {
            // Shalai: the burn has no legal target
            let n: i32 = rem.and_then(|k| k.strip_prefix("dmg")).and_then(|x| x.parse().ok()).unwrap_or(0);
            lose_life(g, face, n + has(g, p, Tag::Thor) as i32, Some(p), "burn", None)?;
        }
    } else if let (Some(kind), Some(tg)) = (rem, ctx.target) {
        apply_removal(g, Some(p), tg, kind, Some(c))?;
    }
    if let Some(w) = t.str(Tag::Wipe)
        && ctx.target.is_none()
        && ctx.face.is_none()
    {
        apply_wipe(g, p, intern(w), c, ctx)?;
    }
    if let Some(f) = t.str(Tag::Fill) {
        ai::seph_fill_resolve(g, p, intern(f), ctx)?;
    }
    if t.has(Tag::Rean) {
        ai::seph_rean_resolve(g, p, c, ctx)?;
    }
    if t.has(Tag::Burn) {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        if let Some(q) = min_by(&opps, |o| g.player(o).life as f64) {
            let n = 5 + total_mana(g, p, false) as i32;
            lose_life(g, q, n, Some(p), "burn", None)?;
        }
    }
    if t.has(Tag::Tokx) {
        make_tokens(
            g,
            p,
            Tokens {
                warrior: t.has(Tag::Warrior),
                sick: false,
                lifelink: t.has(Tag::Toklife),
                ..Tokens::new(ctx.x.max(0) as u32, 1)
            },
        )?;
    }
    if t.has(Tag::Clue) {
        g.player_mut(p).clues += 1;
    }
    if let Some(spec) = t.str(Tag::Mktok) {
        // auto-tagged token spells  n:power:flying
        let mut parts: Vec<i32> = spec.split(':').map(|x| x.parse().unwrap_or(1)).collect();
        parts.resize(3, 0);
        make_tokens(g, p, Tokens { fly: parts[2] == 1, sick: false, ..Tokens::new(parts[0].max(0) as u32, parts[1]) })?;
    }
    if let Some(n) = t.int(Tag::Drainetb) {
        for q in g.opps(p).collect::<Vec<_>>() {
            lose_life(g, q, n, Some(p), "drain", None)?;
        }
    }
    if t.has(Tag::Edictetb) {
        for q in g.opps(p).collect::<Vec<_>>() {
            edict(g, q, false)?;
        }
    }
    if let Some(n) = t.int(Tag::Pumpall) {
        g.player_mut(p).pumpadd += n;
    }
    if t.has(Tag::Lh) && !t.has(Tag::Lr) {
        land_to_hand(g, p)?;
    }
    if t.has(Tag::Crackle) {
        // HUMAN(phase 9): play/cards.crackle. X = (mana spent - 2)/3; 5X to each of up to X targets
        let x = ctx.x.max(1);
        let dmg = 5 * x;
        enum Tgt {
            Face(PlayerId),
            Perm(PermId),
        }
        let mut cands: Vec<(f64, Tgt)> = vec![];
        for q in g.opps(p).collect::<Vec<_>>() {
            let life = g.player(q).life;
            let v = if life <= dmg { 100.0 + (60 - life) as f64 } else { 0.4 * dmg as f64 };
            cands.push((v, Tgt::Face(q)));
        }
        for q in g.opps(p).collect::<Vec<_>>() {
            for &m in &g.player(q).perms {
                if g.is_creature(m)
                    && !g.perm(m).phased
                    && etgh(g, m) <= dmg
                    && pval(g, m) >= 3.0
                    && !untargetable(g, m)
                {
                    cands.push((1.5 * pval(g, m), Tgt::Perm(m)));
                }
            }
        }
        cands.sort_by(|a, b| b.0.total_cmp(&a.0)); // stable, highest first
        for (_, tgt) in cands.into_iter().take(x as usize) {
            match tgt {
                Tgt::Face(q) => lose_life(g, q, dmg, Some(p), "burn", None)?,
                Tgt::Perm(m) if g.perm(m).on_bf => apply_removal(g, Some(p), m, &format!("dmg{dmg}"), Some(c))?,
                _ => {}
            }
        }
    }
    if t.has(Tag::Drawcre) {
        // Shamanic Revelation
        let cr: Vec<PermId> =
            g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
        draw(g, p, cr.len() as u32, false)?;
        let big = cr.iter().filter(|&&m| epow(g, m) >= 4).count() as i32;
        gain(g, p, 4 * big)?;
    }
    if t.has(Tag::Stampede) {
        // Overwhelming Stampede: +X/+X, X = the greatest power
        let cr: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m)).collect();
        if !cr.is_empty() {
            let x = cr.iter().map(|&m| epow(g, m)).max().unwrap();
            let pl = g.player_mut(p);
            pl.pumpadd += x;
            pl.trample = true;
        }
    }
    if t.has(Tag::Prolif1) {
        cardcode::proliferate_all(g, p)?;
    }
    if t.has(Tag::Unearth) {
        // HUMAN(phase 9): the person picks (hc.pick_cards)
        let cs: Vec<CardId> =
            g.player(p).gy.iter().copied().filter(|&x| g.db.get(x).creature && g.db.get(x).cmc <= 3).collect();
        if let Some(x) = max_by(&cs, |x| {
            let d = g.db.get(x);
            d.tag(Tag::Bowmasters) as i32 as f64 * 5.0 + d.pow as f64 + ai::card_etb_value(g, p, x)
        }) {
            let pl = g.player_mut(p);
            let i = pl.gy.iter().position(|&y| y == x).unwrap();
            pl.gy.remove(i);
            enter(g, p, x, Enter::default())?;
        }
    }
    if t.has(Tag::Fbgrant) {
        flashback_grant(g, p)?;
    }
    if zone == "gy" && t.has(Tag::Fbnib) {
        // Nibelheim Aflame from the graveyard: discard your hand, draw four
        let hand = g.player(p).hand.clone();
        discard_cards(g, p, &hand)?;
        draw(g, p, 4, false)?;
    }
    if t.has(Tag::Yawg) {
        // Yawgmoth's Will: the graveyard as it is now can be played from
        let pl = g.player_mut(p);
        pl.yawg = true;
        pl.yawg_gy = pl.gy.clone();
    }
    if t.has(Tag::Mastery) {
        // Mizzix's Mastery: exile target instant/sorcery from your graveyard (each of them, overloaded); cast copies
        let mut pool: Vec<CardId> = g
            .player(p)
            .gy
            .iter()
            .copied()
            .filter(|&x| x != c && (g.db.get(x).instant || g.db.get(x).sorcery) && !g.db.get(x).tag(Tag::Ctr))
            .collect();
        if !ctx.overload {
            pool.sort_by(|&a, &b| card_worth(g, p, b, true).total_cmp(&card_worth(g, p, a, true)));
            pool.truncate(1);
        }
        for &x in &pool {
            let pl = g.player_mut(p);
            let i = pl.gy.iter().position(|&y| y == x).unwrap();
            pl.gy.remove(i);
            pl.exile.push(x);
        }
        if !pool.is_empty() {
            crate::glog!(g, "    Mizzix's Mastery casts copies of {} spell(s)", pool.len());
        }
        for x in pool {
            copy_spell(g, p, x, None, true)?;
            if g.over {
                return Ok(());
            }
        }
    }
    to_gy_or_exile(g, p, c, ctx, zone, false);
    Ok(())
}

/// Fresh targets for a copy of spell c ("you may choose new targets for the copy"): the best opposing permanent for
/// removal, the lowest life total for burn that kills nothing worthwhile.
pub fn spell_targets(g: &Game, p: PlayerId, c: CardId, ctx: Option<&Ctx>) -> Ctx {
    // HUMAN(phase 9): hc.copy_targets
    let t = &g.db.get(c).tags;
    let mut ctx = ctx.cloned().unwrap_or_default();
    ctx.target = None;
    ctx.face = None;
    if let Some(r) = t.str(Tag::Rem) {
        let kind = ctx.rem_kind.unwrap_or(r);
        let tg: Vec<PermId> = legal_targets(g, p, kind, t.str(Tag::Tgt).unwrap_or("c"), t.has(Tag::Mv4), Some(c))
            .into_iter()
            .filter(|&m| g.perm(m).owner != p)
            .collect();
        let best = max_by(&tg, |m| pval(g, m));
        let opps: Vec<PlayerId> = g.opps(p).filter(|&q| !player_hexproof(g, q)).collect();
        if let Some(b) = best
            && !(t.has(Tag::Face) && pval(g, b) < 3.0 && !opps.is_empty())
        {
            ctx.target = Some(b);
        } else if t.has(Tag::Face) && !opps.is_empty() {
            ctx.face = min_by(&opps, |o| g.player(o).life as f64);
        }
    }
    ctx
}

/// A copy of spell c with new targets. cast=false: a copy put on the stack (Thousand-Year Storm, Jin-Gitaxias, Ral):
/// only "cast or copy" triggers see it. cast=true: "you may cast the copy" (Mizzix's Mastery): a real cast. No card
/// moves either way.
pub fn copy_spell(g: &mut Game, p: PlayerId, c: CardId, ctx: Option<&Ctx>, cast: bool) -> Res {
    if g.db.get(c).perm && !cast {
        enter_token_copy(g, p, c)?;
        return Ok(());
    }
    let ctx = spell_targets(g, p, c, ctx);
    if cast {
        let pl = g.player_mut(p);
        pl.spells_this_turn += 1;
        pl.stat("spells_cast", 1);
        on_cast(g, p, c)?;
    } else {
        magecraft(g, p, Some(c), true)?;
    }
    let t = &g.db.get(c).tags;
    if g.over || t.has(Tag::Ctr) || (t.has(Tag::Crackle) && ctx.x == 0) {
        return Ok(()); // nothing to counter; X copies as 0
    }
    resolve(g, p, c, &ctx, "copy")
}

/// cast a copy of instant/sorcery card c (Panoptic Mirror): a real cast that can be countered; no card moves
pub fn cast_spell_copy(g: &mut Game, p: PlayerId, c: CardId, ctx: Option<&Ctx>) -> Res<bool> {
    let ctx = ctx.cloned().unwrap_or_default();
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    crate::glog!(g, "  {} casts a copy of {}", g.player(p).name, g.db.get(c).name);
    on_cast(g, p, c)?;
    if g.over || !g.player(p).alive {
        return Ok(false);
    }
    let (imp, aff) = spell_imp(g, p, c, &ctx);
    if (imp > 0.0 || !aff.is_empty()) && !counter_window(g, p, c, imp, aff)? {
        return Ok(false);
    }
    let (n_gy, n_ex) = (g.player(p).gy.len(), g.player(p).exile.len());
    resolve(g, p, c, &ctx, "hand")?;
    // the copy ceases to exist instead of going to a zone
    for exile in [false, true] {
        let pl = g.player_mut(p);
        let (zone, n0) = if exile { (&mut pl.exile, n_ex) } else { (&mut pl.gy, n_gy) };
        if let Some(i) = (n0..zone.len()).rev().find(|&i| zone[i] == c) {
            zone.remove(i);
            break;
        }
    }
    check_state(g)?;
    Ok(true)
}

/// "You may cast a copy of ..." (a back-face instant/sorcery): a real cast (magecraft, opponents' cast triggers,
/// Thousand-Year Storm), no card moves. Named, it goes on the stack, where it can be countered. PORT(M5): its card
/// effects come with the cards that use it (Emeritus of Conflict, Kefka).
pub fn cast_copy_counted(g: &mut Game, p: PlayerId) -> Res {
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    count_is_cast(g, p);
    magecraft(g, p, None, false)?;
    for q in g.opps(p).collect::<Vec<_>>() {
        if has(g, q, Tag::Sauron) {
            amass(g, q, 1)?;
        }
        if has(g, q, Tag::Rhystic) && cardcode::rhystic_unpaid(g, p)? {
            draw(g, q, 1, false)?;
        }
    }
    check_state(g)
}
