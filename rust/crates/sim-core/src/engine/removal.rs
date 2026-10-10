//! Removal and wipes (engine.py's "removal" section): legal targets, removing a permanent (with protection, ward and
//! taxes), enters-the-battlefield removal and Auras that lock their target, board wipes.

use crate::ai;
use crate::cardcode;
use crate::cards::{Colors, Types};
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, pay};
use crate::engine::values::*;
use crate::engine::zones::*;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

fn dmg_amount(kind: &str, prefix: &str) -> Option<i32> {
    kind.strip_prefix(prefix).and_then(|x| x.parse().ok())
}

/// the opposing permanents p may target with removal of this kind (`tgt`: which permanent types). Python's
/// `legal_targets`.
pub fn legal_targets(g: &Game, p: PlayerId, kind: &str, tgt: &str, mv4: bool, spell: Option<CardId>) -> Vec<PermId> {
    let mut res = vec![];
    let st = spell.map(|s| g.db.get(s));
    let stag = |t: Tag| st.is_some_and(|s| s.tag(t));
    for q in g.opps(p) {
        for &m in &g.player(q).perms {
            if untargetable(g, m) {
                continue;
            }
            let is_c = g.is_creature(m);
            let cd = g.perm(m).cd.map(|c| g.db.get(c));
            let ty = cd.map_or(Types::CREATURE, |d| d.types);
            let ok = match tgt {
                "c" => is_c,
                "cp" => is_c || ty.has(Types::PLANESWALKER),
                "cap" => is_c || ty.has(Types::ARTIFACT) || ty.has(Types::PLANESWALKER),
                "ce" => is_c || ty.has(Types::ENCHANTMENT),
                "nl" | "p" | "blue" => true,
                "a" => ty.has(Types::ARTIFACT),
                "cna" => is_c && !ty.has(Types::ARTIFACT),
                "ae" => ty.has(Types::ARTIFACT) || ty.has(Types::ENCHANTMENT),
                "ac" => is_c || ty.has(Types::ARTIFACT),
                _ => is_c,
            };
            if tgt == "blue" && !cd.is_some_and(|d| d.pips.contains('U')) {
                continue;
            }
            if stag(Tag::Nonblack) && colors_of(g, m).has('B') {
                continue;
            }
            if stag(Tag::Evil) && !(ty.has(Types::ENCHANTMENT) || (is_c && etgh(g, m) >= 4)) {
                continue;
            }
            if stag(Tag::Verdict) && !g.in_combat.contains(&m) {
                continue;
            }
            if LOCK_KINDS.contains(&kind) && !g.auras.is_empty() && cardcode::lock_factor(g, m) < 1.0 {
                continue;
            }
            if let (Some(s), Some(d)) = (st, cd) {
                if &*s.name == "Fatal Push" && d.cmc > 2 && !cardcode::revolt(g, p) {
                    continue;
                }
                if &*s.name == "Bloodchief's Thirst" && d.cmc > 2 && !can_pay(g, p, 2, "BB", false) {
                    continue;
                }
            }
            if stag(Tag::Alsoart) && ty.has(Types::ARTIFACT) && !is_c {
                res.push(m); // Abrade: destroy target artifact mode
                continue;
            }
            if !ok {
                continue;
            }
            if mv4 && cd.is_some_and(|d| d.cmc > 4) {
                continue;
            }
            if let (Some(d), Some(s)) = (cd, st)
                && d.ward > 0
                && !can_pay(g, p, s.generic + d.ward, &s.pips, false)
            {
                continue;
            }
            if cd.is_some_and(|d| d.tag(Tag::Sauron)) && ward_legends(g, p).is_empty() {
                continue; // ward: sacrifice a legend
            }
            if let Some(s) = st
                && protected_from(g, m, Colors::from_letters(&s.pips))
            {
                continue;
            }
            if let Some(n) = dmg_amount(kind, "dmg")
                && (!is_c || etgh(g, m) > n || no_damage(g, m))
            {
                continue;
            }
            if let Some(n) = dmg_amount(kind, "shrink")
                && (!is_c || etgh(g, m) > n)
            {
                continue;
            }
            if kind == "zero" && (!is_c || etgh(g, m) - g.perm(m).tgh > 0) {
                continue; // base 0/0: dies unless pumped
            }
            if stag(Tag::Newonly) && !entered_since_last_turn(g, p, m) {
                continue;
            }
            if (kind == "destroy" || kind.starts_with("dmg")) && indestructible(g, m) {
                continue;
            }
            res.push(m);
        }
    }
    res
}

