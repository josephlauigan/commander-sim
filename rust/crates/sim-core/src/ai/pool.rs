//! The outside decks' AI (Python's `ai/pool_ai.py`, and `cardimpl.spell_importance`): a cast priority built from
//! card tags, per-deck configuration (`plans`), protection and wipe answers from a table of protectors, generic plays
//! (equip, Skullclamp, Deadly Dispute, reanimation, X spells) and attack rules.

use super::brain::Situation;
use super::{Style, decks, plans};
use crate::cards::Types;
use crate::engine::cast::{castable, discard_worst, on_cast, sac_fodder};
use crate::engine::combat::can_block;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::ability_window;
use crate::engine::values::{commander_out, epow, etgh, find, has_type, pval, stopped, untargetable};
use crate::engine::zones::{Enter, die, draw, enter, leave, max_by, min_by, searchable, teferis_protection};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt};
use crate::ids::{CardId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{Ctx, Game};
use crate::sym::Sym;
use crate::tag::Tag;
use std::cell::Cell;

pub const DEFAULT_STYLE: Style = Style { temp: 1.0, aggression: 0.6, caution: 0.5 };

// ------------------------------------------------------------------ cast priority
thread_local! {
    /// valuing a tutor as another tutor's target: no recursion (pool_ai._TUTOR_DEPTH)
    static TUTOR_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// pool_ai.tutor_prio: high when the deck's wish list or a missing combo piece is in the library, otherwise by the
/// impact of the best card it would find; lower with a full hand, and not when its life cost would leave you exposed
pub fn tutor_prio(g: &Game, p: PlayerId, kind: &str, life: i32) -> i32 {
    if TUTOR_DEPTH.get() > 0 {
        return 45;
    }
    if life > 0 && g.player(p).life - life < 6 {
        return 0; // Vampiric Tutor / Imperial Seal: 2 life is a real cost only near death
    }
    let kind = match kind {
        "is" | "art" | "perm" | "ubr" | "cre2" | "cre" | "ench" => kind,
        _ => "any",
    };
    TUTOR_DEPTH.set(TUTOR_DEPTH.get() + 1);
    let v = if tutor_pick(g, p, |c| decks::tutor_ok(g, kind, c)).is_some() {
        Some(72.0)
    } else {
        let cands: Vec<CardId> =
            searchable(g, p).into_iter().filter(|&c| decks::tutor_ok(g, kind, c) && !g.db.get(c).land).collect();
        if cands.is_empty() {
            None
        } else {
            let best = cands.iter().map(|&c| decks::tutor_value(g, p, c)).fold(f64::NEG_INFINITY, f64::max);
            Some(25.0 + (best / 2.5).min(40.0))
        }
    };
    TUTOR_DEPTH.set(TUTOR_DEPTH.get() - 1);
    let Some(mut v) = v else { return 0 };
    if g.player(p).hand.len() >= 7 {
        v -= 10.0;
    }
    v as i32
}

fn creatures_of(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m)).count()
}

