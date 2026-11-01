//! The Tymna partner decks (`thrasios`: Tymna the Weaver + Thrasios, Triton Hero; `kraum`: Tymna + Kraum, Ludevic's
//! Opus). The seat has two commanders (`Player::cmd` and `Player::cmd2`); the engine casts both from the command
//! zone with the generic commander priority. The decks' own AI plans are written later: for now both use the
//! default priority.

use crate::ai::hand_key; // --- phase4-hand-off
use crate::engine::mana::can_pay;
use crate::engine::values::card_name;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

const TYMNA: &str = "Tymna the Weaver";
const THRASIOS: &str = "Thrasios, Triton Hero";
const KRAUM: &str = "Kraum, Ludevic's Opus";
const BASTION: &str = "Bastion Protector";
const LAGRELLA: &str = "Lagrella, the Magpie";

/// a creature that is hard to block: a flier, menace, shadow, or one of the deck's named evasive creatures
fn evasive_card(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.creature
        && (d.tag(Tag::Fly)
            || d.has_kw("flying")
            || d.has_kw("menace")
            || d.has_kw("unblockable_shadow")
            || matches!(&*d.name, "Silhana Ledgewalker" | "Exit Specialist" | "Spectral Lynx" | "Nightveil Predator"))
}

/// evasive creatures p has on the battlefield
fn evasive_out(g: &Game, p: PlayerId) -> usize {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && g.is_creature(m) && x.cd.is_some_and(|c| evasive_card(g, c)) && !x.is_cmd
        })
        .count()
}

fn out(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && card_name(g, m) == Some(name))
}

/// the shared plan of the two Tymna decks (cast priority 0-90). The ramp and the commanders come first, then the
/// evasive creatures until every opponent can be hit, then the generic order (the finishers).
fn partner_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let pl = g.player(p);
    let d = g.db.get(c);
    let base = crate::ai::pool::generic_prio(g, p, c);
    let t = &d.tags;
    if pl.is_commander(c) {
        let tax = 3 * pl.tax_of(c) as i32;
        return match &*d.name {
            TYMNA => 91 - tax,    // as soon as she can be cast (turn 3)
            THRASIOS => 86 - tax, // the cheap engine: turn 2 after a one-mana ramp spell
            KRAUM => 84 - tax,    // five mana: haste, so precombat (the AI casts in the first main phase)
            _ => base,
        };
    }
    if let Some(v) = crate::impls::newcards_b::partner_prio(g, p, c) {
        return v;
    }
    let ramp = t.has(Tag::Rock) || t.has(Tag::Dork) || t.has(Tag::Lr) || t.has(Tag::Fastmana);
    if ramp && pl.turns <= 5 {
        // one-mana ramp first, then two-mana ramp (turns 1-2: mana creatures and rocks)
        return if d.cmc <= 1 {
            89
        } else if d.cmc == 2 {
            87 - (pl.turns > 3) as i32 * 4
        } else {
            82
        };
    }
    if d.creature && evasive_card(g, c) {
        let opps = g.opps(p).filter(|&q| g.player(q).alive).count();
        if evasive_out(g, p) < opps && d.cmc <= 4 {
            return 78 + (4 - d.cmc as i32).max(0) / 2 - 2 * (d.pow >= 4) as i32; // a cheap one for each opponent
        }
        return base.max(66);
    }
    if hand_key(g, p) == "kraum" {
        if t.has(Tag::Breach) && !d.creature {
            return breach_prio(g, p);
        }
        if d.creature && g.registry.get(c).is_some_and(|h| h.live() && h.attack_tax.is_some()) {
            // Baird: every creature that attacks you costs {1}; valued like the pillowfort cards, not as a vanilla body
            let power: i32 = g
                .opps(p)
                .flat_map(|q| g.player(q).perms.iter().copied())
                .filter(|&m| g.is_creature(m) && !g.perm(m).phased)
                .map(|m| epow(g, m))
                .sum();
            let n = g.opps(p).count().max(1) as f64;
            return base.max(55 + ((1.5 * power as f64 / n) as i32).min(25));
        }
    }
    if &*d.name == LAGRELLA {
        // its enters trigger exiles the best creature of every opponent; the compiled tags lack the `etb` tag the
        // generic priority needs for an enters-removal creature, so it was valued 0 and never cast
        let worth: f64 = g
            .opps(p)
            .filter(|&q| g.player(q).alive)
            .filter_map(|q| {
                g.player(q)
                    .perms
                    .iter()
                    .filter(|&&m| g.perm(m).on_bf && !g.perm(m).phased && g.is_creature(m))
                    .map(|&m| pval(g, m))
                    .fold(None, |a: Option<f64>, v| Some(a.map_or(v, |x| x.max(v))))
            })
            .sum();
        return if worth >= 2.0 { 45 + (4.0 * worth).min(30.0) as i32 } else { 35 };
    }
    if &*d.name == BASTION {
        let both = out(g, p, TYMNA) && (out(g, p, THRASIOS) || out(g, p, KRAUM));
        return if both { 74 } else { 40 };
    }
    if &*d.name == "Gilgamesh, Master-at-Arms" {
        return 68; // six mana: casting him is the whole turn, and his trigger finds the rest of the Equipment
    }
    if d.has_subtype("equipment") {
        // below the creatures (they are the carriers), above the generic 10 that left it in the hand: the two with a Hero
        // of their own and the Blade (which attaches to the next legend) any time, the rest once a creature is out
        let carriers = g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.is_creature(m)).count();
        return match &*d.name {
            "Dragoon's Lance" | "Samurai's Katana" => 63,
            "Hero's Blade" => 58,
            "Conqueror's Flail" if carriers > 0 => 60,
            _ if carriers > 0 => 56,
            _ => 25,
        };
    }
    base
}