/// actor removes m (kind: 'destroy', 'exile', 'bounce', 'tuck', 'top', 'dmgN', 'shrinkN', 'zero', a lock ...) with
/// spell, if any. Python's `apply_removal`.
pub fn apply_removal(g: &mut Game, actor: Option<PlayerId>, m: PermId, kind: &str, spell: Option<CardId>) -> Res {
    let owner = g.perm(m).owner;
    let mut kind: Sym = intern(kind);
    if let (Some(n), Some(a)) = (dmg_amount(kind, "dmg"), actor)
        && a != owner
    {
        // Fiery Emancipation: triple damage
        let k = g
            .player(a)
            .perms
            .iter()
            .filter(|&&x| !g.perm(x).phased && card_name(g, x) == Some("Fiery Emancipation"))
            .count() as u32;
        if k > 0 {
            kind = intern(&format!("dmg{}", n * 3i32.pow(k)));
        }
    }
    if !g.perm(m).on_bf || untargetable(g, m) {
        return Ok(());
    }
    if (kind == "destroy" || kind.starts_with("dmg")) && indestructible(g, m) {
        crate::glog!(g, "    {} ({}) is indestructible", g.perm(m).name, g.player(owner).name);
        return Ok(());
    }
    if ai::protect_response(g, owner, m, kind, actor, spell)? {
        crate::glog!(g, "    {} protects {}", g.player(owner).name, g.perm(m).name);
        return Ok(());
    }
    if !g.perm(m).on_bf {
        return Ok(());
    }
    let ward = g.perm(m).cd.map_or(0, |c| g.db.get(c).ward) + cardcode::aura_ward(g, m);
    if ward > 0
        && spell.is_some()
        && let Some(a) = actor.filter(|&a| a != owner)
    {
        // ward {N}: pay or it's countered
        if !can_pay(g, a, ward, "", false) {
            crate::glog!(g, "    ward counters the removal on {}", g.perm(m).name);
            return Ok(());
        }
        pay(g, a, ward, "", false)?;
    }
    if card_tag(g, m, Tag::Sauron)
        && spell.is_some()
        && let Some(a) = actor.filter(|&a| a != owner)
    {
        // Sauron's ward: sacrifice a legendary artifact or creature
        let legs = ward_legends(g, a);
        let Some(x) = min_by(&legs, |x| pval(g, x)) else {
            crate::glog!(g, "    ward counters the removal on {}", g.perm(m).name);
            return Ok(());
        };
        die(g, x, "sac")?;
    }
    if g.perm(m).cd.is_some() && spell.is_some() && actor != Some(owner) && !cardcode::removal_taxes(g, actor, m, kind)?
    {
        return Ok(());
    }
    if kind.starts_with("dmg") && no_damage(g, m) {
        crate::glog!(g, "    the damage to {} is prevented", g.perm(m).name);
        return Ok(());
    }
    if kind.starts_with("dmg") && cardcode::tajic_protects(g, m) {
        crate::glog!(g, "    damage to {} is prevented (Tajic)", g.perm(m).name);
        return Ok(());
    }
    crate::glog!(g, "    {} ({}) is removed: {}", g.perm(m).name, g.player(owner).name, kind);
    g.last_removed = Some((g.perm(m).cd, owner, kind, actor));
    if kind == "exile"
        && let Some(src) = g.rem_src
        && card_name(g, src) == Some("Skyclave Apparition")
    {
        // its owner gets an Illusion when the Apparition leaves
        let cd = g.perm(m).cd;
        cardcode::apparition_note(g, src, cd, owner);
    }
    let name = g.perm(m).name;
    *g.player_mut(owner).lost_names.entry(name).or_insert(0) += 1;
    if pval(g, m) >= 5.0 {
        g.player_mut(owner).stat("threats_lost", 1);
    }
    if g.player(owner).key == "seph" && g.perm(m).cd.is_some_and(|c| g.db.get(c).bomb != 0) {
        g.player_mut(owner).stat("bomb_removed", 1);
        let k: String = kind.chars().take(5).collect();
        g.player_mut(owner).stat(intern(&format!("bomb_removed_{k}")), 1);
    }
    let (power, mv, tgh) = (epow(g, m), g.perm(m).cd.map_or(0, |c| g.db.get(c).cmc), g.perm(m).tgh);
    let st = spell.map(|s| g.db.get(s).tags.clone()).unwrap_or_default();
    if st.has(Tag::Exiledie) && (kind == "destroy" || kind.starts_with("dmg") || kind.starts_with("shrink")) {
        exile_perm(g, m)?; // Scorching Dragonfire: exiled instead of dying
    } else if let Some(n) = dmg_amount(kind, "shrink") {
        if etgh(g, m) <= n {
            die(g, m, "sba")?; // -X/-X: toughness 0, no regeneration
        }
    } else if kind == "zero" {
        if etgh(g, m) - g.perm(m).tgh <= 0 {
            die(g, m, "sba")?; // Multiply by Zero: base 0/0 (counters still count)
        }
    } else if kind == "destroy" || kind.starts_with("dmg") {
        g.noregen = st.has(Tag::Noregen);
        let prev = std::mem::replace(&mut g.destroyer, actor); // Karmic Justice: who destroyed it
        let r = die(g, m, "destroy");
        g.noregen = false;
        g.destroyer = prev;
        r?;
    } else if kind == "exile" {
        exile_perm(g, m)?;
    } else if kind == "top" {
        leave(g, m)?; // Plan for All Outcomes: on top of the library
        to_zone_card(g, m, Zone::Top);
    } else if kind == "bounce" {
        bounce(g, m)?;
    } else if kind == "tuck" {
        tuck(g, m)?;
    } else if matches!(kind, "elk" | "mutate" | "forest") {
        cardcode::transform_away(g, m, kind)?;
    } else if LOCK_KINDS.contains(&kind) {
        cardcode::apply_lock(g, actor, m, kind)?; // Arrest, Encrust ...
    }
    if let Some(s) = spell {
        let t = g.db.get(s).tags.clone();
        let sname = g.db.get(s).name.to_string();
        if t.has(Tag::Rgain) {
            gain(g, owner, power)?; // Swords to Plowshares
        }
        if t.has(Tag::Rland) {
            land_ramp(g, owner, 1, true)?; // Path to Exile
        }
        if let Some(n) = t.int(Tag::Rtok) {
            let color = if sname.contains("Reality") { "" } else { "G" };
            make_tokens(g, owner, Tokens { color: Some(Colors::from_letters(color)), ..Tokens::new(1, n) })?;
        }
        if let Some(a) = actor {
            if t.has(Tag::Losemv) {
                lose_life(g, a, mv as i32, Some(a), "other", None)?; // Feed the Swarm
            }
            if t.has(Tag::Gaintgh) {
                gain(g, a, tgh)?; // Noxious Gearhulk
            }
        }
        if t.has(Tag::Enddraw) && !g.in_combat.contains(&m) {
            draw(g, owner, 1, false)?; // Prophesied End
        }
    }
    check_state(g)
}

