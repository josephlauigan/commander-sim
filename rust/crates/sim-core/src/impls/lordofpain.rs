//! The lordofpain candidate deck (decklists/JD/candidates/lordofpain.md): its card code and AI plan, and the cards the
//! lordofpain and smaug decks share (Charforger, Kalain, Lagomos, Hero's Heirloom, Ultimate Magic: Meteor).
//!
//! Modeled here: The Lord of Pain, Charforger, Kalain Reclusive Painter, Lagomos Hand of Hatred, Hero's Heirloom,
//! Ultimate Magic: Meteor (foretell included). Smaug is in `smaug.rs`.

use crate::cards::{CardDb, Colors};
use crate::engine::cast::cast_card;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::stack::trigger_window;
use crate::engine::values::{etgh, pval};
use crate::engine::zones::{Tokens, die, make_tokens};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};

const LORD: &str = "The Lord of Pain";
const METEOR: &str = "Ultimate Magic: Meteor";
const CHARFORGER: &str = "Charforger";

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn live_creature(g: &Game, m: PermId) -> bool {
    live(g, m) && g.is_creature(m)
}

fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

/// p controls a live permanent with this card name
pub fn controls_named(g: &Game, p: PlayerId, n: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && name_of(g, m) == n)
}

/// cards p has cast this turn
fn casts_this_turn(g: &Game, p: PlayerId) -> usize {
    match &g.player(p).turn_casts {
        Some((s, cs)) if *s == g.turn_stamp() => cs.len(),
        _ => 0,
    }
}

// ======================================================== The Lord of Pain
/// Your opponents can't gain life: the compiled static (`no_lifegain`). Whenever a player casts their first spell each
/// turn, choose another target player; the Lord deals damage equal to that spell's mana value to them. It triggers on
/// every player's first spell, yours included; the chosen player can't be the caster. The AI picks the opponent (of
/// the Lord's controller) who is not the caster and is closest to dying (life left after the damage, then the biggest
/// threat); when no such opponent exists (a two-player game, the opponent casts) the Lord's controller must be chosen.
fn lord_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if !live(g, src) || casts_this_turn(g, caster) != 1 {
        return Ok(());
    }
    let n = g.db.get(c).cmc as i32;
    if n <= 0 {
        return Ok(());
    }
    let o = owner(g, src);
    let alive = |g: &Game, q: PlayerId| g.player(q).alive && q != caster;
    let mut opts: Vec<PlayerId> = g.opps(o).filter(|&q| alive(g, q)).collect();
    if opts.is_empty() && alive(g, o) {
        opts.push(o);
    }
    let Some(t) = crate::engine::zones::min_by(&opts, |q| {
        (g.player(q).life - n) as f64 * 100.0 - crate::engine::values::threat(g, o, q).min(99.0)
    }) else {
        return Ok(());
    };
    let label = format!("deal {n} damage to {} (first spell this turn)", g.player(t).name);
    if trigger_window(g, o, Some(src), &label, Some(if g.player(t).life <= n { 9.0 } else { 5.0 }))? {
        lose_life(g, t, n, Some(o), "triggers", None)?;
    }
    Ok(())
}

/// the Lord's AI plan: on my own turn, before any spell is cast, the most expensive spell I can pay for goes first (only
/// the first spell counts), and a spell two or more mana cheaper waits behind it (ramp and lands are not held back).
pub fn lordofpain_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let base = crate::ai::pool::generic_prio(g, p, c);
    if base <= 0 || g.active != Some(p) || casts_this_turn(g, p) > 0 || controls_named(g, p, LORD).is_none() {
        return base;
    }
    let d = g.db.get(c);
    if d.land || c == g.player(p).cmd || d.tags.has(crate::tag::Tag::Rock) || d.tags.has(crate::tag::Tag::Dork) {
        return base;
    }
    let mana = total_mana(g, p, false);
    let best = g
        .player(p)
        .hand
        .iter()
        .map(|&x| g.db.get(x))
        .filter(|x| !x.land && x.cmc <= mana)
        .map(|x| x.cmc)
        .max()
        .unwrap_or(0);
    if best < 3 {
        base
    } else if d.cmc == best {
        (base + 12).min(95)
    } else if d.cmc + 2 <= best {
        (base - 30).max(1)
    } else {
        base
    }
}