/// ai/decks.rs `deck_prio` for the kraum seat
pub fn kraum_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    partner_prio(g, p, c)
}

// ------------------------------------------------------------------ measurement
/// `ty_tt_<name>` / `ty_tn_<name>`: the first turn (of the owner's own turn count) a partner-deck commander or Tymna came
/// down, summed over games, and the number of games it did (the analysis report prints the mean).
fn landed(g: &mut Game, src: crate::hooks::Src, _p: PlayerId, m: crate::ids::PermId) -> crate::flow::Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let name =
        crate::engine::values::card_name(g, src).unwrap_or("?").split(',').next().unwrap_or("?").replace(' ', "_");
    let turns = g.player(o).turns as i64;
    let pl = g.player_mut(o);
    let key = crate::sym::intern(&format!("ty_tn_{name}"));
    if pl.stats.contains_key(&key) {
        pl.stat(crate::sym::intern(&format!("ty_re_{name}")), 1); // cast again after it died
        return Ok(());
    }
    pl.stat(crate::sym::intern(&format!("ty_tt_{name}")), turns);
    pl.stat(key, 1);
    Ok(())
}

pub fn register(r: &mut crate::hooks::Registry, db: &crate::cards::CardDb) -> Result<(), String> {
    for name in ["Tymna the Weaver", "Thrasios, Triton Hero", "Kraum, Ludevic's Opus"] {
        if db.id(name).is_some() {
            r.card(db, name)?.etb = Some(landed);
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ spreading the attack
use crate::engine::combat::{attack_restrictions, can_block, double_strike, fiery, kw};
use crate::engine::values::{epow, pval, shielded};
use crate::ids::PermId;

/// Tymna's draw (or Reconnaissance Mission's / The Aetherspark's) is waiting on the battlefield
fn hit_engine_out(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        x.on_bf
            && !x.phased
            && matches!(card_name(g, m), Some("Tymna the Weaver" | "Reconnaissance Mission" | "The Aetherspark"))
    })
}

/// `a` would connect with player q: none of q's untapped creatures can block it (menace: not even one blocker)
fn connects(g: &Game, q: PlayerId, a: PermId) -> bool {
    let n = g
        .player(q)
        .perms
        .iter()
        .filter(|&&b| g.is_creature(b) && !g.perm(b).tapped && !g.perm(b).phased && can_block(g, b, a))
        .count();
    n == 0 || (n == 1 && kw(g, a, "menace"))
}

/// combat.rs, after the normal split: while Tymna (or the other hit-engines) is out, make sure every living
/// opponent who can be hit takes at least one attacker that will connect, the cheapest such attacker first, without
/// breaking an attack that is lethal. `pool` is every creature that could attack. Only the kraum seat.
pub fn spread(
    g: &Game,
    p: PlayerId,
    groups: Vec<(PlayerId, Vec<PermId>)>,
    pool: &[PermId],
) -> Vec<(PlayerId, Vec<PermId>)> {
    if hand_key(g, p) != "kraum" || !hit_engine_out(g, p) {
        return groups;
    }
    let hit = |a: PermId| epow(g, a) * if double_strike(g, a) { 2 } else { 1 } * fiery(g, p);
    let mut groups = groups;
    let opps: Vec<PlayerId> =
        g.opps(p).filter(|&q| g.player(q).alive && !shielded(g, q) && attack_restrictions(g, p, q).0 == 0).collect();
    let assigned = |gr: &[(PlayerId, Vec<PermId>)], a: PermId| gr.iter().position(|x| x.1.contains(&a));
    // lethal groups: the damage that can't be blocked reaches the life total
    let lethal = |gr: &[(PlayerId, Vec<PermId>)], i: usize| {
        let (q, xs) = &gr[i];
        xs.iter().filter(|&&a| connects(g, *q, a)).map(|&a| hit(a)).sum::<i32>() >= g.player(*q).life
    };
    let covered = |gr: &[(PlayerId, Vec<PermId>)], q: PlayerId| {
        gr.iter().any(|x| x.0 == q && x.1.iter().any(|&a| epow(g, a) > 0 && connects(g, q, a)))
    };
    // the opponents that can be hit by the fewest creatures first
    let cands = |q: PlayerId| -> Vec<PermId> {
        pool.iter().copied().filter(|&a| epow(g, a) > 0 && connects(g, q, a)).collect()
    };
    let mut order: Vec<PlayerId> = opps.into_iter().filter(|&q| !covered(&groups, q)).collect();
    order.sort_by_key(|&q| cands(q).len());
    for q in order {
        let mut cs: Vec<PermId> = cands(q);
        cs.sort_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)));
        for a in cs {
            let from = assigned(&groups, a);
            if from.is_none() && (g.perm(a).is_cmd || card_name(g, a) == Some("Tymna the Weaver")) {
                continue; // a held-back commander stays home
            }
            if let Some(i) = from {
                let (gq, xs) = (groups[i].0, &groups[i].1);
                if gq == q {
                    continue;
                }
                // not the only attacker that hits its opponent, nor part of a lethal attack
                let other_hit = xs.iter().any(|&x| x != a && epow(g, x) > 0 && connects(g, gq, x));
                if connects(g, gq, a) && !other_hit {
                    continue;
                }
                if lethal(&groups, i) {
                    let mut t = groups.clone();
                    t[i].1.retain(|&x| x != a);
                    if !lethal(&t, i) {
                        continue;
                    }
                }
                groups[i].1.retain(|&x| x != a);
            }
            match groups.iter().position(|x| x.0 == q) {
                Some(j) => groups[j].1.push(a),
                None => groups.push((q, vec![a])),
            }
            break;
        }
    }
    groups.retain(|x| !x.1.is_empty());
    groups
}

