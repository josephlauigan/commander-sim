//! Zones: drawing, milling, tokens, permanents entering, leaving and dying, lands, discards
//! (engine.py's "zones" section, and enter / do_etb / etb_once / lands / discards from later in the file).

use crate::ai;
use crate::cardcode;
use crate::cards::{Colors, LandEtb};
use crate::engine::hooks::{self, fire_trigger};
use crate::engine::life::{gain, lose_life};
use crate::engine::stack::{queue_triggers, trigger_window};
use crate::engine::values::*;
use crate::flow::Res;
use crate::hooks::{Call, Event, Sacrificed, TrigAct, Trigger};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, PermData, Val};
use crate::sym::Sym;
use crate::tag::Tag;

/// cards that win instead of losing when drawing from an empty library
pub const LABMEN: [&str; 2] = ["Laboratory Maniac", "Jace, Wielder of Mysteries"];
/// creature tokens per player; beyond this the board is lethal many times over and games crawl
pub const TOKEN_CAP: usize = 250;
/// undying / persist returns of one card in one turn; past that it stays dead
pub const RETURN_CAP: u32 = 10;
pub const KEYSPELL: [Tag; 3] = [Tag::Rean, Tag::Tut, Tag::Yawg];
pub const TEFERIS_PROTECTION: &str = "Teferi's Protection";

fn is_keyspell(g: &Game, c: CardId) -> bool {
    KEYSPELL.iter().any(|&t| g.db.get(c).tag(t))
}

// ------------------------------------------------------------------ drawing
/// p draws n cards (step: the first is the draw step's own draw). Python's `draw`.
pub fn draw(g: &mut Game, p: PlayerId, n: u32, step: bool) -> Res {
    for k in 0..n {
        if !g.player(p).alive {
            return Ok(());
        }
        let st = g.turn_stamp();
        if g.player(p).draw_st != Some(st) {
            let pl = g.player_mut(p);
            pl.draw_st = Some(st);
            pl.draw_n = 0;
        }
        if g.player(p).draw_n >= 1
            && (g.opps(p).any(|q| has(g, q, Tag::Narset))
                || g.players.iter().any(|q| q.alive && has(g, q.id, Tag::Labyrinth)))
        {
            g.player_mut(p).stat("narset_denied", (n - k) as i64); // Narset / Spirit of the Labyrinth
            return Ok(());
        }
        let extra = !(step && k == 0);
        if !g.hooks.is_empty() && extra {
            // Notion Thief: an opponent's extra draws are stolen (614.5: each replacement applies once per draw)
            let chain = g.thief_chain.clone();
            let thief = hooks::hooked(g, Event::StealDraw)
                .into_iter()
                .find(|(src, imp)| !chain.contains(src) && (imp.steal_draw.unwrap())(g, *src, p))
                .map(|x| x.0);
            if let Some(t) = thief {
                g.thief_chain.push(t);
                let r = draw(g, g.perm(t).owner, 1, false);
                g.thief_chain = chain;
                r?;
                continue;
            }
        }
        if g.player(p).library.is_empty() {
            let labman = g
                .player(p)
                .perms
                .iter()
                .any(|&m| !g.perm(m).phased && card_name(g, m).is_some_and(|n| LABMEN.contains(&n)));
            if labman {
                return ai::win(g, p, "Laboratory Maniac", None);
            }
            g.player_mut(p).decked = true;
            return Ok(());
        }
        if g.player(p).urabrask == Some(g.turn_stamp()) {
            // Urabrask, Heretic Praetor: exiled instead, playable this turn (no draw, so no draw triggers)
            let pl = g.player_mut(p);
            pl.urabrask = None;
            let c = pl.library.pop().unwrap();
            pl.hand.push(c);
            pl.impulse.push(c);
            pl.seen.insert(c);
            crate::glog!(g, "    {} exiles {} instead of drawing it (Urabrask)", g.player(p).name, g.db.get(c).name);
            continue;
        }
        let pl = g.player_mut(p);
        pl.draw_n += 1;
        let c = pl.library.pop().unwrap();
        pl.hand.push(c);
        if pl.draw_n == 1 {
            pl.miracle = Some((st, c));
        }
        pl.seen.insert(c);
        pl.stat("cards_drawn", 1);
        if has(g, p, Tag::Ironman)
            && let Some(a) = army_of(g, p)
        {
            cardcode::add_counters(g, a, 1);
        }
        if g.dsl_on {
            crate::dsl::fire(g, "draw", crate::dsl::Fired { player: Some(p), ..Default::default() })?;
        }
        if !g.hooks.is_empty() {
            fire_trigger(g, Event::Draw, Call::Player { p })?;
        }
        if !g.player(p).emblems.is_empty() {
            cardcode::emblem_draw(g, p)?;
        }
        let opps: Vec<PlayerId> = g.opps(p).collect();
        for q in opps {
            if has(g, q, Tag::SheoA) {
                lose_life(g, p, 2, Some(q), "drain", None)?; // Sheoldred, the Apocalypse: they lose 2
            }
            if has(g, q, Tag::Tithe) && cardcode::tithe_unpaid(g, p)? {
                add_treasure(g, q, 1)?;
            }
            if extra && has(g, q, Tag::Bowmasters) {
                // HUMAN(phase 9): each Bowmasters, the person's target (hc.deal_damage)
                for _ in find(g, q, Tag::Bowmasters) {
                    amass(g, q, 1)?;
                    lose_life(g, p, 1, Some(q), "triggers", None)?;
                }
            }
        }
        if has(g, p, Tag::SheoA) {
            gain(g, p, 2)?;
        }
    }
    Ok(())
}

pub fn mill(g: &mut Game, p: PlayerId, n: usize) -> Res {
    if !g.hooks.is_empty() && n > 0 {
        let lib = &g.player(p).library;
        let cards = lib[lib.len().saturating_sub(n)..].to_vec();
        fire_trigger(g, Event::CardsToGy, Call::Cards { p, cards })?;
    }
    for _ in 0..n {
        let Some(c) = g.player_mut(p).library.pop() else { return Ok(()) };
        g.player_mut(p).gy.push(c);
        if is_keyspell(g, c) {
            let pl = g.player_mut(p);
            pl.stat("key_milled", 1);
            if !pl.milled_keys.contains(&c) {
                pl.milled_keys.push(c);
            }
        }
    }
    Ok(())
}

/// amass n: put +1/+1 counters on p's Army, making a 0/0 Orc Army first if there is none
pub fn amass(g: &mut Game, p: PlayerId, n: i32) -> Res {
    let bonus = if has(g, p, Tag::Mauhur) { 1 } else { 0 };
    let a = match army_of(g, p) {
        Some(a) => a,
        None => {
            let a = g.new_perm(p, None, "Orc Army", 0, 0);
            let mimic = has(g, p, Tag::Mimic);
            let x = g.perm_mut(a);
            x.army = true;
            x.colors = Colors::from_letters("B");
            x.sick = true;
            x.on_bf = true;
            if mimic {
                x.plus += 1;
            }
            g.player_mut(p).perms.push(a);
            cardcode::kindred_enter(g, p, a)?; // Kindred Discovery: an Orc Army entered
            a
        }
    };
    g.perm_mut(a).plus += n + bonus;
    Ok(())
}

// ------------------------------------------------------------------ tokens
/// default colour of a deck's tokens
pub fn token_color(key: &str) -> Colors {
    Colors::from_letters(match key {
        "najeela" | "galadriel" | "yshtola" | "jodah" => "W",
        "seph" | "sauron" => "B",
        "veyran" => "R",
        "alela" => "U",
        _ => "",
    })
}

/// What tokens to make (Python's `make_tokens` keyword arguments).
#[derive(Debug, Clone)]
pub struct Tokens {
    pub n: u32,
    pub pow: i32,
    pub tgh: Option<i32>,
    pub fly: bool,
    pub warrior: bool,
    pub attacking: bool,
    pub lifelink: bool,
    pub sick: bool,
    pub dt: bool,
    pub color: Option<Colors>,
    pub types: Vec<Sym>,
    /// set on each token before its toughness is checked (a 0/0 Construct that grows with artifacts)
    pub data: PermData,
}