/// pool_ai.generic_prio: a 0-90 cast priority from what the card is tagged to do. 0: not cast proactively (held
/// interaction, or no tags: the heuristic AI then falls back to the ability language's value estimate).
pub fn generic_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let pl = g.player(p);
    let d = g.db.get(c);
    let t = &d.tags;
    let cfg = plans::config(pl.key);
    if let Some(&(_, v)) = cfg.key_cards.iter().find(|x| *x.0 == *d.name) {
        return v;
    }
    if let Some(f) = cfg.prio_fn
        && let Some(r) = f(g, p, c)
    {
        return r;
    }
    let imp = g.registry.get(c);
    if let Some(f) = imp.and_then(|i| i.prio) {
        return f(g, p, c);
    }
    if c == pl.cmd {
        let dflt = if imp.is_some() { 85 } else { 75 }; // an engine commander comes first
        return if pl.turns >= cfg.cmd_turn.unwrap_or(2) {
            40.max(cfg.cmd_prio.unwrap_or(dflt) - 3 * pl.tax as i32)
        } else {
            0
        };
    }
    if d.perm {
        let ci = crate::cardcode::combo_imp(g, p, c);
        if ci >= 7.0 {
            return (50.0 + 4.0 * ci) as i32; // a combo piece first (Grim Monolith with Power Artifact)
        }
    }
    if t.has(Tag::Chromemox) {
        // before 'rock': its imprint decides
        let spare = pl
            .hand
            .iter()
            .filter(|&&x| {
                let xd = g.db.get(x);
                !xd.land && !xd.types.has(Types::ARTIFACT) && xd.pips.chars().any(|ch| pl.ident.has(ch))
            })
            .count();
        return if spare >= 2 { plans::ramp_prio(g, p, c, 1) } else { 0 };
    }
    if t.has(Tag::Moxd) {
        // Mox Diamond: needs a land to discard (it carries rock=1:A too, so this check comes first)
        let n = pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
        return if n >= 2 || (n >= 1 && pl.land_turn == pl.turns as i32) { plans::ramp_prio(g, p, c, 1) } else { 0 };
    }
    if let Some(r) = t.str(Tag::Rock)
        && r.split(':').next().and_then(|x| x.parse::<i32>().ok()).unwrap_or(0) >= 2
        && d.cmc <= 1
    {
        return 88; // Sol Ring, Mana Vault: always
    }
    if t.has(Tag::Grim) {
        return plans::ramp_prio(g, p, c, 3); // Grim Monolith: three now, {4} to untap
    }
    if t.has(Tag::Rock) || t.has(Tag::Dork) || t.has(Tag::Lr) || t.has(Tag::Fastmana) {
        return if pl.turns <= 5 { 85 } else { 38 };
    }
    if t.str(Tag::Prot) == Some("boots") || t.has(Tag::Cloak) {
        return if creatures_of(g, p) > 0 { 45 } else { 25 };
    }
    if let Some(rk) = t.str(Tag::Rem)
        && t.has(Tag::Etb)
        && d.perm
    {
        // a creature or enchantment with an enters removal
        let tg = legal_targets(g, p, rk, t.str(Tag::Tgt).unwrap_or("c"), t.has(Tag::Mv4), Some(c));
        let best = tg.iter().map(|&m| pval(g, m)).fold(0.0, f64::max);
        return if best >= 2.0 {
            45 + (6.0 * best).min(30.0) as i32
        } else if d.creature {
            35
        } else {
            0
        };
    }
    if t.has(Tag::Ctr) || t.has(Tag::Rem) || t.has(Tag::Wipe) || t.has(Tag::Prot) {
        return 0; // held / cast by the response logic
    }
    const SPECIAL: [Tag; 6] = [Tag::Rean, Tag::Fill, Tag::Yawg, Tag::Avarice, Tag::Mastery, Tag::Crackle];
    if SPECIAL.iter().any(|&k| t.has(k)) && !d.has_dsl() && !d.creature {
        return 0;
    }
    if t.has(Tag::Tokx) {
        return if total_mana(g, p, false) >= 5 { 50 } else { 0 };
    }
    if t.has(Tag::Tithe) {
        return crate::cardcode::tithe_prio(g, p, c); // Smothering Tithe: from the Treasures it will make
    }
    if t.has(Tag::Necro) {
        return decks::necro_prio(g, p, c);
    }
    if t.has(Tag::Rhystic) {
        return plans::rhystic_prio(g, p, c);
    }
    if t.has(Tag::Eng) {
        return 64;
    }
    if t.has(Tag::Seal) {
        return tutor_prio(g, p, "any", 2); // Vampiric Tutor / Imperial Seal
    }
    if t.has(Tag::Jeska) {
        return plans::jeska_prio(g, p, c);
    }
    if t.has(Tag::Intuition) || t.has(Tag::Gifts) {
        return tutor_prio(g, p, "any", 0);
    }
    if t.has(Tag::Adnaus) {
        return plans::adnaus_prio(g, p, c);
    }
    if t.has(Tag::Citadel) {
        return plans::citadel_prio(g, p, c);
    }
    if t.has(Tag::Agent) {
        return plans::agent_prio(g, p, c);
    }
    if t.has(Tag::Braids) {
        return plans::braids_prio(g, p, c);
    }
    if t.has(Tag::Birgi) {
        return 45;
    }
    if t.has(Tag::Rabble) {
        return 55;
    }
    if t.has(Tag::Stampede) {
        return if creatures_of(g, p) >= 6 { 55 } else { 0 };
    }
    if t.has(Tag::Drawcre) {
        return if creatures_of(g, p) >= 4 { 55 } else { 20 };
    }
    if t.has(Tag::Clamp) {
        return 55;
    }
    if t.has(Tag::Crusade) || t.has(Tag::Anth) || t.has(Tag::Warleader) {
        return 60;
    }
    const BOARD: [Tag; 9] = [
        Tag::Tokup,
        Tag::Tokatk,
        Tag::Tok,
        Tag::Spelltok,
        Tag::Ping,
        Tag::Kiln,
        Tag::Spelldraw,
        Tag::Drain,
        Tag::Bartist,
    ];
    if BOARD.iter().any(|&k| t.has(k)) {
        return 62;
    }
    if let Some(k) = t.str(Tag::Tut) {
        return tutor_prio(g, p, k, 0);
    }
    if t.has(Tag::Draw) && (d.instant || d.sorcery) {
        return 46;
    }
    if t.has(Tag::Draw) {
        return 52;
    }
    if t.has(Tag::Treas) || t.has(Tag::Mktok) || t.has(Tag::Drainetb) || t.has(Tag::Edictetb) {
        return 50;
    }
    if t.has(Tag::Xtutor) {
        return 0; // its own priority decides (X spells)
    }
    if t.has(Tag::Aura) {
        // an Aura needs a creature to go on
        if !pl.perms.iter().any(|&m| g.is_creature(m) && !g.perm(m).phased) {
            return 0;
        }
        return cfg.aura_prio.unwrap_or(55);
    }
    if t.has(Tag::Threedreams) {
        return 60;
    }
    if t.has(Tag::Explore) {
        return if pl.hand.iter().any(|&x| x != c && g.db.get(x).land) { 50 } else { 30 };
    }
    if t.has(Tag::Loam) {
        return if pl.gy.iter().filter(|&&x| g.db.get(x).land).count() >= 2 { 45 } else { 0 };
    }
    if t.has(Tag::Rishkar) {
        let big = pl.perms.iter().filter(|&&m| g.is_creature(m)).map(|&m| epow(g, m)).max().unwrap_or(0);
        return if big >= 4 { 55 } else { 0 };
    }
    if t.has(Tag::Krasis) || t.has(Tag::Combatspell) {
        return 0; // cast by special_options with X / its own priority
    }
    if t.has(Tag::Fable) {
        return 58;
    }
    if d.perm {
        let ci = crate::cardcode::combo_imp(g, p, c);
        if ci >= 7.0 {
            return (50.0 + 4.0 * ci) as i32; // before the creature rule (Mikaeus, Kiki-Jiki, Felidar)
        }
    }
    if d.creature {
        return 42 + (2 * d.pow).min(16) + if t.has(Tag::Fly) { 4 } else { 0 };
    }
    if d.perm
        && let Some(h) = imp
    {
        if h.attack_tax.is_some() || h.attack_cap.is_some() {
            // pillowfort: worth what it keeps off you
            let power: i32 = g
                .opps(p)
                .flat_map(|q| g.player(q).perms.iter().copied())
                .filter(|&m| g.is_creature(m) && !g.perm(m).phased)
                .map(|m| epow(g, m))
                .sum();
            let n = g.opps(p).count().max(1) as f64;
            return 45 + ((1.5 * power as f64 / n) as i32).min(25);
        }
        if (h.cost.is_some() || h.can_cast.is_some() || h.min_cost.is_some()) && !d.creature {
            return if pl.turns <= 5 { 68 } else { 52 }; // stax: best early
        }
        return cfg.hooked_prio.unwrap_or(55);
    }
    if d.perm && crate::cardcode::is_sac_outlet(&d.name) {
        return 50; // sacrifice outlets (Goblin Bombardment ...)
    }
    if d.perm {
        let ci = crate::cardcode::combo_imp(g, p, c);
        if ci > 0.0 {
            return (50.0 + 4.0 * ci) as i32; // a combo piece: 9 = completes it, 7 = one short
        }
        if crate::cardcode::is_combo_piece(g, c) {
            return 48;
        }
    }
    if t.str(Tag::Prot) == Some("boots") || t.has(Tag::Sac) {
        return 40;
    }
    0 // compiled abilities: the interpreter's value decides; combat tricks: no proactive value
}

