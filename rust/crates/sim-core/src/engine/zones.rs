//! Zones: drawing, milling, tokens, permanents entering, leaving and dying, lands, discards
//! (engine.py's "zones" section, and enter / do_etb / etb_once / lands / discards from later in the file).

use crate::ai;
use crate::cardcode;
use crate::cards::Colors;
use crate::engine::hooks::{self, fire_trigger};
use crate::engine::life::{gain, lose_life};
use crate::engine::repl;
use crate::engine::stack::{queue_triggers, trigger_window};
use crate::engine::values::*;
use crate::flow::Res;
use crate::hooks::{Call, Event, Sacrificed, TrigAct, Trigger};
use crate::ids::{CardId, PermId, PlayerId};
use crate::satsum::SatSum;
use crate::state::{CopyExc, CopyOf, DataKey, Game, PermData, Val};
use crate::sym::Sym;
use crate::tag::Tag;

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
            pl.drawn_cards.clear();
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
            let labman = repl::labman(g, p);
            if labman {
                return ai::win(g, p, "Laboratory Maniac", None);
            }
            g.player_mut(p).decked = true;
            return Ok(());
        }
        if !g.hooks.is_empty() && crate::impls::t0_anikthea::abundance_draw(g, p)? {
            continue; // Abundance: a land or a nonland card instead of the draw
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
        if crate::impls::t45_prosper::uba_draw(g, p)? {
            continue; // Uba Mask: exiled instead of drawn
        }
        let pl = g.player_mut(p);
        pl.draw_n += 1;
        let c = pl.library.pop().unwrap();
        pl.hand.push(c);
        pl.drawn_cards.push(c);
        if pl.draw_n == 1 {
            pl.miracle = Some((st, c));
        }
        let new = pl.seen.insert(c); // --- ai-flow: new to the player (the flow stats)
        pl.stat("cards_drawn", 1);
        crate::gf_audit::draw(g, p, c); // --- ai-audit
        crate::t6stats::flow_draw(g, p, step && k == 0, new); // --- ai-flow
        if has(g, p, Tag::Ironman) && crate::human::is_human(g, p) {
            crate::human::triggers_t1::ironman_counter(g, p)?; // the person picks the creature
        } else if has(g, p, Tag::Ironman) {
            // "put a +1/+1 counter on target creature": the AI picks the creature it values most (the Army before,
            // whatever else it had; the Army still wins a tie)
            // (our own creature with shroud or protection from Iron Man's colours is no legal target; hexproof is)
            let cols = find(g, p, Tag::Ironman).first().map_or(Colors::NONE, |&m| colors_of(g, m));
            let own: Vec<PermId> = g
                .player(p)
                .perms
                .iter()
                .copied()
                .filter(|&m| g.is_creature(m) && !g.perm(m).phased && !untargetable_by_its_controller(g, m, cols))
                .collect();
            if let Some(t) = max_by(&own, |m| pval(g, m) + if g.perm(m).army { 0.01 } else { 0.0 }) {
                repl::put_plus(g, t, 1);
            } else {
                // none of ours is a legal target (all have shroud or protection from blue: Akroma's Will) and the trigger
                // must target, so the counter can only help an opponent. Paying a ward is optional and buys nothing here:
                // the AI never pays one. It aims at a creature with a ward (the ward counters the ability, nothing is
                // put), else at the opposing creature worth least
                let theirs: Vec<PermId> = g
                    .opps(p)
                    .flat_map(|q| g.player(q).perms.iter().copied())
                    .filter(|&m| g.is_creature(m) && !untargetable(g, m) && !protected_from(g, m, cols))
                    .collect();
                let key = |m: PermId| -pval(g, m) + if crate::engine::ward::has_ward(g, m) { 1000.0 } else { 0.0 };
                if let Some(t) = max_by(&theirs, key) {
                    if crate::engine::ward::has_ward(g, t) {
                        crate::glog!(g, "    ward counters Iron Man's counter on {}", g.perm(t).name);
                    } else {
                        repl::put_plus(g, t, 1);
                    }
                }
            }
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
                let turns = g.player(q).turns;
                g.player_mut(q).milestone.entry("sheoldred").or_insert(turns); // the sheoldred deck's plan online
            }
            if has(g, q, Tag::Tithe) && cardcode::tithe_unpaid(g, p)? {
                add_treasure(g, q, 1)?;
            }
            if extra && has(g, q, Tag::Bowmasters) {
                let human = crate::human::is_human(g, q);
                for bm in find(g, q, Tag::Bowmasters) {
                    for _ in 0..1 + crate::engine::stack::extra_for(g, bm) {
                        amass(g, q, 1)?;
                        if human {
                            // practice mode: each Bowmasters, your target
                            crate::human::choices::deal_damage_from(
                                g,
                                q,
                                1,
                                "Orcish Bowmasters",
                                "triggers",
                                Some(bm),
                            )?;
                        } else {
                            let ll = crate::engine::life::has_lifelink(g, bm);
                            let dealt = crate::engine::life::lose_life_dealt(g, p, 1, Some(q), "triggers", None)?;
                            crate::engine::life::lifelink(g, bm, ll, dealt)?;
                        }
                    }
                }
            }
        }
        if has(g, p, Tag::SheoA) {
            gain(g, p, 2)?;
        }
    }
    Ok(())
}

/// card c goes to owner o's graveyard; Blightsteel Colossus is revealed and shuffled into o's library instead ("if it
/// would be put into a graveyard from anywhere": milled, discarded or countered; from the battlefield `to_zone_card`)
pub fn put_in_gy(g: &mut Game, o: PlayerId, c: CardId) {
    if crate::impls::karn_cards::shuffles_back(g, c) {
        crate::glog!(
            g,
            "    {} is revealed and shuffled into {}'s library instead",
            g.db.get(c).name,
            g.player(o).name
        );
        let n = g.player(o).library.len() as u64;
        let i = g.rng.below(n + 1) as usize;
        g.player_mut(o).library.insert(i, c);
    } else {
        g.player_mut(o).gy.push(c);
    }
}