impl Tokens {
    pub fn new(n: u32, pow: i32) -> Tokens {
        Tokens {
            n,
            pow,
            tgh: None,
            fly: false,
            warrior: false,
            attacking: false,
            lifelink: false,
            sick: true,
            dt: false,
            color: None,
            types: vec![],
            data: PermData::default(),
        }
    }
}

/// p creates tokens; returns the ones that stayed on the battlefield. Python's `make_tokens`.
pub fn make_tokens(g: &mut Game, p: PlayerId, spec: Tokens) -> Res<Vec<PermId>> {
    let mut out = vec![];
    let mut n = spec.n * crate::dsl::token_mult(g, p);
    let have = g.player(p).perms.iter().filter(|&&m| g.perm(m).token).count();
    n = n.min(TOKEN_CAP.saturating_sub(have) as u32); // runaway token growth is already lethal
    if n == 0 {
        return Ok(out);
    }
    let blank = opp_has(g, p, Tag::Mother);
    let (mut pw, mut tg, mut sick) = (spec.pow, spec.tgh, spec.sick);
    if has(g, p, Tag::Jinnie) && pw < 2 && !spec.attacking {
        (pw, tg, sick) = (2, Some(2), false); // Jinnie Fay: 2/2 Cats with haste instead
    }
    let color = spec.color.unwrap_or_else(|| token_color(g.player(p).key));
    for _ in 0..n {
        let m = g.new_perm(p, None, "Token", pw, tg.unwrap_or(pw));
        let x = g.perm_mut(m);
        x.fly = spec.fly;
        x.warrior = spec.warrior;
        x.lifelink = spec.lifelink;
        x.sick = sick;
        x.dt = spec.dt;
        x.colors = color;
        x.ttypes = spec.types.clone();
        x.data = spec.data.clone();
        x.on_bf = true;
        if spec.attacking {
            x.tapped = true;
        }
        g.player_mut(p).perms.push(m);
        if etgh(g, m) <= 0 {
            die(g, m, "sba")?;
            continue;
        }
        out.push(m);
    }
    let k = out.len() as i32;
    if k > 0 && !g.no_fang {
        cardcode::chatterfang_squirrels(g, p, k)?;
    }
    if k > 0 && !blank {
        if has(g, p, Tag::Crusade) {
            let copies = 1 + hooks::total_trigger_copies(g, p, "etb", Some(out[0]));
            let n = k * find(g, p, Tag::Crusade).len() as i32 * copies;
            for m in g.player(p).perms.clone() {
                if g.is_creature(m) {
                    let x = g.perm_mut(m);
                    x.plus = (x.plus + n).min(999);
                }
            }
        }
        if has(g, p, Tag::Warleader) {
            for q in g.opps(p).collect::<Vec<_>>() {
                lose_life(g, q, k, Some(p), "drain", Some(true))?; // Warleader's Call deals damage
            }
        }
        if has(g, p, Tag::Wisp) && pw <= 2 {
            // Wispdrinker Vampire
            let opps: Vec<_> = g.opps(p).collect();
            for &q in &opps {
                lose_life(g, q, k, Some(p), "drain", None)?;
            }
            gain(g, p, k * opps.len() as i32)?;
        }
    }
    shards_trigger(g, p, k)?;
    if g.dsl_on {
        for &x in &out {
            crate::dsl::fire(g, "etb", crate::dsl::Fired { perm: Some(x), owner: Some(p), ..Default::default() })?;
        }
    }
    if k > 0 && !g.hooks.is_empty() {
        fire_trigger(g, Event::TokensEnter, Call::Tokens { p, toks: out.clone() })?; // Plumecreed Mentor
    }
    Ok(out)
}

/// Aura Shards: whenever a creature enters under your control, destroy target artifact or enchantment
pub fn shards_trigger(g: &mut Game, p: PlayerId, k: i32) -> Res {
    if k <= 0 || !has(g, p, Tag::Shards) || opp_has(g, p, Tag::Mother) {
        return Ok(());
    }
    let reps = k * if has(g, p, Tag::Mother) { 2 } else { 1 };
    // HUMAN(phase 9): you may destroy one, your pick (hc.choose)
    for _ in 0..reps {
        let tg: Vec<PermId> = g
            .opps(p)
            .flat_map(|q| g.player(q).perms.iter().copied())
            .filter(|&m| {
                let x = g.perm(m);
                x.cd.is_some_and(|c| {
                    let d = g.db.get(c);
                    d.types.has(crate::cards::Types::ARTIFACT) || d.types.has(crate::cards::Types::ENCHANTMENT)
                }) && !x.phased
                    && !untargetable(g, m)
                    && !indestructible(g, m)
            })
            .collect();
        let Some(best) = max_by(&tg, |m| pval(g, m)) else { return Ok(()) };
        if pval(g, best) < 1.0 {
            return Ok(());
        }
        g.player_mut(p).stat("shards_kill", 1);
        crate::engine::removal::apply_removal(g, Some(p), best, "destroy", None)?;
        if g.over {
            return Ok(());
        }
    }
    Ok(())
}

/// the first item with the highest key (Python's `max(xs, key=...)`)
pub fn max_by<T: Copy>(xs: &[T], key: impl Fn(T) -> f64) -> Option<T> {
    let mut best: Option<(T, f64)> = None;
    for &x in xs {
        let k = key(x);
        if best.is_none_or(|b| k > b.1) {
            best = Some((x, k));
        }
    }
    best.map(|b| b.0)
}

/// the first item with the lowest key (Python's `min(xs, key=...)`)
pub fn min_by<T: Copy>(xs: &[T], key: impl Fn(T) -> f64) -> Option<T> {
    let mut best: Option<(T, f64)> = None;
    for &x in xs {
        let k = key(x);
        if best.is_none_or(|b| k < b.1) {
            best = Some((x, k));
        }
    }
    best.map(|b| b.0)
}

/// a creature entered under p's control (as it enters: 0 toughness, Crusade, Warleader's Call, Wispdrinker ...)
pub fn creature_entered(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if etgh(g, m) <= 0 && !g.perm(m).army {
        return die(g, m, "sba");
    }
    if !opp_has(g, p, Tag::Mother) {
        if has(g, p, Tag::Crusade) {
            let n = find(g, p, Tag::Crusade).len() as i32 * (1 + hooks::total_trigger_copies(g, p, "etb", Some(m)));
            for x in g.player(p).perms.clone() {
                if g.is_creature(x) {
                    let y = g.perm_mut(x);
                    y.plus = (y.plus + n).min(999);
                }
            }
        }
        if has(g, p, Tag::Warleader) {
            for q in g.opps(p).collect::<Vec<_>>() {
                lose_life(g, q, 1, Some(p), "drain", Some(true))?; // Warleader's Call deals damage
            }
        }
        if has(g, p, Tag::Wisp) && epow(g, m) <= 2 && !card_tag(g, m, Tag::Wisp) {
            let opps: Vec<_> = g.opps(p).collect();
            for &q in &opps {
                lose_life(g, q, 1, Some(p), "drain", None)?;
            }
            gain(g, p, opps.len() as i32)?;
        }
        if has(g, p, Tag::Uprising) && epow(g, m) >= 4 {
            draw(g, p, 1, false)?;
        }
    }
    shards_trigger(g, p, 1)
}

// ------------------------------------------------------------------ leaving
/// Equipment and Auras of m's controller come off it
pub fn detach(g: &mut Game, m: PermId) {
    let owner = g.perm(m).owner;
    for e in g.player(owner).perms.clone() {
        if g.perm(e).attached == Some(m) {
            g.perm_mut(e).attached = None;
        }
    }
}

