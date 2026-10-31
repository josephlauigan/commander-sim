//! Card models, part 3 (10-31): engine features and card code for the pool's last Partial and Unmodeled cards. Each
//! function says which Oracle sentence it plays; `tests/cards_models3.rs` checks them.

use crate::cards::CardDb;
use crate::engine::stack::trigger_window;
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

// ================================================================== cipher (Hidden Strings)
/// The `Ctx::mode` of a cipher copy: its resolve must not encode it again.
pub const CIPHER_COPY: &str = "cipher_copy";

/// The creature p encodes a cipher card on: the one likeliest to deal combat damage to a player (unblockable or
/// flying first, then power), never a defender or a 0-power creature.
pub fn cipher_host(g: &Game, p: PlayerId) -> Option<PermId> {
    use crate::dsl::has_kw;
    let score = |m: PermId| {
        let evasive = has_kw(g, m, "flying")
            || has_kw(g, m, "shadow")
            || g.perm(m).data.get(DataKey::Unblockable).is_some_and(|v| v.truthy());
        (evasive as i32) * 100 + g.perm(m).pow.min(99)
    };
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m) && g.perm(m).pow > 0 && !has_kw(g, m, "defender"))
        .max_by_key(|&m| (score(m), std::cmp::Reverse(m.0)))
}

/// Cipher, called by a cipher spell's resolve as its last step: "Then you may exile this spell card encoded on a
/// creature you control." Returns where the card goes: "exile" when it was encoded (the creature remembers it in
/// `DataKey::Cipher`), else "gy". A cipher copy (`CIPHER_COPY`) is not encoded again.
pub fn cipher_encode(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> &'static str {
    if ctx.mode == Some(CIPHER_COPY) || ctx.owner.is_some_and(|o| o != p) {
        return "gy";
    }
    let Some(m) = cipher_host(g, p) else { return "gy" };
    let mut list = match g.perm(m).data.get(DataKey::Cipher) {
        Some(Val::List(v)) => v.clone(),
        _ => vec![],
    };
    list.push(Val::Card(c));
    g.perm_mut(m).data.set(DataKey::Cipher, Val::List(list));
    crate::glog!(g, "    {} is encoded on {}", g.db.get(c).name, g.perm(m).name);
    "exile"
}

/// "Whenever that creature deals combat damage to a player, its controller may cast a copy of the encoded card
/// without paying its mana cost." Called by `cardcode::combat_damage_cards` for each creature that connects. An
/// encoded card must still be in exile (its owner's); the AI always casts the copy.
pub fn cipher_hit(g: &mut Game, p: PlayerId, a: PermId) -> Res {
    let Some(Val::List(list)) = g.perm(a).data.get(DataKey::Cipher) else { return Ok(()) };
    let cards: Vec<CardId> = list
        .iter()
        .filter_map(|v| match v {
            Val::Card(c) => Some(*c),
            _ => None,
        })
        .collect();
    for c in cards {
        if g.over || !g.player(p).alive {
            break;
        }
        let o = g.perm(a).orig;
        if !g.player(p).exile.contains(&c) && !g.player(o).exile.contains(&c) {
            continue;
        }
        let what = format!("cast a copy of {} (cipher)", g.db.get(c).name);
        if !trigger_window(g, p, Some(a), &what, None)? {
            continue;
        }
        let ctx = Ctx { mode: Some(CIPHER_COPY), ..Ctx::default() };
        crate::engine::cast::cast_spell_copy(g, p, c, Some(&ctx))?;
    }
    Ok(())
}

// ================================================================== battles (Invasion of Ikoria)
/// The defense counters of a battle (0 for any other permanent).
pub fn defense(g: &Game, w: PermId) -> i32 {
    g.perm(w).data.int(DataKey::Defense) as i32
}

/// A Siege's protector: the opponent its controller chose, or, when that player has left the game, the opponent
/// with the weakest board (rule 310.11a: its controller chooses a new one).
pub fn protector(g: &Game, w: PermId) -> Option<PlayerId> {
    match g.perm(w).data.get(DataKey::Protector) {
        Some(Val::Player(q)) if g.player(*q).alive => Some(*q),
        _ => pick_protector(g, g.perm(w).owner),
    }
}

