//! The titania deck (decklists/JD/titania-lands.md): mono-green, lands going to the graveyard under Titania, Protector
//! of Argoth. Its card code and AI plan. The brief is rust/agents/archive/titania-cards.md; the verdicts are in
//! data/card_overrides.toml.
//!
//! Cards modeled here (the rest of the deck is compiled abilities or shared card code):
//! - **Titania herself**: the enter trigger (t1.rs, shared) returns the fetch land first for this deck, untapped, and
//!   cracks every fetch land at once; the graveyard trigger is `land_gy` (t1.rs): `turn::crack_fetch` fires it for
//!   Evolving Wilds, Terramorphic Expanse and Fabled Passage (even with nothing left to find), `lands::sac_land` and
//!   `lands::destroy_land` for the other ways a land dies.
//! - **Follow the Lumarets** (infusion: two cards if you gained life this turn), **Something Worth Saving**, **Scout the
//!   City** (both modes), **Malevolent Rumble** (the Eldrazi Spawn can be sacrificed for {C}), **Glorious Decay** (all
//!   three modes), **Annoyed Altisaur** (cascade), **Arachnogenesis** (X Spiders; non-Spider combat damage is prevented:
//!   `Game::fog_spiders`, read by `cardcode::damage_prevented`), **Infestation Expert** (daybound and nightbound:
//!   `Game::day_night`, judged as each turn begins by `day_night_tick`).
//!
//! The AI plan: Titania as soon as five mana allows; the graveyard fillers first; fetch lands cracked at once while
//! Titania is out and held on the battlefield while she can be cast next turn (`hold_fetch`, `land_adjust`);
//! Garruk's Uprising before the 5/3s; Royal Treatment and Blossoming Defense when removal is aimed at Titania (and the
//! fetch lands cracked in response when nothing saves her); Arachnogenesis against a big attack; Overwhelming Stampede
//! and End-Raze Forerunners when the attack is lethal through the blockers.

use crate::cards::CardDb;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{counter_window, trigger_window};
use crate::engine::values::{card_name, epow, etgh, pval};
use crate::engine::zones::{Tokens, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

pub const TITANIA: &str = "Titania, Protector of Argoth";
const KEY: &str = "titania";
const OMNATH_KEY: &str = "omnath";

/// the decks that share this plan: titania, and omnath (Titania in the 99)
pub fn is_deck(key: &str) -> bool {
    key == KEY || key == OMNATH_KEY
}
const EXPERT: &str = "Infestation Expert";
const ARACHNO: &str = "Arachnogenesis";
const ROYAL: &str = "Royal Treatment";
const BLOSSOM: &str = "Blossoming Defense";
const UPRISING: &str = "Garruk's Uprising";

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

fn creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

/// p's Titania, if she is out
pub fn titania_of(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && card_name(g, m) == Some(TITANIA))
}

/// p's Omnath, Locus of Rage, if she is out (each fetch land cracked is a landfall for her)
fn omnath_out(g: &Game, p: PlayerId) -> bool {
    g.player(p).key == OMNATH_KEY && g.player(p).perms.iter().any(|&m| live(g, m) && card_name(g, m) == Some("Omnath, Locus of Rage"))
}

fn is_fetch(g: &Game, c: CardId) -> bool {
    g.db.get(c).tag(Tag::F)
}

/// the fetch lands p has on the battlefield (not cracked yet)
fn fetch_lands(g: &Game, p: PlayerId) -> Vec<LandId> {
    g.player(p)
        .lands
        .iter()
        .copied()
        .filter(|&l| {
            let x = g.land(l);
            // a fetch land Primeval Titan put in tapped can not pay its {T} until it untaps
            x.on_bf && is_fetch(g, x.cd) && !(x.tapped && x.data.truthy(DataKey::TitanTapped))
        })
        .collect()
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == name)
}

/// Titania can be cast (in the command zone or in hand) and costs this much now
fn titania_cost(g: &Game, p: PlayerId) -> Option<u32> {
    let pl = g.player(p);
    if !is_deck(pl.key) || titania_of(g, p).is_some() {
        return None;
    }
    // the titania deck: Titania is the commander; the omnath deck: she is a card in hand
    let card = if pl.key == KEY {
        if !(pl.cmd_in_zone || pl.hand.contains(&pl.cmd)) {
            return None;
        }
        pl.cmd
    } else {
        in_hand(g, p, TITANIA)?
    };
    let (gn, pips) = cost_of(g, p, card);
    Some(gn + pips.chars().count() as u32)
}