/// m leaves the battlefield (any way). Python's `leave`.
pub fn leave(g: &mut Game, m: PermId) -> Res {
    let p = g.perm(m).owner;
    let on = g.perm(m).on_bf;
    if g.is_creature(m) && g.perm(m).plus > 0 && on && has(g, p, Tag::Ozolith) {
        let n = g.perm(m).plus;
        g.player_mut(p).ozolith_counters += n; // The Ozolith keeps the counters
    }
    if on {
        g.player_mut(p).perms.retain(|&x| x != m);
        g.perm_mut(m).on_bf = false;
    }
    g.bf_ver += 1;
    g.player_mut(p).left_turn = Some(g.turn_stamp()); // revolt
    if !g.auras.is_empty() {
        cardcode::aura_fall(g, m)?; // before detach, which would unhook your own Auras from it
    }
    detach(g, m);
    if g.perm(m).data.truthy(DataKey::Bestow) {
        cardcode::bestow_fall(g, m)?;
    }
    if let Some(i) = g.hooks.iter().position(|&x| x == m) {
        g.hooks.remove(i);
        let imp = g.perm(m).cd.and_then(|c| g.registry.get(c).copied());
        if let Some(imp) = imp
            && imp.leaves.is_some()
        {
            if imp.runs_at_once(Event::Leaves) {
                (imp.leaves.unwrap())(g, m, m)?; // an effect ending
            } else {
                let t = Trigger {
                    controller: p,
                    src: Some(m),
                    act: TrigAct::Hook { event: Event::Leaves, call: Call::Leaves { m } },
                    name: None,
                    known: false,
                    imp: None,
                    cast_etb: false,
                };
                queue_triggers(g, vec![t])?; // "when it leaves" (Oblivion Ring)
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Gy,
    Exile,
    Hand,
    /// into the library at a random position
    Lib,
    /// on top of the library
    Top,
}

/// a nontoken card that left goes to a zone of its owner's; a commander goes to the command zone
pub fn to_zone_card(g: &mut Game, m: PermId, zone: Zone) {
    let x = g.perm(m);
    if x.token || x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Tokpw)) {
        return; // a token with a card face (the Jace token) just ceases to exist
    }
    let owner = x.orig;
    if let Some(cd) = x.phys {
        let pl = g.player_mut(owner);
        match zone {
            Zone::Exile => pl.exile.push(cd),
            Zone::Hand => pl.hand.push(cd),
            _ => pl.gy.push(cd),
        }
        return;
    }
    let cd = x.cd.unwrap();
    if cd == g.player(owner).cmd {
        g.player_mut(owner).cmd_in_zone = true;
        return;
    }
    match zone {
        Zone::Gy => g.player_mut(owner).gy.push(cd),
        Zone::Exile => g.player_mut(owner).exile.push(cd),
        Zone::Hand => g.player_mut(owner).hand.push(cd),
        Zone::Lib => {
            let n = g.player(owner).library.len() as u64;
            let i = g.rng.below(n + 1) as usize;
            g.player_mut(owner).library.insert(i, cd);
        }
        Zone::Top => g.player_mut(owner).library.push(cd),
    }
}

/// put n -1/-1 counters on creature m (none under Melira); true if they were put
pub fn minus_counter(g: &mut Game, m: PermId, n: i32) -> bool {
    if melira(g, g.perm(m).owner) {
        return false;
    }
    g.perm_mut(m).plus -= n;
    true
}

/// A card that keeps dying the moment it returns (Kitchen Finks under Melira while an opponent's -X/-X effect leaves
/// it at 0 toughness) is a mandatory loop, a draw under the rules: the simulator lets it stay dead instead.
pub fn returns_left(g: &mut Game, cd: CardId) -> bool {
    let st = g.turn_stamp();
    if g.returns_turn != Some(st) {
        g.returns_turn = Some(st);
        g.returns.clear();
    }
    let k = g.returns.entry(cd).or_insert(0);
    if *k >= RETURN_CAP {
        return false;
    }
    *k += 1;
    true
}

/// Teferi's Protection: until your next turn your life total can't change and you have protection from everything;
/// all your permanents phase out (lands too: no mana until your untap step)
pub fn teferis_protection(g: &mut Game, p: PlayerId) {
    let pl = g.player_mut(p);
    pl.life_locked = true;
    pl.ring_prot = true;
    pl.floating = Default::default();
    let (perms, lands) = (pl.perms.clone(), pl.lands.clone());
    for m in perms {
        g.perm_mut(m).phased = true;
    }
    for l in lands {
        g.land_mut(l).tapped = true;
    }
    g.bf_ver += 1;
    crate::glog!(
        g,
        "    {} casts Teferi's Protection: life can't change, protection from everything, all phased out",
        g.player(p).name
    );
}

/// q casts Teferi's Protection from hand at instant speed, if it can; true if it resolved
pub fn cast_teferis_protection(g: &mut Game, q: PlayerId, imp: f64) -> Res<bool> {
    let Some(c) = g.db.id(TEFERIS_PROTECTION).filter(|c| g.player(q).hand.contains(c)) else { return Ok(false) };
    let d = g.db.get(c);
    let (gn, pips) = (d.generic, d.pips.to_string());
    if g.player(q).life_locked
        || silenced(g, q)
        || !crate::engine::cast::castable(g, q, c, "hand")
        || !crate::engine::mana::can_pay(g, q, gn, &pips, false)
    {
        return Ok(false);
    }
    g.player_mut(q).hand.retain(|&x| x != c);
    crate::engine::mana::pay(g, q, gn, &pips, false)?;
    let pl = g.player_mut(q);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::engine::cast::on_cast(g, q, c)?;
    g.player_mut(q).exile.push(c);
    if !crate::engine::stack::counter_window(g, q, c, imp, vec![])? {
        return Ok(false);
    }
    teferis_protection(g, q);
    g.player_mut(q).stat("protection_used", 1);
    Ok(true)
}

/// q is about to lose to damage or life loss: Teferi's Protection if it has it
pub fn last_chance(g: &mut Game, q: PlayerId) -> Res<bool> {
    if g.humans.get(q).is_some() {
        return Ok(g.player(q).life_locked); // practice mode: the person casts it themselves
    }
    Ok(g.player(q).life_locked || cast_teferis_protection(g, q, 9.0)?)
}

// ------------------------------------------------------------------ dying
/// Hushbringer: creatures entering or dying don't cause abilities to trigger
pub fn hushed(g: &Game) -> bool {
    g.players.iter().filter(|q| q.alive).flat_map(|q| q.perms.iter()).any(|&m| {
        let x = g.perm(m);
        !x.phased && !x.neutered && card_name(g, m) == Some("Hushbringer")
    })
}

/// m dies (cause: 'destroy', 'sac', 'sba', 'combat' ...). Python's `die`.
pub fn die(g: &mut Game, m: PermId, cause: Sym) -> Res {
    let p = g.perm(m).owner;
    if !g.perm(m).on_bf {
        return Ok(());
    }
    if cause == "destroy" && indestructible(g, m) {
        return Ok(());
    }
    if cause == "sac" && no_sac(g, m) {
        return Ok(());
    }
    if cause == "destroy" && !g.auras.is_empty() && cardcode::umbra_save(g, m)? {
        return Ok(());
    }
    let creature = g.is_creature(m);
    let token = g.perm(m).token;
    if matches!(cause, "destroy" | "combat")
        && creature
        && !token
        && !g.noregen
        && (cardcode::try_regenerate(g, m)? || cardcode::ezuri_regen(g, m)?)
    {
        return Ok(());
    }
    if matches!(cause, "destroy" | "combat") && !g.noregen && cardcode::self_regen(g, m)? {
        return Ok(());
    }
    if cause == "destroy" && creature && g.player(p).regen_turn == Some(g.turn_stamp()) && !g.noregen {
        g.perm_mut(m).tapped = true;
        crate::glog!(g, "    {} regenerates", g.perm(m).name);
        return Ok(());
    }
    let selfdies = g.perm(m).cd.and_then(|c| g.registry.get(c).copied()).filter(|i| i.self_dies.is_some());
    if !g.hooks.is_empty() && hooks::total_player(g, Event::NoGraveyard, p) != 0 {
        leave(g, m)?; // Rest in Peace: exiled, so it never dies
        if !token {
            to_zone_card(g, m, Zone::Exile);
        }
        return Ok(());
    }
    if creature && hushed(g) {
        leave(g, m)?;
        if !token {
            to_zone_card(g, m, Zone::Gy);
        }
        return Ok(());
    }
    let prev = g.dying.replace(m); // Auras that care how their creature left (Gift of Immortality)
    let r = leave(g, m);
    g.dying = prev;
    r?;
    if creature {
        g.died_turn = Some(g.turn_stamp()); // Barad-dûr
    }
    g.resolving += 1; // its dies triggers wait until the card is in the graveyard
    let r = die_triggers(g, m, p, cause, selfdies.is_some(), creature);
    g.resolving -= 1;
    r?;
    if g.resolving == 0 && !g.trig_queue.is_empty() && !g.over {
        crate::engine::stack::flush_triggers(g)?;
    }
    Ok(())
}

fn die_triggers(g: &mut Game, m: PermId, p: PlayerId, cause: Sym, selfdies: bool, creature: bool) -> Res {
    let token = g.perm(m).token;
    if g.marchesa_on && creature && !token {
        cardcode::marchesa_dies(g, p, m)?;
    }
    if g.dsl_on {
        let fired = crate::dsl::Fired { perm: Some(m), owner: Some(p), dying: Some(m), ..Default::default() };
        crate::dsl::fire(g, "dies", fired)?;
    }
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::Dies, Call::Dies { m, cause })?;
        if cause == "sac" {
            fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what: Sacrificed::Perm(m) })?;
        }
    }
    if selfdies {
        let n = 1 + hooks::total_trigger_copies(g, p, "dies", Some(m));
        let imp = g.registry.get(g.perm(m).cd.unwrap()).copied().unwrap();
        for _ in 0..n {
            if imp.runs_at_once(Event::SelfDies) {
                (imp.self_dies.unwrap())(g, m, m, cause)?;
            } else {
                let t = Trigger {
                    controller: p,
                    src: Some(m),
                    act: TrigAct::Hook { event: Event::SelfDies, call: Call::Dies { m, cause } },
                    name: None,
                    known: false,
                    imp: None,
                    cast_etb: false,
                };
                queue_triggers(g, vec![t])?;
            }
        }
    }
    // death triggers of tagged cards
    for i in 0..g.players.len() {
        let q = PlayerId(i as u8);
        if !g.player(q).alive {
            continue;
        }
        for _ in 0..1 + hooks::total_trigger_copies(g, q, "dies", Some(m)) {
            hand_tag_death(g, q, p, m, creature)?;
        }
    }
    if card_tag_str(g, m, Tag::Fill) == Some("stitcher") {
        mill(g, p, 3)?;
    }
    die_rest(g, m, p, cause, creature)
}