/// The opponent p makes protect its Siege: the one whose creatures can block least (fewest untapped creatures, then
/// the least total toughness), so p's attackers get through to the battle.
fn pick_protector(g: &Game, p: PlayerId) -> Option<PlayerId> {
    g.opps(p).min_by_key(|&q| {
        let cs: Vec<PermId> = g.player(q).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
        let tough: i32 = cs.iter().map(|&m| g.perm(m).tgh.max(0)).sum();
        (cs.len(), tough, q.0)
    })
}

/// A battle card c that p cast enters the battlefield with `def` defense counters; a Siege's protector is chosen as it
/// enters. Returns the battle (a noncreature permanent among p's `perms`).
pub fn battle_enter(g: &mut Game, p: PlayerId, c: CardId, def: i32) -> Res<PermId> {
    let how = crate::engine::zones::Enter { was_cast: true, ..Default::default() };
    let w = crate::engine::zones::enter(g, p, c, how)?;
    if !g.perm(w).on_bf {
        return Ok(w);
    }
    g.perm_mut(w).data.set(DataKey::Defense, Val::Int(def as i64));
    if let Some(q) = pick_protector(g, p) {
        g.perm_mut(w).data.set(DataKey::Protector, Val::Player(q));
        crate::glog!(
            g,
            "    {} enters with {def} defense counters, protected by {}",
            g.db.get(c).name,
            g.player(q).name
        );
    }
    Ok(w)
}

/// The battles p can attack when it attacks player d: the Sieges p controls that d protects.
pub fn battles_for(g: &Game, p: PlayerId, d: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&w| live(g, w) && defense(g, w) > 0 && protector(g, w) == Some(d))
        .collect()
}

/// Combat damage dealt to a battle removes that many defense counters; with none left it is defeated. Called by
/// `combat::resolve_blocks` for an attacker that `combat::walker_attacks` sent at battle w.
pub fn battle_damage(g: &mut Game, p: PlayerId, a: PermId, w: PermId, dmg: i32) -> Res {
    let left = (defense(g, w) - dmg).max(0);
    g.perm_mut(w).data.set(DataKey::Defense, Val::Int(left as i64));
    crate::glog!(g, "    {} deals {} to {} (defense {})", g.perm(a).name, dmg, g.perm(w).name, left);
    if left == 0 {
        battle_defeated(g, p, w)?;
    }
    Ok(())
}

