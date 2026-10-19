//! New cards, batch b: Cecil, Dark Knight // Cecil, Redeemed Paladin and Lightning, Army of One (Tymna/Kraum), and
//! Marvin, Murderous Mimic (Kefka). The audit verdicts are in data/card_overrides.toml.
//!
//! - **Cecil**: `cecil_deals` is called by combat.rs whenever a creature deals combat damage (to a player or a
//!   creature); it does nothing for other cards.
//! - **Lightning**: the Stagger mark is `Player::stagger`, read by `engine::life::stagger_mult` in `lose_life`
//!   (player damage), `apply_removal` (damage to permanents) and the combat kill checks.
//! - **Marvin**: borrows the `{T}` abilities of four of Kefka's creatures (`dsl::borrowed_tap_options`).

use crate::cards::CardDb;
use crate::dsl;
use crate::engine::life::lose_life;
use crate::engine::values::card_name;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};

pub const CECIL: &str = "Cecil, Dark Knight";
pub const LIGHTNING: &str = "Lightning, Army of One";
pub const MARVIN: &str = "Marvin, Murderous Mimic";
/// the commander starting life total (Cecil flips at half of it)
const START_LIFE: i32 = 40;

fn card_if<'a>(r: &'a mut Registry, db: &CardDb, name: &str) -> Result<Option<&'a mut crate::hooks::CardImpl>, String> {
    if db.id(name).is_none() {
        return Ok(None);
    }
    Ok(Some(r.card(db, name)?))
}

// ================================================================== Cecil, Dark Knight // Cecil, Redeemed Paladin
pub fn redeemed(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Flipped)
}

/// Darkness: whenever Cecil deals damage, its controller loses that much life. Then if their life total is half their
/// starting life total or less, Cecil untaps and transforms (a 4/4 lifelink without deathtouch).
pub fn cecil_deals(g: &mut Game, m: PermId, dmg: i32) -> Res {
    if dmg <= 0 || card_name(g, m) != Some(CECIL) || redeemed(g, m) {
        return Ok(());
    }
    let p = g.perm(m).owner;
    crate::glog!(g, "    Cecil's Darkness: {} loses {}", g.player(p).name, dmg);
    lose_life(g, p, dmg, None, "other", Some(false))?;
    if g.perm(m).on_bf && g.player(p).life <= START_LIFE / 2 {
        let x = g.perm_mut(m);
        x.tapped = false;
        x.data.set(DataKey::Flipped, Val::Bool(true));
        x.pow = 4;
        x.tgh = 4;
        x.dt = false;
        x.lifelink = true;
        crate::glog!(g, "    Cecil transforms into Cecil, Redeemed Paladin (4/4 lifelink)");
    }
    Ok(())
}

/// Protect: whenever Cecil, Redeemed Paladin attacks, the other attacking creatures gain indestructible until end of turn
fn cecil_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p || !atk.contains(&src) || !redeemed(g, src) {
        return Ok(vec![]);
    }
    for &a in atk {
        if a != src && g.perm(a).on_bf {
            super::partials::eot_kw(g, a, "indestructible");
        }
    }
    crate::glog!(g, "    Cecil's Protect: the other attackers are indestructible this turn");
    Ok(vec![])
}

// ================================================================== Lightning, Army of One
/// Stagger: when Lightning deals combat damage to a player, until its controller's next turn all damage that would be
/// dealt to that player or a permanent they control is doubled
fn lightning_hit(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if a != src || dmg <= 0 || d == p {
        return Ok(());
    }
    let turns = g.player(p).turns;
    g.player_mut(d).stagger = Some((p, turns));
    crate::glog!(
        g,
        "    Lightning's Stagger: damage to {} is doubled until {}'s next turn",
        g.player(d).name,
        g.player(p).name
    );
    Ok(())
}

// ================================================================== Marvin, Murderous Mimic
/// the creatures whose {T} abilities Marvin copies, best first (Niv's draw first)
const COPIED: [&str; 4] = ["Niv-Mizzet, the Firemind", "Rummaging Goblin", "Viashino Fangtail", "Blightspeaker"];

fn marvin_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || !g.perm(src).on_bf || g.perm(src).phased {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for (rank, name) in COPIED.iter().enumerate() {
        let Some(cd) = g.db.id(name) else { continue };
        let here = g.player(p).perms.iter().any(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && x.cd == Some(cd) && g.is_creature(m)
        });
        if !here {
            continue;
        }
        // Blightspeaker's {4}, {T} search has a mana cost and isn't copied (the compiled search takes any permanent)
        for (idx, u) in dsl::borrowed_tap_options(g, p, src, cd)? {
            let bonus = (COPIED.len() - rank) as f64 * 0.1;
            out.push(Opt {
                utility: u + bonus,
                label: format!("Marvin copies {name}"),
                act: Some(Action::Ability { src, f: marvin_go, arg: super::partials::pack(cd.0 as u32, idx as u32) }),
            });
        }
    }
    Ok(out)
}

fn marvin_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (cd, idx) = super::partials::unpack(arg);
    dsl::activate_borrowed(g, p, src, CardId(cd as _), idx as u16)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if let Some(c) = card_if(r, db, CECIL)? {
        c.attack = Some(cecil_attack);
    }
    if let Some(c) = card_if(r, db, LIGHTNING)? {
        c.combat_damage = Some(lightning_hit);
    }
    if let Some(c) = card_if(r, db, MARVIN)? {
        c.options = Some(marvin_options);
    }
    Ok(())
}

// ================================================================== AI
/// the Tymna/Kraum cast priorities of Cecil and Lightning: cheap attackers that make Tymna draw (None for other cards).
/// Cecil costs life with every point of damage, so he waits when the life total is low and he hasn't flipped yet.
pub fn partner_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let name = &*g.db.get(c).name;
    if name == CECIL {
        return Some(if g.player(p).life <= START_LIFE / 2 + 6 { 40 } else { 76 });
    }
    if name == LIGHTNING {
        return Some(78);
    }
    None
}

/// Marvin's cast priority in Kefka's deck: a cheap body (52), much better when a creature whose {T} ability he copies
/// is already out (72; Niv's draw 80)
pub fn marvin_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    if &*g.db.get(c).name != MARVIN {
        return None;
    }
    let out = |n: &str| {
        g.player(p).perms.iter().any(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && card_name(g, m) == Some(n)
        })
    };
    Some(if out(COPIED[0]) {
        80
    } else if COPIED[1..].iter().any(|n| out(n)) {
        72
    } else {
        52
    })
}

/// choose_defender: a bonus for the opponent Lightning would connect with (no untapped creature of theirs can block it):
/// its Stagger then doubles the rest of the attack, and the lifelink pays for it
pub fn stagger_bonus(g: &Game, p: PlayerId, q: PlayerId) -> f64 {
    let ready = g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.on_bf && !x.tapped && !x.sick && !x.noatk && !x.phased && card_name(g, m) == Some(LIGHTNING)
    });
    let Some(l) = ready else { return 0.0 };
    let blockers = g
        .player(q)
        .perms
        .iter()
        .filter(|&&b| {
            g.is_creature(b) && !g.perm(b).tapped && !g.perm(b).phased && crate::engine::combat::can_block(g, b, l)
        })
        .count();
    if blockers == 0 { 2.5 } else { 0.0 }
}