fn card_tag_str(g: &Game, m: PermId, t: Tag) -> Option<&str> {
    g.perm(m).cd.and_then(|c| g.db.get(c).tags.str(t))
}

fn hand_tag_death(g: &mut Game, q: PlayerId, p: PlayerId, _m: PermId, creature: bool) -> Res {
    if has(g, q, Tag::Bartist) {
        let opps: Vec<PlayerId> = g.opps(q).collect();
        if let Some(t) = max_by(&opps, |o| threat(g, q, o)) {
            lose_life(g, t, 1, Some(q), "drain", None)?; // Blood Artist
            gain(g, q, 1)?;
        }
    }
    if q == p && has(g, q, Tag::Drain) && creature {
        for o in g.opps(q).collect::<Vec<_>>() {
            lose_life(g, o, 1, Some(q), "drain", None)?;
        }
        gain(g, q, 1)?;
    }
    Ok(())
}

fn die_rest(g: &mut Game, m: PermId, p: PlayerId, cause: Sym, creature: bool) -> Res {
    if creature {
        for q in g.opps(p).collect::<Vec<_>>() {
            if has(g, q, Tag::Wurmdrain) {
                let src = find(g, q, Tag::Wurmdrain)[0];
                let name = format!("{} loses 2 life", g.player(p).name);
                if trigger_window(g, q, Some(src), &name, None)? {
                    lose_life(g, p, 2, Some(q), "drain", None)?; // Massacre Wurm
                }
            }
            for h in find(g, q, Tag::Heir) {
                g.perm_mut(h).plus += 1; // Sephiroth, Planet's Heir
            }
        }
    }
    if card_tag(g, m, Tag::Wurmcoil) && trigger_window(g, p, Some(m), "two 3/3 Wurms", Some(5.0))? {
        make_tokens(g, p, Tokens { dt: true, color: Some(Colors::NONE), ..Tokens::new(1, 3) })?;
        make_tokens(g, p, Tokens { lifelink: true, color: Some(Colors::NONE), ..Tokens::new(1, 3) })?;
    }
    let x = g.perm(m).clone();
    if x.token {
        return Ok(());
    }
    let cd = x.cd.unwrap();
    let db = g.db.clone();
    let d = db.get(cd);
    if d.bomb != 0 && x.orig == p && !g.player(p).removed_bombs.contains(&cd) {
        g.player_mut(p).removed_bombs.push(cd);
    }
    let is_cmd_card = cd == g.player(p).cmd;
    // undying from Mikaeus
    if has(g, p, Tag::Mikaeus)
        && d.creature
        && !d.tag(Tag::Human)
        && !d.tag(Tag::Mikaeus)
        && !x.undying
        && x.plus <= 0
        && x.orig == p
        && !is_cmd_card
        && trigger_window(g, p, Some(m), "undying", None)?
    {
        let n = enter(g, p, cd, Enter { undying: true, ..Enter::default() })?;
        g.perm_mut(n).plus = 1;
        g.perm_mut(n).undying = true;
        g.player_mut(p).stat("undying", 1);
        return Ok(());
    }
    let (undying, persist) = (d.has_kw("undying"), d.has_kw("persist"));
    if (undying || persist)
        && x.orig == p
        && !is_cmd_card
        && !(!g.hooks.is_empty() && hooks::total_player(g, Event::NoGraveyard, p) != 0)
        && returns_left(g, cd)
    {
        if undying && x.plus <= 0 && trigger_window(g, p, Some(m), "undying", None)? {
            enter(g, p, cd, Enter { undying: true, ..Enter::default() })?; // back with a +1/+1 counter
            g.player_mut(p).stat("undying", 1);
            return Ok(());
        }
        if persist && x.plus >= 0 && trigger_window(g, p, Some(m), "persist", None)? {
            // back with a -1/-1 counter (none under Melira)
            let plus = if melira(g, p) { 0 } else { -1 };
            enter(g, p, cd, Enter { plus, ..Enter::default() })?;
            g.player_mut(p).stat("persist", 1);
            return Ok(());
        }
    }
    to_zone_card(g, m, Zone::Gy);
    if !g.hooks.is_empty() && creature {
        fire_trigger(g, Event::CreatureToGy, Call::Leaves { m })?; // Nim Deathmantle
    }
    if cause == "sac" {
        tergrid_steal(g, p, x.phys.unwrap_or(cd), x.orig)?;
    }
    if creature {
        cardcode::gy_dies(g, x.orig, m)?;
    }
    Ok(())
}

pub fn exile_perm(g: &mut Game, m: PermId) -> Res {
    leave(g, m)?;
    to_zone_card(g, m, Zone::Exile);
    Ok(())
}

pub fn bounce(g: &mut Game, m: PermId) -> Res {
    leave(g, m)?;
    to_zone_card(g, m, Zone::Hand);
    Ok(())
}