fn other_land_in_hand(g: &Game, p: PlayerId) -> bool {
    g.player(p).hand.iter().any(|&x| g.db.get(x).land && !is_fetch(g, x))
}

// ================================================================== fetch lands
/// Hold a fetch land that was just played instead of cracking it: Titania isn't out, she can be cast next turn without
/// the basic this fetch land would find (that basic enters tapped, so cracking it now would add nothing to that
/// cast), and the 5/3 comes when she arrives. With Garruk's Uprising still to be cast while Titania is out, the crack
/// waits for it too (`crack_held` casts it first).
pub fn hold_fetch(g: &Game, p: PlayerId) -> bool {
    if !is_deck(g.player(p).key) {
        return false;
    }
    if omnath_out(g, p) && titania_of(g, p).is_none() {
        return false; // every crack is a landfall now
    }
    if titania_of(g, p).is_some() {
        return uprising_first(g, p);
    }
    let Some(need) = titania_cost(g, p) else { return false };
    let ramp = g.player(p).hand.iter().any(|&x| {
        let t = &g.db.get(x).tags;
        t.has(Tag::Lr) || t.has(Tag::Dork)
    });
    total_mana(g, p, false) + other_land_in_hand(g, p) as u32 + ramp as u32 >= need
}

/// the fetch land to play this turn (`engine::turn::land_score`): with Titania out every fetch land is a 5/3; while she
/// can be cast this turn without the land drop, play it and hold it; while the land drop is what makes her castable
/// this turn, play the basic
pub fn land_adjust(g: &Game, p: PlayerId, c: CardId) -> f64 {
    if !is_deck(g.player(p).key) || !is_fetch(g, c) {
        return 0.0;
    }
    if titania_of(g, p).is_some() || omnath_out(g, p) {
        return 6.0;
    }
    let Some(need) = titania_cost(g, p) else { return 0.0 };
    let m = total_mana(g, p, false);
    if m >= need {
        6.0
    } else if m + 1 >= need && other_land_in_hand(g, p) {
        -6.0
    } else {
        0.0
    }
}

fn uprising_first(g: &Game, p: PlayerId) -> bool {
    let on_bf = g.player(p).perms.iter().any(|&m| live(g, m) && card_name(g, m) == Some(UPRISING));
    !on_bf
        && in_hand(g, p, UPRISING).is_some_and(|c| {
            let (gn, pips) = cost_of(g, p, c);
            can_pay(g, p, gn, &pips, false) && crate::engine::cast::castable(g, p, c, "hand")
        })
}

/// crack every fetch land p has on the battlefield (Titania is out): Garruk's Uprising first when it can be cast
fn crack_held(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    if titania_of(g, p).is_none() {
        return Ok(false);
    }
    if uprising_first(g, p)
        && let Some(c) = in_hand(g, p, UPRISING)
    {
        crate::ai::brain::do_cast_with(g, p, c, None, Ctx::default())?;
    }
    let mut any = false;
    for l in fetch_lands(g, p) {
        if g.over || !g.land(l).on_bf {
            continue;
        }
        crate::engine::turn::crack_fetch(g, p, l)?;
        any = true;
    }
    Ok(any)
}

/// Titania enters: the fetch land in the graveyard comes back (untapped), then every fetch land is cracked. The
/// shared version (t1.rs) serves the other decks: it returns the land with the most colors, tapped.
fn titania_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if !is_deck(g.player(o).key) {
        return super::t1::titania(g, src, p, m);
    }
    if m != src {
        return Ok(());
    }
    if g.player(o).gy.iter().any(|&c| g.db.get(c).land)
        && trigger_window(g, o, Some(src), "return a land from the graveyard", None)?
    {
        let ls: Vec<CardId> = g.player(o).gy.iter().copied().filter(|&c| g.db.get(c).land).collect();
        let pick = ls.iter().copied().find(|&c| is_fetch(g, c)).or_else(|| ls.first().copied());
        if let Some(c) = pick {
            if let Some(i) = g.player(o).gy.iter().position(|&x| x == c) {
                g.player_mut(o).gy.remove(i);
            }
            g.add_land(o, c, false);
            crate::engine::zones::landfall(g, o)?;
        }
    }
    // her trigger is on the battlefield now: the fetch lands held back make their 5/3s
    for l in fetch_lands(g, o) {
        if g.over || !g.land(l).on_bf {
            continue;
        }
        crate::engine::turn::crack_fetch(g, o, l)?;
    }
    Ok(())
}

