//! Python's `cards/impl/rules.py`: rules the pool cards were approximating, made exact (mana sources, counter
//! interactions, removal restrictions and taxes, "becomes a 3/3" effects, Council's Judgment's vote, Fact or Fiction's
//! split, ability locks, emblems, and single-card clauses). The Tier 1 and Sauron cards so far; the planeswalker
//! ultimates (common's WALKERS) and the other decks' cards come with phase 6.

use super::partials::{
    at_once, best_opp_creature, best_opp_nonland, first_max, first_min, name_of, of_type, on, pack, remove_card,
    unpack,
};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{castable, on_cast};
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{counter_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{colors_of, equipped, etgh, has_type, pval, shielded, threat};
use crate::engine::zones::{
    Enter, Tokens, Zone, die, draw, enter, exile_perm, leave, make_tokens, max_by, min_by, searchable, to_zone_card,
};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

// ================================================================== mana sources
/// Springleaf Drum, Jaspera Sentinel: mana only while another untapped creature can be tapped
fn drum(g: &Game, p: PlayerId, m: PermId) -> u32 {
    g.player(p).perms.iter().any(|&x| x != m && g.is_creature(x) && !g.perm(x).tapped && !g.perm(x).phased) as u32
}

/// tap the least useful untapped creature (a summoning-sick one first)
fn drum_tap(g: &mut Game, p: PlayerId, _m: PermId, _used: u32) -> Res {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&x| g.is_creature(x) && !g.perm(x).tapped && !g.perm(x).phased)
        .collect();
    if let Some(x) = first_min(&cs, |x| (!g.perm(x).sick, pval(g, x))) {
        g.perm_mut(x).tapped = true;
    }
    Ok(())
}

/// Lotus Petal: sacrificed as it's tapped for mana
fn petal_sac(g: &mut Game, p: PlayerId, m: PermId, _used: u32) -> Res {
    if on(g, p, m) {
        leave(g, m)?;
        to_zone_card(g, m, Zone::Gy);
    }
    Ok(())
}

/// rules.halfling_colors: Delighted Halfling's coloured mana only pays for legendary spells (None: colourless)
pub fn halfling_colors(g: &Game, p: PlayerId) -> Option<Colors> {
    let c = g.pay_for?;
    g.db.get(c).tag(Tag::Leg).then(|| g.player(p).ident).filter(|x| !x.is_empty())
}

/// rules.dryad_colors: Dryad of the Ilysian Grove: p's lands tap for any colour in its identity
pub fn dryad_colors(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| !g.perm(m).phased && name_of(g, m) == "Dryad of the Ilysian Grove")
}

// ================================================================== tax payments
/// rules.REPLACED_TAG_ENGINES: engines whose card code replaces the generic 'eng' upkeep draw
pub fn replaced_tag_engine(name: &str) -> bool {
    matches!(name, "Mystic Remora" | "Sylvan Library")
}

// ================================================================== counters
/// your decks' keys (engine.CTHRESH): they never answer with Veil of Summer
const CTHRESH: [&str; 8] = ["seph", "veyran", "sauron", "galadriel", "yshtola", "alela", "jodah", "najeela"];

/// rules.veil_response: q is about to counter p's spell with a blue or black counterspell: p answers with Veil of
/// Summer (its spells can't be countered this turn, it draws a card)
pub fn veil_response(g: &mut Game, p: PlayerId, _q: PlayerId, ctr: CardId) -> Res<bool> {
    if g.humans.get(p).is_some() {
        return Ok(false); // HUMAN(phase 9): a person casts their own Veil
    }
    let d = g.db.get(ctr);
    let blue_black = d.pips.contains('U') || d.pips.contains('B') || matches!(&*d.name, "Force of Will" | "Force of Negation");
    if CTHRESH.contains(&g.player(p).key) || !blue_black {
        return Ok(false);
    }
    let Some(v) = g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == "Veil of Summer") else {
        return Ok(false);
    };
    if !can_pay(g, p, 0, "G", false) || !castable(g, p, v, "hand") {
        return Ok(false);
    }
    pay(g, p, 0, "G", false)?;
    remove_card(&mut g.player_mut(p).hand, v);
    g.player_mut(p).gy.push(v);
    on_cast(g, p, v)?;
    crate::glog!(g, "    {} responds with Veil of Summer: the spell can't be countered", pname(g, p));
    draw(g, p, 1, false)?;
    Ok(true)
}