/// "When the last defense counter is removed from this battle, it's defeated": a Siege's controller exiles it, then
/// may cast it transformed without paying its mana cost (Invasion of Ikoria: Zilortha, Apex of Ikoria).
fn battle_defeated(g: &mut Game, p: PlayerId, w: PermId) -> Res {
    let Some(c) = g.perm(w).cd else { return Ok(()) };
    let o = g.perm(w).orig;
    crate::engine::zones::leave(g, w)?;
    crate::engine::zones::to_zone_card(g, w, crate::engine::zones::Zone::Exile);
    crate::glog!(g, "    {} is defeated", g.db.get(c).name);
    if &*g.db.get(c).name == super::t6_green2::IKORIA && o == p {
        zilortha_cast(g, p, c)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Zilortha, Apex of Ikoria
pub const ZILORTHA: &str = "Zilortha, Apex of Ikoria";

/// Invasion of Ikoria's back face, cast from exile without paying its mana cost: a legendary 8/8 Dinosaur with reach
/// (the card's own keywords). The card's permanent is made a creature (`DataKey::Anim`) under Zilortha's name.
fn zilortha_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let ex = &mut g.player_mut(p).exile;
    let Some(i) = ex.iter().rposition(|&x| x == c) else { return Ok(()) };
    ex.remove(i);
    crate::glog!(g, "  {} casts {ZILORTHA} (transformed, free)", g.player(p).name);
    crate::engine::cast::on_cast(g, p, c)?;
    let how = crate::engine::zones::Enter { was_cast: true, ..Default::default() };
    let m = crate::engine::zones::enter(g, p, c, how)?;
    let x = g.perm_mut(m);
    x.data.set(DataKey::Anim, Val::Bool(true));
    x.name = crate::sym::intern(ZILORTHA);
    x.pow = 8;
    x.tgh = 8;
    g.bf_ver += 1;
    Ok(())
}

/// "For each non-Human creature you control, you may have that creature assign its combat damage as though it
/// weren't blocked." Called by `combat::resolve_blocks` for attacker a blocked by b: true when p controls Zilortha, a
/// is not a Human, and p takes the damage to the player (always, unless the blocker would die and is worth 4+).
pub fn zilortha_unblocked(g: &Game, p: PlayerId, a: PermId, b: PermId, b_dies: bool) -> bool {
    let has = g.player(p).perms.iter().any(|&m| live(g, m) && &*g.perm(m).name == ZILORTHA && g.is_creature(m));
    if !has || g.perm(a).cd.is_some_and(|c| g.db.get(c).has_subtype("human")) {
        return false;
    }
    !(b_dies && crate::engine::values::pval(g, b) >= 4.0)
}

// ================================================================== "becomes tapped" (Magda, Brazen Outlaw)
/// The "becomes tapped" event for a watcher `src` (a card whose ability triggers when a permanent becomes tapped),
/// read at state checks (`hooks::fire_sba`): each permanent of `cands` that src saw untapped and that is tapped now
/// became tapped. A permanent that entered tapped, or was tapped when src arrived, never triggers it. Tapping and
/// untapping again between two state checks is not seen (the mana and attack events report those at once, through
/// `tapped_now`).
pub fn newly_tapped(g: &mut Game, src: PermId, cands: &[PermId]) -> Vec<PermId> {
    let mut out = vec![];
    for &m in cands {
        let mut armed = match g.perm(m).data.get(DataKey::TapArmed) {
            Some(Val::List(v)) => v.clone(),
            _ => vec![],
        };
        let i = armed.iter().position(|v| *v == Val::Perm(src));
        match (g.perm(m).tapped, i) {
            (false, None) => armed.push(Val::Perm(src)),
            (true, Some(i)) => {
                armed.remove(i);
                out.push(m);
            }
            _ => continue,
        }
        g.perm_mut(m).data.set(DataKey::TapArmed, Val::List(armed));
    }
    out
}

/// m just became tapped and src's trigger was reported by the event that tapped it (mana, an attack): disarm src
/// for m, so that the next state check does not report it again.
pub fn tapped_now(g: &mut Game, src: PermId, m: PermId) {
    if let Some(Val::List(v)) = g.perm(m).data.get(DataKey::TapArmed) {
        let v: Vec<Val> = v.iter().filter(|x| **x != Val::Perm(src)).cloned().collect();
        g.perm_mut(m).data.set(DataKey::TapArmed, Val::List(v));
    }
}

// ================================================================== Astral Cornucopia (atraxa-infect)
pub const CORNUCOPIA: &str = "Astral Cornucopia";

/// "{T}: Choose a color. Add one mana of that color for each charge counter on this artifact." (its `rock=1:A` tag
/// gives the colours; the amount is its charge counters, `DataKey::Charge`, which proliferate adds to)
fn cornucopia_mana(g: &Game, _p: PlayerId, m: PermId) -> u32 {
    g.perm(m).data.int(DataKey::Charge).max(0) as u32
}

/// The X the AI casts it for: all its spare mana, three per X
fn cornucopia_x(g: &Game, p: PlayerId) -> u32 {
    crate::engine::mana::total_mana(g, p, false) / 3
}

/// {X}{X}{X}: the AI casts it in its main phase with the largest X its mana pays (X >= 1), as a hand option: the
/// generic path would cast it for X = 0 (`cornucopia_prio` keeps it out).
fn cornucopia_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<crate::hooks::Opt>> {
    if post.is_none() || g.active != Some(p) || !g.stack.is_empty() || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    if crate::engine::cast::spell_locked(g, p, c) {
        return Ok(vec![]);
    }
    let x = cornucopia_x(g, p);
    if x == 0 {
        return Ok(vec![]);
    }
    Ok(vec![crate::hooks::Opt {
        utility: if x == 1 { 0.4 } else { 0.3 + 0.3 * x as f64 },
        label: format!("cast {CORNUCOPIA} (X = {x})").into(),
        act: Some(crate::hooks::Action::Plan { f: cornucopia_cast, arg: c.0 as i64 }),
    }])
}

/// Pay {X}{X}{X} and cast it; it enters with X charge counters (the counters step: Doubling Season, Vorinclex)
pub fn cornucopia_cast(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let x = cornucopia_x(g, p);
    if x == 0 || !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    g.pay_for = Some(c);
    let paid = crate::engine::mana::pay(g, p, 3 * x, "", false);
    g.pay_for = None;
    if !paid? {
        return Ok(false);
    }
    crate::glog!(g, "  {} casts {CORNUCOPIA} (X = {x})", g.player(p).name);
    let n0 = g.perms.len();
    let ctx = Ctx { x: x as i32, ..Ctx::default() };
    if !crate::engine::cast::cast_card(g, p, c, "hand", ctx)? {
        return Ok(true);
    }
    let m = (n0..g.perms.len())
        .map(|i| PermId(i as u32))
        .find(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c) && g.perm(m).data.get(DataKey::Charge).is_none());
    if let Some(m) = m {
        use crate::engine::repl::{Ctr, On, Path, counters};
        let k = counters(g, p, On::Perm(m), Ctr::Other, x as i32, Path::ALL);
        g.perm_mut(m).data.set(DataKey::Charge, Val::Int(k as i64));
        crate::glog!(g, "    {CORNUCOPIA} enters with {k} charge counters");
    }
    Ok(true)
}