/// m entered the battlefield after p's last turn ended (Premature Burial)
pub fn entered_since_last_turn(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).born > g.player(p).last_turn_end
}

/// a permanent's enters-the-battlefield removal (or an Aura's lock: attached as it resolves)
pub fn etb_removal(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let cd = g.perm(m).cd.unwrap();
    let db = g.db.clone();
    let rem = db.get(cd).tags.str(Tag::Rem).unwrap_or("destroy");
    if !LOCK_KINDS.contains(&rem) {
        let prev = g.rem_src.replace(m);
        let r = etb_removal_pick(g, p, m);
        g.rem_src = prev;
        return r;
    }
    if g.aura_put {
        return Ok(()); // put onto the battlefield (Zur): no target; placed by Zur
    }
    let prev = g.rem_src.replace(m);
    let r = match g.cast_target.filter(|&t| g.perm(t).on_bf && !untargetable(g, t)) {
        Some(t) => apply_removal(g, Some(p), t, rem, Some(cd)), // the target chosen as it was cast
        None => etb_removal_pick(g, p, m),
    };
    g.rem_src = prev;
    r?;
    if g.perm(m).on_bf && g.perm(m).attached.is_none() {
        leave(g, m)?; // nothing to enchant: the Aura goes to the graveyard
        to_zone_card(g, m, Zone::Gy);
    }
    Ok(())
}

fn etb_removal_pick(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    // HUMAN(phase 9): hc.etb_removal
    let cd = g.perm(m).cd.unwrap();
    let t = g.db.get(cd).tags.clone();
    let rem = t.str(Tag::Rem).unwrap_or("destroy");
    let tg = legal_targets(g, p, rem, t.str(Tag::Tgt).unwrap_or("c"), t.has(Tag::Mv4), Some(cd));
    let Some(best) = max_by(&tg, |x| pval(g, x)) else { return Ok(()) };
    if pval(g, best) < 2.0 {
        return Ok(());
    }
    apply_removal(g, Some(p), best, rem, Some(cd))
}

/// a board wipe of this kind resolves (card: the wipe spell, whose tags matter). Python's `apply_wipe`.
/// card: the wipe spell (its tags matter: Toxic Deluge), None for a wipe with no card
pub fn apply_wipe(g: &mut Game, p: PlayerId, kind: Sym, card: Option<CardId>, ctx: &Ctx) -> Res {
    g.batch += 1; // creatures destroyed together die simultaneously
    let prev = g.destroyer.replace(p); // Karmic Justice: who destroyed them
    g.resolving += 1;
    let r = apply_wipe_inner(g, p, kind, card, ctx);
    g.destroyer = prev;
    g.resolving -= 1;
    r?;
    if g.resolving == 0 && !g.trig_queue.is_empty() {
        crate::engine::stack::flush_triggers(g)?;
    }
    Ok(())
}

