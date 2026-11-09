//! New cards, batch b: Cecil, Dark Knight // Cecil, Redeemed Paladin and Lightning, Army of One (Tymna/Kraum), and
//! Marvin, Murderous Mimic (Kefka). The audit verdicts are in data/card_overrides.toml.
//!
//! - **Cecil**: `cecil_deals` is called by combat.rs whenever a creature deals combat damage (to a player or a
//!   creature); it does nothing for other cards.
//! - **Lightning**: the Stagger mark is `Player::stagger`, read by `engine::life::stagger_mult` in `lose_life`
//!   (player damage), `apply_removal` (damage to permanents) and the combat kill checks.
//! - **Marvin**: has the compiled activated abilities of the creatures its controller has (`dsl::borrowed_options`).

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
/// Marvin has all activated abilities of the creatures its controller has that don't share its name. The compiled
/// ones (`dsl::borrowed_abilities`: cost and effects as printed, mana and sacrifice costs included, token copies
/// too) are offered here; Ioreth's untaps (t6_legends1b) and Krenko's and Kiki-Jiki's (t45_krenko) are the
/// hand-written ones also borrowed. Left out: any other creature's card-code ability, and mana abilities used to pay
/// for costs (Marvin taps for mana only through the engine's mana tags, which he doesn't have).
pub(crate) fn marvin_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || !g.perm(src).on_bf || g.perm(src).phased {
        return Ok(vec![]);
    }
    let sorcery_ok = g.active == Some(p) && matches!(g.step, crate::state::Step::Main1 | crate::state::Step::Main2);
    let mut out = vec![];
    for (cd, idx, u) in dsl::borrowed_options(g, p, src, sorcery_ok)? {
        out.push(Opt {
            utility: u,
            label: format!("Marvin copies {}", g.db.get(cd).name),
            act: Some(Action::Ability { src, f: marvin_go, arg: super::partials::pack(cd.0 as u32, idx as u32) }),
        });
    }
    // --- model-gaps-a3: the hand-written (card code) activated abilities: the `options` hook of each other creature
    // is run with Marvin as the source, so Marvin pays the {T}, the sacrifice of "this creature", the counters ...
    // (Krenko, Kiki-Jiki and Ioreth are borrowed by their own code, Kefka's {8} below, the sacrifice outlets by
    // `common::outlets`, the creatures' mana abilities by `mana::mana_units`)
    if !crate::cardcode::ability_locked(g, src, p) {
        let me = g.perm(src).name;
        let donors: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                let x = g.perm(m);
                m != src && x.name != me && x.cd.is_some() && x.on_bf && !x.phased && !x.neutered && g.is_creature(m)
            })
            .collect();
        let mut seen: Vec<crate::sym::Sym> = vec![];
        for m in donors {
            let (nm, cd) = (g.perm(m).name, g.perm(m).cd.unwrap());
            if seen.contains(&nm) || crate::engine::values::stopped(g, &g.db.get(cd).name) {
                continue;
            }
            seen.push(nm);
            if &*g.db.get(cd).name == "Kefka, Court Mage // Kefka, Ruler of Ruin" {
                for mut o in super::mine::kefka_borrowed(g, src, p, post, m) {
                    o.label = format!("Marvin copies {}: {}", g.db.get(cd).name, o.label);
                    out.push(o);
                }
                continue;
            }
            if matches!(
                &*g.db.get(cd).name,
                "Krenko, Mob Boss" | "Kiki-Jiki, Mirror Breaker" | "Ioreth of the Healing House"
            ) {
                continue;
            }
            let Some(f) = g.registry.get(cd).and_then(|c| c.options) else { continue };
            let dn = g.db.get(cd).name.clone();
            if NOT_ACTIVATED.contains(&&*dn) {
                continue; // its `options` are casting permissions / prepared spells, not activated abilities
            }
            for mut o in f(g, src, p, post)? {
                // "Transform this creature" does nothing to Marvin (it has no other face): not offered
                if &*dn == "Bloodline Keeper // Lord of Lineage" && o.label.starts_with("transform") {
                    continue;
                }
                // the abilities that sacrifice their source: the AI weighs Marvin as the thing given up
                if matches!(&*dn, "Sakura-Tribe Elder" | "Cankerbloom") {
                    o.utility -= crate::engine::values::pval(g, src);
                }
                o.label = format!("Marvin copies {dn}: {}", o.label);
                out.push(o);
            }
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
        c.combat_damage = Some(super::partials::hope_hit); // Hope of Ghirapur's ability: dealt combat damage by it
        c.blocks = Some(super::partials::hordechief_blocks); // Brutal Hordechief's ability: you choose how they block
        c.grant_kw = Some(super::partials::tekuthal_kw); // Drivnod's / Tekuthal's indestructible counter, put on Marvin by its ability
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

/// --- model-gaps-a4: creatures whose `options` hook offers what is not an activated ability (casting from the top of the
/// library or the graveyard, a prepared copy of a spell, the paired creature's ability): Marvin does not get these
pub const NOT_ACTIVATED: &[&str] = &[
    "Abigale, Poet Laureate // Heroic Stanza",
    "Blazing Firesinger",
    "Blazing Firesinger // Seething Song",
    "Conspicuous Snoop",
    "Deadeye Navigator",
    "Emeritus of Conflict // Lightning Bolt",
    "Emeritus of Ideation // Ancestral Recall",
    "Emeritus of Woe // Demonic Tutor",
    "Grave Researcher // Reanimate",
    "Korlessa, Scale Singer",
    "Leech Collector // Bloodletting",
    "Muldrotha, the Gravetide",
    "Realmwalker",
    "Restless Bloodseeker // Bloodsoaked Reveler",
    "Rivaz of the Claw",
    "Sanar, Unfinished Genius // Wild Idea",
    "Silas Renn, Seeker Adept",
    "Six",
    "Skycoach Conductor // All Aboard",
    "Studious First-Year",
    "The Reality Chip",
    "Valgavoth, Terror Eater",
];