/// never cast by the generic path (X would be 0): `cornucopia_options` casts it
fn cornucopia_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

// ================================================================== Mandate of Peace (dack-fayden)
pub const MANDATE: &str = "Mandate of Peace";

/// a card's own timing restriction, for `cast::castable`: "Cast this spell only during combat." (Mandate of Peace)
pub fn cast_restricted(g: &Game, c: CardId) -> bool {
    &*g.db.get(c).name == MANDATE && !super::t6_stax::in_combat_now(g)
}

/// The AI holding Mandate of Peace casts it when an attack on it would be lethal, or would deal half its life (10 or
/// more): it ends the combat phase (no combat damage) and its opponents can't cast spells this turn. Called once the
/// blockers are declared (`cardcode::defend_hooks`); `combat::resolve_blocks` stops when the combat ended.
fn mandate_defend(
    g: &mut Game,
    c: CardId,
    d: PlayerId,
    p: PlayerId,
    atk: &mut Vec<PermId>,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    use crate::engine::{combat::double_strike, values::epow};
    if !g.player(d).hand.contains(&c) || !crate::engine::cast::castable(g, d, c, "hand") {
        return Ok(());
    }
    let unblocked: i32 = atk
        .iter()
        .filter(|&&a| g.perm(a).on_bf && g.perm(a).owner == p && !assign.iter().any(|x| x.0 == a))
        .map(|&a| epow(g, a).max(0) * if double_strike(g, a) { 2 } else { 1 })
        .sum();
    let life = g.player(d).life;
    if unblocked < life && (unblocked < 10 || 2 * unblocked < life) {
        return Ok(());
    }
    let (gn, pips) = crate::engine::mana::cost_of(g, d, c);
    if !crate::engine::mana::can_pay(g, d, gn, &pips, false) {
        return Ok(());
    }
    g.pay_for = Some(c);
    let paid = crate::engine::mana::pay(g, d, gn, &pips, false);
    g.pay_for = None;
    if !paid? {
        return Ok(());
    }
    crate::glog!(g, "  {} casts {MANDATE} ({unblocked} damage coming)", g.player(d).name);
    crate::engine::cast::cast_card(g, d, c, "hand", Ctx::default())?;
    Ok(())
}

// ================================================================== Innkeeper's Talent (atraxa-infect)
pub const INNKEEPER: &str = "Innkeeper's Talent";

fn class_level(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.get(DataKey::Level).map_or(1, Val::int)
}

