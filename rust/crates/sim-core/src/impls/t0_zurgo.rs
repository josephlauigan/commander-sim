//! Tier 0 pool deck card code: the cards of the zurgo precon (decklists/pool/t0-b2-precons/) that weren't modeled.
//! Filled in by the t0-zurgo agent (rust/agents/t0-zurgo.md).
//!
//! The mechanics this module owns, and where the engine calls into it (each call is one line in a shared file):
//!   - myriad (`attack_keywords`, called by `combat::attack_triggers_once`): printed on Goldlust Triad, granted by
//!     Legion Loyalty, an equipped Blade of Selves and Ironwill Forger's lieutenant ability. The token copies attack
//!     their own player (`Game::atk_targets`, then `route_new_attackers`) and are exiled at end of combat
//!     (`end_of_combat`, `DataKey::MyriadTok`). A legendary creature is never copied (the legend rule would put the
//!     copy straight back into the graveyard; the ability is a "may").
//!   - mobilize (Zurgo's attack hook; the tokens carry `DataKey::EotSac`, which the end step already sacrifices),
//!   - goad (`DataKey::Goaded`, read by `goad_attackers` in `combat::combat`; Grenzo),
//!   - fabricate (Angel of Invention's enter hook),
//!   - hideaway (Windbrisk Heights),
//!   - lieutenant (Ironwill Forger's beginning-of-combat hook),
//!   - token doubling and replacement: Kaya's -2 (`kaya_mult`, read by `dsl::token_mult`) and Divine Visitation
//!     (`divine_visitation`, read by `make_tokens`), which also apply to the token copies made here,
//!   - "a creature token leaves the battlefield" (`token_left`, called by `zones::leave`): Zurgo, Twilight Drover.

use crate::cards::{CardDb, Colors};
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, ability_window_card, trigger_window};
use crate::engine::values::{card_name, commander_out, epow, etgh, has_type, pval};
use crate::engine::zones::{Tokens, die, draw, enter_token_copy, exile_perm, make_tokens, max_by, min_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::common::{WalkerAb, walker};
use crate::state::{Ctx, DataKey, Game, PermData, Val};
use crate::sym::Sym;
use crate::tag::Tag;

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn on_bf(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// p's untapped-or-not battlefield permanents with this card name
fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| !g.perm(m).phased && card_name(g, m) == Some(name)).collect()
}

fn creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect()
}

fn legendary(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Leg))
}

fn is_warrior(g: &Game, m: PermId) -> bool {
    g.perm(m).warrior || has_type(g, m, "warrior")
}

/// the player attacker m is attacking, `d` (the main defender) if the engine doesn't know
fn defender_of(g: &Game, m: PermId, d: PlayerId) -> PlayerId {
    g.atk_targets.iter().find(|x| x.0 == m).map_or(d, |x| x.1)
}

/// two permanent ids in one option argument
fn pack(a: PermId, b: PermId) -> i64 {
    ((b.0 as i64) << 32) | a.0 as i64
}

fn unpack(x: i64) -> (PermId, PermId) {
    (PermId((x & 0xffff_ffff) as u32), PermId((x >> 32) as u32))
}

fn remove_first(v: &mut Vec<CardId>, c: CardId) -> bool {
    match v.iter().position(|&x| x == c) {
        Some(i) => {
            v.remove(i);
            true
        }
        None => false,
    }
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

// ------------------------------------------------------------------ rules read by the engine
/// Divine Visitation: p controls it, so creature tokens p creates are 4/4 white Angels with flying and vigilance
pub fn divine_visitation(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| !g.perm(m).phased && card_name(g, m) == Some("Divine Visitation"))
}

/// Kaya, Geist Hunter -2: twice as many tokens are created under p's control this turn
pub fn kaya_mult(g: &Game, p: PlayerId) -> u32 {
    if g.player(p).flag_turn.get("kaya_double") == Some(&g.turn_stamp()) { 2 } else { 1 }
}

/// how many times p has cast its commander from the command zone this game (Commander's Insignia)
pub fn command_casts(g: &Game, p: PlayerId) -> i32 {
    let pl = g.player(p);
    ((pl.tax + pl.tax2) / 2) as i32
}