// ================================================================== the graveyard fillers
/// how much a permanent card found by a mill is worth in hand: a land while lands are short, a fetch land only when
/// Titania can use it at once (it is better left in the graveyard for her), otherwise by mana value
fn found_score(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let d = g.db.get(c);
    if d.land {
        if is_fetch(g, c) {
            return if titania_of(g, p).is_some() { 3.0 } else { -1.0 };
        }
        let have = g.player(p).hand.iter().filter(|&&x| g.db.get(x).land).count();
        return if have < 2 { 4.0 } else { 1.0 };
    }
    if !d.perm {
        return -9.0;
    }
    1.0 + 0.6 * (d.cmc as f64).min(6.0) + if d.creature { 0.5 } else { 0.0 }
}

/// the best permanent card among the n cards just put into the graveyard goes to the hand
fn take_best(g: &mut Game, p: PlayerId, from: usize) {
    let new: Vec<CardId> = g.player(p).gy[from..].to_vec();
    let mut best: Option<(f64, CardId)> = None;
    for &c in &new {
        let v = found_score(g, p, c);
        if v > 0.0 && best.is_none_or(|b| v > b.0) {
            best = Some((v, c));
        }
    }
    if let Some((_, c)) = best {
        let pl = g.player_mut(p);
        if let Some(i) = pl.gy.iter().rposition(|&x| x == c) {
            pl.gy.remove(i);
        }
        pl.hand.push(c);
        pl.seen.insert(c);
        crate::glog!(g, "    {} takes {} into its hand", g.player(p).name, g.db.get(c).name);
    }
}

/// mill n cards, then take the best permanent card among them
fn mill_take(g: &mut Game, p: PlayerId, n: usize) -> Res {
    let from = g.player(p).gy.len();
    crate::engine::zones::mill(g, p, n)?;
    take_best(g, p, from);
    Ok(())
}

/// Something Worth Saving: mill four, a permanent card from among them to your hand, gain 1 life
fn worth_saving(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    mill_take(g, p, 4)?;
    crate::engine::life::gain(g, p, 1)?;
    Ok(intern("gy"))
}

/// Malevolent Rumble: reveal the top four, a permanent card to your hand and the rest into the graveyard; an Eldrazi
/// Spawn (0/1, "Sacrifice this token: Add {C}")
fn rumble(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    mill_take(g, p, 4)?;
    let spec = Tokens {
        color: Some(crate::cards::Colors::NONE),
        types: vec!["eldrazi", "spawn"],
        tgh: Some(1),
        ..Tokens::new(1, 0)
    };
    make_tokens(g, p, spec)?;
    Ok(intern("gy"))
}

fn is_flier(g: &Game, m: PermId) -> bool {
    g.is_creature(m) && (g.perm(m).fly || crate::dsl::has_kw(g, m, "flying"))
}

/// the opposing creature with flying worth destroying, if any
fn flier_target(g: &Game, p: PlayerId, ctx: &Ctx, max_toughness: i32) -> Option<PermId> {
    let ok = |g: &Game, m: PermId| {
        live(g, m)
            && is_flier(g, m)
            && g.perm(m).owner != p
            && !crate::engine::values::untargetable(g, m)
            && etgh(g, m) <= max_toughness
    };
    ctx.target.filter(|&t| ok(g, t)).or_else(|| {
        let cands: Vec<PermId> = g.opps(p).flat_map(|q| g.player(q).perms.clone()).filter(|&m| ok(g, m)).collect();
        max_by(&cands, |m| pval(g, m))
    })
}

/// Scout the City, {1}{G}: mill three, a permanent card from among them to your hand, gain 3 life; or destroy target
/// creature with flying. The flier mode is chosen when an opposing flier is worth a card.
fn scout_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    // a person chose the mode (and the flier) as they cast it
    let human = crate::human::is_human(g, p);
    let kill = if human {
        ctx.target.filter(|&t| ctx.mode == Some("flier") && live(g, t))
    } else {
        flier_target(g, p, ctx, i32::MAX).filter(|&t| pval(g, t) >= 3.0)
    };
    if let Some(t) = kill {
        apply_removal(g, Some(p), t, "destroy", None)?;
    } else {
        mill_take(g, p, 3)?;
        crate::engine::life::gain(g, p, 3)?;
    }
    Ok(intern("gy"))
}

/// Follow the Lumarets, {1}{G}: look at the top four, reveal a creature or land card and put it into your hand (two if
/// you gained life this turn); the rest go to the bottom in a random order
fn lumarets(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let k = if super::green_cards::gained_life(g, p) { 2 } else { 1 };
    super::t1::look_take(g, p, 4, &|g, c| g.db.get(c).creature || g.db.get(c).land, "hand", k)?;
    Ok(intern("gy"))
}