// ================================================================== removal restrictions and taxes
/// rules.removal_taxes: targeting costs: Terror of the Peaks (3 life); damage to Phyrexian Obliterator costs
/// permanents. False would stop the removal (nothing here does).
pub fn removal_taxes(g: &mut Game, actor: Option<PlayerId>, m: PermId, kind: Sym) -> Res<bool> {
    let name = name_of(g, m).to_string();
    let Some(a) = actor else { return Ok(true) };
    if name == "Terror of the Peaks" {
        lose_life(g, a, 3, Some(a), "other", None)?;
    }
    if name == "Phyrexian Obliterator" && kind.starts_with("dmg") {
        let n: i32 = kind[3..].parse().unwrap_or(0);
        for _ in 0..n {
            let ps: Vec<PermId> = g.player(a).perms.iter().copied().filter(|&x| !g.perm(x).phased).collect();
            if let Some(x) = min_by(&ps, |x| pval(g, x)) {
                die(g, x, "sac")?;
            }
        }
    }
    Ok(true)
}

/// rules.transform_away: 'becomes a 3/3 Elk' (Kenrith's Transformation, Oko), 'becomes a 0/1 indestructible Insect'
/// (Darksteel Mutation), 'becomes a Forest' (Song of the Dryads)
pub fn transform_away(g: &mut Game, m: PermId, kind: Sym) -> Res {
    if !g.perm(m).on_bf {
        return Ok(());
    }
    if kind == "forest" {
        let (o, tapped) = (g.perm(m).owner, g.perm(m).tapped);
        leave(g, m)?;
        let forest = g.db.id("Forest").expect("Forest is in the card database");
        g.add_land(o, forest, tapped);
        crate::glog!(g, "    {} becomes a Forest", g.perm(m).name);
        return Ok(());
    }
    let x = g.perm_mut(m);
    x.neutered = true;
    if kind == "elk" {
        (x.pow, x.tgh) = (3, 3);
        x.data.set(DataKey::Elk, Val::Bool(true));
    } else {
        (x.pow, x.tgh) = (0, 1);
        x.data.set(DataKey::Indestr, Val::Bool(true));
    }
    (x.fly, x.dt, x.lifelink) = (false, false, false);
    if kind == "mutate" {
        x.plus = x.plus.min(0);
    }
    g.bf_ver += 1;
    crate::glog!(
        g,
        "    {} becomes a {} with no abilities",
        g.perm(m).name,
        if kind == "elk" { "3/3 Elk" } else { "0/1 Insect" }
    );
    Ok(())
}

// ================================================================== Council's Judgment: the vote
/// cast when the best opposing nonland permanent is worth exiling
fn judgment_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match best_opp_nonland(g, p, |_, _| true) {
        Some(t) if pval(g, t) >= 4.0 => 58,
        _ => 0,
    }
}

/// will of the council: each player votes for a nonland permanent they don't control (their biggest threat); every
/// permanent with the most votes is exiled
fn judgment(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut votes: Vec<(PermId, u32)> = vec![];
    let voters: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for &v in &voters {
        let mut cands: Vec<PermId> = voters
            .iter()
            .filter(|&&q| q != v)
            .flat_map(|&q| g.player(q).perms.iter().copied())
            .filter(|&m| !g.perm(m).phased)
            .collect();
        if v == p {
            cands.retain(|&m| g.perm(m).owner != p);
        }
        let Some(pick) = max_by(&cands, |m| {
            let o = g.perm(m).owner;
            pval(g, m) * (1.0 + 0.3 * if o != v { threat(g, v, o) } else { 0.0 })
        }) else {
            continue;
        };
        match votes.iter_mut().find(|x| x.0 == pick) {
            Some(x) => x.1 += 1,
            None => votes.push((pick, 1)),
        }
    }
    let Some(top) = votes.iter().map(|x| x.1).max() else { return Ok("gy") };
    for (m, n) in votes {
        if n == top && g.perm(m).on_bf {
            crate::glog!(g, "    Council's Judgment exiles {} ({} votes)", g.perm(m).name, n);
            exile_perm(g, m)?;
        }
    }
    Ok("gy")
}

// ================================================================== Fact or Fiction: the split
fn fof_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    45
}