// ------------------------------------------------------------------ what opponents think of a card
/// cardimpl._threat_value: what removing a permanent of card c is worth beyond its tags, fixed per card: card code
/// that taxes attacks, locks or runs an engine, and compiled abilities
pub fn card_threat_value(g: &Game, c: CardId) -> f64 {
    let imp = g.registry.get(c);
    if let Some(v) = imp.and_then(|i| i.pval) {
        return v;
    }
    let mut v: f64 = 0.0;
    if let Some(h) = imp {
        if h.attack_tax.is_some() || h.attack_cap.is_some() {
            v = v.max(4.5);
        }
        const LOCK: [Event; 6] =
            [Event::Cost, Event::CanCast, Event::MinCost, Event::Uncounterable, Event::NoLifegain, Event::NoGraveyard];
        if LOCK.iter().any(|&e| h.handles(e)) {
            v = v.max(5.0);
        }
        // the events with slots so far; M5 adds sacrifice, extra_lands, discard, land_gy
        const ENGINE: [Event; 13] = [
            Event::Options,
            Event::Cast,
            Event::Dies,
            Event::Etb,
            Event::Attack,
            Event::Upkeep,
            Event::EndStep,
            Event::Landfall,
            Event::Draw,
            Event::CombatDamage,
            Event::TriggerCopies,
            Event::LandMana,
            Event::GrantKw,
        ];
        if ENGINE.iter().any(|&e| h.handles(e)) {
            v = v.max(3.0);
        }
    }
    let d = g.db.get(c);
    if d.has_dsl() && !d.creature {
        use crate::dsl::model::Ability;
        let n: f64 = d
            .abilities
            .iter()
            .filter(|a| a.static_kind() != Some("note"))
            .map(|a| match a {
                Ability::Triggered { .. } | Ability::Static { .. } | Ability::Replacement { .. } => 1.5,
                Ability::Activated { .. } => 1.0,
                _ => 0.0,
            })
            .sum();
        v = v.max(6.0f64.min(1.0 + n));
    }
    v
}

/// cardimpl.spell_importance: how much opponents want to counter spell c (0-9; 6-7 is the usual threshold)
pub fn spell_importance(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let imp = g.registry.get(c);
    if let Some(f) = imp.and_then(|i| i.spell_imp) {
        return f(g, p, c);
    }
    let d = g.db.get(c);
    let mut v: f64 = 0.0;
    if d.perm {
        v = (d.bomb as f64).max(card_threat_value(g, c) + 1.0);
        if d.creature {
            v = v.max(0.9 * d.pow as f64 + if d.tag(Tag::Fly) { 1.0 } else { 0.0 } - 0.5);
        }
        if c == g.player(p).cmd {
            v = (v + 1.0).max(6.0);
        }
    } else {
        let t = &d.tags;
        if t.has(Tag::Tut) || t.has(Tag::Seal) {
            v = 5.0;
        }
        if let Some(n) = t.int(Tag::Draw) {
            v = v.max(if n > 0 { 1.5 * n as f64 } else { 3.0 });
        }
        if imp.is_some_and(|i| i.resolve.is_some()) {
            v = v.max(4.0);
        }
    }
    v.min(9.0)
}

// ------------------------------------------------------------------ tutors
/// pool_ai.tutor_pick: the deck's wish list first (combo pieces it's missing): the first one it may find that isn't
/// in hand or on the battlefield; None: the caller falls back to priority
pub fn tutor_pick(g: &Game, p: PlayerId, ok: impl Fn(CardId) -> bool) -> Option<CardId> {
    let names: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| ok(c)).collect();
    let pl = g.player(p);
    let have = |c: CardId| pl.hand.contains(&c) || pl.perms.iter().any(|&m| g.perm(m).cd == Some(c));
    plans::wish_raw(g, p).into_iter().find(|&w| names.contains(&w) && !have(w))
}

// ------------------------------------------------------------------ generic plays
/// equip costs of the protection Equipment the outside decks play
const EQUIP_COST: [(&str, u32); 3] = [("Lightning Greaves", 0), ("Swiftfoot Boots", 1), ("Whispersilk Cloak", 2)];

fn can_pay_any(g: &Game, p: PlayerId, c: CardId) -> bool {
    let (cg, cp) = cost_of(g, p, c);
    can_pay(g, p, cg, &cp, false) && castable(g, p, c, "hand")
}