/// Glorious Decay, {1}{G}: destroy target artifact; or 4 damage to target creature with flying; or exile target card
/// from a graveyard and draw a card. A target the AI aims it at (`tgt=fa`) picks the first two modes; with none it
/// exiles the best card of an opponent's graveyard.
fn decay_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let art_ok = |g: &Game, m: PermId| {
        live(g, m)
            && g.perm(m).owner != p
            && !crate::engine::values::untargetable(g, m)
            && g.perm(m).cd.is_some_and(|d| g.db.get(d).types.has(crate::cards::Types::ARTIFACT))
    };
    let mut cands: Vec<PermId> = vec![];
    if let Some(t) = ctx.target {
        cands.push(t);
    } else {
        cands.extend(g.opps(p).flat_map(|q| g.player(q).perms.clone()));
    }
    let pick = max_by(
        &cands
            .into_iter()
            .filter(|&m| art_ok(g, m) || (is_flier(g, m) && etgh(g, m) <= 4 && live(g, m)))
            .collect::<Vec<_>>(),
        |m| pval(g, m),
    );
    if let Some(t) = pick {
        if art_ok(g, t) {
            apply_removal(g, Some(p), t, "destroy", Some(c))?;
        } else {
            apply_removal(g, Some(p), t, "dmg4", Some(c))?;
        }
        return Ok(intern("gy"));
    }
    // exile a card from a graveyard (an opponent's best creature, or a land), then draw
    let mut best: Option<(f64, PlayerId, CardId)> = None;
    for q in g.opps(p).collect::<Vec<_>>() {
        for &x in &g.player(q).gy {
            let d = g.db.get(x);
            let v = if d.creature { 2.0 + d.cmc as f64 } else { 0.5 };
            if best.is_none_or(|b| v > b.0) {
                best = Some((v, q, x));
            }
        }
    }
    if let Some((_, q, x)) = best {
        let pl = g.player_mut(q);
        if let Some(i) = pl.gy.iter().position(|&y| y == x) {
            pl.gy.remove(i);
            pl.exile.push(x);
        }
    }
    crate::engine::zones::draw(g, p, 1, false)?;
    Ok(intern("gy"))
}

// ================================================================== Annoyed Altisaur (cascade)
/// cascade: exile cards from the top until a nonland card that costs less (mana value 7 here), cast it free
fn altisaur_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let mv = g.db.get(c).cmc;
    super::legends::cascade(g, p, mv, "Annoyed Altisaur")?;
    Ok(())
}

// ================================================================== Arachnogenesis
/// Combat damage by a non-Spider creature is prevented this turn (read by `cardcode::damage_prevented`)
pub fn spider_fog(g: &Game, m: PermId) -> bool {
    g.fog_spiders == Some(g.turn_stamp())
        && !(g.perm(m).ttypes.contains(&"spider") || g.perm(m).cd.is_some_and(|c| g.db.get(c).has_subtype("spider")))
}

/// Arachnogenesis, {2}{G}: X 1/2 green Spider creature tokens with reach, X the creatures attacking you; prevent all
/// combat damage that would be dealt this turn by non-Spider creatures. Cast with X known (`ctx.x`), else the
/// creatures in combat that are not yours.
fn arachno_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = if ctx.x > 0 {
        ctx.x
    } else {
        g.in_combat.iter().filter(|&&m| live(g, m) && g.perm(m).owner != p).count() as i32
    };
    if x > 0 {
        let mut spec = Tokens {
            color: Some(crate::cards::Colors::from_letters("G")),
            types: vec!["spider"],
            tgh: Some(2),
            ..Tokens::new(x as u32, 1)
        };
        spec.data.set(DataKey::Kws, Val::List(vec![Val::Str(intern("reach"))]));
        make_tokens(g, p, spec)?;
    }
    g.fog_spiders = Some(g.turn_stamp());
    crate::glog!(g, "    non-Spider creatures deal no combat damage this turn");
    Ok(intern("gy"))
}