// ------------------------------------------------------------------ a creature token leaves the battlefield
/// zones::leave: creature token m (controlled by p) left the battlefield. Zurgo Stormrender: whenever a creature
/// token you control leaves the battlefield, draw a card if it was attacking, otherwise each opponent loses 1 life.
/// Twilight Drover: whenever a creature token leaves the battlefield (anyone's), a +1/+1 counter on it.
pub fn token_left(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if !g.is_creature(m) {
        return Ok(());
    }
    let attacking = g.in_combat.contains(&m);
    for z in named(g, p, "Zurgo Stormrender") {
        if !g.perm(z).on_bf || g.over {
            continue;
        }
        if attacking {
            crate::glog!(g, "    Zurgo Stormrender: an attacking token left, {} draws", g.player(p).name);
            draw(g, p, 1, false)?;
        } else {
            crate::glog!(g, "    Zurgo Stormrender: a token left, each opponent loses 1");
            for q in g.opps(p).collect::<Vec<_>>() {
                lose_life(g, q, 1, Some(p), "drain", None)?;
            }
        }
    }
    let all: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for q in all {
        for d in named(g, q, "Twilight Drover") {
            if g.perm(d).on_bf {
                crate::cardcode::add_counters(g, d, 1);
            }
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ goad
/// the player goading m, while the goad lasts (until that player's next turn)
fn goaded_by(g: &Game, m: PermId) -> Option<PlayerId> {
    if let Some(Val::List(v)) = g.perm(m).data.get(DataKey::Goaded)
        && let (Some(Val::Player(q)), Some(Val::Int(t))) = (v.first(), v.get(1))
        && g.player(*q).alive
        && g.player(*q).turns as i64 == *t
    {
        return Some(*q);
    }
    // Ghoulish Impetus (t0_anikthea): the enchanted creature is goaded by the Aura's controller while it stays attached
    g.players.iter().flat_map(|pl| pl.perms.iter().copied()).find_map(|a| {
        let x = g.perm(a);
        (x.on_bf
            && x.attached == Some(m)
            && !x.neutered
            && x.cd.is_some_and(|c| &*g.db.get(c).name == "Ghoulish Impetus"))
        .then_some(x.owner)
        .filter(|&q| g.player(q).alive)
    })
}

fn goad(g: &mut Game, goader: PlayerId, m: PermId) {
    let turns = g.player(goader).turns as i64;
    g.perm_mut(m).data.set(DataKey::Goaded, Val::List(vec![Val::Player(goader), Val::Int(turns)]));
}

/// combat::combat: goaded creatures attack if able, and a player other than the goader if able
pub fn goad_attackers(g: &mut Game, p: PlayerId, groups: &mut Vec<(PlayerId, Vec<PermId>)>, _d: PlayerId) -> Res {
    let cands: Vec<(PermId, PlayerId)> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter_map(|m| {
            let x = g.perm(m);
            if !x.on_bf
                || x.tapped
                || x.phased
                || !g.is_creature(m)
                || epow(g, m) <= 0
                || (x.sick && !crate::engine::combat::has_haste(g, m))
            {
                return None;
            }
            goaded_by(g, m).map(|q| (m, q))
        })
        .collect();
    for (m, goader) in cands {
        let others: Vec<PlayerId> =
            g.opps(p).filter(|&q| q != goader && !crate::engine::values::shielded(g, q)).collect();
        let target = others.iter().copied().min_by_key(|&q| g.player(q).life);
        let cur = groups.iter().position(|(_, xs)| xs.contains(&m));
        let dest = match (cur, target) {
            (Some(i), Some(t)) if groups[i].0 == goader => {
                groups[i].1.retain(|&x| x != m);
                t
            }
            (None, Some(t)) => t,
            (None, None) if !crate::engine::values::shielded(g, goader) => goader,
            _ => continue,
        };
        match groups.iter_mut().find(|x| x.0 == dest) {
            Some(gr) => gr.1.push(m),
            None => groups.push((dest, vec![m])),
        }
    }
    groups.retain(|(_, xs)| !xs.is_empty());
    Ok(())
}

// ------------------------------------------------------------------ attack: counts, myriad, Within Range
/// does attacker m have myriad: printed, granted until end of turn (Ironwill Forger), by Legion Loyalty, or by an
/// equipped Blade of Selves
fn has_myriad(g: &Game, m: PermId, loyalty: bool) -> bool {
    let x = g.perm(m);
    if x.eot_kw.contains(&"myriad") || loyalty {
        return true;
    }
    if x.cd.is_some_and(|c| !x.neutered && g.db.get(c).has_kw("myriad")) {
        return true;
    }
    g.player(x.owner).perms.iter().any(|&e| g.perm(e).attached == Some(m) && card_name(g, e) == Some("Blade of Selves"))
}

/// combat::attack_triggers_once: p attacked with `atk` (d: the main defender). Records how many creatures attacked
/// and whether a commander did (Windbrisk Heights, Neriv), then runs myriad. Returns the token copies, which enter
/// tapped and attacking.
pub fn attack_keywords(g: &mut Game, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let st = g.turn_stamp();
    let n = atk.len() as u32;
    let cmd = atk.iter().any(|&m| g.perm(m).is_cmd);
    let pl = g.player_mut(p);
    pl.atk_count = match pl.atk_count {
        (Some(s), k) if s == st => (Some(st), k.max(n)),
        _ => (Some(st), n),
    };
    if cmd {
        pl.cmd_atk = Some(st);
        neriv_release(g, p);
    }
    let loyalty = g.player(p).perms.iter().any(|&m| !g.perm(m).phased && card_name(g, m) == Some("Legion Loyalty"));
    let mut new = vec![];
    for &m in atk {
        if !(g.perm(m).on_bf && g.perm(m).owner == p && has_myriad(g, m, loyalty)) || legendary(g, m) {
            continue;
        }
        let dd = defender_of(g, m, d);
        let others: Vec<PlayerId> = g.opps(p).filter(|&q| q != dd).collect();
        if others.is_empty() || !trigger_window(g, p, Some(m), "myriad: token copies", Some(4.0))? {
            continue;
        }
        crate::glog!(g, "    {} has myriad: token copies attack the other opponents", g.perm(m).name);
        for q in others {
            new.extend(myriad_copies(g, p, m, q)?);
        }
    }
    Ok(new)
}

/// a token copy of attacker m, tapped and attacking q, exiled at end of combat. A token is copied by what it is (its
/// size, types and keywords; counters aren't copied); a card by `enter_token_copy`, so its enter effects happen.
/// Divine Visitation turns the copy into an Angel and Kaya's -2 doubles it, as for any token.
fn myriad_copies(g: &mut Game, p: PlayerId, m: PermId, q: PlayerId) -> Res<Vec<PermId>> {
    let mut made = vec![];
    if g.perm(m).token {
        let x = g.perm(m).clone();
        let mut data = PermData::default();
        if let Some(v) = x.data.get(DataKey::Kws).cloned() {
            data.set(DataKey::Kws, v);
        }
        let spec = Tokens {
            tgh: Some(x.tgh),
            fly: x.fly,
            warrior: x.warrior,
            attacking: true,
            lifelink: x.lifelink,
            sick: false,
            dt: x.dt,
            color: Some(x.colors),
            types: x.ttypes.clone(),
            data,
            ..Tokens::new(1, x.pow)
        };
        for t in make_tokens(g, p, spec)? {
            g.perm_mut(t).vig |= x.vig;
            made.push(t);
        }
    } else if let Some(cd) = g.perm(m).cd {
        if divine_visitation(g, p) {
            made = make_tokens(g, p, Tokens { attacking: true, sick: false, ..Tokens::new(1, 4) })?;
        } else {
            for _ in 0..crate::dsl::token_mult(g, p) {
                if let Some(t) = enter_token_copy(g, p, cd)? {
                    let x = g.perm_mut(t);
                    x.tapped = true;
                    x.sick = false;
                    made.push(t);
                }
            }
        }
    }
    made.retain(|&t| g.perm(t).on_bf);
    for &t in &made {
        let x = g.perm_mut(t);
        x.tapped = true;
        x.sick = false;
        x.temp = true; // if end of combat is somehow missed, the end step exiles it
        x.data.set(DataKey::MyriadTok, Val::Bool(true));
        g.atk_targets.push((t, q));
    }
    Ok(made)
}

/// combat::combat: the tokens made by attack triggers attack their own player, not always the main defender; then
/// the triggers that count attackers (Within Range) resolve, last, so they see every attacker
pub fn route_new_attackers(
    g: &mut Game,
    p: PlayerId,
    atk: &[PermId],
    new: &[PermId],
    d: PlayerId,
    groups: &mut Vec<(PlayerId, Vec<PermId>)>,
) -> Res {
    for &m in new {
        let known = g.atk_targets.iter().find(|x| x.0 == m).map(|x| x.1);
        let q = known.filter(|&q| g.player(q).alive).unwrap_or(d);
        if known.is_none() {
            g.atk_targets.push((m, q));
        }
        match groups.iter_mut().find(|x| x.0 == q) {
            Some(gr) => gr.1.push(m),
            None => groups.push((q, vec![m])),
        }
    }
    // Within Range: whenever you attack, each opponent loses life equal to the number of creatures attacking them
    for w in named(g, p, "Within Range") {
        if g.over || !g.perm(w).on_bf || atk.is_empty() {
            continue;
        }
        let counts: Vec<(PlayerId, i32)> = g
            .opps(p)
            .map(|q| {
                let n = atk.iter().filter(|&&m| g.perm(m).on_bf && defender_of(g, m, d) == q).count() as i32;
                (q, n)
            })
            .filter(|x| x.1 > 0)
            .collect();
        if counts.is_empty() || !trigger_window(g, p, Some(w), "each opponent loses life per attacker", Some(5.0))? {
            continue;
        }
        crate::glog!(g, "    Within Range: each opponent loses life for the creatures attacking them");
        for (q, n) in counts {
            lose_life(g, q, n, Some(p), "drain", None)?;
        }
    }
    Ok(())
}

/// combat::combat: end of combat. The myriad tokens are exiled (still attacking, so Zurgo draws for each).
pub fn end_of_combat(g: &mut Game, p: PlayerId) -> Res {
    for m in g.player(p).perms.clone() {
        if g.perm(m).on_bf && g.perm(m).data.truthy(DataKey::MyriadTok) {
            exile_perm(g, m)?;
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ end of turn
/// turn::end_step: Kaya's deathtouch ends; Neriv's cards that were put in hand for the turn go back to exile
pub fn end_cleanup(g: &mut Game, p: PlayerId) {
    for m in g.player(p).perms.clone() {
        if g.perm(m).eot_kw.contains(&"kaya_dt") {
            let x = g.perm_mut(m);
            x.dt = false;
            x.eot_kw.retain(|k| *k != "kaya_dt");
        }
    }
    if g.player(p).neriv.is_empty() {
        return;
    }
    let list = std::mem::take(&mut g.player_mut(p).neriv);
    let mut keep = vec![];
    for c in list {
        let pl = g.player_mut(p);
        if remove_first(&mut pl.hand, c) {
            pl.exile.push(c);
            keep.push(c);
        } else if pl.exile.contains(&c) {
            keep.push(c);
        }
    }
    g.player_mut(p).neriv = keep;
}

/// the cards Neriv exiled can be played during this turn (a commander attacked): into the hand. A land goes there
/// for good (the same card id can be a basic land in the hand too, so it can't be told apart again).
fn neriv_release(g: &mut Game, p: PlayerId) {
    for c in g.player(p).neriv.clone() {
        let land = g.db.get(c).land;
        let pl = g.player_mut(p);
        if remove_first(&mut pl.exile, c) {
            pl.hand.push(c);
            if land {
                remove_first(&mut pl.neriv, c);
            }
        }
    }
}

// ================================================================== Zurgo Stormrender
/// Mobilize 1: whenever it attacks, a tapped and attacking 1/1 red Warrior token, sacrificed at the beginning of the
/// next end step. (The second ability, `token_left`, is in the engine's leave path.)
fn zurgo_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !atk.contains(&src) || !on_bf(g, src) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "mobilize 1: a tapped and attacking Warrior", Some(4.0))? {
        return Ok(vec![]);
    }
    let dd = defender_of(g, src, d);
    let spec = Tokens {
        attacking: true,
        sick: false,
        warrior: true,
        color: Some(Colors::from_letters("R")),
        types: vec!["warrior"],
        ..Tokens::new(1, 1)
    };
    crate::glog!(g, "    {} mobilizes: a tapped and attacking Warrior", g.perm(src).name);
    let toks = make_tokens(g, p, spec)?;
    for &t in &toks {
        g.perm_mut(t).data.set(DataKey::EotSac, Val::Bool(true));
        g.atk_targets.push((t, dd));
    }
    Ok(toks)
}

// ================================================================== Ainok Strike Leader
/// whenever you attack with this creature and/or your commander, for each opponent, a 1/1 red Goblin token tapped and
/// attacking that player
fn ainok_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !on_bf(g, src) {
        return Ok(vec![]);
    }
    if !atk.iter().any(|&m| m == src || (g.perm(m).is_cmd && g.perm(m).owner == p)) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "a Goblin attacking each opponent", Some(4.0))? {
        return Ok(vec![]);
    }
    crate::glog!(g, "    Ainok Strike Leader: a Goblin attacks each opponent");
    let mut out = vec![];
    for q in g.opps(p).collect::<Vec<_>>() {
        let spec = Tokens {
            attacking: true,
            sick: false,
            color: Some(Colors::from_letters("R")),
            types: vec!["goblin"],
            ..Tokens::new(1, 1)
        };
        for t in make_tokens(g, p, spec)? {
            g.atk_targets.push((t, q));
            out.push(t);
        }
    }
    Ok(out)
}

// ================================================================== Angel of Invention
/// Fabricate 2: two +1/+1 counters or two 1/1 Servos. The Servos: with the anthem each is a 2/2, and tokens are what
/// the rest of this deck feeds on.
fn angel_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "fabricate 2", None)? {
        return Ok(());
    }
    crate::glog!(g, "    Angel of Invention fabricates");
    let servos = creatures(g, o).len() >= 2 || divine_visitation(g, o) || named(g, o, "Zurgo Stormrender").len() == 1;
    if servos || !g.perm(src).on_bf {
        let spec = Tokens { color: Some(Colors::NONE), types: vec!["servo", "artifact"], ..Tokens::new(2, 1) };
        make_tokens(g, o, spec)?;
    } else {
        crate::cardcode::add_counters(g, src, 2);
    }
    Ok(())
}