/// pool_ai.special_options: generic activated plays for outside decks: equip Boots / Greaves / Cloak, Skullclamp,
/// special spells, and the card code's own (adventures, evoke, aristocrats, Food, miracles, incubate)
pub fn special_options(g: &mut Game, p: PlayerId, s: &Situation, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut o = spell_options(g, p, s, post);
    o.extend(crate::cardcode::pool_card_options(g, p, post)?);
    let Some(_post) = post else {
        if crate::cardcode::aid_active(g, p) {
            // Sigarda's Aid: Auras at instant speed
            let any_cr = g.player(p).perms.iter().any(|&m| g.is_creature(m));
            for c in g.player(p).hand.clone() {
                if g.db.get(c).has_subtype("aura") && can_pay_any(g, p, c) && any_cr {
                    o.push(Opt {
                        utility: 2.0,
                        label: format!("{} (flash)", g.db.get(c).name),
                        act: Some(Action::Cast { card: c, zone: None }),
                    });
                }
            }
        }
        return Ok(o); // end-of-turn window: nothing below is instant speed
    };
    let post = post.unwrap();
    for e in g.player(p).perms.clone() {
        let x = g.perm(e);
        let Some(cd) = x.cd else { continue };
        if x.phased {
            continue;
        }
        let Some(&(_, n)) = EQUIP_COST.iter().find(|y| *y.0 == *g.db.get(cd).name) else { continue };
        if x.attached.is_some_and(|a| g.perm(a).on_bf && g.perm(a).owner == p && !g.perm(a).phased) {
            continue;
        }
        if !can_pay(g, p, n, "", false) {
            continue;
        }
        let cr: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| g.is_creature(m) && !g.perm(m).phased && !g.perm(m).noatk)
            .collect();
        let Some(best) = max_by(&cr, |m| {
            let y = g.perm(m);
            y.is_cmd as i32 as f64 * 5.0 + pval(g, m) + if y.sick { 2.0 } else { 0.0 }
        }) else {
            continue;
        };
        let u = 2.0 + 0.4 * pval(g, best) + if g.perm(best).sick && !post { 1.5 } else { 0.0 } - 0.5 * n as f64;
        o.push(Opt {
            utility: u,
            label: format!("equip {}", g.db.get(cd).name),
            act: Some(Action::Plan { f: equip_protection, arg: pack(e, best, n) }),
        });
    }
    for e in find(g, p, Tag::Clamp) {
        // Skullclamp: equip {1} to an X/1, it dies, draw two
        if stopped(g, g.perm(e).name) || !can_pay(g, p, 1, "", false) {
            continue;
        }
        let fod: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                g.is_creature(m) && !g.perm(m).phased && etgh(g, m) == 1 && (g.perm(m).token || pval(g, m) < 3.0)
            })
            .collect();
        let Some(m) = min_by(&fod, |x| pval(g, x)) else { continue };
        o.push(Opt {
            utility: 4.0 - 0.3 * pval(g, m),
            label: "Skullclamp".into(),
            act: Some(Action::Plan { f: skullclamp, arg: pack(e, m, 0) }),
        });
    }
    Ok(o)
}

/// two permanents and a small number in one plan argument
fn pack(a: PermId, b: PermId, n: u32) -> i64 {
    (a.0 as i64) | ((b.0 as i64) << 24) | ((n as i64) << 48)
}

fn unpack(x: i64) -> (PermId, PermId, u32) {
    (PermId((x & 0xff_ffff) as u32), PermId(((x >> 24) & 0xff_ffff) as u32), (x >> 48) as u32)
}

fn equip_protection(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (e, best, n) = unpack(arg);
    if !g.perm(best).on_bf || g.perm(best).owner != p || !can_pay(g, p, n, "", false) {
        return Ok(false);
    }
    pay(g, p, n, "", false)?;
    let en = g.perm(e).name;
    crate::glog!(g, "  {} equips {} to {}", g.player(p).name, en, g.perm(best).name);
    let label = format!("equip to {}", g.perm(best).name);
    if ability_window(g, p, Some(e), &label, None, Some(best))? && g.perm(best).on_bf && g.perm(e).on_bf {
        g.perm_mut(e).attached = Some(best);
    }
    Ok(true)
}

fn skullclamp(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (e, m, _) = unpack(arg);
    if !g.perm(m).on_bf || g.perm(m).owner != p || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    g.perm_mut(e).attached = Some(m);
    crate::glog!(g, "  {} Skullclamps {}", g.player(p).name, g.perm(m).name);
    die(g, m, "sba")?;
    draw(g, p, 2, false)?;
    g.player_mut(p).stat("clamp_draws", 2);
    Ok(true)
}