/// the AI at its declare-attackers priority (`ai::brain::attack_response`): Arachnogenesis when the attack would take a
/// fifth of its life or more, or three attackers, or 12 damage
pub fn attack_fog(g: &mut Game, d: PlayerId, p: PlayerId, atk: &[PermId]) -> Res<bool> {
    let Some(c) = in_hand(g, d, ARACHNO) else { return Ok(false) };
    if g.fog == Some(g.turn_stamp()) || g.fog_spiders == Some(g.turn_stamp()) {
        return Ok(false);
    }
    let mine: Vec<PermId> = atk.iter().copied().filter(|&m| live(g, m) && g.perm(m).owner == p).collect();
    let incoming: i32 =
        mine.iter().map(|&m| epow(g, m) * if crate::engine::combat::double_strike(g, m) { 2 } else { 1 }).sum();
    let life = g.player(d).life;
    let cmd_threat = mine.iter().any(|&m| g.perm(m).is_cmd && g.player(d).cmd_dmg[p.index()] + epow(g, m) >= 21);
    // a fifth of the life, three or more attackers (three Spider blockers and a fogged turn) or a 12-point swing
    if incoming * 5 < life && mine.len() < 3 && incoming < 12 && !cmd_threat && incoming < life {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, d, c);
    if !can_pay(g, d, gn, &pips, false) {
        return Ok(false);
    }
    crate::glog!(
        g,
        "  {} answers the attack with Arachnogenesis ({} attackers, {} damage)",
        g.player(d).name,
        mine.len(),
        incoming
    );
    crate::ai::brain::do_cast_with(g, d, c, None, Ctx { x: mine.len() as i32, ..Ctx::default() })
}

// ================================================================== Infestation Expert (daybound, nightbound)
fn night(g: &Game) -> bool {
    g.day_night == 2
}

/// the Expert's face now: Infestation Expert 3/4 by day, Infested Werewolf 4/5 by night
fn set_face(g: &mut Game, m: PermId) {
    let (pw, th) = if night(g) { (4, 5) } else { (3, 4) };
    let x = g.perm_mut(m);
    x.pow = pw;
    x.tgh = th;
}

fn experts(g: &Game) -> Vec<PermId> {
    g.players
        .iter()
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| live(g, m) && card_name(g, m) == Some(EXPERT))
        .collect()
}

/// As a turn begins (day and night): if it is day and the player whose turn just ended cast no spell during it, it
/// becomes night; if it is night and that player cast two or more, it becomes day. The permanents with daybound or
/// nightbound transform. Only does anything once an Infestation Expert has made it day.
pub fn day_night_tick(g: &mut Game, p: PlayerId) -> Res {
    let last = g.dn_last.replace(p);
    if g.day_night == 0 {
        return Ok(());
    }
    let Some(q) = last else { return Ok(()) };
    let n = g.player(q).spells_this_turn;
    let next = if g.day_night == 1 && n == 0 {
        2
    } else if g.day_night == 2 && n >= 2 {
        1
    } else {
        g.day_night
    };
    if next != g.day_night {
        g.day_night = next;
        crate::glog!(g, "  it becomes {}", if next == 2 { "night" } else { "day" });
        for m in experts(g) {
            set_face(g, m);
        }
    }
    Ok(())
}

/// Infestation Expert / Infested Werewolf: whenever it enters or attacks, a 1/1 green Insect (two at night). As a
/// daybound permanent enters and it is neither day nor night, it becomes day.
fn insects(g: &mut Game, p: PlayerId) -> Res {
    let n = if night(g) { 2 } else { 1 };
    let spec = Tokens {
        color: Some(crate::cards::Colors::from_letters("G")),
        types: vec!["insect"],
        tgh: Some(1),
        ..Tokens::new(n, 1)
    };
    make_tokens(g, p, spec)?;
    Ok(())
}

fn expert_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || g.perm(src).owner != p {
        return Ok(());
    }
    if g.day_night == 0 {
        g.day_night = 1;
        crate::glog!(g, "  it becomes day");
    }
    set_face(g, m);
    if trigger_window(g, p, Some(src), "create a 1/1 Insect", Some(2.0))? {
        insects(g, p)?;
    }
    Ok(())
}

fn expert_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src)
        && g.perm(src).owner == p
        && trigger_window(g, p, Some(src), "create a 1/1 Insect", Some(2.0))?
    {
        insects(g, p)?;
    }
    Ok(vec![])
}