pub fn tuck(g: &mut Game, m: PermId) -> Res {
    leave(g, m)?;
    to_zone_card(g, m, Zone::Lib);
    Ok(())
}

// ------------------------------------------------------------------ entering
/// How a permanent enters (Python's `enter` keyword arguments).
#[derive(Debug, Clone, Copy)]
pub struct Enter {
    /// its owner, when another player controls it
    pub orig: Option<PlayerId>,
    pub sick: bool,
    pub was_cast: bool,
    pub undying: bool,
    pub plus: i32,
}

impl Default for Enter {
    fn default() -> Enter {
        Enter { orig: None, sick: true, was_cast: false, undying: false, plus: 0 }
    }
}

/// Card cd enters the battlefield under p's control: its as-it-enters effects, then its enter triggers (on the
/// stack once the spell resolving has finished). Python's `enter`.
pub fn enter(g: &mut Game, p: PlayerId, cd: CardId, how: Enter) -> Res<PermId> {
    g.tick()?;
    let mut cd = cd;
    if g.db.get(cd).tag(Tag::Moxd) && !how.was_cast {
        // Mox Diamond put onto the battlefield (Urza's Saga): the same replacement as when cast
        let lands: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
        if lands.is_empty() {
            g.player_mut(p).gy.push(cd);
            crate::glog!(g, "    Mox Diamond goes to the graveyard (no land to discard)");
            return Ok(g.new_perm(p, Some(cd), "Token", 0, 0)); // never on the battlefield
        }
        let x = min_by(&lands, |c| mox_discard_key(g, c)).unwrap();
        let pl = g.player_mut(p);
        let i = pl.hand.iter().position(|&c| c == x).unwrap();
        pl.hand.remove(i);
        pl.gy.push(x);
    }
    let mut phys = None;
    if g.db.get(cd).tag(Tag::Clone) {
        // Phyrexian Metamorph: copy the best creature or artifact on the battlefield
        let cands: Vec<PermId> = g
            .players
            .iter()
            .filter(|q| q.alive)
            .flat_map(|q| q.perms.iter().copied())
            .filter(|&x| {
                let y = g.perm(x);
                y.cd.is_some_and(|c| {
                    let d = g.db.get(c);
                    c != g.player(y.owner).cmd
                        && (g.is_creature(x) || d.types.has(crate::cards::Types::ARTIFACT))
                        && !d.tag(Tag::Clone)
                })
            })
            .collect();
        if let Some(b) = max_by(&cands, |x| pval(g, x)) {
            phys = Some(cd);
            cd = g.perm(b).cd.unwrap();
        }
    }
    let m = g.new_perm(p, Some(cd), "Token", 0, 0);
    g.enter_no += 1;
    let d = g.db.get(cd);
    let (haste, has_dsl, loyalty) = (d.tag(Tag::Haste), d.has_dsl(), d.start_loyalty);
    let doubling = g.player(p).perms.iter().any(|&x| !g.perm(x).phased && card_name(g, x) == Some("Doubling Season"));
    let born = g.enter_no;
    let x = g.perm_mut(m);
    x.orig = how.orig.unwrap_or(p);
    x.sick = how.sick && !haste;
    x.phys = phys;
    x.born = born;
    if let Some(l) = loyalty {
        x.loyalty = Some(if doubling { l * 2 } else { l }); // Doubling Season: twice the loyalty
    }
    if how.undying {
        x.plus = 1; // returns with its +1/+1 counter (so it survives -X/-X effects)
        x.undying = true;
    }
    if how.plus != 0 {
        x.plus = how.plus; // enters with counters (persist: -1)
    }
    x.on_bf = true;
    if has_dsl {
        g.dsl_on = true;
    }
    g.player_mut(p).perms.push(m);
    g.bf_ver += 1;
    if g.registry.get(cd).is_some() {
        g.hooks.push(m);
    }
    cardcode::as_enters(g, p, m)?; // naming a creature type
    let st = g.turn_stamp();
    g.entered.retain(|e| e.0 == st);
    g.entered.push((st, m));
    let creature = g.db.get(cd).creature;
    let quiet = g.skip_etb || (creature && (hushed(g) || cardcode::tidebinder_response(g, p, m)?));
    if quiet {
        // an enchantment's own enters hook still runs (it sets itself up)
        let d = g.db.get(cd);
        if let Some(imp) = g.registry.get(cd).copied()
            && g.perm(m).on_bf
            && d.types.has(crate::cards::Types::ENCHANTMENT)
            && !d.creature
            && let Some(f) = imp.etb
        {
            f(g, m, p, m)?;
        }
        return Ok(m);
    }
    if creature {
        creature_entered(g, p, m)?;
    }
    if g.perm(m).on_bf {
        do_etb(g, p, m)?;
    }
    if g.dsl_on && g.perm(m).on_bf {
        let fired = crate::dsl::Fired { perm: Some(m), owner: Some(p), was_cast: how.was_cast, ..Default::default() };
        crate::dsl::fire(g, "etb", fired)?;
    }
    if !g.hooks.is_empty() && g.perm(m).on_bf {
        g.last_cast_etb = how.was_cast;
        let r = fire_trigger(g, Event::Etb, Call::Etb { p, m });
        g.last_cast_etb = false;
        r?;
    }
    Ok(m)
}

/// which land Mox Diamond discards: the one making the fewest colours, a tapped land first
fn mox_discard_key(g: &Game, c: CardId) -> f64 {
    let t = &g.db.get(c).tags;
    let cols = t.str(Tag::C).map_or(0, |s| s.chars().count()) as f64;
    cols * 2.0 - if t.has(Tag::T) { 1.0 } else { 0.0 }
}

/// a token that's a copy of card cd (Scute Swarm, Kiki-Jiki, Helm of the Host ...)
pub fn enter_token_copy(g: &mut Game, p: PlayerId, cd: CardId) -> Res<Option<PermId>> {
    let have = g.player(p).perms.iter().filter(|&&m| g.perm(m).token).count();
    if have >= TOKEN_CAP {
        return Ok(None);
    }
    let m = enter(g, p, cd, Enter::default())?;
    g.perm_mut(m).token = true;
    Ok(Some(m))
}

/// the tag-driven enter effects: as it enters (etb_static), then its triggered ones on the stack (etb_once)
pub fn do_etb(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let cd = g.perm(m).cd.unwrap();
    if g.db.get(cd).has_dsl() || opp_has(g, p, Tag::Mother) {
        return Ok(()); // the ability language handles its abilities
    }
    let mut reps = if has(g, p, Tag::Mother) && g.db.get(cd).creature { 2 } else { 1 };
    etb_static(g, p, m)?;
    if !etb_triggers(g, cd) {
        return Ok(());
    }
    let (name, imp) = etb_text(g, cd);
    if !g.hooks.is_empty() {
        reps += hooks::total_trigger_copies(g, p, "etb", Some(m)); // Panharmonicon
    }
    for _ in 0..reps {
        if g.over {
            return Ok(());
        }
        let t = Trigger {
            controller: p,
            src: Some(m),
            act: TrigAct::EtbOnce { p, m },
            name: Some(name),
            known: true,
            imp: Some(imp),
            cast_etb: false,
        };
        queue_triggers(g, vec![t])?;
    }
    Ok(())
}

/// enters-the-battlefield tags that are triggered abilities (the rest: as it enters, or static)
pub const ETB_TRIGGER_TAGS: [Tag; 18] = [
    Tag::Chromemox,
    Tag::Recruit,
    Tag::Titan,
    Tag::Archon,
    Tag::Gray,
    Tag::Wurm,
    Tag::Rsd,
    Tag::Witness,
    Tag::Wall,
    Tag::Atraxa,
    Tag::Skate,
    Tag::Bowmasters,
    Tag::Heir,
    Tag::Suntitan,
    Tag::Dualcaster,
    Tag::Uprising,
    Tag::Tok,
    Tag::Tokbig,
];
pub const ETB_SCRYFALL_TAGS: [Tag; 4] = [Tag::Tut, Tag::Treas, Tag::Drainetb, Tag::Edictetb];
/// removal kinds that are Auras locking the target down
pub const LOCK_KINDS: [&str; 4] = ["arrest", "pacify", "encrust", "kasmina"];