/// the most threatened opponent splits the five into the piles that leave you least; you take the better pile
fn fof(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut top = vec![];
    for _ in 0..g.player(p).library.len().min(5) {
        top.push(g.player_mut(p).library.pop().unwrap());
    }
    if top.is_empty() {
        return Ok("gy");
    }
    let w: Vec<f64> = top.iter().map(|&x| card_worth(g, p, x, false).max(0.0)).collect();
    let all = w.iter().copied().psum();
    let mut best: Option<(f64, usize, bool)> = None;
    for mask in 0..(1usize << top.len()) {
        let a = (0..top.len()).filter(|i| mask >> i & 1 == 1).map(|i| w[i]).psum();
        let b = all - a;
        if best.is_none_or(|x| a.max(b) < x.0) {
            best = Some((a.max(b), mask, a >= b));
        }
    }
    let (_, mask, take_a) = best.unwrap();
    let pile: Vec<CardId> = (0..top.len()).filter(|&i| (mask >> i & 1 == 1) == take_a).map(|i| top[i]).collect();
    for x in top {
        let pl = g.player_mut(p);
        if pile.contains(&x) {
            pl.hand.push(x);
            pl.seen.insert(x);
        } else {
            pl.gy.push(x);
        }
    }
    Ok("gy")
}

// ================================================================== Boros Charm
/// 4 damage to a player at 4 life or less (the indestructible mode is the protection AI's)
fn boros_charm(g: &mut Game, c: CardId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if !can_pay(g, p, 0, "RW", false) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let lethal = g.opps(p).find(|&q| g.player(q).life <= 4 && !shielded(g, q));
    let Some(q) = lethal else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 9.0,
        label: "Boros Charm (4 damage, lethal)".into(),
        act: Some(Action::Plan { f: boros_go, arg: pack(c.0 as u32, q.0 as u32) }),
    }])
}

fn boros_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, q) = unpack(arg);
    let (c, q) = (CardId(c as u16), PlayerId(q as u8));
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "RW", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 0, "RW", false)?;
    on_cast(g, p, c)?;
    if counter_window(g, p, c, 6.0, vec![])? {
        lose_life(g, q, 4, Some(p), "burn", Some(true))?;
    }
    g.player_mut(p).gy.push(c);
    Ok(true)
}

// ================================================================== ability locks
/// rules.ability_locked: can src's activated abilities not be used right now? (Arrest, Encrust; Collector Ouphe /
/// Karn: artifacts; Cursed Totem: creatures; Grand Abolisher: nothing of yours during its controller's turn)
pub fn ability_locked(g: &Game, src: PermId, p: PlayerId) -> bool {
    if !g.auras.is_empty() && crate::cardcode::locked(g, src, "noact") {
        return true;
    }
    let x = g.perm(src);
    if g.hooks.is_empty() || x.cd.is_none() && !g.is_creature(src) {
        return false;
    }
    let art = of_type(g, src, Types::ARTIFACT) || (x.token && x.ttypes.contains(&"artifact"));
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in &q.perms {
            let y = g.perm(m);
            if y.cd.is_none() || y.phased || y.neutered {
                continue;
            }
            match name_of(g, m) {
                "Collector Ouphe" if art => return true,
                "Karn, the Great Creator" if art && p != q.id => return true,
                "Cursed Totem" if g.is_creature(src) => return true,
                "Grand Abolisher" if p != q.id && g.active == Some(q.id) => return true,
                _ => {}
            }
        }
    }
    false
}

// ================================================================== emblems
/// rules.emblem_cast: Chandra's emblem (5 damage per spell), Venser's (exile a permanent per spell)
pub fn emblem_cast(g: &mut Game, p: PlayerId, _c: CardId) -> Res {
    if g.player(p).emblems.contains(&"chandra") {
        let t = best_opp_creature(g, p, |g, m| etgh(g, m) <= 5);
        let lethal = g.opps(p).find(|&q| g.player(q).life <= 5);
        if let Some(q) = lethal {
            lose_life(g, q, 5, Some(p), "burn", Some(true))?;
        } else if let Some(t) = t.filter(|&t| pval(g, t) >= 4.0) {
            apply_removal(g, Some(p), t, "dmg5", None)?;
        } else {
            let opps: Vec<PlayerId> = g.opps(p).collect();
            if let Some(q) = first_min(&opps, |q| g.player(q).life) {
                lose_life(g, q, 5, Some(p), "burn", Some(true))?;
            }
        }
    }
    if g.player(p).emblems.contains(&"venser") {
        if let Some(t) = best_opp_nonland(g, p, |_, _| true).filter(|&t| pval(g, t) >= 1.0) {
            apply_removal(g, Some(p), t, "exile", None)?;
        }
    }
    Ok(())
}