// ================================================================== protecting Titania
/// cast an instant from the hand for {G} in response to removal on m (`gives`: what it does to m)
pub fn cast_protect(g: &mut Game, p: PlayerId, name: &str, m: PermId, gives: fn(&mut Game, PermId)) -> Res<bool> {
    let Some(c) = in_hand(g, p, name) else { return Ok(false) };
    if !can_pay(g, p, 0, "G", false) || !crate::engine::cast::castable(g, p, c, "hand") {
        return Ok(false);
    }
    if !pay(g, p, 0, "G", false)? {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    if let Some(i) = pl.hand.iter().position(|&x| x == c) {
        pl.hand.remove(i);
    }
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts {} -> {}", g.player(p).name, name, g.perm(m).name);
    crate::engine::cast::on_cast(g, p, c)?;
    let ok = counter_window(g, p, c, 6.0, vec![])?;
    g.player_mut(p).gy.push(c);
    if !ok {
        return Ok(false);
    }
    gives(g, m);
    Ok(true)
}

pub fn blossom_apply(g: &mut Game, m: PermId) {
    super::partials::eot_kw(g, m, "hexproof");
    let x = g.perm_mut(m);
    x.eot_pt = (x.eot_pt.0 + 2, x.eot_pt.1 + 2);
}

/// removal is aimed at m: Royal Treatment or Blossoming Defense on Titania (a creature worth 5 or more); a removal
/// spell that targets nothing (an edict, a sweeper) can't be stopped. When nothing saves Titania, every fetch land is
/// cracked in response while she is still on the battlefield.
pub fn titania_protect(
    g: &mut Game,
    p: PlayerId,
    m: PermId,
    kind: &str,
    actor: Option<PlayerId>,
    _spell: Option<CardId>,
) -> Res<bool> {
    if actor.is_none() || actor == Some(p) || g.perm(m).cd.is_none() || !g.is_creature(m) {
        return Ok(false);
    }
    let is_titania = card_name(g, m) == Some(TITANIA);
    if matches!(kind, "edict" | "wipe" | "sacrifice") {
        if is_titania && kind == "wipe" {
            crack_before_death(g, p)?;
        }
        return Ok(false);
    }
    if !is_titania && pval(g, m) < 5.0 {
        return Ok(false);
    }
    if cast_protect(g, p, ROYAL, m, super::green_cards::royal_apply)? || cast_protect(g, p, BLOSSOM, m, blossom_apply)? {
        return Ok(true);
    }
    if is_titania {
        crack_before_death(g, p)?;
    }
    Ok(false)
}

fn crack_before_death(g: &mut Game, p: PlayerId) -> Res {
    for l in fetch_lands(g, p) {
        if g.over || !g.land(l).on_bf {
            continue;
        }
        crate::engine::turn::crack_fetch(g, p, l)?;
    }
    Ok(())
}

// ================================================================== the finishers
/// the damage p's attack deals to q through the blockers when every attacker gets `each` more power and trample (and
/// `extra` more creatures of `extra_pow` arrive with haste): each blocker soaks up its toughness from one attacker
fn trample_through(g: &Game, p: PlayerId, q: PlayerId, each: i32, extra: i32, extra_pow: i32) -> i32 {
    use crate::engine::combat::can_block;
    let mut atk: Vec<(Option<PermId>, i32)> = (0..extra).map(|_| (None, extra_pow + each)).collect();
    for m in creatures(g, p) {
        let x = g.perm(m);
        if x.tapped || x.noatk || (x.sick && !crate::engine::combat::kw(g, m, "haste")) {
            continue;
        }
        let pw = (epow(g, m) + each).max(0);
        if pw > 0 {
            atk.push((Some(m), pw));
        }
    }
    atk.sort_by_key(|a| -a.1);
    let mut blockers: Vec<PermId> =
        g.player(q).perms.iter().copied().filter(|&b| live(g, b) && g.is_creature(b) && !g.perm(b).tapped).collect();
    blockers.sort_by_key(|&b| -etgh(g, b));
    let mut total = 0;
    for (a, pw) in atk {
        let blocker = blockers.iter().position(|&b| a.is_none_or(|a| can_block(g, b, a)));
        match blocker {
            Some(i) => {
                let b = blockers.remove(i);
                total += (pw - etgh(g, b).max(0)).max(0);
            }
            None => total += pw,
        }
    }
    total
}

fn lowest_life(g: &Game, p: PlayerId) -> i32 {
    g.opps(p).map(|q| g.player(q).life).min().unwrap_or(40)
}

/// Overwhelming Stampede and End-Raze Forerunners: held until the alpha strike, with trample, is lethal for some
/// opponent through the blockers
fn finisher_prio(g: &Game, p: PlayerId, name: &str, base: i32) -> i32 {
    let lethal = |each: i32, extra: i32, extra_pow: i32| {
        g.opps(p).any(|q| trample_through(g, p, q, each, extra, extra_pow) >= g.player(q).life)
    };
    match name {
        "Overwhelming Stampede" => {
            let x = creatures(g, p).into_iter().map(|m| epow(g, m)).max().unwrap_or(0).max(0);
            if lethal(x, 0, 0) { 92 } else { 0 }
        }
        _ => {
            if creatures(g, p).len() >= 1 && lethal(2, 1, 7) {
                92
            } else {
                // as a creature: a big body once the board is built
                let _ = lowest_life(g, p);
                base.min(30)
            }
        }
    }
}

// ================================================================== the AI plan
/// titania's cast priority: ramp first; the graveyard fillers before Titania so her enter trigger has a fetch land to
/// return; Titania as soon as she can be cast (priority 97); Garruk's Uprising before the 5/3s; the finishers only
/// when they win; the protection spells and Arachnogenesis only in response
pub fn titania_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let base = crate::ai::pool::generic_prio(g, p, c);
    let pl = g.player(p);
    if c == pl.cmd {
        return if pl.turns >= 2 { (97 - 3 * pl.tax as i32).max(40) } else { 0 };
    }
    let name = &*g.db.get(c).name;
    let out = titania_of(g, p).is_some();
    let fetch_in_gy = pl.gy.iter().any(|&x| g.db.get(x).land && is_fetch(g, x));
    let filler = !out && !fetch_in_gy && pl.library.len() > 20;
    match name {
        // the omnath deck: Titania (in the 99) as soon as she can be cast, behind Omnath
        TITANIA if pl.key == OMNATH_KEY => 90,
        ROYAL | BLOSSOM | ARACHNO => 0,
        "Overwhelming Stampede" | "End-Raze Forerunners" => finisher_prio(g, p, name, base),
        // the graveyard fillers: before Titania while no fetch land waits in the graveyard
        "Satyr Wayfinder" | "Malevolent Rumble" | "Something Worth Saving" | "Scout the City" => {
            if pl.library.len() <= 12 {
                0
            } else if filler {
                86
            } else {
                base.max(44)
            }
        }
        "Follow the Lumarets" => {
            if pl.library.len() > 12 {
                48
            } else {
                0
            }
        }
        // ramp and landfall in one card (two lands from the top seven, tapped): early, before the threats
        "Cartographer's Survey" => {
            if pl.lands.len() <= 7 { 74 } else { base.max(40) }
        }
        // a 6/6 trampling haste threat that doubles every counter the deck makes: ahead of the other top-end cards
        "Vorinclex, Monstrous Raider" => base.max(62),
        UPRISING => {
            if out || creatures(g, p).iter().any(|&m| epow(g, m) >= 4) {
                70
            } else {
                40
            }
        }
        _ => base,
    }
}