// ======================================================== Charforger
fn oil(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Oil)
}

fn add_oil(g: &mut Game, m: PermId) {
    let n = oil(g, m) + 1;
    g.perm_mut(m).data.set(DataKey::Oil, Val::Int(n));
}

/// Whenever another creature or artifact you control is put into a graveyard from the battlefield (a permanent dies,
/// token or not), put an oil counter on Charforger
fn charforger_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m == src || !live(g, src) || g.perm(m).owner != owner(g, src) {
        return Ok(());
    }
    let artifact = g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT))
        || g.perm(m).ttypes.contains(&"artifact");
    if !(g.is_creature(m) || artifact) {
        return Ok(());
    }
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "put an oil counter on Charforger", Some(1.0))? && live(g, src) {
        add_oil(g, src);
    }
    Ok(())
}

/// the same for a Treasure / Food / Clue token sacrificed (they are artifacts going to the graveyard)
fn charforger_sac(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p != owner(g, src) || !live(g, src) || !matches!(what, Sacrificed::Token(_)) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "put an oil counter on Charforger", Some(1.0))? && live(g, src) {
        add_oil(g, src);
    }
    Ok(())
}

/// Remove three oil counters: exile the top card of your library; you may play it this turn (the AI plays it if it
/// can: a land, or a spell it can pay for). Offered in the main phases when it has three counters.
fn charforger_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || oil(g, src) < 3 || g.player(p).library.is_empty() {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 3.0,
        label: "Charforger: remove three oil counters, exile the top card".into(),
        act: Some(Action::Ability { src, f: charforger_go, arg: 0 }),
    }])
}

fn charforger_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if oil(g, src) < 3 || !live(g, src) {
        return Ok(false);
    }
    let Some(c) = g.player_mut(p).library.pop() else { return Ok(false) };
    let left = oil(g, src) - 3;
    g.perm_mut(src).data.set(DataKey::Oil, Val::Int(left));
    let pl = g.player_mut(p);
    pl.seen.insert(c);
    pl.hand.push(c);
    pl.impulse.push(c);
    crate::glog!(g, "    Charforger exiles {} (playable this turn)", g.db.get(c).name);
    Ok(true)
}

// ======================================================== Kalain, Reclusive Painter
/// Other creatures you control enter with an additional +1/+1 counter for each mana from a Treasure spent to cast
/// them (the engine counts the Treasures paid for the spell: `Game::cast_treasure`). When Kalain enters, create a
/// Treasure token: the compiled trigger. The AI doesn't plan its Treasure spending around this.
fn kalain_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src || !live(g, src) || p != owner(g, src) || !g.is_creature(m) {
        return Ok(());
    }
    let (Some(c), Some((cc, k)), Some((cur, _, _))) = (g.perm(m).cd, g.cast_treasure, g.cur_cast.as_ref()) else {
        return Ok(());
    };
    if c != cc || c != *cur || k == 0 {
        return Ok(());
    }
    g.cast_treasure = None;
    g.perm_mut(m).plus += k as i32;
    crate::glog!(g, "    {} enters with {k} +1/+1 counter(s) (Kalain)", g.perm(m).name);
    Ok(())
}