/// the levels of the Innkeeper's Talents p controls
fn innkeepers(g: &Game, p: PlayerId) -> impl Iterator<Item = i64> + '_ {
    g.player(p)
        .perms
        .iter()
        .filter(move |&&m| live(g, m) && &*g.perm(m).name == INNKEEPER)
        .map(move |&m| class_level(g, m))
}

/// "At the beginning of combat on your turn, put a +1/+1 counter on target creature you control." The AI picks the
/// creature with the most power (an infect or toxic creature first: its counter is poison).
fn innkeeper_combat(g: &mut Game, src: Src, p: PlayerId) -> Res {
    use crate::dsl::has_kw;
    let o = g.perm(src).owner;
    if !live(g, src) || p != o || g.active != Some(o) {
        return Ok(());
    }
    let pick = g.player(o).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).max_by_key(|&m| {
        ((has_kw(g, m, "infect") || has_kw(g, m, "toxic")) as i32, crate::engine::values::epow(g, m), m.0)
    });
    let Some(t) = pick else { return Ok(()) };
    if trigger_window(g, o, Some(src), "put a +1/+1 counter on target creature you control", None)? && live(g, t) {
        crate::engine::repl::put_plus(g, t, 1);
    }
    Ok(())
}

/// Level 2 ({G}) and Level 3 ({3}{G}), gained one at a time as a sorcery: the AI takes Level 2 once a permanent of
/// its has counters, and Level 3 whenever it can pay (it doubles every counter it puts, poison included).
fn innkeeper_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<crate::hooks::Opt>> {
    if post.is_none() || g.active != Some(p) || !g.stack.is_empty() || g.perm(src).owner != p || !live(g, src) {
        return Ok(vec![]);
    }
    let (generic, level) = match class_level(g, src) {
        1 => (0, 2),
        2 => (3, 3),
        _ => return Ok(vec![]),
    };
    let countered = g.player(p).perms.iter().any(|&m| live(g, m) && has_counters(g, m));
    if (level == 2 && !countered) || !crate::engine::mana::can_pay(g, p, generic, "G", false) {
        return Ok(vec![]);
    }
    Ok(vec![crate::hooks::Opt {
        utility: if level == 3 { 2.0 } else { 0.8 },
        label: format!("{INNKEEPER} level {level}"),
        act: Some(crate::hooks::Action::Ability { src, f: innkeeper_level_up, arg: level }),
    }])
}

fn innkeeper_level_up(g: &mut Game, src: PermId, p: PlayerId, level: i64) -> Res<bool> {
    let generic = if level == 2 { 0 } else { 3 };
    if !live(g, src)
        || g.perm(src).owner != p
        || class_level(g, src) != level - 1
        || !crate::engine::mana::can_pay(g, p, generic, "G", false)
    {
        return Ok(false);
    }
    crate::engine::mana::pay(g, p, generic, "G", false)?;
    if crate::engine::stack::ability_window(g, p, Some(src), &format!("level {level}"), None, None)? && live(g, src) {
        g.perm_mut(src).data.set(DataKey::Level, Val::Int(level));
        crate::glog!(g, "  {} levels {INNKEEPER} to {level}", g.player(p).name);
    }
    Ok(true)
}

/// a permanent with counters on it (+1/+1 or -1/-1, loyalty, charge, lore, time, the engine's other counters)
fn has_counters(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.plus != 0
        || x.loyalty.is_some_and(|l| l > 0)
        || [DataKey::Charge, DataKey::Counters, DataKey::Lore, DataKey::Vanish].iter().any(|&k| x.data.int(k) > 0)
}

/// Level 2: "Permanents you control with counters on them have ward {1}." (`cardcode::aura_ward`)
pub fn innkeeper_ward(g: &Game, m: PermId) -> u32 {
    let o = g.perm(m).owner;
    if g.hooks.is_empty() || !has_counters(g, m) {
        return 0;
    }
    innkeepers(g, o).filter(|&l| l >= 2).count() as u32
}