pub fn mill(g: &mut Game, p: PlayerId, n: usize) -> Res {
    if !g.hooks.is_empty() && n > 0 {
        let lib = &g.player(p).library;
        let cards = lib[lib.len().saturating_sub(n)..].to_vec();
        fire_trigger(g, Event::CardsToGy, Call::Cards { p, cards })?;
    }
    for _ in 0..n {
        let Some(c) = g.player_mut(p).library.pop() else { return Ok(()) };
        if crate::impls::karn_cards::shuffles_back(g, c) {
            put_in_gy(g, p, c); // Blightsteel Colossus: shuffled into the library instead
            continue;
        }
        g.player_mut(p).gy.push(c);
        crate::impls::t12_brago_meren::note_gy(g, c); // --- Tiers 1/2 (brago-meren): Twilight Diviner's record
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
    repl::put_plus(g, a, n); // the counters step: Mauhúr's one more, Hardened Scales, doublers
    Ok(())
}

// ------------------------------------------------------------------ tokens
/// default colour of a deck's tokens
pub fn token_color(key: &str) -> Colors {
    Colors::from_letters(match key {
        "najeela" | "galadriel" | "yshtola" => "W",
        "sauron" => "B",
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
pub fn make_tokens(g: &mut Game, p: PlayerId, mut spec: Tokens) -> Res<Vec<PermId>> {
    let mut out = vec![];
    let mut n = repl::tokens(g, p, spec.n); // the tokens step: doublers
    let have = g.player(p).perms.iter().filter(|&&m| g.perm(m).token).count();
    n = n.min(TOKEN_CAP.saturating_sub(have) as u32); // runaway token growth is already lethal
    if n == 0 {
        return Ok(out);
    }
    let (mut pw, mut tg, mut sick) = (spec.pow, spec.tgh, spec.sick);
    if has(g, p, Tag::Jinnie) && pw < 2 && !spec.attacking {
        (pw, tg, sick) = (2, Some(2), false); // Jinnie Fay: 2/2 Cats with haste instead
    }
    let angels = crate::impls::t0_zurgo::divine_visitation(g, p);
    if angels {
        // Divine Visitation: 4/4 white Angels with flying and vigilance instead
        (pw, tg) = (4, Some(4));
        spec.fly = true;
        spec.warrior = false;
        spec.lifelink = false;
        spec.dt = false;
        spec.color = Some(Colors::from_letters("W"));
        spec.types = vec!["angel"];
        spec.data = PermData::default();
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
        x.vig = angels;
        x.on_bf = true;
        g.enter_no += 1;
        g.perm_mut(m).born = g.enter_no;
        let x = g.perm_mut(m);
        if spec.attacking {
            x.tapped = true;
        }
        g.player_mut(p).perms.push(m);
        if repl::enters_tapped(g, p, Some(m)) {
            g.perm_mut(m).tapped = true; // before the tokens' enter triggers
        }
        if etgh(g, m) <= 0 {
            die(g, m, "sba")?;
            continue;
        }
        out.push(m);
    }
    let k = out.len() as i32;
    // an opponent's Elesh Norn, or Hushbringer for creature tokens: what their entering would trigger does not
    let blank = hooks::enter_silenced(g, p, out.first().copied());
    note_tokens(g, p, k as u32);
    crate::impls::t45_krenko::note_entered(g, p, &out); // --- Tiers 4/5 (krenko): Hobgoblin Bandit Lord
    if k > 0 && !g.no_fang {
        cardcode::chatterfang_squirrels(g, p, k)?;
    }
    if k > 0 && !blank {
        if has(g, p, Tag::Crusade) {
            let copies = 1 + hooks::enter_copies(g, p, Some(out[0])) as i32;
            let n = k * find(g, p, Tag::Crusade).len() as i32 * copies;
            for m in g.player(p).perms.clone() {
                if g.is_creature(m) {
                    let x = g.perm_mut(m);
                    x.plus = (x.plus + n).min(999);
                }
            }
        }
        if has(g, p, Tag::Warleader) {
            let copies = 1 + hooks::enter_copies(g, p, Some(out[0])) as i32;
            for q in g.opps(p).collect::<Vec<_>>() {
                lose_life(g, q, k * copies, Some(p), "drain", Some(true))?; // Warleader's Call deals damage
            }
        }
        if has(g, p, Tag::Wisp) && pw <= 2 {
            // I1: event copies and each listener's source extras add once.
            for w in find(g, p, Tag::Wisp) {
                for _ in 0..1 + hooks::enter_copies(g, p, Some(out[0])) as usize + crate::engine::stack::extra_for(g, w)
                {
                    let opps: Vec<_> = g.opps(p).collect();
                    for &q in &opps {
                        lose_life(g, q, k, Some(p), "drain", None)?;
                    }
                    gain(g, p, k)?;
                }
            }
        }
    }
    if k > 0 && !blank && has(g, p, Tag::Uprising) {
        // Garruk's Uprising: a card for each token with power 4 or more ("whenever a creature with power 4 or
        // greater you control enters" counts tokens; Titania's 5/3 Elementals)
        let big: Vec<_> = out.iter().copied().filter(|&m| epow(g, m) >= 4).collect();
        if let Some(&m) = big.first() {
            draw(g, p, big.len() as u32 * (1 + hooks::enter_copies(g, p, Some(m))), false)?;
        }
    }
    if let Some(&m) = out.first() {
        shards_trigger(g, p, k, Some(m))?;
    }
    if g.dsl_on {
        for &x in &out {
            crate::dsl::fire(g, "etb", crate::dsl::Fired { perm: Some(x), owner: Some(p), ..Default::default() })?;
        }
    }
    if k > 0 && !g.hooks.is_empty() {
        // a person's "damage to any target" trigger of each token (Terror of the Peaks): asked once for the batch
        let batch = out.len() > 1 && crate::human::choices::batch_begin(g, p);
        let r = fire_trigger(g, Event::TokensEnter, Call::Tokens { p, toks: out.clone() }); // Plumecreed Mentor
        crate::human::choices::batch_end(g, batch);
        r?;
    }
    Ok(out)
}

/// Aura Shards: whenever a creature enters under your control, destroy target artifact or enchantment
pub fn shards_trigger(g: &mut Game, p: PlayerId, k: i32, entering: Option<PermId>) -> Res {
    if k <= 0 || !has(g, p, Tag::Shards) || hooks::enter_silenced(g, p, entering) {
        return Ok(());
    }
    let reps = k * (1 + hooks::enter_copies(g, p, entering) as i32);
    if crate::human::is_human(g, p) {
        // practice mode: you may destroy one, your pick
        for _ in 0..reps {
            let tg: Vec<PermId> = g
                .players
                .iter()
                .filter(|q| q.alive)
                .flat_map(|q| q.perms.iter().copied())
                .filter(|&m| {
                    let x = g.perm(m);
                    x.cd.is_some_and(|c| {
                        let d = g.db.get(c);
                        d.types.has(crate::cards::Types::ARTIFACT) || d.types.has(crate::cards::Types::ENCHANTMENT)
                    }) && !x.phased
                        && !(x.owner != p && untargetable(g, m))
                })
                .collect();
            if tg.is_empty() {
                return Ok(());
            }
            let labels: Vec<String> =
                tg.iter().map(|&m| crate::human::legal::describe_target(g, p, crate::human::Target::Perm(m))).collect();
            let prompt = "Aura Shards: destroy target artifact or enchantment?";
            let Some(k2) = crate::human::choices::choose(g, p, "target", prompt, &labels, Some("none"), None)? else {
                return Ok(());
            };
            g.player_mut(p).stat("shards_kill", 1);
            crate::engine::removal::apply_removal(g, Some(p), tg[k2], "destroy", None)?;
            if g.over {
                return Ok(());
            }
        }
        return Ok(());
    }
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
    if !hooks::enter_silenced(g, p, Some(m)) {
        if has(g, p, Tag::Crusade) {
            let n = find(g, p, Tag::Crusade).len() as i32 * (1 + hooks::enter_copies(g, p, Some(m)) as i32);
            for x in g.player(p).perms.clone() {
                if g.is_creature(x) {
                    let y = g.perm_mut(x);
                    y.plus = (y.plus + n).min(999);
                }
            }
        }
        if has(g, p, Tag::Warleader) {
            let copies = 1 + hooks::enter_copies(g, p, Some(m)) as i32;
            for q in g.opps(p).collect::<Vec<_>>() {
                lose_life(g, q, copies, Some(p), "drain", Some(true))?; // Warleader's Call deals damage
            }
        }
        if has(g, p, Tag::Wisp) && epow(g, m) <= 2 {
            for w in find(g, p, Tag::Wisp).into_iter().filter(|&w| w != m) {
                for _ in 0..1 + hooks::enter_copies(g, p, Some(m)) as usize + crate::engine::stack::extra_for(g, w) {
                    let opps: Vec<_> = g.opps(p).collect();
                    for &q in &opps {
                        lose_life(g, q, 1, Some(p), "drain", None)?;
                    }
                    gain(g, p, 1)?;
                }
            }
        }
        if has(g, p, Tag::Uprising) && epow(g, m) >= 4 {
            draw(g, p, 1 + hooks::enter_copies(g, p, Some(m)), false)?;
        }
    }
    shards_trigger(g, p, 1, Some(m))
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
    // a creature dying under Hushbringer causes no triggered ability (what it set up before, and a duration that ends, still do)
    let hushed = g.dying == Some(m) && g.is_creature(m) && repl::hushed(g);
    if on {
        crate::impls::vondam_elspeth::leaving(g, m); // Syr Vondam: his power as he leaves, and the wipe he leaves in
        crate::engine::stack::note_leaving(g, m);
    }
    if g.is_creature(m) && g.perm(m).plus > 0 && on && has(g, p, Tag::Ozolith) {
        let n = g.perm(m).plus;
        g.player_mut(p).ozolith_counters += n; // The Ozolith keeps the counters
    }
    if on {
        g.player_mut(p).perms.retain(|&x| x != m);
        if g.perm(m).token {
            g.player_mut(p).lands.retain(|&x| x != m); // (a creature token land is listed in both: Astral Dragon)
        }
        g.perm_mut(m).on_bf = false;
        g.perm_mut(m).data.remove(DataKey::PhaseInWith);
        if !g.control_links.is_empty() {
            crate::engine::control::end_links(g);
        }
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
            // a leave-caused trigger happens once more for each Teysa Karlov, Drivnod or Gandalf the White; a duration that
            // ends with the leaving ("until this leaves the battlefield") is not a trigger
            let trigger = hooks::leave_hook_is_trigger(&imp, Event::Leaves);
            let times = match (trigger, hushed) {
                (true, true) => 0,
                (true, false) => hooks::leave_trigger_times(g, p, m),
                (false, _) => 1,
            };
            let mut entries = vec![];
            for k in 0..times {
                if imp.runs_at_once(Event::Leaves) {
                    (imp.leaves.unwrap())(g, m, m)?; // an effect ending, or a trigger that has no window (Oblivion Ring)
                } else {
                    let act = TrigAct::Hook { event: Event::Leaves, call: Call::Leaves { m } };
                    entries.push(Trigger { repeat: k > 0, ..Trigger::new(p, Some(m), act) }); // "when it leaves"
                }
            }
            queue_triggers(g, entries)?;
        }
    }
    if on && !hushed {
        for s in crate::impls::t0_giada::sleuths_for(g, m) {
            let o = g.perm(s).owner;
            let act = TrigAct::Hook { event: Event::Leaves, call: Call::Leaves { m } };
            let entries = (0..hooks::leave_trigger_times(g, o, m))
                .map(|k| Trigger { repeat: k > 0, ..Trigger::new(o, Some(s), act.clone()) })
                .collect();
            queue_triggers(g, entries)?; // Angelic Sleuth: another permanent of yours with counters left
        }
    }
    if on && g.perm(m).token {
        crate::impls::karn_cards::ugin_token_left(g, m); // Ugin, the Ineffable: the Spirit's exiled card
        if !hushed {
            crate::impls::t0_zurgo::token_left(g, p, m)?; // t0-zurgo: Zurgo Stormrender, Twilight Drover
        }
    }
    if on {
        crate::impls::t0_anikthea::calix_link_left(g, m)?; // Calix's -3 link
        crate::impls::t12_brago_meren::creature_left(g, m)?; // --- Tiers 1/2 (brago-meren): Dour Port-Mage
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
    to_zone_card_inner(g, m, zone, false);
}

/// `to_zone_card`; with `keep_cmd` a commander is left in the zone it was sent to instead of moving to the command zone
/// (CR 903.9a: the owner may put it there, and a card that returns on its own, as Flickerwisp's does, is left in exile)
fn to_zone_card_inner(g: &mut Game, m: PermId, zone: Zone, keep_cmd: bool) {
    // unearth: "if it would leave the battlefield, exile it instead of putting it anywhere else"
    let zone = if g.perm(m).data.truthy(DataKey::Unearth) { Zone::Exile } else { zone };
    if zone == Zone::Exile {
        crate::impls::bulk_white::note_exile(g, m); // Ennis: cards were put into exile this turn
    }
    let x = g.perm(m);
    if x.token || x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Tokpw)) {
        return; // a token with a card face (the Jace token) just ceases to exist
    }
    let owner = x.orig;
    if let Some(cd) = x.phys {
        if !keep_cmd && g.player(owner).is_commander(cd) {
            g.player_mut(owner).set_cmd_zone(cd, true); // --- scion: Scion as a copy is still the commander
            return;
        }
        let pl = g.player_mut(owner);
        match zone {
            Zone::Exile => pl.exile.push(cd),
            Zone::Hand => pl.hand.push(cd),
            _ => pl.gy.push(cd),
        }
        return;
    }
    let cd = x.cd.unwrap();
    let zone = if zone == Zone::Gy && crate::impls::karn_cards::shuffles_back(g, cd) { Zone::Lib } else { zone }; // Blightsteel Colossus
    if !keep_cmd && g.player(owner).is_commander(cd) {
        if zone == Zone::Hand && crate::impls::cloud::cmd_stays_in_hand(g, owner) {
            g.player_mut(owner).hand.push(cd); // rule 903.9b: the owner may leave the commander in hand (no tax)
            return;
        }
        g.player_mut(owner).set_cmd_zone(cd, true);
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
    if !repl::minus_ok(g, m) {
        return false; // Melira, Solemnity
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
    pl.mana.clear();
    let (perms, lands) = (pl.perms.clone(), pl.lands.clone());
    for m in perms {
        g.perm_mut(m).phased = true;
    }
    for l in lands {
        g.perm_mut(l).tapped = true;
    }
    if !g.control_links.is_empty() {
        crate::engine::control::end_links(g);
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
        || !crate::engine::mana::can_pay_spell(g, q, c, gn, &pips, false)
    {
        return Ok(false);
    }
    g.player_mut(q).hand.retain(|&x| x != c);
    crate::engine::mana::pay_spell(g, q, c, gn, &pips, false)?;
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
    Ok(g.player(q).life_locked
        || cast_teferis_protection(g, q, 9.0)?
        || crate::impls::t45_winota::flare_last_chance(g, q)? // --- Tiers 4/5 (winota): Flare of Fortitude
        || grace_last_chance(g, q)?) // --- card-models-3: Angel's Grace (can't lose this turn)
}

/// Angel's Grace in force for q this turn, or cast now (a Tier 6 seat holding it with {W} open): q can't lose
fn grace_last_chance(g: &mut Game, q: PlayerId) -> Res<bool> {
    use crate::impls::t6_stax::grace_active;
    Ok(grace_active(g, q) || (crate::impls::t6_plans::is_t6(g, q) && crate::ai::t6_table::grace_save(g, q)))
}

// ------------------------------------------------------------------ dying
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
    if cause == "sac" && g.is_creature(m) {
        g.player_mut(p).stat("creatures_sacrificed", 1);
    }
    if matches!(cause, "destroy" | "combat") && !g.auras.is_empty() && cardcode::umbra_save(g, m)? {
        return Ok(());
    }
    // --- Tiers 4/5 (urza-yawgmoth): Welding Jar regenerates an artifact
    if matches!(cause, "destroy" | "combat") && !g.noregen && crate::impls::t45_urza_yawgmoth::welding_jar(g, m)? {
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
    if repl::no_graveyard(g, p) {
        leave(g, m)?; // Rest in Peace: exiled, so it never dies
        if !token {
            to_zone_card(g, m, Zone::Exile);
            crate::impls::t12_kaalia_sythis::exiled_instead(g, m); // --- Tiers 1/2 (kaalia-sythis): Valgavoth
        }
        return exiled_from_bf(g, m, None); // exiled, not dead: Soulherder, Syr Vondam
    }
    if creature {
        g.player_mut(p).creature_died = Some(g.turn_stamp()); // Essenceknit Scholar
        crate::impls::t12_brago_meren::note_died(g, m); // --- Tiers 1/2 (brago-meren): Twilight Diviner's record
        let st = g.turn_stamp();
        g.died_count = if g.died_count.0 == Some(st) { (Some(st), g.died_count.1 + 1) } else { (Some(st), 1) }; // Lagomos
    }
    if creature && repl::hushed(g) {
        // still a death (Python doesn't record it here): common.aura_fall returns Angelic Destiny for it
        let prev = g.dying.replace(m);
        let r = leave(g, m);
        g.dying = prev;
        r?;
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
    // Marchesa, Avacyn and Liesa return the card once whatever the number of times their trigger happens: the second
    // instance finds it already on its way back, so Teysa Karlov and Gandalf the White add nothing here
    if g.marchesa_on && creature && !token {
        cardcode::marchesa_dies(g, p, m)?;
    }
    if !g.t12_batch.is_empty() && creature && !token {
        crate::impls::t12_kaalia_sythis::batch_dies(g, p, m)?; // --- Tiers 1/2 (kaalia-sythis): Avacyn, Liesa
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
    if creature && g.cur_batch.is_some() {
        crate::impls::vondam_elspeth::left_with(g, m, Some(cause))?; // Syr Vondam, if he left earlier in this wipe
    }
    if selfdies {
        let imp = g.registry.get(g.perm(m).cd.unwrap()).copied().unwrap();
        let n = if hooks::leave_hook_is_trigger(&imp, Event::SelfDies) {
            1 + repl::trigger_copies(g, p, "dies", Some(m))
        } else {
            1
        };
        for k in 0..n {
            if imp.runs_at_once(Event::SelfDies) {
                (imp.self_dies.unwrap())(g, m, m, cause)?;
            } else {
                let act = TrigAct::Hook { event: Event::SelfDies, call: Call::Dies { m, cause } };
                queue_triggers(g, vec![Trigger { repeat: k > 0, ..Trigger::new(p, Some(m), act) }])?;
            }
        }
    }
    // death triggers of tagged cards
    for i in 0..g.players.len() {
        let q = PlayerId(i as u8);
        if !g.player(q).alive {
            continue;
        }
        for _ in 0..1 + repl::trigger_copies(g, q, "dies", Some(m)) {
            hand_tag_death(g, q, p, m, creature)?;
        }
    }
    // the engine's own "when this dies" triggers happen once more for each Teysa Karlov, Drivnod or Gandalf the White
    if creature && g.perm(m).data.get(DataKey::Rabid) == Some(&Val::Stamp(g.turn_stamp())) {
        for _ in 0..dies_times(g, p, m) {
            crate::impls::sephfs_cards::rabid_dies(g, g.perm(m).owner)?; // Rabid Attack: draw a card
        }
    }
    if creature && g.perm(m).data.truthy(DataKey::DemonGift) {
        for _ in 0..dies_times(g, p, m) {
            crate::impls::t45_edgar::demon_dies(g, m)?; // --- Tiers 4/5 (edgar): Clavileño's gift
        }
    }
    if creature && g.players.iter().any(|pl| pl.emblems.contains(&"seph")) {
        cardcode::emblem_dies(g)?; // Sephiroth, One-Winged Angel's emblem
    }
    if card_tag_str(g, m, Tag::Fill) == Some("stitcher") {
        for _ in 0..dies_times(g, p, m) {
            mill(g, p, 3)?;
        }
    }
    die_rest(g, m, p, cause, creature)?;
    // die_rest puts the card in the graveyard; the granted trigger must resolve after that move.
    if creature
        && !token
        && let Some(Val::Int(n)) = g.perm(m).data.get(DataKey::RivazExile).cloned()
    {
        let times = 1 + repl::trigger_copies(g, p, "dies", Some(m));
        for _ in 0..n {
            // one grant, once per time its trigger happens: the first is the original, the rest are repeats
            for k in 0..times {
                let t = Trigger { repeat: k > 0, ..Trigger::new(p, Some(m), TrigAct::RivazExile { m }) };
                queue_triggers(g, vec![t])?;
            }
        }
    }
    Ok(())
}

/// how many times a "when a creature dies" trigger of p's happens for m dying: once, and once more for each Teysa Karlov,
/// Drivnod or Gandalf the White of p's
fn dies_times(g: &Game, p: PlayerId, m: PermId) -> i32 {
    1 + repl::trigger_copies(g, p, "dies", Some(m)).max(0)
}

fn card_tag_str(g: &Game, m: PermId, t: Tag) -> Option<&str> {
    g.perm(m).cd.and_then(|c| g.db.get(c).tags.str(t))
}

fn hand_tag_death(g: &mut Game, q: PlayerId, p: PlayerId, m: PermId, creature: bool) -> Res {
    // how many of q's permanents with tag t see this death: the ones still on the battlefield, and the dying creature
    // itself ("this creature or another creature dies"), which has already left by now (`leave` comes first), read as
    // it last was: q's, with its abilities
    let seeing = |g: &Game, t: Tag| -> usize {
        let on_bf = find(g, q, t).iter().filter(|&&a| a != m && !g.perm(a).neutered).count();
        let itself = p == q && card_tag(g, m, t) && !g.perm(m).neutered;
        on_bf + usize::from(itself)
    };
    // Blood Artist: "this creature or another creature dies": one drain per Artist you control (two drained once)
    let artists = if creature { seeing(g, Tag::Bartist) } else { 0 };
    for _ in 0..artists {
        let opps: Vec<PlayerId> = g.opps(q).collect();
        let target = if crate::human::is_human(g, q) {
            crate::human::gaps_t2::pick_player(g, q, "Blood Artist")? // "target player": the person's pick, you included
        } else {
            max_by(&opps, |o| threat(g, q, o))
        };
        if let Some(t) = target {
            lose_life(g, t, 1, Some(q), "drain", None)?; // Blood Artist
            gain(g, q, 1)?;
        }
    }
    // Zulaport Cutthroat: "this creature or another creature you control dies, each opponent loses 1 life and you gain
    // 1 life": once per Zulaport (two drained once), and for its own death too (it had left before it was counted)
    let zulaports = if q == p && creature { seeing(g, Tag::Drain) } else { 0 };
    for _ in 0..zulaports {
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
            // each of their triggers happens once more per Teysa Karlov, Drivnod or Gandalf the White of theirs
            if has(g, q, Tag::Wurmdrain) {
                let src = find(g, q, Tag::Wurmdrain)[0];
                let name = format!("{} loses 2 life", g.player(p).name);
                for _ in 0..dies_times(g, q, m) {
                    if trigger_window(g, q, Some(src), &name, None)? {
                        lose_life(g, p, 2, Some(q), "drain", None)?; // Massacre Wurm
                    }
                }
            }
            for h in find(g, q, Tag::Heir) {
                for _ in 0..dies_times(g, q, m) {
                    repl::put_plus(g, h, 1); // Sephiroth, Planet's Heir
                }
            }
        }
    }
    if card_tag(g, m, Tag::Wurmcoil) {
        for _ in 0..dies_times(g, p, m) {
            if !trigger_window(g, p, Some(m), "two 3/3 Wurms", Some(5.0))? {
                continue;
            }
            // Oracle: colorless Phyrexian Wurm *artifact* creature tokens (they count for Karn, Mirrorworks, a Construct)
            let mut art = PermData::default();
            art.set(DataKey::Artifact, Val::Bool(true));
            let wurm =
                Tokens { color: Some(Colors::NONE), types: vec!["phyrexian", "wurm"], data: art, ..Tokens::new(1, 3) };
            make_tokens(g, p, Tokens { dt: true, ..wurm.clone() })?;
            make_tokens(g, p, Tokens { lifelink: true, ..wurm })?;
        }
    }
    // What comes back from the graveyard (undying, persist, Mikaeus, Ghostform, Ferocity, Perigee Beckoner) returns the
    // card once: a second instance of the trigger finds it gone, so the trigger copiers add nothing below
    let x = g.perm(m).clone();
    if x.token {
        return crate::impls::t45_urza_yawgmoth::pest_dies(g, &x); // --- Tiers 4/5 (urza-yawgmoth): Blight Mound
    }
    let cd = x.cd.unwrap();
    let db = g.db.clone();
    let d = db.get(cd);
    if d.bomb != 0 && x.orig == p && !g.player(p).removed_bombs.contains(&cd) {
        g.player_mut(p).removed_bombs.push(cd);
    }
    let is_cmd_card = g.player(p).is_commander(cd);
    // undying from Mikaeus
    let person = crate::human::is_human(g, p);
    if has(g, p, Tag::Mikaeus)
        && d.creature
        && g.is_creature(m) // not a returned Enduring Tenacity / Vitality (an enchantment, `DataKey::NotCreature`)
        && !d.tag(Tag::Human)
        && !d.tag(Tag::Mikaeus)
        && (person || !x.undying) // (a person's creature may come back again: undying only asks that it has no +1/+1 counter now)
        && x.plus <= 0
        && x.orig == p
        && !is_cmd_card
        && trigger_window(g, p, Some(m), "undying", None)?
    {
        let n = enter(g, p, cd, Enter { undying: true, ..Enter::default() })?;
        if !person {
            g.perm_mut(n).plus = 1; // (for a person the enter step's counters stay: Triskelion comes back with 3 + 1)
        }
        g.perm_mut(n).undying = true;
        g.player_mut(p).stat("undying", 1);
        return Ok(());
    }
    let (undying, persist) = (d.has_kw("undying"), d.has_kw("persist"));
    if (undying || persist) && x.orig == p && !is_cmd_card && !repl::no_graveyard(g, p) && returns_left(g, cd) {
        if undying && x.plus <= 0 && trigger_window(g, p, Some(m), "undying", None)? {
            enter(g, p, cd, Enter { undying: true, ..Enter::default() })?; // back with a +1/+1 counter
            g.player_mut(p).stat("undying", 1);
            return Ok(());
        }
        if persist && x.plus >= 0 && trigger_window(g, p, Some(m), "persist", None)? {
            // back with a -1/-1 counter (none under Melira)
            let plus = if repl::melira(g, p) { 0 } else { -1 };
            enter(g, p, cd, Enter { plus, ..Enter::default() })?;
            g.player_mut(p).stat("persist", 1);
            return Ok(());
        }
    }
    if let Some(Val::Player(q)) = x.data.get(DataKey::Ghostform).cloned()
        && returns_left(g, cd)
        && !repl::no_graveyard(g, p)
        && crate::impls::sephfs_cards::ghostform_return(g, m, q)?
    {
        return Ok(()); // Kaya's Ghostform: it returns to the battlefield under its Aura's controller
    }
    if x.data.truthy(DataKey::Ferocity)
        && returns_left(g, cd)
        && !repl::no_graveyard(g, p)
        && trigger_window(g, p, Some(m), "Ferocity of the Hunt: return it to the battlefield tapped", None)?
    {
        let n = enter(g, x.orig, cd, Enter::default())?;
        g.perm_mut(n).tapped = true;
        return Ok(());
    }
    if !x.eot_kw.is_empty() && crate::impls::sheoldred::returns_when_dies(g, &x, cause)? {
        return Ok(()); // Perigee Beckoner: it returns to the battlefield
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
    exile_perm_inner(g, m, false)
}

/// `exile_perm`, but a commander stays in exile (CR 903.9a: the owner may leave it there): for an exile that returns the
/// card by itself (Flickerwisp's own commander, back at the end step; Eerie Interlude's `interlude_exile` does the same by hand)
pub fn exile_perm_keep_cmd(g: &mut Game, m: PermId) -> Res {
    exile_perm_inner(g, m, true)
}

fn exile_perm_inner(g: &mut Game, m: PermId, keep_cmd: bool) -> Res {
    let prev = g.exiling.replace(m); // Kaya's Ghostform sees the exile
    let r = leave(g, m);
    g.exiling = prev;
    r?;
    if let Some(Val::Player(q)) = g.perm(m).data.get(DataKey::Ghostform).cloned() {
        let mark = g.enter_no; // what entered after this was not on the battlefield when m left
        if crate::impls::sephfs_cards::ghostform_return(g, m, q)? {
            return exiled_from_bf_upto(g, m, Some(mark)); // it came back, but it was exiled first
        }
    }
    to_zone_card_inner(g, m, Zone::Exile, keep_cmd);
    exiled_from_bf(g, m, None)
}

/// m was put into exile from the battlefield (any way: an exile, a blink, a token that is gone for good, Rest in Peace's
/// "exiled instead of dying"): "whenever a creature is exiled" (Soulherder), "dies or is put into exile" (Syr Vondam).
/// Call it once, after `leave`, and after the exiled card is back for a blink (`back`: what it came back as), so the
/// triggers wait for its enters triggers as before. Leaves-the-battlefield abilities look back in time (CR 603.10a):
/// `back` and what entered with it never saw m leave, so they hear nothing. The permanent's own trigger
/// (`self_exiled`) is queued here, as `die_triggers` queues `self_dies`, since `leave` took it out of `g.hooks`.
pub fn exiled_from_bf(g: &mut Game, m: PermId, back: Option<PermId>) -> Res {
    exiled_from_bf_upto(g, m, back.map(|n| g.perm(n).born.saturating_sub(1)))
}

/// `exiled_from_bf`, with the bound given as the entry number (`Game::enter_no`) of the last object that was on the
/// battlefield when m left: only listeners born by then hear it (None: all of them)
fn exiled_from_bf_upto(g: &mut Game, m: PermId, last_born: Option<u32>) -> Res {
    if !g.hooks.is_empty() {
        hooks::fire_trigger_where(
            g,
            Event::ExiledFromBf,
            Call::Leaves { m },
            &|g, s| last_born.is_none_or(|b| g.perm(s).born <= b),
            false,
        )?;
    }
    if g.cur_batch.is_some() && g.is_creature(m) {
        crate::impls::vondam_elspeth::left_with(g, m, None)?; // Syr Vondam, if he left earlier in this wipe
    }
    let imp = g.perm(m).cd.and_then(|c| g.registry.get(c).copied()).filter(|i| i.self_exiled.is_some());
    if let Some(imp) = imp {
        let o = g.perm(m).owner;
        // put into exile from the battlefield: a leaving that is not a death
        let times = if hooks::leave_hook_is_trigger(&imp, Event::SelfExiled) {
            1 + hooks::total_trigger_copies(g, o, "leaves", Some(m)).max(0) as u32
        } else {
            1
        };
        let mut entries = vec![];
        for k in 0..times {
            if imp.runs_at_once(Event::SelfExiled) {
                (imp.self_exiled.unwrap())(g, m, m)?;
            } else {
                let act = TrigAct::Hook { event: Event::SelfExiled, call: Call::Leaves { m } };
                entries.push(Trigger { repeat: k > 0, ..Trigger::new(o, Some(m), act) });
            }
        }
        queue_triggers(g, entries)?;
    }
    Ok(())
}

pub fn bounce(g: &mut Game, m: PermId) -> Res {
    let note = crate::impls::bulk_white::bounce_note(g, m); // Stormfront Riders sees creatures return to hand
    leave(g, m)?;
    to_zone_card(g, m, Zone::Hand);
    crate::impls::bulk_white::bounced(g, m, note)?;
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
    pub rivaz_exile: u32,
    pub undying: bool,
    pub plus: i32,
    /// the copy exceptions of a token copy ("except it's a 0/1 white Illusion"); a clone's come from its `enter_copy`
    pub exc: Option<CopyExc>,
    /// it is a token from the start (a token copy): whatever takes it away on entering (an Aura with nothing to enchant)
    /// leaves no card behind
    pub token: bool,
    /// the X the permanent spell was cast with (or a copy of that spell has): kept on the permanent as `DataKey::CastX`,
    /// where its enter trigger reads it (The Meathook Massacre). 0 for a permanent that did not enter from a cast
    pub x: i32,
    /// it enters as a noncreature (Enduring Tenacity / Vitality's return): marked `DataKey::NotCreature` before any
    /// "a creature enters" trigger looks at it
    pub not_creature: bool,
}

impl Default for Enter {
    fn default() -> Enter {
        Enter {
            orig: None,
            sick: true,
            was_cast: false,
            rivaz_exile: 0,
            undying: false,
            plus: 0,
            exc: None,
            token: false,
            x: 0,
            not_creature: false,
        }
    }
}

/// Card cd enters the battlefield under p's control: its as-it-enters effects, then its enter triggers (on the
/// stack once the spell resolving has finished). Python's `enter`.
pub fn enter(g: &mut Game, p: PlayerId, cd: CardId, how: Enter) -> Res<PermId> {
    g.tick()?;
    let from_gy = crate::impls::t12_brago_meren::from_gy(g, cd); // --- Tiers 1/2 (brago-meren): Twilight Diviner
    let mut cd = cd;
    if g.db.get(cd).tag(Tag::Moxd) && !how.was_cast {
        // Mox Diamond put onto the battlefield (Urza's Saga): the same replacement as when cast
        let lands: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
        if lands.is_empty() {
            g.player_mut(p).gy.push(cd);
            crate::glog!(g, "    Mox Diamond goes to the graveyard (no land to discard)");
            return Ok(g.new_perm(p, Some(cd), "Token", 0, 0)); // never on the battlefield
        }
        let x = min_by(&lands, |c| mox_discard_key(g, p, c)).unwrap();
        let pl = g.player_mut(p);
        let i = pl.hand.iter().position(|&c| c == x).unwrap();
        pl.hand.remove(i);
        pl.gy.push(x);
    }
    // "enters as a copy of ...": a replacement effect, applied as the permanent is created (no 0/0 stand-in, no
    // enters triggers of the clone itself: the copied card's are the ones that fire)
    let mut phys = None;
    let mut exc = how.exc;
    let copy = if let Some(f) = g.registry.get(cd).and_then(|i| i.enter_copy) {
        f(g, p, cd)
    } else if g.db.get(cd).tag(Tag::Clone) {
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
                    !g.player(y.owner).is_commander(c)
                        && (g.is_creature(x) || d.types.has(crate::cards::Types::ARTIFACT))
                        && !d.tag(Tag::Clone)
                })
            })
            .collect();
        max_by(&cands, |x| pval(g, x)).map(|b| CopyOf { cd: g.perm(b).cd.unwrap(), exc: CopyExc::default() })
    } else {
        None
    };
    if let Some(c) = copy {
        phys = Some(cd);
        cd = c.cd;
        exc = exc.or(Some(c.exc));
        // --- urdragon: the copied card may itself say "as this enters, you may have it become a copy" (Sculpting Steel
        // copying Cursed Mirror; a token copy of Cursed Mirror): that replacement applies too, in addition
        if let Some(c2) = g.registry.get(cd).and_then(|i| i.enter_copy).and_then(|f| f(g, p, cd)) {
            cd = c2.cd;
            let (a, b) = (exc.unwrap_or_default(), c2.exc);
            exc = Some(CopyExc { haste: a.haste || b.haste, revert_to: b.revert_to.or(a.revert_to), ..a });
        }
    }
    let m = g.new_perm(p, Some(cd), "Token", 0, 0);
    g.perm_mut(m).token = how.token;
    g.enter_no += 1;
    let d = g.db.get(cd);
    let (haste, has_dsl, loyalty) = (d.tag(Tag::Haste), d.has_dsl(), d.start_loyalty);
    let born = g.enter_no;
    // Spark Double's copy: an additional +1/+1 counter if it's a creature, an additional loyalty counter if it's a
    // planeswalker
    let spark = crate::impls::common::spark_bonus(g, phys);
    let (spark_plus, loyalty) = match loyalty {
        Some(l) => (0, Some(l + spark)),
        None => (if d.creature { spark } else { 0 }, None),
    };
    let plus = how.plus + spark_plus + crate::impls::green_cards::enters_with(g, p, cd, how.was_cast) // Wildgrowth Archaic
        + crate::impls::karn_engines::hangar_counters(g, cd, how.was_cast, how.x); // Hangarback Walker: X counters
    // the enter step's counters (614.1c): Doubling Season doubles a planeswalker's loyalty (once: the old `etb` hook
    // in impls/t3.rs doubled it again), undying's counter and the counters it enters with; Scales, Vorinclex, Solemnity
    let (v_loy, v_undying, v_plus) = repl::enter_counters(g, p, m, loyalty, plus);
    let x = g.perm_mut(m);
    x.orig = how.orig.unwrap_or(p);
    x.sick = how.sick && !haste;
    x.phys = phys;
    x.born = born;
    if how.rivaz_exile > 0 {
        x.data.set(DataKey::RivazExile, Val::Int(how.rivaz_exile as i64));
    }
    if how.x > 0 {
        x.data.set(DataKey::CastX, Val::Int(how.x as i64));
    }
    if how.not_creature {
        x.data.set(DataKey::NotCreature, Val::Bool(true));
    }
    if loyalty.is_some() {
        x.loyalty = Some(v_loy); // Doubling Season: twice the loyalty; Vorinclex
    }
    if how.undying {
        x.plus = v_undying; // returns with its +1/+1 counter (so it survives -X/-X effects)
        x.undying = true;
    }
    if plus != 0 {
        x.plus = v_plus; // enters with counters (persist: -1)
    }
    if let Some(e) = exc {
        x.exc = e;
        if let Some((pw, th)) = e.pt {
            (x.pow, x.tgh) = (pw, th);
        }
        x.fly |= e.fly;
        x.sick = x.sick && !e.haste;
        if let Some(s) = e.add_subtype.or(e.only_subtype) {
            x.ttypes.push(s);
        }
        if e.vanishing > 0 {
            x.data.set(DataKey::Vanish, crate::state::Val::Int(e.vanishing as i64)); // --- card-models-2
        }
    }
    x.on_bf = true;
    if has_dsl {
        g.dsl_on = true;
    }
    g.player_mut(p).perms.push(m);
    g.bf_ver += 1;
    if from_gy {
        crate::impls::t12_brago_meren::mark_from_gy(g, m); // --- Tiers 1/2 (brago-meren)
    }
    if g.registry.get(cd).is_some_and(|i| i.live()) {
        g.hooks.push(m);
    }
    // a replacement effect: it is already tapped as it enters, before any enters trigger sees it
    if repl::enters_tapped(g, p, Some(m)) {
        g.perm_mut(m).tapped = true;
    }
    cardcode::as_enters(g, p, m)?; // naming a creature type
    let st = g.turn_stamp();
    g.entered.retain(|e| e.0 == st);
    g.entered.push((st, m));
    crate::impls::t45_krenko::note_entered(g, p, &[m]); // --- Tiers 4/5 (krenko): Hobgoblin Bandit Lord
    let creature = g.db.get(cd).creature && !how.not_creature;
    // (Hushbringer silences what the creature entering triggers, not how it enters: `enter_silenced`; there is no enters
    // trigger for Tishana's Tidebinder to counter then)
    let quiet = g.skip_etb || (creature && !repl::hushed(g) && cardcode::tidebinder_response(g, p, m)?);
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

/// which land Mox Diamond discards (the lowest key): the one making the fewest colours, a tapped land first.
/// A Tier 6 player also keeps the only source of a colour its hand needs (a 2-land hand lost its Plains, g6) and the
/// lands that make two mana or do a job (Ancient Tomb, City of Traitors, Gaea's Cradle).
pub fn mox_discard_key(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let t = &g.db.get(c).tags;
    let mine = t.str(Tag::C).unwrap_or("");
    let mut key = mine.chars().count() as f64 * 2.0 - if t.has(Tag::T) { 1.0 } else { 0.0 };
    if !crate::impls::t6_plans::is_t6(g, p) {
        return key;
    }
    if matches!(g.db.get(c).name.as_ref(), "Ancient Tomb" | "City of Traitors" | "Gaea's Cradle" | "Gemstone Caverns") {
        key += 20.0;
    }
    let pl = g.player(p);
    let others: Vec<CardId> = pl
        .hand
        .iter()
        .copied()
        .filter(|&x| x != c && g.db.get(x).land)
        .chain(pl.lands.iter().map(|&l| g.perm(l).card()))
        .collect();
    let covered = |ch: char| others.iter().any(|&o| g.db.get(o).tags.str(Tag::C).is_some_and(|s| s.contains(ch)));
    let needed = |ch: char| pl.hand.iter().any(|&x| !g.db.get(x).land && g.db.get(x).pips.contains(ch));
    for ch in mine.chars() {
        if !covered(ch) {
            key += if needed(ch) { 6.0 } else { 1.5 };
        }
    }
    key
}

/// a token that's a copy of card cd (Scute Swarm, Kiki-Jiki, Helm of the Host ...)
pub fn enter_token_copy(g: &mut Game, p: PlayerId, cd: CardId) -> Res<Option<PermId>> {
    enter_token_copy_inner(g, p, cd, None, 0)
}

/// the token a copy of a permanent spell becomes: it keeps the spell's X (CR 707.10)
pub fn enter_token_copy_x(g: &mut Game, p: PlayerId, cd: CardId, x: i32) -> Res<Option<PermId>> {
    enter_token_copy_inner(g, p, cd, None, x)
}

/// a token copy with exceptions ("except it's a 0/1 white Illusion", "it's an artifact in addition"): they apply as it
/// enters, so its own enters triggers see them
pub fn enter_token_copy_with(g: &mut Game, p: PlayerId, cd: CardId, exc: Option<CopyExc>) -> Res<Option<PermId>> {
    enter_token_copy_inner(g, p, cd, exc, 0)
}

/// Data keys that define a token made by `make_tokens` (abilities or characteristics its creator gave it in `spec.data`,
/// CR 707.2): keywords and markers (`Kws`, `Manatok`, `Construct`, `WeirdToken`, `KujaWiz`), "legendary" (`Legendary`),
/// "is an artifact" (`Artifact`: how Hangarback Walker's Thopters, Urza's Constructs, Sai's Thopters and others are
/// artifacts without "artifact" in `ttypes`; copied only as `Val::Bool`, since a `Val::Stamp` is a this-turn effect on a
/// permanent), and the named-token flags (`Indestr`: Marit Lage and Kaldra, `Kaldra`, `MordDog`, `Voja`). Every other key is
/// treated as state (counters, marks, effects) and is never copied. A token-defining key that a new card stores in `data`
/// must be added here, or a copy of its token loses it. Limit: `Indestr` is also a one-shot grant on any permanent
/// (`rules.rs`, `partials.rs`), which a copy of that permanent would inherit.
pub const COPIABLE_TOKEN_DATA: [DataKey; 11] = [
    DataKey::Kws,
    DataKey::Manatok,
    DataKey::Construct,
    DataKey::WeirdToken,
    DataKey::KujaWiz,
    DataKey::Legendary,
    DataKey::Artifact,
    DataKey::Indestr,
    DataKey::Kaldra,
    DataKey::MordDog,
    DataKey::Voja,
];

/// p creates a token that's a copy of permanent `src` (CR 707.2-707.3), once for each token a doubler makes
/// (`repl::tokens`), and returns the tokens that stayed on the battlefield:
/// - a card-backed permanent: its card, with the copy exceptions it was itself made with (`exc`, a copiable value under
///   707.9b: a Helm of the Host token stays nonlegendary, a Cursed Mirror copy keeps "has haste") except `revert_to`, the
///   duration of the Mirror's own effect;
/// - a face-down permanent: a 2/2 colourless creature with no name, types or abilities (708.2);
/// - a token with no card record: the characteristics `make_tokens` gave it (name, base power and toughness, colours,
///   types, fly, deathtouch, lifelink, vigilance, warrior, army, noatk, and the `COPIABLE_TOKEN_DATA` keys).
///
/// Replacement effects of creating a token (614) apply to every branch alike, as they do in `make_tokens`: with Divine
/// Visitation out a creature token is a 4/4 white flying vigilance Angel instead, and with Jinnie Fay out a token whose
/// power is under 2 is a 2/2 with haste. Such a token is not a copy (nothing of the source is copied onto it); it is made
/// by `make_tokens` alone, which also applies the doublers.
///
/// Never copied: counters (`plus`, `DataKey::Counters`...), damage, tapped, sick (the token enters sick), attachments,
/// `eot_pt`/`eot_kw`, phasing, `neutered`, `temp`, `undying`, loyalty, and every other data key.
/// `marks` are set on each new token (the ones that stayed on the battlefield) before any of its triggers resolve and
/// before the state-based actions, whatever it turned out to be. (A hook that runs inside `enter` itself, Thassa's
/// Oracle's `at_once` enters, runs before the marks are set: nothing reads them there.)
/// At top level (`g.resolving == 0` on entry) the state-based actions are checked and only then are the queued triggers
/// flushed (704.3 before 603.3: the legend rule removes a legendary copy before its enters trigger resolves; the trigger
/// still resolves). Nested inside another resolution, it does neither: the outer resolution does both.
pub fn token_copy_of(g: &mut Game, p: PlayerId, src: PermId, marks: &[(DataKey, Val)]) -> Res<Vec<PermId>> {
    let top = g.resolving == 0;
    g.resolving += 1;
    let r = (|| -> Res<Vec<PermId>> {
        let x = g.perm(src).clone(); // last known information: src may have left in response
        let face_down = x.cd.is_some() && x.data.truthy(DataKey::Facedown);
        let creature = x.cd.is_none() || face_down || g.is_creature(src);
        let (base_pow, base_tgh) = match x.cd {
            _ if face_down => (2, 2),
            Some(cd) => (g.db.get(cd).pow, g.db.get(cd).tgh),
            None => (x.pow, x.tgh),
        };
        let replaced =
            (creature && crate::impls::t0_zurgo::divine_visitation(g, p)) || (has(g, p, Tag::Jinnie) && base_pow < 2);
        let made = if replaced {
            make_tokens(g, p, Tokens { tgh: Some(base_tgh), ..Tokens::new(1, base_pow) })? // not a copy
        } else if face_down || x.cd.is_none() {
            let mut spec = if face_down {
                Tokens { color: Some(Colors::NONE), ..Tokens::new(1, 2) }
            } else {
                Tokens {
                    fly: x.fly,
                    warrior: x.warrior,
                    lifelink: x.lifelink,
                    dt: x.dt,
                    color: Some(x.colors),
                    types: x.ttypes.clone(),
                    ..Tokens { tgh: Some(x.tgh), ..Tokens::new(1, x.pow) }
                }
            };
            if !face_down {
                for k in COPIABLE_TOKEN_DATA {
                    // an artifact stamp is an effect of this turn on the permanent, not what the token is
                    if let Some(v) = x.data.get(k).filter(|v| k != DataKey::Artifact || matches!(v, Val::Bool(_))) {
                        spec.data.set(k, v.clone());
                    }
                }
            }
            let made = make_tokens(g, p, spec)?; // doublers, TOKEN_CAP
            if !face_down {
                for &t in &made {
                    let y = g.perm_mut(t);
                    y.name = x.name;
                    y.vig |= x.vig;
                    y.army = x.army;
                    y.noatk = x.noatk;
                }
            }
            made
        } else {
            let exc = CopyExc { revert_to: None, ..x.exc };
            let n = repl::tokens(g, p, 1);
            let mut made = vec![];
            for _ in 0..n {
                let Some(t) = enter_token_copy_with(g, p, x.cd.unwrap(), (exc != CopyExc::default()).then_some(exc))?
                else {
                    break; // TOKEN_CAP
                };
                if g.perm(t).on_bf {
                    made.push(t);
                }
            }
            made
        };
        for &t in &made {
            for (k, v) in marks {
                g.perm_mut(t).data.set(*k, v.clone());
            }
        }
        Ok(made)
    })();
    g.resolving -= 1;
    let made = r?;
    if top {
        crate::engine::life::check_state(g)?;
        if !g.trig_queue.is_empty() && !g.over {
            crate::engine::stack::flush_triggers(g)?;
        }
    }
    Ok(made)
}

fn enter_token_copy_inner(g: &mut Game, p: PlayerId, cd: CardId, exc: Option<CopyExc>, x: i32) -> Res<Option<PermId>> {
    let have = g.player(p).perms.iter().filter(|&&m| g.perm(m).token).count();
    if have >= TOKEN_CAP {
        return Ok(None);
    }
    // --- Tiers 4/5 (urdragon): its enter triggers wait until it is a token ("another nontoken Dragon": Miirym)
    g.resolving += 1;
    let r = enter(g, p, cd, Enter { exc, token: true, x, ..Enter::default() });
    if let Ok(m) = r {
        g.perm_mut(m).token = true;
    }
    g.resolving -= 1;
    let m = r?;
    note_tokens(g, p, 1);
    if g.resolving == 0 && !g.trig_queue.is_empty() && !g.over {
        crate::engine::stack::flush_triggers(g)?;
    }
    Ok(Some(m))
}

/// the tag-driven enter effects: as it enters (etb_static), then its triggered ones on the stack (etb_once)
pub fn do_etb(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let cd = g.perm(m).cd.unwrap();
    if g.db.get(cd).has_dsl() {
        return Ok(()); // the ability language handles its abilities
    }
    etb_static(g, p, m)?; // as it enters, not triggered: an opponent's Elesh Norn leaves it alone
    if hooks::enter_silenced(g, p, Some(m)) || !etb_triggers(g, cd) {
        return Ok(());
    }
    let (name, imp) = etb_text(g, cd);
    let mut reps = 1;
    if !g.hooks.is_empty() {
        reps += repl::trigger_copies(g, p, "etb", Some(m)); // Panharmonicon, Elesh Norn
    }
    for k in 0..reps {
        if g.over {
            return Ok(());
        }
        let t = Trigger {
            name: Some(name),
            known: true,
            imp: Some(imp),
            repeat: k > 0,
            ..Trigger::new(p, Some(m), TrigAct::EtbOnce { p, m })
        };
        queue_triggers(g, vec![t])?;
    }
    Ok(())
}

/// enters-the-battlefield tags that are triggered abilities (the rest: as it enters, or static)
pub const ETB_TRIGGER_TAGS: [Tag; 19] = [
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
    Tag::Endraze,
];
pub const ETB_SCRYFALL_TAGS: [Tag; 4] = [Tag::Tut, Tag::Treas, Tag::Drainetb, Tag::Edictetb];
/// removal kinds that are Auras locking the target down
pub const LOCK_KINDS: [&str; 5] = ["arrest", "pacify", "encrust", "kasmina", "coerce"];
/// removal kinds that are Auras changing what the enchanted permanent is (Kenrith's Transformation, Darksteel Mutation,
/// Song of the Dryads, Imprisoned in the Moon, Reprobation): static effects of the Aura, not enters triggers
pub const STATIC_AURA_KINDS: [&str; 5] = ["elk", "mutate", "forest", "moon", "coward"];

/// is a `rem` kind with `etb` a static effect of an Aura (all the lock and static Aura kinds are Auras; attached as
/// it resolves), not a triggered ability?
pub fn rem_is_static(kind: &str) -> bool {
    LOCK_KINDS.contains(&kind) || STATIC_AURA_KINDS.contains(&kind)
}

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
        && !rem_is_static(r)
    {
        return true;
    }
    g.db.get(cd).source_scryfall() && ETB_SCRYFALL_TAGS.iter().any(|&k| t.has(k))
}

/// what an enters trigger is called on the stack, and how much it matters
pub fn etb_text(g: &Game, cd: CardId) -> (Sym, f64) {
    let t = &g.db.get(cd).tags;
    if t.has(Tag::Rem) && t.has(Tag::Etb) && !t.str(Tag::Rem).is_some_and(rem_is_static) {
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
        && rem_is_static(r)
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
        repl::put_plus(g, m, 2 * toks.len() as i32);
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
    if t.has(Tag::Endraze) {
        // End-Raze Forerunners: other creatures you control get +2/+2 and gain vigilance and trample until end of turn:
        // the ones there as it resolves (CR 611.2c), not the source and not a later entrant
        let others: Vec<PermId> =
            g.player(p).perms.iter().copied().filter(|&x| x != m && g.is_creature(x) && !g.perm(x).phased).collect();
        for x in others {
            let y = g.perm_mut(x);
            y.eot_pt.0 += 2;
            y.eot_pt.1 += 2;
            crate::impls::partials::eot_kw(g, x, "vigilance");
            crate::impls::partials::eot_kw(g, x, "trample");
        }
    }
    if t.has(Tag::Recruit) {
        crate::engine::tutors::tutor(g, p, "cre2", "Imperial Recruiter")?; // Imperial Recruiter: creature with power 2 or less
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
            .map(|c| g.db.get(c).devotion_to("B") as i32)
            .ssum();
        let dev = dev.min(10);
        for &q in &opps {
            lose_life(g, q, dev, Some(p), "drain", None)?;
        }
        gain(g, p, dev * opps.len() as i32)?;
    }
    if t.has(Tag::Wurm) {
        for &q in &opps {
            for x in g.player(q).perms.clone() {
                if g.is_creature(x) && !g.perm(x).phased {
                    g.perm_mut(x).eot_pt.0 -= 2;
                    g.perm_mut(x).eot_pt.1 -= 2;
                    if etgh(g, x) <= 0 {
                        die(g, x, "sba")?; // the 2 life is its dies trigger ('wurmdrain', in die_rest)
                    }
                }
            }
        }
    }
    if t.has(Tag::Rsd) {
        crate::engine::tutors::tutor(g, p, "any", "Rune-Scarred Demon")?;
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
        if crate::human::is_human(g, p) {
            crate::human::choices::deal_damage_from(g, p, 1, "Orcish Bowmasters", "triggers", Some(m))?;
        } else {
            let ll = crate::engine::life::has_lifelink(g, m);
            let mut dealt = 0;
            let x1: Vec<PermId> = opps
                .iter()
                .flat_map(|&q| g.player(q).perms.iter().copied())
                .filter(|&x| g.is_creature(x) && etgh(g, x) <= 1 && pval(g, x) >= 2.0 && !untargetable(g, x))
                .collect();
            if let Some(b) = max_by(&x1, |x| pval(g, x)) {
                if crate::engine::ward::pay_ward(g, p, b)? {
                    dealt = crate::engine::life::creature_damage(g, p, Some(m), b, 1);
                    die(g, b, "destroy")?;
                }
            } else if let Some(q) = max_by(&opps, |o| threat(g, p, o)) {
                dealt = crate::engine::life::lose_life_dealt(g, q, 1, Some(p), "triggers", None)?;
            }
            crate::engine::life::lifelink(g, m, ll, dealt)?;
        }
        amass(g, p, 1)?;
    }
    if let Some(r) = t.str(Tag::Rem)
        && t.has(Tag::Etb)
        && !rem_is_static(r)
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
        sun_titan(g, p, m)?;
    }
    if t.has(Tag::Dualcaster) {
        // copy a spell: approximated as copying your last instant/sorcery (no card carries the tag now: Dualcaster Mage
        // copies through impls/threats.rs and `cast::copy_spell`). The copy is yours, so your magecraft triggers.
        let last = g.player(p).gy.iter().rev().copied().find(|&x| g.db.get(x).instant || g.db.get(x).sorcery);
        if let Some(last) = last {
            crate::engine::cast::magecraft(g, p, Some(last), true)?;
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
            let by = g.db.get(cd).name.to_string();
            let k = crate::ai::decks::tut_kind(g, p, k);
            crate::engine::tutors::tutor(g, p, &k, &by)?;
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
            // the titania deck leaves its fetch lands in the graveyard, where Titania returns them
            let keep_fetch = crate::impls::titania::is_deck(g.player(p).key);
            if let Some(i) = top.iter().position(|&c| g.db.get(c).land && !(keep_fetch && g.db.get(c).tag(Tag::F))) {
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
    let q = if q.is_none() && crate::human::is_human(g, p) {
        Some(crate::human::choices::target_opponent(g, p, "Archon of Cruelty")?) // the person picks the opponent
    } else {
        q
    };
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
    edict_nth(g, q, least_power, 1, 1)
}

/// `edict` for the i-th of n sacrifices (a person's prompt says "(1 of 2)" when n is more than one)
pub fn edict_nth(g: &mut Game, q: PlayerId, least_power: bool, i: usize, n: usize) -> Res {
    let mut cr: Vec<PermId> =
        g.player(q).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased && !no_sac(g, m)).collect();
    if !cr.is_empty() && least_power {
        let lo = cr.iter().map(|&m| epow(g, m)).min().unwrap();
        cr.retain(|&m| epow(g, m) == lo);
    }
    if crate::human::is_human(g, q) && !cr.is_empty() {
        let why = match n {
            1 => "Sacrifice a creature (edict)".to_string(),
            _ => format!("Sacrifice a creature (edict) ({i} of {n})"),
        };
        return crate::human::choices::sacrifice_creature(g, q, &cr, &why);
    }
    if let Some(m) = crate::ai::values::read_as(g, q, || {
        min_by(&cr, |x| sac_worth(g, x) + crate::ai::table_fixes::steal_cost(g, q, x)) // --- ai-table-fixes: AI_SACSTEAL
    }) {
        let name = g.perm(m).name.to_string();
        die(g, m, "sac")?;
        if !g.perm(m).on_bf {
            crate::glog!(g, "    {} sacrifices {} (edict)", g.player(q).name, name);
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ lands
/// the basic land cards (Wastes is basic: a colourless deck's Wayfarer's Bauble, Solemn Simulacrum and Burnished Hart find it)
pub const BASIC_LANDS: [&str; 6] = ["Forest", "Island", "Plains", "Swamp", "Mountain", "Wastes"];

fn basics_in(g: &Game, p: PlayerId) -> Vec<CardId> {
    searchable(g, p).into_iter().filter(|&c| BASIC_LANDS.contains(&&*g.db.get(c).name)).collect()
}

/// search for n basic lands and put them onto the battlefield (tapped or not)
pub fn land_ramp(g: &mut Game, p: PlayerId, n: u32, tapped: bool) -> Res {
    let human = crate::human::is_human(g, p);
    for _ in 0..n {
        if human {
            let prompt =
                format!("Search for a basic land to put onto the battlefield{}", if tapped { " tapped" } else { "" });
            let Some(c) = crate::human::choices::pick_basic(g, p, &prompt)? else { return Ok(()) };
            g.add_land_entering(p, c, tapped);
            landfall(g, p)?;
            continue;
        }
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

/// the lands that just entered under p's control: the land-enters event for card code (`Event::LandEnters`), and their
/// compiled enter triggers (a Temple's scry, a gain land's life, a surveil land's surveil), which go on the stack
/// through the ability language (Phase 3: one path; the engine's own `land_etb_fx` is gone). Returns the lands it just
/// swept: the ones that entered.
pub fn lands_entered(g: &mut Game, p: PlayerId) -> Res<Vec<PermId>> {
    let mut entered = vec![];
    for l in g.player(p).lands.clone() {
        if g.perm(l).data.truthy(DataKey::In) {
            continue;
        }
        g.perm_mut(l).data.set(DataKey::In, Val::Bool(true));
        entered.push(l);
        // the land-enters event (Yoshimaru): once per land, whatever put it onto the battlefield; before the Blood
        // Moon check, which takes the land's own abilities but not what other permanents see
        if !g.hooks.is_empty() && !g.over {
            fire_trigger(g, Event::LandEnters, Call::Etb { p, m: l })?;
        }
        // Blood Moon: a plain Mountain has no enters trigger; nor has a card played as its land face (the compiled
        // trigger is the front face's: Witch Enchanter's destroy)
        if crate::engine::mana::moon_land(g, l) || g.perm(l).data.truthy(DataKey::LandFace) {
            continue;
        }
        let etb =
            g.db.get(g.perm(l).card())
                .abilities
                .iter()
                .any(|a| matches!(a, crate::dsl::model::Ability::Triggered { event, .. } if event == "etb"));
        if etb && !g.over {
            crate::dsl::fire(g, "etb", crate::dsl::Fired { perm: Some(l), owner: Some(p), ..Default::default() })?;
        }
    }
    Ok(entered)
}

/// A land entered under p's control: landfall. The land the sweep found is the cause of the event for the doublers that
/// care what entered (Gandalf the White: a legendary or artifact land; Panharmonicon: a creature or artifact land). One
/// call is one land; if the sweep found none (a state check swept it first), no doubler adds anything.
pub fn landfall(g: &mut Game, p: PlayerId) -> Res {
    let land = lands_entered(g, p)?.first().copied();
    let inline =
        (has(g, p, Tag::Landfall2) || g.player(p).lands.iter().any(|&l| card_tag(g, l, Tag::Fotd))).then_some(p);
    let copies = hooks::listener_copies_for(g, Event::Landfall, "landfall", land, inline);
    let passes = 1 + copies.iter().copied().max().unwrap_or(0);
    // one landfall event: no queued trigger of it resolves before every pass has triggered (CR 603.3), so what an
    // earlier pass makes (a Scute Swarm's copy) is not a listener of a later one. Held only when there is a later pass.
    let hold = passes > 1;
    if hold {
        g.resolving += 1;
    }
    let mut r = Ok(());
    for k in 0..passes {
        r = landfall_once(g, p, k, &copies, land); // each pass after the first is a repeat of the landfall triggers
        if r.is_err() || g.over {
            break;
        }
    }
    if hold {
        g.resolving -= 1;
    }
    r?;
    if hold && g.resolving == 0 && !g.trig_queue.is_empty() && !g.over {
        crate::engine::stack::flush_triggers(g)?;
    }
    Ok(())
}

fn landfall_once(g: &mut Game, p: PlayerId, pass: i32, copies: &[i32], land: Option<PermId>) -> Res {
    let repeat = pass > 0;
    let own_pass = copies[p.0 as usize] >= pass;
    // an opponent's Elesh Norn, Mother of Machines: the permanents' landfall effects below don't trigger (the
    // compiled and card-code ones are blanked where they fire, a graveyard card's not at all)
    let blanked = hooks::enter_silenced(g, p, land);
    let felidars = if blanked || !own_pass { vec![] } else { find(g, p, Tag::Landfall2) };
    for r in felidars {
        if crate::human::is_human(g, p) {
            // "choose one": a 2/2 Cat Beast, or a +1/+1 counter on each creature you control and vigilance until end of turn
            if !trigger_window(g, p, Some(r), "choose one: a 2/2 Cat, or +1/+1 counters and vigilance", None)? {
                continue;
            }
            let labels = vec![
                "Create a 2/2 white Cat Beast creature token".to_string(),
                "Put a +1/+1 counter on each creature you control; they gain vigilance until end of turn".to_string(),
            ];
            let k = crate::human::choices::choose_one(g, p, "choose", "Felidar Retreat: choose a mode", &labels)?;
            if k == 0 {
                make_tokens(g, p, Tokens::new(1, 2))?;
            } else {
                for m in g.player(p).perms.clone() {
                    if g.is_creature(m) {
                        repl::put_plus(g, m, 1);
                        let x = g.perm_mut(m);
                        if !x.eot_kw.contains(&"vigilance") {
                            x.eot_kw.push("vigilance");
                        }
                    }
                }
            }
            continue;
        }
        make_tokens(g, p, Tokens::new(1, 2))?; // Felidar Retreat: 2/2 Cat per land
    }
    let lands = g.player(p).lands.clone();
    let fields = lands.iter().filter(|&&l| g.db.get(g.perm(l).card()).tag(Tag::Fotd)).count();
    let mut names: Vec<CardId> = lands.iter().map(|&l| g.perm(l).card()).collect();
    names.sort();
    names.dedup();
    if own_pass && !blanked && fields > 0 && names.len() >= 7 {
        // Field of the Dead: 7+ lands with different names
        for _ in 0..fields {
            make_tokens(g, p, Tokens { color: Some(Colors::from_letters("B")), ..Tokens::new(1, 2) })?;
        }
    }
    if g.dsl_on {
        crate::dsl::fire_where(
            g,
            "landfall",
            crate::dsl::Fired { player: Some(p), perm: land, repeat, ..Default::default() },
            &|_, q| copies[q.0 as usize] >= pass,
        )?;
    }
    if !g.hooks.is_empty() {
        hooks::fire_trigger_entering(
            g,
            Event::Landfall,
            Call::Player { p },
            &|g, s| copies[g.perm(s).owner.0 as usize] >= pass,
            repeat,
            land,
        )?;
    }
    // a graveyard card's ability is no permanent's: once; Hushbringer stops abilities in every zone
    let hushed_land = land.is_some_and(|l| g.is_creature(l)) && repl::hushed(g);
    if repeat || hushed_land { Ok(()) } else { cardcode::gy_landfall(g, p) }
}

/// search for a basic land and put it into your hand. Every such search in the pool reveals the land (landcycling,
/// Land Tax, Cultivate, Strixhaven Skycoach), so the log names it for everyone
pub fn land_to_hand(g: &mut Game, p: PlayerId) -> Res {
    if crate::human::is_human(g, p) {
        if let Some(c) = crate::human::choices::pick_basic(g, p, "Search for a basic land to put into your hand")? {
            g.player_mut(p).hand.push(c);
            crate::glog!(g, "    {} reveals {} and puts it into their hand", g.player(p).name, g.db.get(c).name);
        }
        crate::human::choices::shuffle_library(g, p);
        return Ok(());
    }
    let basics = basics_in(g, p);
    if let Some(&c) = g.rng.choice(&basics) {
        let lib = &mut g.player_mut(p).library;
        let i = lib.iter().position(|&x| x == c).unwrap();
        lib.remove(i);
        match agent_for(g, p) {
            Some(a) => agent_take(g, a, p, c),
            None => {
                g.player_mut(p).hand.push(c);
                crate::glog!(g, "    {} reveals {} and puts it into their hand", g.player(p).name, g.db.get(c).name);
            }
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
    make_artifact_tokens_tapped(g, p, kind, n, false)
}

/// the same, and `tapped`: the Treasures come in tapped (Tataru Taru) -- they also do under an "artifacts enter tapped"
/// replacement. A tapped Treasure can't make mana (its ability needs {T}) until its owner's untap step.
pub fn make_artifact_tokens_tapped(g: &mut Game, p: PlayerId, kind: Sym, n: i32, tapped: bool) -> Res {
    if n <= 0 {
        return Ok(());
    }
    // u32 then i32 a third of its range: Manufactor's three kinds still sum inside it (the doublers are capped)
    let n = (n as u32).saturating_mul(repl::token_mult(g, p)).min(i32::MAX as u32 / 3) as i32;
    let manufactor = repl::manufactor(g, p);
    let kinds: Vec<Sym> = if manufactor { vec!["Treasure", "Food", "Clue"] } else { vec![kind] };
    let tap_all = repl::enters_tapped(g, p, None);
    for &k in &kinds {
        let pl = g.player_mut(p);
        match k {
            "Treasure" => {
                pl.treasures = pl.treasures.saturating_add(n as u32);
                pl.tapped_treasures = pl.tapped_treasures.saturating_add((tapped || tap_all) as u32 * n as u32);
            }
            "Food" => pl.foods = pl.foods.saturating_add(n as u32),
            _ => pl.clues = pl.clues.saturating_add(n as u32),
        }
    }
    note_tokens(g, p, (n * kinds.len() as i32) as u32);
    if !g.hooks.is_empty() {
        let total = n * kinds.len() as i32;
        fire_trigger(g, Event::TokenCreated, Call::ArtifactTokens { p, kinds, n: total })?;
    }
    Ok(())
}

/// return the best permanent card with mana value 3 or less from graveyard to the battlefield (a person picks one, or
/// none: "you may return target permanent card")
pub fn sun_titan(g: &mut Game, p: PlayerId, src: PermId) -> Res {
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
    let human = crate::human::is_human(g, p);
    let x = if human && !cs.is_empty() {
        if !trigger_window(g, p, Some(src), "return a permanent card with mana value 3 or less", None)? {
            return Ok(());
        }
        let cs: Vec<CardId> = cs.iter().copied().filter(|c| g.player(p).gy.contains(c)).collect(); // (the window)
        let labels: Vec<String> = cs.iter().map(|&c| crate::human::choices::card_label(g, c)).collect();
        let prompt =
            "Sun Titan: return which permanent card with mana value 3 or less from your graveyard to the battlefield?";
        match crate::human::choices::choose(g, p, "target", prompt, &labels, Some("none"), None)? {
            Some(k) => cs[k],
            None => return Ok(()),
        }
    } else {
        let Some(x) = max_by(&cs, key) else { return Ok(()) };
        x
    };
    let gy = &mut g.player_mut(p).gy;
    let i = gy.iter().position(|&c| c == x).unwrap();
    gy.remove(i);
    if g.db.get(x).land {
        if human {
            let tapped = crate::engine::turn::land_enters_tapped(g, p, x);
            g.add_land_entering(p, x, tapped);
            landfall(g, p)?;
        } else {
            g.add_land_entering(p, x, true);
        }
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
        if pl.is_commander(c) {
            pl.set_cmd_zone(c, true); // a commander in hand (Command Beacon): to the command zone
            continue;
        }
        if necro {
            pl.exile.push(c)
        } else {
            put_in_gy(g, q, c); // (Blightsteel Colossus is shuffled into the library)
        }
        if !necro {
            crate::impls::t12_brago_meren::note_gy(g, c); // --- Tiers 1/2 (brago-meren)
        }
        if !g.hooks.is_empty() {
            fire_trigger(g, Event::Discard, Call::Discard { p: q, c })?;
            g.player_mut(q).discarded_turn = Some(g.turn_stamp());
        }
    }
    if !g.hooks.is_empty() && !necro {
        fire_trigger(g, Event::CardsToGy, Call::Cards { p: q, cards: done.clone() })?;
    }
    crate::impls::kefka::discarded(g, q, &done)?; // mayhem / madness
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
    if g.player(q).is_commander(c) {
        g.player_mut(q).set_cmd_zone(c, true);
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
    put_in_gy(g, q, c); // (Blightsteel Colossus is shuffled into the library)
    crate::impls::t12_brago_meren::note_gy(g, c); // --- Tiers 1/2 (brago-meren)
    crate::impls::kefka::discarded(g, q, &[c])?; // mayhem / madness
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

/// p created k tokens: counted for "the number of tokens you created this turn" (Thalisse, Reverent Medium)
pub fn note_tokens(g: &mut Game, p: PlayerId, k: u32) {
    let st = g.turn_stamp();
    let pl = g.player_mut(p);
    // saturating: a safety net only (the doublers are capped, `repl::MULT_CAP`)
    pl.tok_made = if pl.tok_made.0 == Some(st) { (Some(st), pl.tok_made.1.saturating_add(k)) } else { (Some(st), k) };
}