// ======================================================== Lagomos, Hand of Hatred
/// At the beginning of combat on your turn, create a 2/1 red Elemental token with trample and haste; sacrifice it at
/// the beginning of the next end step (`DataKey::EotSac`, swept by `engine::turn::end_step`).
fn lagomos_combat(g: &mut Game, src: Src, p: PlayerId) -> Res<Vec<PermId>> {
    if p != owner(g, src) || !live(g, src) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "create a 2/1 Elemental with trample and haste", Some(4.0))? {
        return Ok(vec![]);
    }
    let spec = Tokens {
        sick: false,
        color: Some(Colors::from_letters("R")),
        types: vec![intern("elemental")],
        ..Tokens::new(1, 2)
    };
    let spec = Tokens { tgh: Some(1), ..spec };
    for t in make_tokens(g, p, spec)? {
        let x = g.perm_mut(t);
        x.data.set(DataKey::EotSac, Val::Bool(true));
        let tr = intern("trample");
        if !x.eot_kw.contains(&tr) {
            x.eot_kw.push(tr);
        }
    }
    Ok(vec![])
}

/// {T}: Search your library for a card, put it into your hand, then shuffle. Activate only if five or more creatures
/// died this turn (`Game::died_count`).
fn lagomos_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let died = g.died_count.0 == Some(g.turn_stamp()) && g.died_count.1 >= 5;
    let x = g.perm(src);
    if post.is_none() || !died || x.tapped || x.sick || !live(g, src) || g.player(p).library.is_empty() {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 6.0,
        label: "Lagomos: tap, search for a card".into(),
        act: Some(Action::Ability { src, f: lagomos_go, arg: 0 }),
    }])
}

fn lagomos_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let died = g.died_count.0 == Some(g.turn_stamp()) && g.died_count.1 >= 5;
    if !died || g.perm(src).tapped || g.perm(src).sick || !live(g, src) {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    crate::engine::tutors::tutor(g, p, "any")?;
    Ok(true)
}

// ======================================================== Hero's Heirloom
/// Equipped creature gets +2/+1 (compiled `equip_bonus`) and, as long as it is legendary, trample and haste
fn heirloom_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    (kw == "trample" || kw == "haste")
        && live(g, src)
        && g.perm(src).attached == Some(m)
        && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(crate::tag::Tag::Leg))
}

// ======================================================== Ultimate Magic: Meteor
/// Ultimate Magic: Meteor deals 7 damage to each creature (the compiled spell: `dsl::resolve_spell`). Foretell {5}{R}:
/// for {2} the card is exiled face down (kept in the hand, marked in `impulse_long` like a plotted card) and cast on a
/// later turn for {5}{R}; cast from exile, it also destroys, for each opponent, an artifact (a worthwhile one) or
/// else the best land that player controls. The AI foretells with spare mana in its second main phase, and casts the
/// foretold card when it kills more of theirs than of mine.
fn meteor_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let from_exile = g.player(p).flag_turn.get(&intern("meteor_exile")) == Some(&g.turn_stamp());
    crate::dsl::resolve_spell(g, p, c, ctx)?;
    if from_exile {
        g.player_mut(p).flag_turn.shift_remove(&intern("meteor_exile"));
        for q in g.opps(p).collect::<Vec<_>>() {
            if !g.player(q).alive {
                continue;
            }
            let art = crate::engine::zones::max_by(
                &g.player(q)
                    .perms
                    .iter()
                    .copied()
                    .filter(|&m| {
                        live(g, m)
                            && g.perm(m).cd.is_some_and(|d| g.db.get(d).types.has(crate::cards::Types::ARTIFACT))
                            && !crate::engine::values::indestructible(g, m)
                    })
                    .collect::<Vec<_>>(),
                |m| pval(g, m),
            );
            let land = crate::engine::zones::max_by(&g.player(q).lands.clone(), |l| {
                let t = &g.db.get(g.land(l).cd).tags;
                (t.int(crate::tag::Tag::Amt).unwrap_or(1) * 10
                    + t.str(crate::tag::Tag::C).map_or(0, |s| s.chars().count()) as i32 * 3) as f64
                    + if g.db.get(g.land(l).cd).basic { 0.0 } else { 5.0 }
            });
            match (art, land) {
                (Some(a), _) if pval(g, a) >= 2.0 || land.is_none() => {
                    crate::glog!(g, "    Meteor destroys {}", g.perm(a).name);
                    die(g, a, "destroy")?;
                }
                (_, Some(l)) => super::fixes::destroy_land(g, q, l)?,
                (Some(a), None) => die(g, a, "destroy")?,
                (None, None) => {}
            }
        }
    }
    Ok(intern("gy"))
}