pub fn etb_triggers(g: &Game, cd: CardId) -> bool {
    let d = g.db.get(cd);
    let t = &d.tags;
    if ETB_TRIGGER_TAGS.iter().any(|&k| t.has(k)) || matches!(t.str(Tag::Fill), Some("stitcher" | "wayfinder")) {
        return true;
    }
    if t.has(Tag::Draw) && !(d.instant || d.sorcery) {
        return true;
    }
    if let Some(r) = t.str(Tag::Rem)
        && t.has(Tag::Etb)
        && !LOCK_KINDS.contains(&r)
    {
        return true;
    }
    g.db.get(cd).source_scryfall() && ETB_SCRYFALL_TAGS.iter().any(|&k| t.has(k))
}

/// what an enters trigger is called on the stack, and how much it matters
pub fn etb_text(g: &Game, cd: CardId) -> (Sym, f64) {
    let t = &g.db.get(cd).tags;
    if t.has(Tag::Rem) && t.has(Tag::Etb) {
        return ("remove a permanent", 6.0);
    }
    for (k, v) in [
        (Tag::Archon, ("opponent sacrifices, discards, loses 3", 7.0)),
        (Tag::Gray, ("drain", 6.0)),
        (Tag::Wurm, ("destroy small creatures", 6.0)),
        (Tag::Heir, ("opponents' creatures get -2/-2", 6.0)),
        (Tag::Bowmasters, ("1 damage; amass 1", 5.0)),
        (Tag::Suntitan, ("return a permanent", 4.0)),
        (Tag::Titan, ("two 2/2 Zombies", 3.0)),
        (Tag::Atraxa, ("reveal ten", 5.0)),
        (Tag::Rsd, ("search your library", 4.0)),
        (Tag::Recruit, ("search your library", 4.0)),
    ] {
        if t.has(k) {
            return v;
        }
    }
    ("enters", 3.0)
}

/// what happens as it enters, or is true while it's out (not a triggered ability)
pub fn etb_static(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let cd = g.perm(m).cd.unwrap();
    let t = g.db.get(cd).tags.clone();
    if t.has(Tag::Flute) {
        // Disruptor Flute: choose a card name
        if let Some((name, _)) = ai::flute_pick(g, p) {
            g.flutes.push((m, name));
            g.player_mut(p).stat("flute_named", 1);
            crate::glog!(g, "    Disruptor Flute names {}", g.db.get(name).name);
        }
    }
    if t.has(Tag::Prepare) {
        g.perm_mut(m).data.set(DataKey::Prepared, Val::Bool(true)); // its back-face spell can be cast as a copy
    }
    if let Some(r) = t.str(Tag::Rem)
        && t.has(Tag::Etb)
        && LOCK_KINDS.contains(&r)
    {
        crate::engine::removal::etb_removal(g, p, m)?; // an Aura: attached as it resolves
    }
    if t.has(Tag::Mycoloth) {
        // devour 2: eat up to three tokens
        let toks: Vec<PermId> =
            g.player(p).perms.iter().copied().filter(|&x| g.perm(x).token && g.is_creature(x)).take(3).collect();
        for &x in &toks {
            die(g, x, "sac")?;
        }
        g.perm_mut(m).plus += 2 * toks.len() as i32;
    }
    if t.has(Tag::Endraze) {
        let pl = g.player_mut(p);
        pl.pumpadd += 2;
        pl.trample = true;
    }
    if t.has(Tag::Spider) {
        // Old Fat Spider: hexproof on your best creature (like Boots)
        let cr: Vec<PermId> =
            g.player(p).perms.iter().copied().filter(|&x| g.is_creature(x) && g.perm(x).cd.is_some()).collect();
        if let Some(b) = max_by(&cr, |x| if card_tag(g, x, Tag::Veyran) { 10.0 } else { 0.0 } + pval(g, x)) {
            g.perm_mut(m).attached = Some(b);
        }
    }
    Ok(())
}