// ================================================================== Chittering Witch
/// when it enters, a 1/1 black Rat for each opponent
fn witch_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "a Rat for each opponent", None)? {
        return Ok(());
    }
    let n = g.opps(o).count() as u32;
    let spec = Tokens { color: Some(Colors::from_letters("B")), types: vec!["rat"], ..Tokens::new(n, 1) };
    make_tokens(g, o, spec)?;
    Ok(())
}

/// -N/-N until end of turn: it dies if its toughness is N or less
fn shrink(g: &mut Game, p: PlayerId, m: PermId, n: i32) -> Res {
    if !g.perm(m).on_bf {
        return Ok(());
    }
    if etgh(g, m) <= n {
        return apply_removal(g, Some(p), m, &format!("shrink{n}"), None);
    }
    let x = g.perm_mut(m);
    x.eot_pt = (x.eot_pt.0 - n, x.eot_pt.1 - n);
    Ok(())
}

/// the creature p would sacrifice: the cheapest, never the commander, nothing worth more than `cap`
fn fodder(g: &Game, p: PlayerId, except: Option<PermId>, cap: f64) -> Option<PermId> {
    let cands: Vec<PermId> = creatures(g, p)
        .into_iter()
        .filter(|&m| Some(m) != except && !g.perm(m).is_cmd && !crate::engine::values::no_sac(g, m))
        .collect();
    let m = min_by(&cands, |m| pval(g, m))?;
    (pval(g, m) <= cap).then_some(m)
}

