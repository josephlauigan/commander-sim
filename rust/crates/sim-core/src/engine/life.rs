//! Life, damage to players, state-based checks and elimination (engine.py: lose_life ... eliminate).

use crate::engine::hooks;
use crate::engine::values::{chasm, has};
use crate::flow::Res;
use crate::hooks::{Call, Event};
use crate::ids::PlayerId;
use crate::state::Game;
use crate::sym::{Sym, intern};
use crate::tag::Tag;

/// lose_life kinds that are damage (prevented by protection); 'drain' and 'other' are life loss. 'triggers' is mostly
/// damage (Orcish Bowmasters, Kaervek, the ability language's damage); Undermine passes damage = Some(false).
pub const DAMAGE_KINDS: [&str; 4] = ["combat", "burn", "aether", "triggers"];

/// p loses n life (or is dealt n damage) from src. Python's `lose_life`.
pub fn lose_life(g: &mut Game, p: PlayerId, n: i32, src: Option<PlayerId>, kind: Sym, damage: Option<bool>) -> Res {
    let pl = g.player(p);
    if n <= 0 || !pl.alive || pl.life_locked {
        return Ok(());
    }
    let damage = damage.unwrap_or(DAMAGE_KINDS.contains(&kind));
    let mut n = n;
    if damage && let Some(s) = src.filter(|&s| s != p) {
        // Fiery Emancipation: triple damage, once per copy
        let k = g
            .player(s)
            .perms
            .iter()
            .filter(|&&m| !g.perm(m).phased && crate::engine::values::card_name(g, m) == Some("Fiery Emancipation"))
            .count() as u32;
        n *= 3i32.pow(k);
    }
    if damage && prevents_damage(g, p, src) {
        g.player_mut(p).stat("dmg_prevented", n as i64);
        crate::glog!(g, "    {} damage to {} is prevented", n, g.player(p).name);
        return Ok(());
    }
    if damage && crate::impls::alela::serra_floor(g, p, &mut n) {
        return Ok(()); // Serra the Benevolent's emblem left nothing to deal
    }
    if damage && kind != "combat" {
        g.player_mut(p).nc_dmg_this = true;
    }
    if damage && crate::impls::tymna_cards::unlife_infect(g, p, n) {
        return Ok(()); // Phyrexian Unlife at 0 or less life: the damage is dealt as poison counters
    }
    let st = g.turn_stamp();
    let pl = g.player_mut(p);
    pl.life -= n;
    let before = pl.lost_turn.filter(|x| x.0 == st).map_or(0, |x| x.1);
    pl.lost_turn = Some((st, before + n)); // Bloodsoaked Insight, Archfiend, Y'shtola
    if !g.hooks.is_empty() && n > 0 {
        hooks::fire_trigger(g, Event::LoseLife, Call::Life { p, n })?;
    }
    match src {
        Some(s) if s != p => {
            let pl = g.player_mut(p);
            pl.last_src = Some(s);
            pl.last_kind = kind;
            let sp = g.player_mut(s);
            sp.stat("dmg_dealt", n as i64);
            sp.stat(intern(&format!("dmgk_{kind}")), n as i64);
            sp.hit_turn = sp.turns as i32;
        }
        None => g.player_mut(p).last_src = None,
        _ => {}
    }
    crate::ai::note_damage(g, p, src, n);
    Ok(())
}

/// Glacial Chasm prevents all damage to you; The One Ring's protection prevents damage from any source, the controller's own too
pub fn prevents_damage(g: &Game, p: PlayerId, src: Option<PlayerId>) -> bool {
    if g.unpreventable() {
        return false; // Impractical Joke
    }
    if chasm(g, p) {
        return true;
    }
    if crate::impls::bulk_white::damage_shield(g, p) {
        return true; // Ultimate Magic: Holy cast from exile
    }
    if !g.hooks.is_empty() && src != Some(p) && hooks::any_prevent_damage(g, p) {
        return true; // Solitary Confinement
    }
    // protection from everything stops damage from every source, our own included (Pestilence, Ancient Tomb)
    g.player(p).ring_prot
}

/// p gains n life. Python's `gain`.
pub fn gain(g: &mut Game, p: PlayerId, n: i32) -> Res {
    let pl = g.player(p);
    if !pl.alive || pl.life_locked {
        return Ok(());
    }
    if g.opps(p).any(|q| has(g, q, Tag::Erebos)) {
        return Ok(()); // Erebos: opponents can't gain life
    }
    if crate::dsl::no_lifegain(g, p) || (!g.hooks.is_empty() && hooks::any_no_lifegain(g, p)) {
        return Ok(());
    }
    let n = n + crate::impls::t0_giada::gain_bonus(g, p, n); // Angel of Vitality: plus 1
    g.player_mut(p).life += n;
    // (recorded with no hooked permanent out too: infusion cards cast later this turn read it)
    if n > 0 {
        let st = g.turn_stamp();
        let pl = g.player_mut(p);
        let before = pl.gained_turn.filter(|x| x.0 == st).map_or(0, |x| x.1);
        pl.gained_turn = Some((st, before + n));
    }
    if !g.hooks.is_empty() {
        hooks::fire_trigger(g, Event::GainLife, Call::Life { p, n })?;
    }
    Ok(())
}