/// pool_ai.spell_options: hand-tagged spells whose casting logic lives in your decks' AI, made usable by outside
/// decks: Grisly Salvage, Deadly Dispute, reanimation, Hydroid Krasis, Primal Might, Return of the Wildspeaker
pub fn spell_options(g: &Game, p: PlayerId, s: &Situation, post: Option<bool>) -> Vec<Opt> {
    let mut o = vec![];
    let Some(post) = post else { return o };
    let pl = g.player(p);
    for &c in &pl.hand {
        let d = g.db.get(c);
        let t = &d.tags;
        let fill = t.str(Tag::Fill);
        let payable = || castable(g, p, c, "hand") && can_pay(g, p, d.generic, &d.pips, false);
        if fill == Some("grisly")
            && payable()
            && (pl.lands.len() < 6
                || pl
                    .perms
                    .iter()
                    .any(|&m| g.perm(m).cd.is_some_and(|x| &*g.db.get(x).name == "Meren of Clan Nel Toth")))
        {
            o.push(Opt {
                utility: 2.5,
                label: d.name.to_string(),
                act: Some(Action::Plan { f: plain_cast, arg: c.0 as i64 }),
            });
        }
        if fill == Some("dispute") && payable() {
            let ok = match sac_fodder(g, p, "artifact or creature", None) {
                None => false,
                Some(crate::engine::cast::Fodder::Treasure) => true,
                Some(crate::engine::cast::Fodder::Perm(m)) => pval(g, m) < 3.0,
            };
            if ok {
                o.push(Opt {
                    utility: 3.5,
                    label: d.name.to_string(),
                    act: Some(Action::Plan { f: dispute, arg: c.0 as i64 }),
                });
            }
        }
        if let Some(r) = t.str(Tag::Rean)
            && matches!(r, "animate" | "reanimate" | "evil" | "zombify")
            && payable()
        {
            let mut tg = decks::rean_targets(g, p, r);
            if r == "reanimate" {
                tg.retain(|x| pl.life - g.db.get(x.1).cmc as i32 >= 12);
            }
            if let Some(&(v, cd, src)) = tg.first() {
                o.push(Opt {
                    utility: v * 0.9 * (1.0 - 0.35 * s.ctr_risk),
                    label: format!("{} -> {}", d.name, g.db.get(cd).name),
                    act: Some(Action::Plan {
                        f: reanimate,
                        arg: (c.0 as i64)
                            | ((cd.0 as i64) << 16)
                            | ((src.0 as i64) << 32)
                            | (((v * 100.0) as i64) << 40),
                    }),
                });
            }
        }
        if t.has(Tag::Krasis) && castable(g, p, c, "hand") {
            // Hydroid Krasis: X = spare mana
            let x = total_mana(g, p, false) as i32 - 2;
            if x >= 4 {
                o.push(Opt {
                    utility: 1.0 + 0.6 * x as f64,
                    label: format!("Hydroid Krasis X={x}"),
                    act: Some(Action::Plan { f: krasis, arg: c.0 as i64 }),
                });
            }
        }
        if t.has(Tag::Primalmight) && castable(g, p, c, "hand") {
            // Primal Might: pump and fight
            let mine: Vec<PermId> =
                pl.perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
            let x = total_mana(g, p, false) as i32 - 1;
            if let Some(me) = max_by(&mine, |m| epow(g, m) as f64)
                && x >= 1
            {
                let tg: Vec<PermId> = g
                    .opps(p)
                    .flat_map(|q| g.player(q).perms.iter().copied())
                    .filter(|&m| g.is_creature(m) && !untargetable(g, m) && etgh(g, m) <= epow(g, me) + x)
                    .collect();
                if let Some(foe) = max_by(&tg, |m| pval(g, m))
                    && pval(g, foe) >= 3.0
                {
                    o.push(Opt {
                        utility: pval(g, foe) - 2.5,
                        label: format!("Primal Might -> {}", g.perm(foe).name),
                        act: Some(Action::Plan {
                            f: primal_might,
                            arg: (c.0 as i64) | ((me.0 as i64) << 16) | ((foe.0 as i64) << 40),
                        }),
                    });
                }
            }
        }
        if t.has(Tag::Rotw) && payable() {
            // Return of the Wildspeaker
            let nh: Vec<PermId> = pl
                .perms
                .iter()
                .copied()
                .filter(|&m| g.is_creature(m) && !g.perm(m).phased && !has_type(g, m, "human"))
                .collect();
            if nh.is_empty() {
                continue;
            }
            let draw_n = nh.iter().map(|&m| epow(g, m)).max().unwrap_or(0);
            let atk = nh.iter().filter(|&&m| !g.perm(m).sick && !g.perm(m).tapped && !g.perm(m).noatk).count();
            if !post && atk >= 3 {
                o.push(Opt {
                    utility: 1.0 + atk as f64,
                    label: format!("{} (+3/+3)", d.name),
                    act: Some(Action::Plan { f: wildspeaker_pump, arg: c.0 as i64 }),
                });
            } else if draw_n >= 3 {
                o.push(Opt {
                    utility: 1.0 + 0.8 * draw_n as f64,
                    label: format!("{} (draw {draw_n})", d.name),
                    act: Some(Action::Plan { f: wildspeaker_draw, arg: c.0 as i64 }),
                });
            }
        }
    }
    o
}

fn card_arg(arg: i64) -> CardId {
    CardId((arg & 0xffff) as u16)
}

/// cast a card from hand for its printed cost, no choices
fn plain_cast(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = card_arg(arg);
    let d = g.db.get(c);
    let (gn, pips) = (d.generic, d.pips.to_string());
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    crate::engine::cast::cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

/// Deadly Dispute: sacrifice an artifact or creature (a Treasure first), then cast it
fn dispute(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    use crate::engine::cast::Fodder;
    let c = card_arg(arg);
    let d = g.db.get(c);
    let (gn, pips) = (d.generic, d.pips.to_string());
    let fod = sac_fodder(g, p, "artifact or creature", None);
    if !g.player(p).hand.contains(&c) || fod.is_none() {
        return Ok(false);
    }
    // the sacrificed Treasure can't also pay for the spell (Python pays first and can sacrifice a Treasure it spent,
    // leaving -1 Treasures; a u32 here would wrap to four billion)
    let tre = fod == Some(Fodder::Treasure);
    g.player_mut(p).treasures -= tre as u32;
    if !can_pay(g, p, gn, &pips, false) {
        g.player_mut(p).treasures += tre as u32;
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    match fod.unwrap() {
        Fodder::Treasure => {
            if !g.hooks.is_empty() {
                crate::engine::hooks::fire_trigger(
                    g,
                    Event::Sacrifice,
                    crate::hooks::Call::Sacrifice { p, what: crate::hooks::Sacrificed::Token("Treasure") },
                )?;
            }
        }
        Fodder::Perm(m) => die(g, m, "sac")?,
    }
    crate::engine::cast::cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

/// a reanimation spell at a creature card in a graveyard
fn reanimate(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = card_arg(arg);
    let cd = CardId(((arg >> 16) & 0xffff) as u16);
    let src = PlayerId(((arg >> 32) & 0xff) as u8);
    let v = (arg >> 40) as f64 / 100.0;
    let d = g.db.get(c);
    let (gn, pips) = (d.generic, d.pips.to_string());
    if !g.player(p).hand.contains(&c) || !g.player(src).gy.contains(&cd) || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    if !g.hooks.is_empty() && crate::cardcode::gy_response(g, p, v, src)? {
        decks::remove_from_hand(g, p, c);
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    let ctx = Ctx { rean_target: Some(cd), rean_src: Some(src), rean_value: v, ..Ctx::default() };
    crate::engine::cast::cast_card(g, p, c, "hand", ctx)?;
    Ok(true)
}

/// Hydroid Krasis for X = all spare mana: draw and gain X/2, a creature with X counters
fn krasis(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = card_arg(arg);
    let mut x = total_mana(g, p, false) as i32 - 2;
    while x >= 1 && !can_pay(g, p, x as u32, "GU", false) {
        x -= 1;
    }
    if !g.player(p).hand.contains(&c) || x < 1 {
        return Ok(false);
    }
    pay(g, p, x as u32, "GU", false)?;
    decks::remove_from_hand(g, p, c);
    let pl = g.player_mut(p);
    pl.cast_names.insert(c);
    pl.spells_this_turn += 1;
    on_cast(g, p, c)?;
    crate::engine::life::gain(g, p, x / 2)?;
    draw(g, p, (x / 2) as u32, false)?;
    let m = enter(g, p, c, Enter { was_cast: true, ..Enter::default() })?;
    g.perm_mut(m).plus += x;
    Ok(true)
}

/// Primal Might: your creature gets +X/+X, then fights theirs
fn primal_might(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = card_arg(arg);
    let me = PermId(((arg >> 16) & 0xff_ffff) as u32);
    let foe = PermId((arg >> 40) as u32);
    let x = total_mana(g, p, false) as i32 - 1;
    if !g.player(p).hand.contains(&c) || x < 0 || !can_pay(g, p, x as u32, "G", false) {
        return Ok(false);
    }
    decks::remove_from_hand(g, p, c);
    pay(g, p, x as u32, "G", false)?;
    g.player_mut(p).gy.push(c);
    on_cast(g, p, c)?;
    let pt = &mut g.perm_mut(me).eot_pt;
    pt.0 += x;
    pt.1 += x;
    if g.perm(foe).on_bf && g.perm(me).on_bf && g.perm(me).owner == p {
        let kind = format!("dmg{}", epow(g, me));
        apply_removal(g, Some(p), foe, &kind, Some(c))?;
    }
    Ok(true)
}

fn wildspeaker_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    let d = g.db.get(c);
    let (gn, pips) = (d.generic, d.pips.to_string());
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    decks::remove_from_hand(g, p, c);
    pay(g, p, gn, &pips, false)?;
    g.player_mut(p).gy.push(c);
    on_cast(g, p, c)?;
    Ok(true)
}

/// Return of the Wildspeaker: non-Humans get +3/+3
fn wildspeaker_pump(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    if !wildspeaker_cast(g, p, card_arg(arg))? {
        return Ok(false);
    }
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) && !has_type(g, m, "human") {
            let pt = &mut g.perm_mut(m).eot_pt;
            pt.0 += 3;
            pt.1 += 3;
        }
    }
    Ok(true)
}

/// Return of the Wildspeaker: draw cards equal to your biggest non-Human's power
fn wildspeaker_draw(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let n = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| g.is_creature(m) && !g.perm(m).phased && !has_type(g, m, "human"))
        .map(|&m| epow(g, m))
        .max()
        .unwrap_or(0);
    if !wildspeaker_cast(g, p, card_arg(arg))? {
        return Ok(false);
    }
    draw(g, p, n as u32, false)?;
    Ok(true)
}