/// {1}{B}, sacrifice a creature: target creature gets -2/-2. Aimed at a creature the -2/-2 kills that is worth more
/// than the sacrifice (a token, usually).
fn witch_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || post.is_none() || !on_bf(g, src) || !can_pay(g, p, 1, "B", false) {
        return Ok(vec![]);
    }
    let Some(f) = fodder(g, p, None, 2.5) else { return Ok(vec![]) };
    let tg: Vec<PermId> = legal_targets(g, p, "destroy", "c", false, None)
        .into_iter()
        .filter(|&m| etgh(g, m) <= 2 && pval(g, m) >= pval(g, f) + 1.5)
        .collect();
    let Some(t) = max_by(&tg, |m| pval(g, m)) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: pval(g, t) - pval(g, f) - 1.0,
        label: format!("Chittering Witch: sacrifice {}, -2/-2 on {}", g.perm(f).name, g.perm(t).name),
        act: Some(Action::Ability { src, f: witch_go, arg: pack(t, f) }),
    }])
}

fn witch_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (t, f) = unpack(arg);
    if !on_bf(g, src) || owner(g, src) != p || !on_bf(g, t) || !on_bf(g, f) || owner(g, f) != p || f == t {
        return Ok(false);
    }
    if !can_pay(g, p, 1, "B", false) {
        return Ok(false);
    }
    pay(g, p, 1, "B", false)?;
    die(g, f, "sac")?;
    if ability_window(g, p, Some(src), "-2/-2", None, Some(t))? {
        shrink(g, p, t, 2)?;
    }
    Ok(true)
}

// ================================================================== Twilight Drover
/// {2}{W}, remove a +1/+1 counter from it: two 1/1 white Spirit tokens with flying. Used with spare mana in the second
/// main phase or at the end of the turn.
fn drover_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p
        || post == Some(false)
        || !on_bf(g, src)
        || g.perm(src).plus < 1
        || !can_pay(g, p, 2, "W", false)
    {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0 + 0.2 * g.perm(src).plus as f64,
        label: "Twilight Drover: two Spirits".into(),
        act: Some(Action::Ability { src, f: drover_go, arg: 0 }),
    }])
}

fn drover_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on_bf(g, src) || owner(g, src) != p || g.perm(src).plus < 1 || !can_pay(g, p, 2, "W", false) {
        return Ok(false);
    }
    pay(g, p, 2, "W", false)?;
    g.perm_mut(src).plus -= 1;
    if ability_window(g, p, Some(src), "two 1/1 flying Spirits", None, None)? {
        let spec =
            Tokens { fly: true, color: Some(Colors::from_letters("W")), types: vec!["spirit"], ..Tokens::new(2, 1) };
        make_tokens(g, p, spec)?;
    }
    Ok(true)
}