/// State-based checks: lands that entered without their enter effects, then each player at 0 life, decked, with 21
/// commander damage from one commander or 10 poison is out; the last player standing wins. Python's `check_state`.
pub fn check_state(g: &mut Game) -> Res {
    if g.over {
        return Ok(());
    }
    g.tick()?;
    if !g.hooks.is_empty() {
        hooks::fire_sba(g)?;
    }
    for i in 0..g.players.len() {
        let q = PlayerId(i as u8);
        if g.player(q).alive && g.player(q).lands.iter().any(|&l| !g.land(l).data.truthy(crate::state::DataKey::In)) {
            crate::engine::zones::lands_entered(g, q)?; // a land put onto the battlefield without landfall's path
        }
    }
    sba_toughness(g)?;
    sba_legend(g)?;
    if g.over {
        return Ok(());
    }
    for i in 0..g.players.len() {
        let pl = &g.players[i];
        if pl.alive && ((pl.life <= 0 && !crate::impls::tymna_cards::unlife(g, PlayerId(i as u8))) || pl.decked || pl.cmd_dmg.iter().chain(pl.cmd2_dmg.iter()).any(|&d| d >= 21) || pl.poison >= 10) && !crate::impls::t0_giada::cant_lose(g, PlayerId(i as u8)) {
            eliminate(g, PlayerId(i as u8));
        }
    }
    let alive: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    if alive.len() <= 1 && !g.goldfish {
        g.over = true;
        g.winner = alive.first().copied();
        if g.wintype.is_none() {
            g.wintype = Some("damage");
        }
    }
    Ok(())
}

/// 704.5f: a creature with toughness 0 or less is put into its owner's graveyard, at every check (not only as it
/// enters). `die(.., "sba")` ignores indestructible and regeneration. Cheap: the full `etgh` runs only for a creature
/// whose stored toughness (base + counters + until-end-of-turn bonus) is below 6, since static effects shift it by
/// a few points at most. Armies (amass) are skipped as on entering.
fn sba_toughness(g: &mut Game) -> Res {
    for i in 0..g.players.len() {
        if !g.players[i].alive {
            continue;
        }
        let doomed: Vec<_> = g.players[i]
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                let x = g.perm(m);
                x.on_bf
                    && !x.phased
                    && !x.army
                    && x.tgh + x.plus + x.eot_pt.1 < 6
                    && g.is_creature(m)
                    && crate::engine::values::etgh(g, m) <= 0
            })
            .collect();
        for m in doomed {
            if g.perm(m).on_bf {
                crate::glog!(g, "    {} has toughness 0 or less and dies", g.perm(m).name);
                crate::engine::zones::die(g, m, "sba")?;
            }
        }
    }
    Ok(())
}

/// 704.5j, the legend rule: a player who controls two or more legendary permanents with the same name chooses one and
/// puts the rest into their owners' graveyards (not destruction: indestructible does not help; `die(.., "sba")`).
/// Legendary-ness reads `mine::is_legendary` (copy exceptions: a Helm of the Host token is not legendary). The AI
/// keeps the better one: untapped, equipped, with counters, the real card over a token copy, then the older one
/// (not `pval`: a token copy of a card has no card to value).
/// Cheap: a player with fewer than two permanents is skipped, and names are compared only among legendary ones.
fn sba_legend(g: &mut Game) -> Res {
    for i in 0..g.players.len() {
        if !g.players[i].alive || g.players[i].perms.len() < 2 {
            continue;
        }
        let leg: Vec<_> = g.players[i]
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                let x = g.perm(m);
                x.on_bf && !x.phased && crate::cardcode::is_legendary(g, m)
            })
            .collect();
        if leg.len() < 2 {
            continue;
        }
        let mut done: Vec<crate::sym::Sym> = vec![];
        for &m in &leg {
            let name = g.perm(m).name;
            if done.contains(&name) {
                continue;
            }
            let same: Vec<_> = leg.iter().copied().filter(|&n| g.perm(n).name == name && g.perm(n).on_bf).collect();
            if same.len() < 2 {
                continue;
            }
            done.push(name);
            let score = |n: crate::ids::PermId| -> f64 {
                let x = g.perm(n);
                let gear = g.players[i].perms.iter().filter(|&&e| g.perm(e).attached == Some(n)).count();
                crate::engine::values::epow(g, n) as f64 * 0.25 + if x.tapped { 0.0 } else { 2.0 } + 1.5 * gear as f64
                    + 0.5 * x.plus.max(0) as f64
                    + if x.token { 0.0 } else { 0.25 }
                    - x.born as f64 * 1e-6
            };
            let keep = same.iter().copied().fold(same[0], |b, n| if score(n) > score(b) { n } else { b });
            for n in same {
                if n != keep && g.perm(n).on_bf {
                    crate::glog!(g, "    legend rule: {} keeps one {}, the other goes to the graveyard", g.players[i].name, g.perm(n).name);
                    crate::engine::zones::die(g, n, "sba")?;
                }
            }
        }
    }
    Ok(())
}