/// rules.emblem_draw: Teferi's emblem exiles an opposing permanent whenever you draw
pub fn emblem_draw(g: &mut Game, p: PlayerId) -> Res {
    if g.player(p).emblems.contains(&"teferi")
        && let Some(t) = best_opp_nonland(g, p, |_, _| true)
    {
        apply_removal(g, Some(p), t, "exile", None)?;
    }
    Ok(())
}

/// rules.emblem_combat: Vraska's emblem (combat damage makes a player lose), Kaito's (a creature from the library)
pub fn emblem_combat(g: &mut Game, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if dmg <= 0 || !g.is_creature(a) {
        return Ok(());
    }
    if g.player(p).emblems.contains(&"vraska") && g.player(d).alive {
        crate::glog!(g, "    Vraska's emblem: {} loses the game", pname(g, d));
        let pl = g.player_mut(d);
        pl.life = 0;
        pl.last_src = Some(p);
        check_state(g)?;
    }
    if g.player(p).emblems.contains(&"kaito") {
        let cs: Vec<CardId> = searchable(g, p)
            .into_iter()
            .filter(|&c| {
                let x = g.db.get(c);
                x.creature && (x.pips.contains('U') || x.pips.contains('B'))
            })
            .collect();
        if let Some(c) = first_max(&cs, |c| (g.db.get(c).bomb, card_worth(g, p, c, false))) {
            remove_card(&mut g.player_mut(p).library, c);
            shuffle_library(g, p);
            enter(g, p, c, Enter::default())?;
        }
    }
    Ok(())
}

/// rules.dovin_blocked: Dovin, Hand of Control's -1: damage to and from m is prevented until its controller's next
/// turn
pub fn dovin_blocked(g: &Game, m: PermId) -> bool {
    match g.perm(m).data.get(DataKey::Dovin) {
        Some(Val::List(v)) => match v.as_slice() {
            [Val::Player(q), Val::Int(t)] => g.player(*q).alive && g.player(*q).turns as i64 == *t,
            _ => false,
        },
        _ => false,
    }
}

// ================================================================== single-card clauses
/// Nature's Claim: its controller gains 4 life
fn natures_claim(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = ctx.target.or_else(|| {
        best_opp_nonland(g, p, |g, m| of_type(g, m, Types::ARTIFACT) || of_type(g, m, Types::ENCHANTMENT))
    });
    if let Some(t) = t.filter(|&t| g.perm(t).on_bf) {
        let q = g.perm(t).owner;
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
        gain(g, q, 4)?;
    }
    Ok("gy")
}