fn foretold_turn(g: &Game, p: PlayerId, c: CardId) -> Option<u32> {
    g.player(p)
        .impulse_long
        .iter()
        .find(|x| x.0 == c && x.1 < u32::MAX && x.1 > u32::MAX / 2)
        .map(|x| u32::MAX - 1 - x.1)
}

fn meteor_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(post) = post else { return Ok(vec![]) };
    if g.active != Some(p) {
        return Ok(vec![]);
    }
    let turns = g.player(p).turns;
    Ok(match foretold_turn(g, p, c) {
        Some(t) if turns > t && can_pay(g, p, 5, "R", false) => {
            let (mut ol, mut ml) = (0.0, 0.0);
            for q in g.players.iter().filter(|x| x.alive).map(|x| x.id).collect::<Vec<_>>() {
                for &m in &g.player(q).perms {
                    if live_creature(g, m) && etgh(g, m) <= 7 {
                        if q == p { ml += pval(g, m) } else { ol += pval(g, m) }
                    }
                }
            }
            let u = (ol - 1.2 * ml) / 2.2 + 1.5 - 1.5;
            if u > 0.5 {
                vec![Opt {
                    utility: u.min(8.0),
                    label: "Ultimate Magic: Meteor (foretold)".into(),
                    act: Some(Action::Plan { f: meteor_cast_foretold, arg: c.0 as i64 }),
                }]
            } else {
                vec![]
            }
        }
        Some(_) => vec![],
        None if post && total_mana(g, p, false) >= 2 && total_mana(g, p, false) < 8 => vec![Opt {
            utility: 1.5,
            label: "foretell Ultimate Magic: Meteor".into(),
            act: Some(Action::Plan { f: meteor_foretell, arg: c.0 as i64 }),
        }],
        None => vec![],
    })
}

fn meteor_foretell(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || foretold_turn(g, p, c).is_some() || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    let pl = g.player_mut(p);
    let mark = u32::MAX - 1 - pl.turns;
    pl.impulse_long.push((c, mark));
    crate::glog!(g, "  {} foretells Ultimate Magic: Meteor", g.player(p).name);
    Ok(true)
}

fn meteor_cast_foretold(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || foretold_turn(g, p, c).is_none_or(|t| g.player(p).turns <= t) {
        return Ok(false);
    }
    let (cg, cp) = cost_of(g, p, c);
    if !can_pay(g, p, cg, &cp, false) {
        return Ok(false);
    }
    pay(g, p, cg, &cp, false)?;
    g.player_mut(p).impulse_long.retain(|x| x.0 != c);
    let st = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(intern("meteor_exile"), st);
    cast_card(g, p, c, "hand", Ctx::default())?;
    g.player_mut(p).flag_turn.shift_remove(&intern("meteor_exile"));
    Ok(true)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, LORD)?.cast = Some(lord_cast);
    let c = r.card(db, CHARFORGER)?;
    c.dies = Some(charforger_dies);
    c.sacrifice = Some(charforger_sac);
    c.options = Some(charforger_options);
    r.card(db, "Kalain, Reclusive Painter")?.etb = Some(kalain_etb);
    let c = r.card(db, "Lagomos, Hand of Hatred")?;
    c.combat_start = Some(lagomos_combat);
    c.options = Some(lagomos_options);
    r.card(db, "Hero's Heirloom")?.grant_kw = Some(heirloom_kw);
    let c = r.card(db, METEOR)?;
    c.resolve = Some(meteor_resolve);
    c.hand_options = Some(meteor_options);
    Ok(())
}