/// the tag-driven enters triggers, as they resolve (Python's `etb_once`)
pub fn etb_once(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let cd = g.perm(m).cd.unwrap();
    let d = g.db.get(cd);
    let t = d.tags.clone();
    let (scryfall, is_spell) = (d.source_scryfall(), d.instant || d.sorcery);
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if t.has(Tag::Chromemox) {
        crate::engine::tutors::chrome_imprint(g, p, m)?;
    }
    if t.has(Tag::Recruit) {
        crate::engine::tutors::tutor(g, p, "cre2")?; // Imperial Recruiter: creature with power 2 or less
    }
    if t.has(Tag::Titan) {
        make_tokens(g, p, Tokens { color: Some(Colors::from_letters("B")), ..Tokens::new(2, 2) })?;
    }
    if t.has(Tag::Archon) && !opps.is_empty() {
        archon_trig(g, p, None)?;
    }
    if t.has(Tag::Gray) {
        let dev: i32 = 2 + g
            .player(p)
            .perms
            .iter()
            .filter(|&&x| x != m)
            .filter_map(|&x| g.perm(x).cd)
            .filter(|&c| g.db.get(c).perm)
            .map(|c| g.db.get(c).pips.chars().filter(|&ch| ch == 'B').count() as i32)
            .sum::<i32>();
        let dev = dev.min(10);
        for &q in &opps {
            lose_life(g, q, dev, Some(p), "drain", None)?;
        }
        gain(g, p, dev * opps.len() as i32)?;
    }
    if t.has(Tag::Wurm) {
        for &q in &opps {
            for x in g.player(q).perms.clone() {
                if g.is_creature(x) && etgh(g, x) <= 2 {
                    die(g, x, "destroy")?; // the 2 life is its dies trigger ('wurmdrain', in die_rest)
                }
            }
        }
    }
    if t.has(Tag::Rsd) {
        crate::engine::tutors::tutor(g, p, "any")?;
    }
    if t.has(Tag::Witness) {
        ai::regrow(g, p, false)?;
    }
    if t.has(Tag::Wall) {
        ai::regrow(g, p, true)?;
    }
    if t.has(Tag::Atraxa) {
        crate::engine::tutors::atraxa_reveal(g, p)?;
    }
    if let Some(n) = t.int(Tag::Draw)
        && !is_spell
    {
        draw(g, p, n.max(0) as u32, false)?;
    }
    if t.has(Tag::Skate) {
        for x in g.player(p).perms.clone() {
            let y = g.perm_mut(x);
            if y.plus > 0 {
                y.plus = (y.plus * 2).min(200);
            }
        }
    }
    if t.has(Tag::Bowmasters) && !opps.is_empty() {
        // HUMAN(phase 9): the person's target (hc.deal_damage), then amass
        let x1: Vec<PermId> = opps
            .iter()
            .flat_map(|&q| g.player(q).perms.iter().copied())
            .filter(|&x| g.is_creature(x) && etgh(g, x) <= 1 && pval(g, x) >= 2.0 && !untargetable(g, x))
            .collect();
        if let Some(b) = max_by(&x1, |x| pval(g, x)) {
            die(g, b, "destroy")?;
        } else if let Some(q) = max_by(&opps, |o| threat(g, p, o)) {
            lose_life(g, q, 1, Some(p), "triggers", None)?;
        }
        amass(g, p, 1)?;
    }
    if let Some(r) = t.str(Tag::Rem)
        && t.has(Tag::Etb)
        && !LOCK_KINDS.contains(&r)
    {
        crate::engine::removal::etb_removal(g, p, m)?;
    }
    if t.has(Tag::Heir) {
        // Sephiroth, Planet's Heir: opponents' creatures get -2/-2
        for &q in &opps {
            for x in g.player(q).perms.clone() {
                if !g.is_creature(x) || g.perm(x).phased {
                    continue;
                }
                let y = g.perm_mut(x);
                y.eot_pt = (y.eot_pt.0 - 2, y.eot_pt.1 - 2);
                if etgh(g, x) <= 0 {
                    die(g, x, "sba")?;
                }
            }
        }
    }
    if t.has(Tag::Suntitan) {
        sun_titan(g, p)?;
    }
    if t.has(Tag::Dualcaster) {
        // copy a spell: approximated as copying your last instant/sorcery
        let last = g.player(p).gy.iter().rev().copied().find(|&x| g.db.get(x).instant || g.db.get(x).sorcery);
        if let Some(last) = last {
            if g.player(p).key == "veyran" {
                crate::engine::cast::magecraft(g, p, Some(last), true)?;
            }
            if let Some(n) = g.db.get(last).tags.int(Tag::Draw) {
                draw(g, p, n.max(0) as u32, false)?;
            }
        }
    }
    if t.has(Tag::Uprising) && g.player(p).perms.iter().any(|&x| g.is_creature(x) && epow(g, x) >= 4) {
        draw(g, p, 1, false)?;
    }
    if let Some(n) = t.int(Tag::Tok) {
        let dt = t.has(Tag::Tokdt);
        make_tokens(
            g,
            p,
            Tokens {
                fly: t.has(Tag::Tokfly),
                lifelink: !dt && !t.has(Tag::Tokp),
                dt,
                color: if dt { Some(Colors::from_letters("G")) } else { None },
                ..Tokens::new(n.max(0) as u32, t.int(Tag::Tokp).unwrap_or(1))
            },
        )?;
    }
    if scryfall {
        // generic enter effects of auto-tagged cards
        if let Some(k) = t.str(Tag::Tut) {
            crate::engine::tutors::tutor(g, p, k)?;
        }
        if let Some(n) = t.int(Tag::Treas) {
            add_treasure(g, p, n)?;
        }
        if let Some(n) = t.int(Tag::Drainetb) {
            for &q in &opps {
                lose_life(g, q, n, Some(p), "drain", None)?;
            }
        }
        if t.has(Tag::Edictetb) {
            for &q in &opps {
                edict(g, q, false)?;
            }
        }
    }
    if t.has(Tag::Tokbig) {
        for sz in [2, 3, 4] {
            make_tokens(g, p, Tokens::new(1, sz))?; // Trostani's Summoner: 2/2, 3/3, 4/4
        }
    }
    match t.str(Tag::Fill) {
        Some("stitcher") => mill(g, p, 3)?,
        Some("wayfinder") => {
            let n = g.player(p).library.len().min(4);
            let mut top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
            if let Some(i) = top.iter().position(|&c| g.db.get(c).land) {
                let l = top.remove(i);
                g.player_mut(p).hand.push(l);
            }
            for c in top {
                g.player_mut(p).gy.push(c);
                if is_keyspell(g, c) {
                    g.player_mut(p).stat("key_milled", 1);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Archon of Cruelty: an opponent sacrifices a creature, discards a card and loses 3 life; you gain 3 and draw
pub fn archon_trig(g: &mut Game, p: PlayerId, q: Option<PlayerId>) -> Res {
    // HUMAN(phase 9): the person picks the opponent (hc.target_opponent)
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let Some(q) = q.or_else(|| max_by(&opps, |o| threat(g, p, o))) else { return Ok(()) };
    edict(g, q, false)?;
    let n = g.player(q).hand.len() as u64;
    if n > 0 {
        let i = g.rng.below(n) as usize;
        discard_index(g, q, i)?;
    }
    lose_life(g, q, 3, Some(p), "drain", None)?;
    gain(g, p, 3)?;
    draw(g, p, 1, false)
}

/// what losing m to a sacrifice costs its controller: its value, or less when Marchesa will return it
pub fn sac_worth(g: &Game, m: PermId) -> f64 {
    let v = pval(g, m);
    let x = g.perm(m);
    if g.player(x.owner).borrowed.contains(&m) {
        return -0.5 * v; // stolen until end of turn: free to lose
    }
    if g.marchesa_on && g.is_creature(m) && !x.token && x.plus > 0 {
        return cardcode::marchesa_sac_worth(g, m, v);
    }
    v
}

/// q sacrifices the creature it values least (least_power: among those with the least power, Witch-king)
pub fn edict(g: &mut Game, q: PlayerId, least_power: bool) -> Res {
    let mut cr: Vec<PermId> =
        g.player(q).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased && !no_sac(g, m)).collect();
    if !cr.is_empty() && least_power {
        let lo = cr.iter().map(|&m| epow(g, m)).min().unwrap();
        cr.retain(|&m| epow(g, m) == lo);
    }
    // HUMAN(phase 9): the person picks (hc.sacrifice_creature)
    if let Some(m) = min_by(&cr, |x| sac_worth(g, x)) {
        die(g, m, "sac")?;
    }
    Ok(())
}

// ------------------------------------------------------------------ lands
pub const BASIC_LANDS: [&str; 5] = ["Forest", "Island", "Plains", "Swamp", "Mountain"];

fn basics_in(g: &Game, p: PlayerId) -> Vec<CardId> {
    searchable(g, p).into_iter().filter(|&c| BASIC_LANDS.contains(&&*g.db.get(c).name)).collect()
}

/// search for n basic lands and put them onto the battlefield (tapped or not)
pub fn land_ramp(g: &mut Game, p: PlayerId, n: u32, tapped: bool) -> Res {
    // HUMAN(phase 9): the person picks each basic (hc.pick_basic)
    for _ in 0..n {
        let basics = basics_in(g, p);
        let Some(&c) = g.rng.choice(&basics) else { return Ok(()) };
        let lib = &mut g.player_mut(p).library;
        let i = lib.iter().position(|&x| x == c).unwrap();
        lib.remove(i);
        if let Some(a) = agent_for(g, p) {
            agent_take(g, a, p, c);
            continue;
        }
        g.add_land_entering(p, c, tapped);
        landfall(g, p)?;
    }
    Ok(())
}

/// the lands that just entered under p's control: their enter triggers (a Temple's scry, a gain land's life)
pub fn lands_entered(g: &mut Game, p: PlayerId) -> Res {
    for l in g.player(p).lands.clone() {
        if g.land(l).data.truthy(DataKey::In) {
            continue;
        }
        g.land_mut(l).data.set(DataKey::In, Val::Bool(true));
        let cd = g.land(l).cd;
        for fx in g.db.get(cd).land_etb_fx.clone() {
            let name = match fx {
                LandEtb::Scry(n) => format!("scry {n}"),
                LandEtb::Gain(n) => format!("gain {n} life"),
            };
            if g.over || !crate::engine::stack::trigger_window_card(g, p, cd, &name)? {
                continue;
            }
            match fx {
                LandEtb::Scry(n) => cardcode::scry(g, p, n, false)?,
                LandEtb::Gain(n) => {
                    gain(g, p, n)?;
                    crate::glog!(g, "    {} gains {} life ({})", g.player(p).name, n, g.db.get(cd).name);
                }
            }
        }
    }
    Ok(())
}

pub fn landfall(g: &mut Game, p: PlayerId) -> Res {
    lands_entered(g, p)?;
    let copies = 1 + hooks::total_trigger_copies(g, p, "landfall", None); // Ancient Greenwarden
    for _ in 0..copies {
        landfall_once(g, p)?;
    }
    Ok(())
}

fn landfall_once(g: &mut Game, p: PlayerId) -> Res {
    for _ in find(g, p, Tag::Landfall2) {
        make_tokens(g, p, Tokens::new(1, 2))?; // Felidar Retreat: 2/2 Cat per land
    }
    let lands = g.player(p).lands.clone();
    let fields = lands.iter().filter(|&&l| g.db.get(g.land(l).cd).tag(Tag::Fotd)).count();
    let mut names: Vec<CardId> = lands.iter().map(|&l| g.land(l).cd).collect();
    names.sort();
    names.dedup();
    if fields > 0 && names.len() >= 7 {
        // Field of the Dead: 7+ lands with different names
        for _ in 0..fields {
            make_tokens(g, p, Tokens { color: Some(Colors::from_letters("B")), ..Tokens::new(1, 2) })?;
        }
    }
    if g.dsl_on {
        crate::dsl::fire(g, "landfall", crate::dsl::Fired { player: Some(p), ..Default::default() })?;
    }
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::Landfall, Call::Player { p })?;
    }
    cardcode::gy_landfall(g, p)
}

/// search for a basic land and put it into your hand
pub fn land_to_hand(g: &mut Game, p: PlayerId) -> Res {
    // HUMAN(phase 9): the person picks (hc.pick_basic), then the library is shuffled
    let basics = basics_in(g, p);
    if let Some(&c) = g.rng.choice(&basics) {
        let lib = &mut g.player_mut(p).library;
        let i = lib.iter().position(|&x| x == c).unwrap();
        lib.remove(i);
        match agent_for(g, p) {
            Some(a) => agent_take(g, a, p, c),
            None => g.player_mut(p).hand.push(c),
        }
    }
    Ok(())
}

/// the cards p can find when searching its library (Aven Mindcensor: only the top four)
pub fn searchable(g: &Game, p: PlayerId) -> Vec<CardId> {
    let lib = &g.player(p).library;
    let lim = hooks::min_search_limit(g, p);
    match lim {
        Some(n) => lib[lib.len().saturating_sub(n as usize)..].to_vec(),
        None => lib.clone(),
    }
}

/// p creates n Treasure tokens (through the token machinery, so Academy Manufactor, Xorn, Chatterfang and token
/// doublers apply)
pub fn add_treasure(g: &mut Game, p: PlayerId, n: i32) -> Res {
    if n <= 0 {
        return Ok(());
    }
    make_artifact_tokens(g, p, "Treasure", n)
}

/// create n Treasure / Food / Clue tokens, applying Academy Manufactor and token doubling (common.make_artifact_tokens)
pub fn make_artifact_tokens(g: &mut Game, p: PlayerId, kind: Sym, n: i32) -> Res {
    if n <= 0 {
        return Ok(());
    }
    let n = n * crate::dsl::token_mult(g, p) as i32;
    let manufactor =
        g.player(p).perms.iter().any(|&m| !g.perm(m).phased && card_name(g, m) == Some("Academy Manufactor"));
    let kinds: Vec<Sym> = if manufactor { vec!["Treasure", "Food", "Clue"] } else { vec![kind] };
    for &k in &kinds {
        let pl = g.player_mut(p);
        match k {
            "Treasure" => pl.treasures += n as u32,
            "Food" => pl.foods += n as u32,
            _ => pl.clues += n as u32,
        }
    }
    if !g.hooks.is_empty() {
        let total = n * kinds.len() as i32;
        fire_trigger(g, Event::TokenCreated, Call::ArtifactTokens { p, kinds, n: total })?;
    }
    Ok(())
}

/// return the best permanent card with mana value 3 or less from graveyard to the battlefield
pub fn sun_titan(g: &mut Game, p: PlayerId) -> Res {
    let cs: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| (g.db.get(c).perm || g.db.get(c).land) && g.db.get(c).cmc <= 3)
        .collect();
    let key = |c: CardId| {
        let d = g.db.get(c);
        (d.creature as i32 as f64) * 1000.0 + d.pow as f64 * 10.0 + d.cmc as f64
    };
    let Some(x) = max_by(&cs, key) else { return Ok(()) };
    let gy = &mut g.player_mut(p).gy;
    let i = gy.iter().position(|&c| c == x).unwrap();
    gy.remove(i);
    if g.db.get(x).land {
        g.add_land_entering(p, x, true);
    } else {
        enter(g, p, x, Enter::default())?;
    }
    Ok(())
}

// ------------------------------------------------------------------ discards, Tergrid, Opposition Agent
/// q discards these cards from hand: to the graveyard (exiled instead under Necropotence), then Tergrid
pub fn discard_cards(g: &mut Game, q: PlayerId, cards: &[CardId]) -> Res {
    let necro = has(g, q, Tag::Necro);
    let mut done = vec![];
    for &c in cards {
        let pl = g.player_mut(q);
        let Some(i) = pl.hand.iter().position(|&x| x == c) else { continue }; // paid away meanwhile
        pl.hand.remove(i);
        done.push(c);
        if c == pl.cmd {
            pl.cmd_in_zone = true; // a commander in hand (Command Beacon): to the command zone
            continue;
        }
        if necro {
            pl.exile.push(c)
        } else {
            pl.gy.push(c)
        }
        if !g.hooks.is_empty() {
            fire_trigger(g, Event::Discard, Call::Discard { p: q, c })?;
            g.player_mut(q).discarded_turn = Some(g.turn_stamp());
        }
    }
    if !g.hooks.is_empty() && !necro {
        fire_trigger(g, Event::CardsToGy, Call::Cards { p: q, cards: done.clone() })?;
    }
    if !necro {
        for c in done {
            tergrid_steal(g, q, c, q)?;
        }
    }
    Ok(())
}

/// q discards the card at position i in hand (random discards)
pub fn discard_index(g: &mut Game, q: PlayerId, i: usize) -> Res {
    let c = g.player_mut(q).hand.remove(i);
    if c == g.player(q).cmd {
        g.player_mut(q).cmd_in_zone = true;
        return Ok(());
    }
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::Discard, Call::Discard { p: q, c })?;
        g.player_mut(q).discarded_turn = Some(g.turn_stamp());
    }
    if has(g, q, Tag::Necro) {
        g.player_mut(q).exile.push(c);
        return Ok(());
    }
    g.player_mut(q).gy.push(c);
    tergrid_steal(g, q, c, q)
}