// ================================================================== Ironwill Forger
/// Lieutenant: at the beginning of combat on your turn, if you control your commander, target nonlegendary creature
/// you control gains myriad until end of turn. The creature is picked from the attackers (the AI has declared them by
/// the time this runs).
fn forger_combat(g: &mut Game, src: Src, p: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || g.active != Some(p) || !on_bf(g, src) || !commander_out(g, p) || g.opps(p).count() < 2 {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> = g
        .atk_targets
        .iter()
        .map(|x| x.0)
        .filter(|&m| on_bf(g, m) && owner(g, m) == p && !legendary(g, m) && !g.perm(m).eot_kw.contains(&"myriad"))
        .collect();
    let Some(best) = max_by(&cands, |m| pval(g, m) + epow(g, m) as f64) else { return Ok(vec![]) };
    if trigger_window(g, p, Some(src), "lieutenant: a creature gains myriad", Some(3.0))? && on_bf(g, best) {
        crate::glog!(g, "    Ironwill Forger gives {} myriad", g.perm(best).name);
        g.perm_mut(best).eot_kw.push("myriad");
    }
    Ok(vec![])
}

// ================================================================== Grenzo, Havoc Raiser
/// whenever a creature you control deals combat damage to a player, choose one: goad target creature that player
/// controls; or exile the top card of that player's library and cast it this turn, mana of any colour. The AI exiles
/// when it has mana to cast what it finds (a card or two a turn), and goads their best creature otherwise.
fn grenzo_damage(g: &mut Game, src: Src, p: PlayerId, _a: PermId, d: PlayerId, _dmg: i32) -> Res {
    let o = owner(g, src);
    if o != p || d == o || !on_bf(g, src) || !g.player(d).alive {
        return Ok(());
    }
    let goadable: Vec<PermId> =
        creatures(g, d).into_iter().filter(|&m| epow(g, m) > 0 && goaded_by(g, m).is_none_or(|q| q != o)).collect();
    let can_exile = !g.player(d).library.is_empty();
    let spare = total_mana(g, o, false) as usize;
    let held = g.player(o).impulse.len();
    let best = max_by(&goadable, |m| pval(g, m));
    let exile =
        can_exile && (best.is_none() || (spare >= 3 && held <= spare / 3) || best.is_some_and(|m| pval(g, m) < 2.0));
    if !exile && best.is_none() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "goad, or exile the top card of their library", Some(3.0))? {
        return Ok(());
    }
    if exile {
        let Some(c) = g.player_mut(d).library.pop() else { return Ok(()) };
        if g.db.get(c).land {
            g.player_mut(d).exile.push(c); // a land can't be cast
        } else {
            crate::impls::yshtola::take_card(g, o, c, d, "Grenzo");
            g.player_mut(o).impulse.push(c);
        }
    } else if let Some(t) = best.filter(|&t| on_bf(g, t)) {
        goad(g, o, t);
        crate::glog!(g, "    {} is goaded by {}", g.perm(t).name, g.player(o).name);
    }
    Ok(())
}

// ================================================================== Mindblade Render
/// whenever your opponents are dealt combat damage, if any of that damage was dealt by a Warrior, you draw a card and
/// you lose 1 life: once for each combat damage step
fn mindblade_damage(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    let o = owner(g, src);
    if d == o || !on_bf(g, src) || !is_warrior(g, a) {
        return Ok(());
    }
    let key = crate::sym::intern(&format!("mindblade_{}_{}", src.0, g.player(g.active.unwrap_or(o)).combat_no));
    let st = g.turn_stamp();
    // (the engine probes a hook up to its trigger window: nothing may be set before it)
    if g.player(o).flag_turn.get(key) != Some(&st)
        && trigger_window(g, o, Some(src), "draw a card, lose 1 life", Some(4.0))?
    {
        g.player_mut(o).flag_turn.insert(key, st);
        crate::glog!(
            g,
            "    Mindblade Render: a Warrior hurt {}, {} draws and loses 1",
            g.player(d).name,
            g.player(o).name
        );
        draw(g, o, 1, false)?;
        lose_life(g, o, 1, Some(o), "life", None)?;
    }
    Ok(())
}

// ================================================================== Myr Battlesphere
/// whenever it attacks, you may tap X untapped Myr you control: it gets +X/+0 and deals X damage to the player it is
/// attacking. The AI taps every untapped Myr (the new tokens can't attack anyway).
fn battlesphere_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !atk.contains(&src) || !on_bf(g, src) {
        return Ok(vec![]);
    }
    let myr: Vec<PermId> =
        creatures(g, p).into_iter().filter(|&m| m != src && !g.perm(m).tapped && has_type(g, m, "myr")).collect();
    if myr.is_empty() || !trigger_window(g, p, Some(src), "tap X Myr: +X/+0 and X damage", Some(5.0))? {
        return Ok(vec![]);
    }
    let myr: Vec<PermId> = myr.into_iter().filter(|&m| on_bf(g, m) && !g.perm(m).tapped).collect();
    let x = myr.len() as i32;
    crate::glog!(g, "    Myr Battlesphere taps {} Myr: +{}/+0 and {} damage", x, x, x);
    for m in myr {
        g.perm_mut(m).tapped = true;
    }
    if x > 0 {
        g.perm_mut(src).eot_pt.0 += x;
        let q = defender_of(g, src, d);
        if g.player(q).alive {
            lose_life(g, q, x, Some(p), "triggers", Some(true))?;
        }
    }
    Ok(vec![])
}

// ================================================================== Redoubled Stormsinger
/// whenever it attacks, for each creature token you control that entered this turn, a tapped and attacking copy of
/// that token, sacrificed at the beginning of the next end step
fn stormsinger_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !atk.contains(&src) || !on_bf(g, src) {
        return Ok(vec![]);
    }
    let toks: Vec<PermId> = creatures(g, p)
        .into_iter()
        .filter(|&m| g.perm(m).token && g.perm(m).born > g.turn_enter && !g.perm(m).data.truthy(DataKey::MyriadTok))
        .collect();
    if toks.is_empty() || !trigger_window(g, p, Some(src), "attacking copies of this turn's tokens", Some(5.0))? {
        return Ok(vec![]);
    }
    crate::glog!(g, "    Redoubled Stormsinger copies {} token(s) that entered this turn", toks.len());
    let dd = defender_of(g, src, d);
    let mut out = vec![];
    for m in toks {
        if !g.perm(m).on_bf {
            continue;
        }
        let x = g.perm(m).clone();
        let mut data = PermData::default();
        if let Some(v) = x.data.get(DataKey::Kws).cloned() {
            data.set(DataKey::Kws, v);
        }
        let spec = Tokens {
            tgh: Some(x.tgh),
            fly: x.fly,
            warrior: x.warrior,
            attacking: true,
            lifelink: x.lifelink,
            sick: false,
            dt: x.dt,
            color: Some(x.colors),
            types: x.ttypes.clone(),
            data,
            ..Tokens::new(1, x.pow)
        };
        for t in make_tokens(g, p, spec)? {
            let y = g.perm_mut(t);
            y.vig |= x.vig;
            y.data.set(DataKey::EotSac, Val::Bool(true));
            g.atk_targets.push((t, dd));
            out.push(t);
        }
    }
    Ok(out)
}