/// Worldly Tutor's creature for Titania's plan (the creature goes on top, so it is the next draw): a mana creature
/// while the lands are few, then a land-finder or a graveyard filler while Titania is a turn or two away, and the big
/// bodies once she is out or the mana is there. End-Raze Forerunners is not a pick (it is cast only for lethal).
pub fn worldly_pick(g: &Game, p: PlayerId, okn: &[CardId]) -> Option<CardId> {
    let pl = g.player(p);
    let first = |ns: &[&str]| ns.iter().filter_map(|n| g.db.id(n)).find(|c| okn.contains(c));
    let mana = pl.lands.len()
        + pl.perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Dork))).count();
    let fetch_in_gy = pl.gy.iter().any(|&x| g.db.get(x).land && is_fetch(g, x));
    if titania_of(g, p).is_none() && mana < 5 {
        if mana <= 3 {
            let ramp = first(&["Llanowar Elves", "Paradise Druid", "Bugenhagen, Wise Elder", "Silhana Starfletcher"]);
            if ramp.is_some() {
                return ramp;
            }
        }
        // land-finders for the fifth mana, then the graveyard filler when no fetch land waits there
        if let Some(c) = first(&["Fblthp, Knows the Way", "Studious First-Year", "Spider-Man, Brooklyn Visionary"]) {
            return Some(c);
        }
        if !fetch_in_gy
            && let Some(c) = first(&["Satyr Wayfinder"])
        {
            return Some(c);
        }
        return first(&["Boughside Wanderers", "Topiary Lecturer", "Hardbristle Bandit"]);
    }
    first(&["Ghalta the Unstoppable", "Annoyed Altisaur", "Cactarantula", "Genesis Hydra", "Kura, the Boundless Sky"])
}