/// Level 3: "If you would put one or more counters on a permanent or player, put twice that many of each of those
/// kinds of counters on that permanent or player instead." How many doublings `actor` gets (`repl::counters`).
pub fn innkeeper_doublings(g: &Game, actor: PlayerId) -> usize {
    if g.hooks.is_empty() {
        return 0;
    }
    innkeepers(g, actor).filter(|&l| l >= 3).count()
}

// ================================================================== Venerated Rotpriest (atraxa-infect)
pub const ROTPRIEST: &str = "Venerated Rotpriest";

/// The permanents spell c targets as it is cast (its target, the creature it is cast on, a person's extra targets).
fn spell_target_perms(ctx: &Ctx) -> Vec<PermId> {
    let mut v: Vec<PermId> = ctx.target.into_iter().chain(ctx.host).chain(ctx.perms.iter().copied()).collect();
    v.dedup();
    v
}

/// "Whenever a creature you control becomes the target of a spell, target opponent gets a poison counter." Called by
/// `cast::cast_card` as a spell is cast (any player's spell). The AI gives the poison counter to the opponent with
/// the most poison (the closest to ten), then the one with the least life.
pub fn spell_targeted(g: &mut Game, ctx: &Ctx) -> Res {
    let Some(rp) = g.db.id(ROTPRIEST) else { return Ok(()) };
    for m in spell_target_perms(ctx) {
        if !live(g, m) || !g.is_creature(m) {
            continue;
        }
        let o = g.perm(m).owner;
        let priests: Vec<PermId> =
            g.player(o).perms.iter().copied().filter(|&y| live(g, y) && g.perm(y).cd == Some(rp)).collect();
        for y in priests {
            let Some(q) = g.opps(o).max_by_key(|&q| (g.player(q).poison, -g.player(q).life, q.0)) else { break };
            if trigger_window(g, o, Some(y), "target opponent gets a poison counter", None)? {
                crate::engine::repl::give_poison(g, o, q, 1, crate::engine::repl::Path::POISON);
                crate::glog!(g, "    {ROTPRIEST}: {} gets a poison counter", g.player(q).name);
            }
        }
    }
    Ok(())
}

// ================================================================== Ajani, Sleeper Agent (atraxa-infect)
pub const AJANI: &str = "Ajani, Sleeper Agent";
const AJANI_EMBLEM: &str = "ajani_sleeper";

fn is_creature_or_walker(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.creature || d.types.has(crate::cards::Types::PLANESWALKER)
}

/// +1: "Reveal the top card of your library. If it's a creature or planeswalker card, put it into your hand.
/// Otherwise, you may put it on the bottom of your library." The AI bottoms a card it values below 4 (a land it
/// still needs stays).
fn ajani_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let Some(&c) = g.player(p).library.first() else { return Ok(()) };
    crate::glog!(g, "    Ajani reveals {}", g.db.get(c).name);
    if is_creature_or_walker(g, c) {
        g.player_mut(p).library.remove(0);
        g.player_mut(p).hand.push(c);
        return Ok(());
    }
    let lands_in_hand = g.player(p).hand.iter().filter(|&&x| g.db.get(x).land).count();
    let keep =
        if g.db.get(c).land { lands_in_hand == 0 } else { crate::engine::tutors::card_worth(g, p, c, false) >= 4.0 };
    if !keep {
        let pl = g.player_mut(p);
        pl.library.remove(0);
        pl.library.push(c);
    }
    Ok(())
}

fn ajani_plus_val(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(1.2)
}

/// the creatures the -3 puts its counters on: all three on the best infect or toxic creature, else one each on the
/// three with the most power
fn ajani_targets(g: &Game, p: PlayerId) -> Vec<PermId> {
    use crate::dsl::has_kw;
    let mut cs: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
    cs.sort_by_key(|&m| std::cmp::Reverse((crate::engine::values::epow(g, m), m.0)));
    if let Some(&i) = cs.iter().find(|&&m| has_kw(g, m, "infect") || has_kw(g, m, "toxic")) {
        return vec![i, i, i];
    }
    cs.truncate(3);
    cs
}