/// p is out of the game: its permanents leave with it. Python's `eliminate`.
pub fn eliminate(g: &mut Game, p: PlayerId) {
    let round = g.round;
    let pl = g.player_mut(p);
    pl.alive = false;
    pl.killer = pl.last_src;
    pl.death_kind = if pl.cmd_dmg.iter().chain(pl.cmd2_dmg.iter()).any(|&d| d >= 21) {
        "commander damage"
    } else if pl.decked {
        "decked"
    } else {
        pl.last_kind
    };
    let (killer, kind) = (pl.killer, pl.death_kind);
    pl.stats.insert("elim_round", round as i64);
    if let Some(k) = killer.filter(|&k| k != p) {
        g.player_mut(k).stat(intern(&format!("kills_{kind}")), 1);
    }
    g.elim.push((p, round));
    crate::glog!(
        g,
        "*** {} is eliminated (life {}) by {}",
        g.player(p).name,
        g.player(p).life,
        killer.map_or("?", |k| g.player(k).name)
    );
    for m in std::mem::take(&mut g.player_mut(p).perms) {
        g.perm_mut(m).on_bf = false;
    }
    g.bf_ver += 1;
    if g.garland {
        crate::cardcode::garland_check(g); // Garland's steals from (or owned by) p end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::table;

    #[test]
    fn damage_and_life_loss_and_elimination() {
        let mut g = table(&["sauron", "veyran"]);
        let (s, v) = (PlayerId(0), PlayerId(1));
        lose_life(&mut g, v, 5, Some(s), "burn", None).unwrap();
        assert_eq!(g.player(v).life, 35);
        assert_eq!(g.player(v).last_src, Some(s));
        assert_eq!(g.player(s).stats["dmgk_burn"], 5);
        assert_eq!(g.player(v).lost_turn.unwrap().1, 5);
        gain(&mut g, v, 3).unwrap();
        assert_eq!(g.player(v).life, 38);
        g.player_mut(v).life_locked = true; // Teferi's Protection: life can't change
        lose_life(&mut g, v, 10, Some(s), "burn", None).unwrap();
        assert_eq!(g.player(v).life, 38);
        g.player_mut(v).life_locked = false;
        lose_life(&mut g, v, 38, Some(s), "drain", None).unwrap();
        check_state(&mut g).unwrap();
        assert!(!g.player(v).alive && g.over && g.winner == Some(s));
        assert_eq!((g.player(v).killer, g.player(v).death_kind), (Some(s), "drain"));
        assert_eq!(g.player(s).stats["kills_drain"], 1);
    }

    #[test]
    fn commander_damage_and_poison_eliminate() {
        let mut g = table(&["sauron", "veyran", "seph"]);
        g.player_mut(PlayerId(1)).cmd_dmg[0] = 21;
        g.player_mut(PlayerId(2)).poison = 10;
        check_state(&mut g).unwrap();
        assert!(!g.player(PlayerId(1)).alive && !g.player(PlayerId(2)).alive);
        assert_eq!(g.player(PlayerId(1)).death_kind, "commander damage");
        assert_eq!(g.winner, Some(PlayerId(0)));
    }

    #[test]
    fn the_one_rings_protection_stops_damage_from_every_source() {
        let mut g = table(&["sauron", "veyran"]);
        let (s, v) = (PlayerId(0), PlayerId(1));
        g.player_mut(v).ring_prot = true;
        lose_life(&mut g, v, 4, Some(s), "combat", None).unwrap();
        assert_eq!(g.player(v).life, 40);
        lose_life(&mut g, v, 4, Some(s), "drain", None).unwrap(); // life loss, not damage
        assert_eq!(g.player(v).life, 36);
        lose_life(&mut g, v, 2, Some(v), "other", Some(true)).unwrap(); // its own Ancient Tomb: damage, so prevented too
        assert_eq!(g.player(v).life, 36);
        lose_life(&mut g, v, 1, Some(v), "burn", None).unwrap(); // its own Pestilence
        assert_eq!(g.player(v).life, 36);
        lose_life(&mut g, v, 2, Some(v), "other", Some(false)).unwrap(); // paying life is not damage
        assert_eq!(g.player(v).life, 34);
    }
}