/// the deck's own plays outside any card's slots (`ai::brain::special_options`): crack the fetch lands held back, now
/// that Titania is out
pub fn titania_special(g: &mut Game, p: PlayerId, post: bool) -> Res<Vec<Opt>> {
    let mut out = super::green_cards::radical_options(g, p, post); // Greenbelt Radical turned face up
    if g.active != Some(p) || titania_of(g, p).is_none() || fetch_lands(g, p).is_empty() {
        return Ok(out);
    }
    out.push(Opt {
        utility: 25.0,
        label: "crack the fetch lands (Titania)".into(),
        act: Some(Action::Plan { f: crack_held, arg: 0 }),
    });
    Ok(out)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, TITANIA)?.etb = Some(titania_etb);
    r.card(db, "Something Worth Saving")?.resolve = Some(worth_saving);
    r.card(db, "Malevolent Rumble")?.resolve = Some(rumble);
    r.card(db, "Scout the City")?.resolve = Some(scout_resolve);
    r.card(db, "Follow the Lumarets")?.resolve = Some(lumarets);
    r.card(db, "Glorious Decay")?.resolve = Some(decay_resolve);
    r.card(db, "Annoyed Altisaur")?.self_cast = Some(altisaur_cast);
    r.card(db, ARACHNO)?.resolve = Some(arachno_resolve);
    let c = r.card(db, EXPERT)?;
    c.etb = Some(expert_etb);
    c.attack = Some(expert_attack);
    register_titan(r, db)?;
    Ok(())
}

// ================================================================== Primeval Titan
const TITAN: &str = "Primeval Titan";

/// how much a land is worth to the Titan's search: the titania deck wants fetch lands first (each is a landfall now
/// and, cracked, a land in the graveyard: a 5/3 under Titania), then utility lands, then basics; every other deck
/// takes the land with the most colors
fn titan_land_score(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let d = g.db.get(c);
    let cols = d.tags.str(Tag::C).map_or(0, |s| s.chars().count()) as f64;
    let basic = crate::engine::zones::BASIC_LANDS.contains(&&*d.name);
    if is_deck(g.player(p).key) {
        let fetch = if is_fetch(g, c) { 100.0 } else { 0.0 };
        return fetch + if basic { 0.0 } else { 10.0 } + cols;
    }
    cols * 10.0 + if basic { 0.0 } else { 1.0 }
}

/// Primeval Titan's trigger: up to two land cards from the library onto the battlefield tapped (each is a landfall),
/// then shuffle. A person is not asked (the best lands are taken); Opposition Agent takes what it can
fn titan_search(g: &mut Game, p: PlayerId) -> Res {
    for _ in 0..2 {
        let lands: Vec<CardId> =
            crate::engine::zones::searchable(g, p).into_iter().filter(|&c| g.db.get(c).land).collect();
        let Some(c) = max_by(&lands, |c| titan_land_score(g, p, c)) else { break };
        let lib = &mut g.player_mut(p).library;
        if let Some(i) = lib.iter().position(|&x| x == c) {
            lib.remove(i);
        }
        if let Some(a) = crate::engine::zones::agent_for(g, p) {
            crate::engine::zones::agent_take(g, a, p, c);
            continue;
        }
        let l = g.add_land_entering(p, c, true);
        if is_fetch(g, c) {
            g.land_mut(l).data.set(DataKey::TitanTapped, Val::Bool(true));
        }
        crate::glog!(g, "    {} puts {} onto the battlefield tapped", g.player(p).name, g.db.get(c).name);
        crate::engine::zones::landfall(g, p)?;
    }
    crate::engine::tutors::shuffle_library(g, p);
    Ok(())
}

fn titan_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && g.perm(src).owner == p && trigger_window(g, p, Some(src), "search for two lands", Some(3.0))? {
        titan_search(g, p)?;
    }
    Ok(())
}

fn titan_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src)
        && g.perm(src).owner == p
        && trigger_window(g, p, Some(src), "search for two lands", Some(3.0))?
    {
        titan_search(g, p)?;
    }
    Ok(vec![])
}

/// Titania's cast priority for the Titan: the six-mana threat that is also ramp, landfall and (with fetch lands left in
/// the library) two 5/3s a swing; ahead of the other top-end cards, behind Titania herself
fn titan_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let fetch_left = g.player(p).library.iter().any(|&x| is_fetch(g, x));
    if titania_of(g, p).is_some() && fetch_left {
        72
    } else {
        64
    }
}

fn register_titan(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, TITAN)?;
    c.etb = Some(titan_etb);
    c.attack = Some(titan_attack);
    c.prio = Some(titan_prio);
    Ok(())
}