/// -3: "Distribute three +1/+1 counters among up to three target creatures. They gain vigilance until end of turn."
fn ajani_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let ts = ajani_targets(g, p);
    for &m in &ts {
        crate::engine::repl::put_plus(g, m, 1);
    }
    for m in ts {
        let x = g.perm_mut(m);
        if !x.eot_kw.contains(&"vigilance") {
            x.eot_kw.push(crate::sym::intern("vigilance"));
        }
    }
    Ok(())
}

fn ajani_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    match ajani_targets(g, p).len() {
        0 => None,
        _ => Some(1.8),
    }
}

/// -6: "You get an emblem with 'Whenever you cast a creature or planeswalker spell, target opponent gets two poison
/// counters.'" (`ajani_emblem_cast`)
fn ajani_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    g.player_mut(p).emblems.push(crate::sym::intern(AJANI_EMBLEM));
    Ok(())
}

fn ajani_ult_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let spells = g.player(p).hand.iter().filter(|&&c| is_creature_or_walker(g, c)).count();
    Some(3.0 + spells as f64)
}

/// the emblem: called by `rules::emblem_cast` for each spell p casts; the opponent closest to ten poison gets two
pub fn ajani_emblem_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let k = g.player(p).emblems.iter().filter(|&&e| e == AJANI_EMBLEM).count();
    if k == 0 || !is_creature_or_walker(g, c) {
        return Ok(());
    }
    for _ in 0..k {
        let Some(q) = g.opps(p).max_by_key(|&q| (g.player(q).poison, -g.player(q).life, q.0)) else { break };
        if trigger_window(g, p, None, "target opponent gets two poison counters", None)? {
            crate::engine::repl::give_poison(g, p, q, 2, crate::engine::repl::Path::POISON);
            crate::glog!(g, "    Ajani's emblem: {} gets two poison counters", g.player(q).name);
        }
    }
    crate::engine::life::check_state(g)
}

const AJANI_ABS: [super::common::WalkerAb; 3] = [
    super::common::WalkerAb { delta: 1, label: "reveal the top card", val: ajani_plus_val, eff: ajani_plus },
    super::common::WalkerAb { delta: -3, label: "three +1/+1 counters", val: ajani_minus_val, eff: ajani_minus },
    super::common::WalkerAb {
        delta: -6,
        label: "emblem: poison per creature spell",
        val: ajani_ult_val,
        eff: ajani_ult,
    },
];

// ================================================================== Ixhel, Scion of Atraxa (atraxa-infect)
pub const IXHEL: &str = "Ixhel, Scion of Atraxa";

/// "Corrupted -- At the beginning of your end step, each opponent who has three or more poison counters exiles the
/// top card of their library face down. You may look at and play those cards for as long as they remain exiled, and
/// you may spend mana as though it were mana of any color to cast those spells." The cards are held in Ixhel's
/// controller's hand as stolen cards (`yshtola::take_card`: any type of mana, back to their owner's graveyard).
fn ixhel_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if !live(g, src) || p != o {
        return Ok(());
    }
    let qs: Vec<PlayerId> = g.opps(o).filter(|&q| g.player(q).poison >= 3 && !g.player(q).library.is_empty()).collect();
    if qs.is_empty() || !trigger_window(g, o, Some(src), "corrupted opponents exile their top card", None)? {
        return Ok(());
    }
    for q in qs {
        if g.player(q).library.is_empty() || !g.player(q).alive {
            continue;
        }
        let c = g.player_mut(q).library.remove(0);
        super::yshtola::take_card(g, o, c, q, IXHEL);
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, IXHEL)?.end_step = Some(ixhel_end);
    super::common::walker(r, db, AJANI, &AJANI_ABS)?;
    let c = r.card(db, INNKEEPER)?;
    c.each_combat = Some(innkeeper_combat);
    c.options = Some(innkeeper_options);
    r.card(db, MANDATE)?.hand_defend = Some(mandate_defend);
    let c = r.card(db, CORNUCOPIA)?;
    c.dyn_mana_perm = Some(cornucopia_mana);
    c.hand_options = Some(cornucopia_options);
    c.prio = Some(cornucopia_prio);
    Ok(())
}