fn apply_wipe_inner(g: &mut Game, p: PlayerId, kind: Sym, card: Option<CardId>, ctx: &Ctx) -> Res {
    crate::glog!(g, "    board wipe resolves ({})", kind);
    let mut victims: Vec<PlayerId> = match kind {
        "rift" | "vandal" => g.opps(p).collect(),
        "rebuke" => ctx.victim.filter(|&v| g.player(v).alive).into_iter().collect(),
        _ => g.players.iter().filter(|q| q.alive).map(|q| q.id).collect(),
    };
    if kind == "minus" && card.is_some_and(|c| g.db.get(c).tag(Tag::Deluge)) {
        let x = match ctx.deluge_x {
            Some(x) => x, // practice mode: the person chose X
            None => {
                let xs: Vec<i32> = g
                    .opps(p)
                    .flat_map(|q| g.player(q).perms.iter().copied())
                    .filter(|&m| g.is_creature(m))
                    .map(|m| etgh(g, m))
                    .collect();
                xs.iter().copied().max().unwrap_or(1).min(10) // Toxic Deluge: pay X life
            }
        };
        lose_life(g, p, x, Some(p), "other", None)?;
    }
    let modes: Vec<Sym> = if matches!(kind, "farewell" | "austere2") {
        let m = ai::wipe_modes(g, p, kind);
        crate::glog!(g, "      modes: {:?}", m);
        m
    } else {
        vec![]
    };
    if kind == "vandal" {
        victims = g.opps(p).collect();
    }
    let destroyish = ["destroy", "dmg13", "austere", "nib", "austere2", "vandal"];
    for q in victims {
        if g.over {
            return Ok(());
        }
        let prot = ai::wipe_response(g, q, kind, p)?;
        if prot == Some("all") {
            continue;
        }
        let (mut biggest, mut x_dmg) = (None, 0);
        if kind == "nib" {
            let cr: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m)).collect();
            biggest = max_by(&cr, |m| epow(g, m) as f64);
            x_dmg = biggest.map_or(0, |b| epow(g, b));
        }
        for m in g.player(q).perms.clone() {
            if !g.perm(m).on_bf {
                continue;
            }
            if g.perm(m).phased || (prot == Some("indes") && destroyish.contains(&kind)) {
                continue;
            }
            let ty = g.perm(m).cd.map_or(Types::CREATURE, |c| g.db.get(c).types);
            let cre = g.is_creature(m);
            let cmc = g.perm(m).cd.map(|c| g.db.get(c).cmc);
            match kind {
                "vandal" => {
                    if ty.has(Types::ARTIFACT) {
                        die(g, m, "destroy")?;
                    }
                }
                "farewell" | "austere2" => {
                    let has_mode = |k: &str| modes.contains(&k);
                    let hit = (has_mode("art") && ty.has(Types::ARTIFACT))
                        || (has_mode("ench") && ty.has(Types::ENCHANTMENT))
                        || (has_mode("cre") && cre)
                        || (has_mode("le3") && cre && cmc.is_none_or(|c| c <= 3))
                        || (has_mode("ge4") && cre && cmc.is_some_and(|c| c >= 4));
                    if hit {
                        if kind == "farewell" { exile_perm(g, m)? } else { die(g, m, "destroy")? }
                    }
                }
                "rift" | "rebuke" => {
                    if g.perm(m).token {
                        leave(g, m)?
                    } else {
                        bounce(g, m)?
                    }
                }
                _ if !cre => {}
                "evac" => {
                    // Evacuation: all creatures to owners' hands
                    if g.perm(m).token { leave(g, m)? } else { bounce(g, m)? }
                }
                "minus" if ctx.deluge_x.is_some() => {
                    if etgh(g, m) <= ctx.deluge_x.unwrap() {
                        die(g, m, "destroy")?; // -X/-X: only toughness X or less
                    }
                }
                "destroy" | "minus" => die(g, m, "destroy")?,
                "exile" => exile_perm(g, m)?,
                "dmg13" => {
                    if etgh(g, m) <= 13 && !no_damage(g, m) {
                        die(g, m, "destroy")?;
                    }
                }
                "austere" => {
                    if cmc.is_some_and(|c| c >= 4) {
                        die(g, m, "destroy")?;
                    }
                }
                "nib" if Some(m) != biggest && etgh(g, m) <= x_dmg && !no_damage(g, m) => die(g, m, "destroy")?,
                _ => {}
            }
        }
    }
    if kind == "farewell" && modes.contains(&"gy") {
        // Farewell: exile all graveyards
        for q in &mut g.players {
            let gy = std::mem::take(&mut q.gy);
            q.exile.extend(gy);
        }
        crate::glog!(g, "    Farewell exiles all graveyards");
    }
    check_state(g)
}