/// Tergrid, God of Fright: whenever an opponent sacrifices a nontoken permanent or discards a permanent card, you may
/// put that card from a graveyard onto the battlefield under your control
pub fn tergrid_steal(g: &mut Game, loser: PlayerId, c: CardId, gy_owner: PlayerId) -> Res {
    let d = g.db.get(c);
    if !(d.perm || d.land) || !g.player(gy_owner).gy.contains(&c) {
        return Ok(());
    }
    for i in 0..g.players.len() {
        let t = PlayerId(i as u8);
        if !g.player(t).alive || t == loser || !has(g, t, Tag::Tergrid) {
            continue;
        }
        let gy = &mut g.player_mut(gy_owner).gy;
        let k = gy.iter().position(|&x| x == c).unwrap();
        gy.remove(k);
        if g.db.get(c).land {
            g.add_land_entering(t, c, false);
        } else {
            enter(g, t, c, Enter { orig: Some(gy_owner), ..Enter::default() })?;
        }
        g.player_mut(t).stat("tergrid_steals", 1);
        crate::glog!(
            g,
            "    Tergrid puts {} onto the battlefield under {}'s control",
            g.db.get(c).name,
            g.player(t).name
        );
        return Ok(());
    }
    Ok(())
}

/// the opponent controlling Opposition Agent while p searches, if any
pub fn agent_for(g: &Game, p: PlayerId) -> Option<PlayerId> {
    g.opps(p).find(|&q| g.player(q).perms.iter().any(|&m| card_tag(g, m, Tag::Agent) && !g.perm(m).phased))
}

/// Opposition Agent: the searching player exiles the card; a may play it (held in a's hand here)
pub fn agent_take(g: &mut Game, a: PlayerId, p: PlayerId, c: CardId) {
    let pl = g.player_mut(a);
    pl.hand.push(c);
    pl.agent_ids.push(c);
    pl.stat("agent_takes", 1);
    crate::glog!(
        g,
        "    Opposition Agent: {} takes {} from {}'s search",
        g.player(a).name,
        g.db.get(c).name,
        g.player(p).name
    );
}