// ------------------------------------------------------------------ protection (outside decks)
#[derive(Clone, Copy, PartialEq)]
enum Where {
    Hand,
    BfSac,
    BfTap,
    BfSelfDiscard,
}

#[derive(Clone, Copy, PartialEq)]
enum How {
    /// any targeted removal of that permanent
    AllTargeted,
    /// destroy or damage only
    Indes,
    /// targeted removal from a coloured source
    Color,
    /// any targeted removal (creature, nontoken)
    Blink,
    HexproofIndes,
    Phase,
}

#[derive(Clone, Copy, PartialEq)]
enum Scope {
    One,
    Creatures,
    Perms,
    Selfish,
}

/// a protector: (name, where, cost, what it saves, scope, free while you control your commander)
type Protector = (&'static str, Where, Option<(u32, &'static str)>, How, Scope, bool);

/// pool_ai.PROTECTORS
const PROTECTORS: [Protector; 15] = [
    ("Heroic Intervention", Where::Hand, Some((1, "G")), How::HexproofIndes, Scope::Perms, false),
    ("Teferi's Protection", Where::Hand, Some((2, "W")), How::Phase, Scope::Perms, false),
    ("Flawless Maneuver", Where::Hand, Some((2, "W")), How::Indes, Scope::Creatures, true),
    ("Boros Charm", Where::Hand, Some((0, "RW")), How::Indes, Scope::Perms, false),
    ("Deflecting Swat", Where::Hand, Some((2, "R")), How::AllTargeted, Scope::One, true),
    ("Gods Willing", Where::Hand, Some((0, "W")), How::Color, Scope::One, false),
    ("Valorous Stance", Where::Hand, Some((1, "W")), How::Indes, Scope::One, false),
    ("Ephemerate", Where::Hand, Some((0, "W")), How::Blink, Scope::One, false),
    ("Restoration Angel", Where::Hand, Some((3, "W")), How::Blink, Scope::One, false),
    ("Selfless Spirit", Where::BfSac, None, How::Indes, Scope::Creatures, false),
    ("Mother of Runes", Where::BfTap, None, How::Color, Scope::One, false),
    ("Giver of Runes", Where::BfTap, None, How::AllTargeted, Scope::One, false), // colorless or a colour
    ("Dream Trawler", Where::BfSelfDiscard, None, How::AllTargeted, Scope::Selfish, false),
    ("Benevolent Bodyguard", Where::BfSac, None, How::Color, Scope::One, false),
    ("Alseid of Life's Bounty", Where::BfSac, Some((1, "")), How::Color, Scope::One, false),
];

const WIPE_INDES: [&str; 6] = ["destroy", "dmg13", "austere", "austere2", "nib", "vandal"];

fn destroyish(kind: &str) -> bool {
    kind == "destroy" || kind.starts_with("dmg")
}

fn protector(name: &str) -> Option<usize> {
    PROTECTORS.iter().position(|x| x.0 == name)
}

/// pool_ai.worth_protecting
fn worth_protecting(g: &Game, p: PlayerId, m: PermId) -> bool {
    let x = g.perm(m);
    let kc = plans::config(g.player(p).key).key_cards;
    pval(g, m) >= 4.0 || x.is_cmd || x.cd.is_some_and(|c| kc.iter().any(|k| *k.0 == *g.db.get(c).name))
}

/// pool_ai._saves
fn saves(g: &Game, how: How, kind: &str, m: Option<PermId>, spell: Option<CardId>, targeted: bool) -> bool {
    match how {
        How::Phase => true,
        How::HexproofIndes => targeted || WIPE_INDES.contains(&kind),
        How::Indes => destroyish(kind) || (!targeted && WIPE_INDES.contains(&kind)),
        _ if !targeted => false,
        How::AllTargeted => true,
        How::Blink => m.is_some_and(|m| g.is_creature(m) && !g.perm(m).token),
        How::Color => spell.is_some_and(|s| g.db.get(s).pips.chars().any(|ch| "WUBRG".contains(ch))),
    }
}