// ------------------------------------------------------------------ commanders come down on time
/// `ai::search::choose_inner`, the kraum seat only: when the heuristic's best play is casting a commander
/// from the command zone, it is made without the look-ahead. The search's position score doesn't value the commanders
/// (Tymna's draw, Thrasios's mana sink) and was putting a ramp spell or a creature ahead of them, which landed
/// Thrasios on turn 4.4-4.7, Tymna on 5.0-5.2 and Kraum on 6.8-7.1 when each was castable about 1.3 turns earlier.
/// `opts` are the main-phase options; returns the index of the commander cast.
pub fn commander_first(g: &Game, p: PlayerId, opts: &[crate::hooks::Opt]) -> Option<usize> {
    if hand_key(g, p) != "kraum" {
        return None;
    }
    let (i, o) =
        opts.iter().enumerate().filter(|(_, o)| o.act.is_some()).max_by(|a, b| a.1.utility.total_cmp(&b.1.utility))?;
    match o.act {
        Some(crate::hooks::Action::Cast { card, .. })
            if g.player(p).is_commander(card) && !g.player(p).hand.contains(&card) =>
        {
            Some(i)
        }
        _ => None,
    }
}

// ------------------------------------------------------------------ Underworld Breach (kraum seat)
/// Underworld Breach for the kraum seat ({1}{R}, gone at the end step), so it is cast only when a card in the graveyard can be escaped
/// with the mana that is left (its cost plus the {1}{R}), 4+ cards in the graveyard (the three exiled and the card itself).
/// `generic_prio` gave it 0, so it sat in the hand in every game. The escape options are `breach_gc_options`.
fn breach_prio(g: &Game, p: PlayerId) -> i32 {
    if g.active != Some(p) {
        return 0;
    }
    let ok = crate::ai::decks::breach_candidates(g, p, false).into_iter().any(|(_, c)| {
        let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
        can_pay_r(g, p, cg, &cp)
    });
    if ok { 72 } else { 0 }
}