// ================================================================== Neriv, Crackling Vanguard
/// the number of differently named tokens p controls (creature tokens by type, Treasures, Food, Clues)
fn token_kinds(g: &Game, p: PlayerId) -> usize {
    let mut seen: Vec<String> = vec![];
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        if x.token && !x.phased {
            let k = if x.ttypes.is_empty() { "token".to_string() } else { x.ttypes.join(" ") };
            if !seen.contains(&k) {
                seen.push(k);
            }
        }
    }
    let pl = g.player(p);
    seen.len() + (pl.treasures > 0) as usize + (pl.foods > 0) as usize + (pl.clues > 0) as usize
}

/// whenever it attacks, exile cards from the top of your library equal to the number of differently named tokens you
/// control; during any turn you attacked with a commander, you may play those cards (they stay in exile otherwise,
/// and come to the hand for each turn a commander attacks)
fn neriv_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !atk.contains(&src) || !on_bf(g, src) {
        return Ok(vec![]);
    }
    let n = token_kinds(g, p).min(g.player(p).library.len());
    if n == 0 || !trigger_window(g, p, Some(src), "exile cards from the top of your library", Some(4.0))? {
        return Ok(vec![]);
    }
    crate::glog!(g, "    Neriv exiles {} card(s) from the top of the library", n);
    for _ in 0..n {
        let Some(c) = g.player_mut(p).library.pop() else { break };
        let pl = g.player_mut(p);
        pl.exile.push(c);
        pl.neriv.push(c);
    }
    if g.player(p).cmd_atk == Some(g.turn_stamp()) {
        neriv_release(g, p);
    }
    Ok(vec![])
}

// ================================================================== Kaya, Geist Hunter
/// the tokens of p, strongest first
fn token_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    creatures(g, p).into_iter().filter(|&m| g.perm(m).token).collect()
}

fn kaya_plus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let n = creatures(g, p).len().min(5) as f64;
    Some(1.0 + 0.4 * n)
}

/// +1: creatures you control gain deathtouch until end of turn; a +1/+1 counter on up to one creature token
fn kaya_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    for m in creatures(g, p) {
        let x = g.perm_mut(m);
        if !x.dt {
            x.dt = true;
            x.eot_kw.push("kaya_dt");
        }
    }
    let toks = token_creatures(g, p);
    if let Some(t) = max_by(&toks, |m| epow(g, m) as f64) {
        crate::cardcode::add_counters(g, t, 1);
    }
    Ok(())
}

/// token makers that will go to work this turn: creatures that make tokens when they attack, and token spells in hand
const ATTACK_TOKEN_MAKERS: [&str; 7] = [
    "Zurgo Stormrender",
    "Hero of Bladehold",
    "Adeline, Resplendent Cathar",
    "Legion Warboss",
    "Ainok Strike Leader",
    "Redoubled Stormsinger",
    "Neriv, Crackling Vanguard",
];
const TOKEN_SPELLS: [&str; 6] = [
    "Lingering Souls",
    "Shadow Summoning",
    "Release the Dogs",
    "Siege-Gang Commander",
    "Beetleback Chief",
    "Myr Battlesphere",
];

fn kaya_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let attackers = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            !x.sick
                && !x.tapped
                && g.is_creature(m)
                && card_name(g, m).is_some_and(|n| ATTACK_TOKEN_MAKERS.contains(&n))
        })
        .count();
    let spells = g
        .player(p)
        .hand
        .iter()
        .filter(|&&c| TOKEN_SPELLS.contains(&&*g.db.get(c).name))
        .filter(|&&c| {
            let (cg, cp) = cost_of(g, p, c);
            can_pay(g, p, cg, &cp, false)
        })
        .count();
    let v = 1.3 * attackers as f64 + 1.0 * spells as f64;
    (v >= 2.0).then_some(1.5 + v)
}

/// -2: until end of turn, if one or more tokens would be created under your control, twice that many are created
fn kaya_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let st = g.turn_stamp();
    g.player_mut(p).flag_turn.insert("kaya_double", st);
    Ok(())
}

fn kaya_ult_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let n: usize = g.players.iter().map(|q| q.gy.len()).sum();
    let mine = g.player(p).gy.len();
    (n >= 8 && n > 2 * mine).then_some(4.0 + 0.2 * n as f64)
}

/// -6: exile all cards from all graveyards, then a 1/1 white Spirit with flying for each card exiled
fn kaya_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let mut n = 0;
    for i in 0..g.players.len() {
        let pl = &mut g.players[i];
        let cards = std::mem::take(&mut pl.gy);
        n += cards.len();
        pl.exile.extend(cards);
    }
    if n > 0 {
        let spec = Tokens {
            fly: true,
            color: Some(Colors::from_letters("W")),
            types: vec!["spirit"],
            ..Tokens::new(n as u32, 1)
        };
        make_tokens(g, p, spec)?;
    }
    Ok(())
}

static KAYA_ABS: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "deathtouch; a counter on a token", val: kaya_plus_val, eff: kaya_plus },
    WalkerAb { delta: -2, label: "tokens are doubled this turn", val: kaya_minus_val, eff: kaya_minus },
    WalkerAb { delta: -6, label: "exile all graveyards, Spirits", val: kaya_ult_val, eff: kaya_ult },
];

