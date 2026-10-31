//! Card models, part 3 (10-31): engine features and card code for the pool's last Partial and Unmodeled cards. Each
//! function says which Oracle sentence it plays; `tests/cards_models3.rs` checks them.

use crate::cards::CardDb;
use crate::engine::stack::trigger_window;
use crate::flow::Res;
use crate::hooks::Registry;
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

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
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