/// Tithe Taker: afterlife 1
fn tithe_afterlife(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(m).owner;
    if trigger_window(g, o, Some(m), "create a 1/1 flying Spirit", None)? {
        let spec =
            Tokens { fly: true, color: Some(Colors::from_letters("W")), types: vec!["spirit"], ..Tokens::new(1, 1) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

/// rules.evasion_blocked: extra blocking restrictions: fear, Amrou Seekers, Signal Pest, intimidate, protection from
/// creature types. Legion Loyalist's battalion (tokens can't block) comes with its attack hook in phase 6: nothing
/// sets `loyalist_turn` before then.
pub fn evasion_blocked(g: &Game, b: PermId, a: PermId) -> bool {
    if g.perm(a).cd.is_none() {
        return false;
    }
    let art_or = |col: char| of_type(g, b, Types::ARTIFACT) || colors_of(g, b).has(col);
    match name_of(g, a) {
        "Shriekmaw" if !art_or('B') => return true,
        "Amrou Seekers" if !art_or('W') => return true,
        "Signal Pest" => {
            let y = g.perm(b);
            let reach = y.cd.is_some_and(|c| g.db.get(c).tag(Tag::Reach));
            if !(y.fly || reach || crate::dsl::has_kw(g, b, "flying") || crate::dsl::has_kw(g, b, "reach")) {
                return true;
            }
        }
        _ => {}
    }
    if equipped(g, a, Tag::Nim) && !art_or('B') {
        return true; // intimidate
    }
    super::rules2::prot_unblockable(g, b, a)
}

/// Brimaz: a 1/1 Cat blocking with it
fn brimaz_block(
    g: &mut Game,
    src: Src,
    _p: PlayerId,
    _atk: &[PermId],
    d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if g.perm(src).owner == d
        && assign.iter().any(|x| x.1 == src)
        && trigger_window(g, d, Some(src), "create a blocking 1/1 Cat", None)?
    {
        let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["cat"], ..Tokens::new(1, 1) };
        make_tokens(g, d, spec)?;
    }
    Ok(())
}

/// Allosaurus Shepherd: your green spells can't be countered
fn shepherd(g: &Game, src: Src, p: PlayerId, c: CardId) -> bool {
    p == g.perm(src).owner && g.db.get(c).pips.contains('G')
}

/// Ezuri: rules.py replaces t1's overrun with no activated abilities (its regeneration is ezuri_regen)
fn ezuri_none(_g: &mut Game, _src: Src, _p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(vec![])
}

/// rules.ezuri_regen: {G}: regenerate another Elf (saves valuable Elves)
pub fn ezuri_regen(g: &mut Game, m: PermId) -> Res<bool> {
    let p = g.perm(m).owner;
    if !has_type(g, m, "elf") || name_of(g, m) == "Ezuri, Renegade Leader" {
        return Ok(false);
    }
    if !g.player(p).perms.iter().any(|&x| !g.perm(x).phased && name_of(g, x) == "Ezuri, Renegade Leader") {
        return Ok(false);
    }
    if pval(g, m) < 3.0 || !can_pay(g, p, 0, "G", false) {
        return Ok(false);
    }
    pay(g, p, 0, "G", false)?;
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "    Ezuri regenerates {}", g.perm(m).name);
    Ok(true)
}

/// Mirkwood Bats: creating tokens drains each opponent
fn bats_make(g: &mut Game, src: Src, p: PlayerId, _kinds: &[Sym], n: i32) -> Res {
    if g.perm(src).owner == p && trigger_window(g, p, Some(src), &format!("each opponent loses {n} life"), None)? {
        for q in g.opps(p).collect::<Vec<_>>() {
            lose_life(g, q, n, Some(p), "drain", None)?;
        }
    }
    Ok(())
}

/// Bloodchief's Thirst: kicked only when needed. Python reads `p.thirst_kicked`, which nothing sets, so a target with
/// mana value 3 or more (chosen when the kicker mana is there) is never destroyed.
fn thirst(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    let mv = g.perm(t).cd.map_or(0, |x| g.db.get(x).cmc);
    let kicked = false; // getattr(p, 'thirst_kicked', False)
    if mv > 2 && !kicked {
        return Ok("gy");
    }
    apply_removal(g, Some(p), t, "destroy", Some(c))?;
    Ok("gy")
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Jaspera Sentinel")?;
    c.dyn_mana_perm = Some(drum);
    c.on_tap_perm = Some(drum_tap);
    r.card(db, "Lotus Petal")?.on_tap_perm = Some(petal_sac);
    let c = r.card(db, "Council's Judgment")?;
    c.resolve = Some(judgment);
    c.prio = Some(judgment_prio);
    let c = r.card(db, "Fact or Fiction")?;
    c.resolve = Some(fof);
    c.prio = Some(fof_prio);
    r.card(db, "Boros Charm")?.hand_options = Some(boros_charm);
    r.card(db, "Nature's Claim")?.resolve = Some(natures_claim);
    let c = r.card(db, "Tithe Taker")?;
    c.self_dies = Some(tithe_afterlife);
    at_once(c, Event::SelfDies, false);
    let c = r.card(db, "Brimaz, King of Oreskos")?;
    c.blocks = Some(brimaz_block);
    at_once(c, Event::Blocks, false);
    r.card(db, "Allosaurus Shepherd")?.uncounterable = Some(shepherd);
    r.card(db, "Ezuri, Renegade Leader")?.options = Some(ezuri_none);
    let c = r.card(db, "Mirkwood Bats")?;
    c.token_created = Some(bats_make);
    at_once(c, Event::TokenCreated, false);
    r.card(db, "Bloodchief's Thirst")?.resolve = Some(thirst);
    Ok(())
}