// ================================================================== the spells
/// Shadow Summoning: two tapped 1/1 white Spirit creature tokens with flying
fn summoning_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let spec = Tokens { fly: true, color: Some(Colors::from_letters("W")), types: vec!["spirit"], ..Tokens::new(2, 1) };
    for t in make_tokens(g, p, spec)? {
        g.perm_mut(t).tapped = true;
    }
    Ok("gy")
}

fn summoning_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    50
}

/// the player with the most creatures, and that count
fn most_creatures(g: &Game, p: PlayerId) -> (PlayerId, i32) {
    let mut best = (p, creatures(g, p).len() as i32);
    for q in g.opps(p).collect::<Vec<_>>() {
        let n = creatures(g, q).len() as i32;
        if n > best.1 {
            best = (q, n);
        }
    }
    best
}

/// the opposing creature `n` damage kills that is worth the most
fn burn_target(g: &Game, p: PlayerId, n: i32, spell: Option<CardId>) -> Option<PermId> {
    let tg: Vec<PermId> =
        legal_targets(g, p, &format!("dmg{n}"), "c", false, spell).into_iter().filter(|&m| etgh(g, m) <= n).collect();
    max_by(&tg, |m| pval(g, m))
}

/// Will of the Mardu: choose one, or both with your commander out: Warrior tokens equal to the creatures a player
/// controls; damage equal to the creatures you control to a creature. The tokens come first, so they count.
fn will_resolve(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let both = commander_out(g, p);
    let (_, theirs) = most_creatures(g, p);
    let mine = creatures(g, p).len() as i32;
    let dmg_a = burn_target(g, p, mine + theirs, Some(c)).map_or(0.0, |m| pval(g, m));
    let dmg_b = burn_target(g, p, mine, Some(c)).map_or(0.0, |m| pval(g, m));
    let (do_tokens, do_burn) = if both {
        (theirs > 0, true)
    } else if dmg_b >= 3.0 && dmg_b >= 0.8 * theirs as f64 {
        (false, true)
    } else {
        (theirs > 0, false)
    };
    let _ = dmg_a;
    if do_tokens {
        let (_, n) = most_creatures(g, p);
        let spec = Tokens {
            warrior: true,
            color: Some(Colors::from_letters("R")),
            types: vec!["warrior"],
            ..Tokens::new(n.max(0) as u32, 1)
        };
        make_tokens(g, p, spec)?;
    }
    if do_burn {
        let n = creatures(g, p).len() as i32;
        if let Some(t) = burn_target(g, p, n, Some(c)) {
            apply_removal(g, Some(p), t, &format!("dmg{n}"), Some(c))?;
        }
    }
    Ok("gy")
}

fn will_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let both = commander_out(g, p);
    let (_, theirs) = most_creatures(g, p);
    let mine = creatures(g, p).len() as i32;
    let tokens = theirs as f64 * 0.8;
    let burn = burn_target(g, p, mine, Some(c)).map_or(0.0, |m| pval(g, m));
    let v = if both { tokens + burn } else { tokens.max(burn) };
    if v < 3.0 { 0 } else { (35.0 + 5.0 * v).min(75.0) as i32 }
}

/// Eliminate the Competition: as an additional cost sacrifice X creatures; destroy X target creatures. The AI picks X
/// (the cheapest bodies for the best targets) when it casts the spell (`eliminate_options`).
fn eliminate_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let mut tg = legal_targets(g, p, "destroy", "c", false, Some(c));
    tg.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
    for t in tg.into_iter().take(ctx.x.max(0) as usize) {
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
    }
    Ok("gy")
}

/// (X, value of what dies, what it costs): sacrificing the cheapest creatures for the best targets while each trade
/// is worth it
fn eliminate_plan(g: &Game, p: PlayerId, c: CardId) -> (usize, f64) {
    let mut tg: Vec<f64> = legal_targets(g, p, "destroy", "c", false, Some(c)).iter().map(|&m| pval(g, m)).collect();
    tg.sort_by(|a, b| b.total_cmp(a));
    let mut fod: Vec<f64> = creatures(g, p)
        .into_iter()
        .filter(|&m| !g.perm(m).is_cmd && !crate::engine::values::no_sac(g, m))
        .map(|m| pval(g, m))
        .collect();
    fod.sort_by(|a, b| a.total_cmp(b));
    let mut x = 0;
    let mut gain = 0.0;
    while x < tg.len() && x < fod.len() && tg[x] >= fod[x] + 1.5 {
        gain += tg[x] - fod[x];
        x += 1;
    }
    (x, gain)
}

fn eliminate_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if crate::human::is_human(g, p) || g.active != Some(p) || post.is_none() {
        return Ok(vec![]);
    }
    let (cg, cp) = cost_of(g, p, c);
    if !crate::engine::cast::castable(g, p, c, "hand") || !can_pay(g, p, cg, &cp, false) {
        return Ok(vec![]);
    }
    let (x, gain) = eliminate_plan(g, p, c);
    if x == 0 || gain < 3.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: gain - 2.0,
        label: format!("Eliminate the Competition (X={x})"),
        act: Some(Action::Plan { f: eliminate_go, arg: c.0 as i64 | ((x as i64) << 16) }),
    }])
}

fn eliminate_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, x) = (CardId((arg & 0xffff) as u16), (arg >> 16) as usize);
    let (cg, cp) = cost_of(g, p, c);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, cg, &cp, false) {
        return Ok(false);
    }
    let (best, _) = eliminate_plan(g, p, c);
    let x = x.min(best);
    if x == 0 {
        return Ok(false);
    }
    let mut fod: Vec<PermId> =
        creatures(g, p).into_iter().filter(|&m| !g.perm(m).is_cmd && !crate::engine::values::no_sac(g, m)).collect();
    fod.sort_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)));
    for &f in fod.iter().take(x) {
        die(g, f, "sac")?;
    }
    crate::ai::brain::do_cast_with(g, p, c, None, Ctx { x: x as i32, ..Ctx::default() })
}