/// the {1}{R} for Breach plus the escape cost: a red source remains after paying `cp` (a rough check on colours)
fn can_pay_r(g: &Game, p: PlayerId, cg: u32, cp: &str) -> bool {
    let with_r = format!("{cp}R");
    let cg = cg + 1; // Breach is {1}{R}
    crate::engine::mana::can_pay(g, p, cg, &with_r, false)
}

// ------------------------------------------------------------------ Thrasios's {4} sink
/// `ai::search::choose_inner`, the kraum seat: the index of Thrasios's {4} ability among the main-phase
/// options. The search's position score doesn't count the card or land it gives, and it kept choosing "stop" with the
/// mana unspent (0.4-0.7 activations a game), so the ability is left out of the search and made when the search's
/// best play among the others is "stop": the spare mana is spent after everything else the deck wanted to do.
pub fn thrasios_sink(g: &Game, p: PlayerId, opts: &[crate::hooks::Opt]) -> Option<usize> {
    if hand_key(g, p) != "kraum" {
        return None;
    }
    opts.iter().position(|o| o.act.is_some() && o.label.starts_with("Thrasios:"))
}

// ------------------------------------------------------------------ protection spells
/// a protection instant the kraum seat keeps mana for when it spends the spare on Thrasios: Dark Endurance, while a
/// commander or a creature worth 4 or more is out to protect. (Not part of `brain::hold_value`: there the look-ahead's
/// hold bonus beat casting two creatures with ten lands in play.)
pub fn protection_held(g: &Game, p: PlayerId, c: CardId) -> bool {
    if hand_key(g, p) != "kraum" {
        return false;
    }
    &*g.db.get(c).name == "Dark Endurance"
        && g.player(p).perms.iter().any(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && g.is_creature(m) && (x.is_cmd || pval(g, m) >= 4.0)
        })
}

/// engine/combat.rs, after the blockers are declared (the kraum seat; the thrasios seat's hand AI was deleted on
/// 2026-11-01, with its Blossoming Defense pump): Dark Endurance (+2/+0, indestructible) on an attacker that would
/// die in its block and is worth saving (a commander or worth 3 or more), or that would win a trade it was losing
/// against a blocker worth 3 or more. `assign` is the (attacker, blocker) pairs. First strike and trample are not
/// worked through: those blocks are left alone.
pub fn combat_trick(g: &mut Game, p: PlayerId, atk: &[PermId], assign: &[(PermId, PermId)]) -> crate::flow::Res {
    if hand_key(g, p) != "kraum" || crate::human::is_human(g, p) {
        return Ok(());
    }
    let find = |g: &Game, name: &str| g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == name);
    for &a in atk {
        let Some(&(_, b)) = assign.iter().find(|x| x.0 == a) else { continue };
        if g.over
            || !g.perm(a).on_bf
            || g.perm(a).owner != p
            || !g.perm(b).on_bf
            || first_strike_either(g, a, b)
            || crate::engine::values::silenced(g, p)
        {
            continue;
        }
        let (pa, ta, pb, tb) = (epow(g, a), crate::engine::values::etgh(g, a), epow(g, b), crate::engine::values::etgh(g, b));
        let dt = g.perm(b).dt;
        let a_dies = pb >= ta || dt;
        let b_dies = pa >= tb;
        let worth_a = g.perm(a).is_cmd || pval(g, a) >= 3.0;
        let worth_b = pval(g, b) >= 3.0;
        let endure = find(g, "Dark Endurance").is_some() && can_pay(g, p, 1, "B", false);
        let save = a_dies && worth_a;
        let trade = !b_dies && worth_b && pa + 2 >= tb && (!a_dies || worth_a);
        if !(save || trade) {
            continue;
        }
        if endure && let Some(c) = find(g, "Dark Endurance") {
            if crate::engine::cast::castable(g, p, c, "hand") && super::tymna_cards::cast_endurance(g, p, c, a)? {
                crate::glog!(g, "    {} pumps {} in its block", g.player(p).name, g.perm(a).name);
            }
        }
    }
    Ok(())
}

fn first_strike_either(g: &Game, a: PermId, b: PermId) -> bool {
    crate::engine::combat::first_strike(g, a) || crate::engine::combat::first_strike(g, b)
}