/// pool_ai._options: (protector index, its source permanent) p could use now
fn protector_options(g: &Game, p: PlayerId, m: Option<PermId>) -> Vec<(usize, Option<PermId>)> {
    let mut out = vec![];
    for &c in &g.player(p).hand {
        if let Some(i) = protector(&g.db.get(c).name)
            && PROTECTORS[i].1 == Where::Hand
        {
            out.push((i, None));
        }
    }
    for &x in &g.player(p).perms {
        let px = g.perm(x);
        let Some(cd) = px.cd else { continue };
        let Some(i) = protector(&g.db.get(cd).name) else { continue };
        if PROTECTORS[i].1 == Where::Hand || px.phased || crate::cardcode::ability_locked(g, x, p) {
            continue;
        }
        if PROTECTORS[i].0 == "Giver of Runes" && Some(x) == m {
            continue; // "another target creature"
        }
        if PROTECTORS[i].4 == Scope::Selfish && Some(x) != m {
            continue;
        }
        out.push((i, Some(x)));
    }
    out
}

/// pool_ai._use: pay for and use protector i; true if it happened
fn use_protector(g: &mut Game, p: PlayerId, i: usize, src: Option<PermId>) -> Res<bool> {
    let (name, wh, cost, _, _, free_cmd) = PROTECTORS[i];
    match wh {
        Where::Hand => {
            let Some(c) = g.player(p).hand.iter().copied().find(|&x| &*g.db.get(x).name == name) else {
                return Ok(false);
            };
            if !castable(g, p, c, "hand") {
                return Ok(false);
            }
            let free = free_cmd && commander_out(g, p);
            let (cg, cp) = cost.unwrap();
            if !free && !can_pay(g, p, cg, cp, false) {
                return Ok(false);
            }
            decks::remove_from_hand(g, p, c); // off the hand before paying: a Treasure payment can trigger responses
            if !free {
                pay(g, p, cg, cp, false)?;
            }
            let pl = g.player_mut(p);
            pl.spells_this_turn += 1;
            pl.stat("spells_cast", 1);
            match name {
                "Ephemerate" => {
                    pl.exile.push(c);
                    pl.rebound.push(c);
                }
                "Teferi's Protection" => pl.exile.push(c),
                "Restoration Angel" => {}
                _ => pl.gy.push(c),
            }
            g.player_mut(p).cast_names.insert(c);
            on_cast(g, p, c)?;
            if name == "Restoration Angel" {
                enter(g, p, c, Enter::default())?;
            }
        }
        _ => {
            let Some(s) = src.filter(|&s| g.perm(s).on_bf && g.perm(s).owner == p) else { return Ok(false) };
            match wh {
                Where::BfTap => {
                    if g.perm(s).tapped || g.perm(s).sick {
                        return Ok(false);
                    }
                    g.perm_mut(s).tapped = true;
                }
                Where::BfSelfDiscard => {
                    if g.player(p).hand.is_empty() {
                        return Ok(false);
                    }
                    discard_worst(g, p, 1)?;
                    g.perm_mut(s).tapped = true;
                }
                _ => {
                    if let Some((cg, cp)) = cost {
                        if !can_pay(g, p, cg, cp, false) {
                            return Ok(false);
                        }
                        pay(g, p, cg, cp, false)?;
                    }
                    die(g, s, "sac")?;
                }
            }
        }
    }
    g.player_mut(p).stat("protection_used", 1);
    crate::glog!(g, "    {} protects with {name}", g.player(p).name);
    Ok(true)
}

/// pool_ai._worth_whole_board: Teferi's Protection on one targeted removal only for the commander or a permanent
/// carrying at least 40% of the board's value
fn worth_whole_board(g: &Game, owner: PlayerId, m: PermId) -> bool {
    if g.perm(m).is_cmd {
        return true;
    }
    let board: f64 = g.player(owner).perms.iter().filter(|&&x| !g.perm(x).phased).map(|&x| pval(g, x)).psum();
    board > 0.0 && pval(g, m) >= 0.4 * board
}

/// pool_ai.protect: targeted removal is aimed at owner's permanent m: respond if it's worth it
pub fn protect(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    _actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if !worth_protecting(g, owner, m) {
        return crate::cardcode::sac_in_response(g, owner, m, kind);
    }
    let mut cands: Vec<(u32, usize, Option<PermId>)> = vec![];
    for (i, src) in protector_options(g, owner, Some(m)) {
        let (name, wh, _, how, scope, _) = PROTECTORS[i];
        if matches!(scope, Scope::One | Scope::Creatures)
            && !g.is_creature(m)
            && !matches!(how, How::HexproofIndes | How::Phase | How::AllTargeted)
        {
            continue;
        }
        if !saves(g, how, kind, Some(m), spell, true) {
            continue;
        }
        if name == "Teferi's Protection" && !worth_whole_board(g, owner, m) {
            continue; // kept for wipes and lethal
        }
        // cheapest first: permanents that tap, then one-shot cards; save the board-wide ones for wipes
        let rank = match wh {
            Where::BfTap | Where::BfSelfDiscard => 0,
            Where::Hand => 1,
            Where::BfSac => 2,
        } + if matches!(scope, Scope::One | Scope::Selfish) { 0 } else { 3 };
        cands.push((rank, i, src));
    }
    cands.sort_by_key(|x| x.0);
    for (_, i, src) in cands {
        if use_protector(g, owner, i, src)? {
            let (name, _, _, how, _, _) = PROTECTORS[i];
            if name == "Teferi's Protection" {
                teferis_protection(g, owner);
            } else if how == How::Phase {
                for x in g.player(owner).perms.clone() {
                    g.perm_mut(x).phased = true;
                }
            } else if how == How::Blink && g.perm(m).on_bf && g.perm(m).owner == owner {
                let (cd, orig, is_cmd) = (g.perm(m).cd, g.perm(m).orig, g.perm(m).is_cmd);
                leave(g, m)?;
                if let Some(cd) = cd {
                    let n = enter(g, owner, cd, Enter { orig: Some(orig), ..Enter::default() })?;
                    g.perm_mut(n).is_cmd = is_cmd;
                }
            }
            return Ok(true);
        }
    }
    crate::cardcode::sac_in_response(g, owner, m, kind)
}