/// Divine Visitation: worth casting once the deck has tokens to turn into Angels
fn visitation_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let makers = creatures(g, p)
        .into_iter()
        .filter(|&m| card_name(g, m).is_some_and(|n| ATTACK_TOKEN_MAKERS.contains(&n)))
        .count() as i32;
    45 + 5 * makers.min(3)
}

/// Legion Loyalty: creatures you control have myriad; worth it against two or more opponents
fn loyalty_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.opps(p).count() < 2 { 0 } else { 40 + 4 * creatures(g, p).len().min(5) as i32 }
}

// ================================================================== Windbrisk Heights
fn hidden(g: &Game, l: LandId) -> Option<CardId> {
    match g.land(l).data.get(DataKey::Hidden) {
        Some(Val::Card(c)) => Some(*c),
        _ => None,
    }
}

/// Hideaway 4: look at the top four cards, exile one face down, the rest go to the bottom in a random order. The AI
/// keeps the most expensive nonland card (it is cast for free).
fn windbrisk_etb(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    let n = 4.min(g.player(p).library.len());
    let mut top: Vec<CardId> = (0..n).filter_map(|_| g.player_mut(p).library.pop()).collect();
    if top.is_empty() {
        return Ok(());
    }
    let mut best = top[0];
    let mut best_key = (false, 0u32);
    for &c in &top {
        let d = g.db.get(c);
        let key = (!d.land, d.cmc);
        if key > best_key {
            best = c;
            best_key = key;
        }
    }
    remove_first(&mut top, best);
    g.rng.shuffle(&mut top);
    g.player_mut(p).library.splice(0..0, top);
    g.land_mut(l).data.set(DataKey::Hidden, Val::Card(best));
    crate::glog!(g, "    Windbrisk Heights hides a card");
    Ok(())
}

/// {W},{T}: play the exiled card without paying its mana cost if you attacked with three or more creatures this turn
fn windbrisk_options(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(c) = hidden(g, l) else { return Ok(vec![]) };
    let pl = g.player(p);
    if post.is_none() || g.land(l).tapped || pl.atk_count.0 != Some(g.turn_stamp()) || pl.atk_count.1 < 3 {
        return Ok(vec![]);
    }
    if !crate::impls::lands::can_pay_without(g, p, l, 0, "W") {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.5 + g.db.get(c).cmc as f64 / 3.0,
        label: format!("Windbrisk Heights ({})", g.db.get(c).name),
        act: Some(Action::Plan { f: windbrisk_go, arg: l.0 as i64 }),
    }])
}

fn windbrisk_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    let Some(c) = hidden(g, l) else { return Ok(false) };
    let pl = g.player(p);
    if !pl.lands.contains(&l) || g.land(l).tapped || pl.atk_count.0 != Some(g.turn_stamp()) || pl.atk_count.1 < 3 {
        return Ok(false);
    }
    if !crate::impls::lands::pay_without(g, p, l, 0, "W")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    g.land_mut(l).data.set(DataKey::Hidden, Val::None);
    let what = format!("play {}", g.db.get(c).name);
    if !ability_window_card(g, p, g.land(l).cd, &what, None, None)? {
        return Ok(true);
    }
    if g.db.get(c).land {
        crate::impls::lands::play_land_card_free(g, p, c);
    } else {
        crate::engine::cast::cast_card(g, p, c, "lib", Ctx::default())?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Zurgo Stormrender")?.attack = Some(zurgo_attack);
    r.card(db, "Ainok Strike Leader")?.attack = Some(ainok_attack);
    r.card(db, "Angel of Invention")?.etb = Some(angel_etb);
    let c = r.card(db, "Chittering Witch")?;
    c.etb = Some(witch_etb);
    c.options = Some(witch_options);
    r.card(db, "Twilight Drover")?.options = Some(drover_options);
    r.card(db, "Ironwill Forger")?.combat_start = Some(forger_combat);
    r.card(db, "Grenzo, Havoc Raiser")?.combat_damage = Some(grenzo_damage);
    r.card(db, "Mindblade Render")?.combat_damage = Some(mindblade_damage);
    r.card(db, "Myr Battlesphere")?.attack = Some(battlesphere_attack);
    r.card(db, "Redoubled Stormsinger")?.attack = Some(stormsinger_attack);
    r.card(db, "Neriv, Crackling Vanguard")?.attack = Some(neriv_attack);
    walker(r, db, "Kaya, Geist Hunter", &KAYA_ABS)?;
    let c = r.card(db, "Shadow Summoning")?;
    c.resolve = Some(summoning_resolve);
    c.prio = Some(summoning_prio);
    let c = r.card(db, "Will of the Mardu")?;
    c.resolve = Some(will_resolve);
    c.prio = Some(will_prio);
    let c = r.card(db, "Eliminate the Competition")?;
    c.resolve = Some(eliminate_resolve);
    c.hand_options = Some(eliminate_options);
    c.prio = Some(zero_prio);
    r.card(db, "Divine Visitation")?.prio = Some(visitation_prio);
    r.card(db, "Legion Loyalty")?.prio = Some(loyalty_prio);
    let c = r.card(db, "Windbrisk Heights")?;
    c.land_etb = Some(windbrisk_etb);
    c.land_options = Some(windbrisk_options);
    Ok(())
}

/// ai::pool::wipe_response: would Ainok Strike Leader's indestructible tokens save most of q's board? (Only the
/// creature tokens survive the wipe, so it is only worth the sacrifice when they are most of what the wipe kills.)
pub fn tokens_carry_board(g: &Game, q: PlayerId) -> bool {
    let (mut tokens, mut all, mut n) = (0.0, 0.0, 0);
    for m in creatures(g, q) {
        let v = pval(g, m).max(0.5);
        all += v;
        if g.perm(m).token {
            tokens += v;
            n += 1;
        }
    }
    n >= 2 && tokens >= 0.75 * all
}