/// pool_ai.wipe_response: a board wipe is about to hit q: 'all' (everything saved), 'indes', or None
pub fn wipe_response(g: &mut Game, q: PlayerId, kind: Sym, caster: PlayerId) -> Res<Option<Sym>> {
    let loss = decks::wipe_loss(g, q, kind, caster); // by the wipe's own rule: Vandalblast spares creatures
    if loss < 6.0 {
        return Ok(None);
    }
    if kind == "destroy" && crate::cardcode::regen_wipe(g, q)? {
        return Ok(Some("indes"));
    }
    let mut opts = protector_options(g, q, None);
    opts.sort_by_key(|&(i, _)| PROTECTORS[i].1 != Where::BfSac);
    for (i, src) in opts {
        let (name, _, _, how, scope, _) = PROTECTORS[i];
        if matches!(scope, Scope::One | Scope::Selfish) || !saves(g, how, kind, None, None, false) {
            continue;
        }
        if use_protector(g, q, i, src)? {
            if name == "Teferi's Protection" {
                teferis_protection(g, q);
                return Ok(Some("all"));
            }
            if how == How::Phase {
                for x in g.player(q).perms.clone() {
                    g.perm_mut(x).phased = true;
                }
                return Ok(Some("all"));
            }
            return Ok(Some("indes"));
        }
    }
    Ok(None)
}

// ------------------------------------------------------------------ attacking (outside decks)
fn evasive(g: &Game, m: PermId) -> bool {
    crate::dsl::has_kw(g, m, "unblockable") || g.perm(m).fly || crate::dsl::has_kw(g, m, "flying")
}

fn untapped_blockers(g: &Game, q: PlayerId) -> Vec<PermId> {
    g.player(q).perms.iter().copied().filter(|&b| g.is_creature(b) && !g.perm(b).tapped && !g.perm(b).phased).collect()
}

/// pool_ai.attack_filter: keep home attackers that would just die to a good block (a blocker that kills it and
/// survives). Evasive creatures, lethal swings and attacks with more attackers than good blockers go ahead.
pub fn attack_filter(g: &mut Game, p: PlayerId, atk: Vec<PermId>, d: PlayerId) -> Vec<PermId> {
    if atk.is_empty() {
        return atk;
    }
    let total: i32 = atk.iter().map(|&m| epow(g, m)).sum();
    if total >= g.player(d).life {
        return atk; // lethal-ish: everyone goes
    }
    if combat_reserve(g, p).is_some() {
        // ninjutsu: every evasive enabler attacks
        let bl: Vec<PermId> =
            g.player(d).perms.iter().copied().filter(|&b| g.is_creature(b) && !g.perm(b).tapped).collect();
        let out: Vec<PermId> =
            atk.iter().copied().filter(|&a| evasive(g, a) || !bl.iter().any(|&b| can_block(g, b, a))).collect();
        return if out.is_empty() { atk } else { out };
    }
    let blockers = untapped_blockers(g, d);
    if blockers.is_empty() {
        return atk;
    }
    let mut keep = vec![];
    let mut risky = vec![];
    for &a in &atk {
        let good = blockers
            .iter()
            .filter(|&&b| {
                can_block(g, b, a)
                    && (epow(g, b) >= etgh(g, a) || g.perm(b).dt)
                    && !(epow(g, a) >= etgh(g, b) || g.perm(a).dt)
            })
            .count();
        if good > 0 { risky.push(a) } else { keep.push(a) }
    }
    if risky.is_empty() {
        return atk;
    }
    let n_good_blockers = blockers.iter().filter(|&&b| risky.iter().any(|&a| can_block(g, b, a))).count();
    if risky.len() > n_good_blockers + 1 {
        return atk; // they can't block everything
    }
    let mut out = keep;
    for a in risky {
        // a cheap token can still attack to push damage
        if g.perm(a).token && epow(g, a) <= 1 && out.len() >= 3 {
            out.push(a);
        }
    }
    if out.len() < atk.len() {
        crate::glog!(
            g,
            "      [{} holds back {} attacker(s) that would die to blocks]",
            g.player(p).name,
            atk.len() - out.len()
        );
    }
    out
}

/// pool_ai.ninja_defender_bonus: a ninjutsu deck attacks the player who can't block its evasive creatures
pub fn ninja_defender_bonus(g: &Game, p: PlayerId, q: PlayerId) -> f64 {
    if combat_reserve(g, p).is_none() {
        return 0.0;
    }
    let ev: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).sick && !g.perm(m).tapped && evasive(g, m))
        .collect();
    if ev.is_empty() {
        return 0.0;
    }
    let blockers = untapped_blockers(g, q);
    let open = ev.iter().filter(|&&a| !blockers.iter().any(|&b| can_block(g, b, a))).count();
    3.0 * open.min(2) as f64 - if open == 0 { 1.5 } else { 0.0 }
}

/// pool_ai.combat_reserve: mana to keep for the combat step: ninjutsu (Yuriko from the command zone, Ninjas in
/// hand) when an evasive attacker is ready. (generic, pips) or None.
pub fn combat_reserve(g: &Game, p: PlayerId) -> Option<(u32, String)> {
    let pl = g.player(p);
    let yuriko = &*g.db.get(pl.cmd).name == "Yuriko, the Tiger's Shadow";
    let costs = crate::cardcode::ninjutsu_costs(g, p);
    if !yuriko && costs.is_empty() {
        return None;
    }
    let ready = pl.perms.iter().any(|&m| {
        let x = g.perm(m);
        g.is_creature(m) && !x.sick && !x.tapped && !x.noatk && evasive(g, m)
    });
    if !ready {
        return None;
    }
    if yuriko && pl.cmd_in_zone {
        return Some((0, "UB".into()));
    }
    costs.into_iter().min_by_key(|x| x.0 + x.1.len() as u32)
}
