//! Tier 4 pool deck `chulane-bant-value-combo` and Tier 5 pool deck `kinnan-simic-mana-combo` (EDHREC's bracket-4
//! average lists): the cards the ability language doesn't cover (rust/agents/t45-chulane-kinnan.md), and the combos
//! the new lists hold that the catalogue lacked. Peregrine Drake's "untap up to five lands" is compiled (dsl_parse.rs,
//! `// --- Tiers 4/5 (chulane-kinnan)`); Azorius Chancery is a bounce land (card_overrides.toml).
//!
//! Chulane:
//! - Cloudstone Curio: a nonartifact permanent of yours enters: you may return another permanent of yours that shares
//!   a permanent type with it. The AI returns a creature it can recast at once for value (free with Aluren, or an
//!   enters-the-battlefield creature it can pay for with Chulane out). Left out: lands entering (the engine keeps
//!   lands apart from permanents, so a land drop doesn't trigger it).
//! - Intruder Alarm: creatures don't untap during their controllers' untap steps (engine/turn.rs asks `alarm_on`);
//!   a creature enters: untap all creatures. The AI casts it only as a combo piece (prio 0).
//! - Mana Breach: whenever a player casts a spell, that player returns a land they control to its owner's hand (their
//!   tapped land with the fewest colours first).
//! - Enduring Vitality (t6_green2.rs): the returned enchantment is no creature now (`DataKey::NotCreature`, read by
//!   `Game::is_creature`).
//!
//! Kinnan:
//! - Barkchannel Pathway // Tidechannel Pathway: the face is chosen as it enters (the colour p's other lands make
//!   less, then the colour its hand needs more); it taps for that colour only.
//! - Koma, Cosmos Serpent: sacrifice another Serpent: Koma gains indestructible (as it would be destroyed, by removal,
//!   a wipe or combat damage: `self_regen`), or tap target permanent: at the beginning of each combat, the AI taps the
//!   active opponent's biggest attackers, or on its own turn the biggest blockers, always keeping one Serpent back.
//!   Left out: "its activated abilities can't be activated this turn" (the tap is used on creatures only).
//! - Nezahal, Primal Tide: discard three cards: exile it, it returns at the beginning of the next end step (the
//!   engine's `eot_returns`): the answer to targeted removal or a steal with three cards in hand (six for a bounce).
//!   Left out: it returns untapped (the engine's end-step returns enter untapped), and the dodge isn't used against
//!   board wipes.
//! - Nyxbloom Ancient: a permanent tapped for mana (lands, mana creatures and rocks, Treasures, the creatures Enduring
//!   Vitality gives the ability) makes three times as much (engine/mana.rs asks `nyxbloom_factor`, before Kinnan's
//!   additional mana). Left out: the tripled mana is one unit of the source's colours, so a source of several colours
//!   can pay pips of different colours with one tap.
//! - Spellskite: {U/P}: a removal spell or ability an opponent aims at a better permanent of yours (worth more than
//!   the Spellskite, or the commander) is redirected to Spellskite when Spellskite is a legal target for it ({U}, or
//!   2 life above 10). Left out: steals and redirecting a counterspell's target (spells aren't permanents).
//! - Void Winnower: opponents can't cast spells with even mana values (`can_cast`) and can't block with creatures
//!   with even mana values (tokens are 0: `grant_kw` "cant_block"). Left out: X spells count their printed mana value.
//! - Wandering Archaic // Explore the Vastlands: an opponent's instant or sorcery: they pay {2} when they can and the
//!   copy is worth 2 or more to you; else you copy it (worth 1 or more) with new targets. The back face is cast from
//!   hand for {3} when the front is out of reach: each player puts a land and an instant or sorcery from their top
//!   five into their hand, the rest on the bottom, and gains 3 life. Left out: the back face counts as a creature
//!   spell for cast triggers (the card database has the front face's types).
//!
//! Combos (`COMBOS`, chained in combos::all):
//! - Intruder Alarm + Shrieking Drake / Whitemane Lion + a mana creature + Chulane: cast the bouncer with creature
//!   mana, it returns itself, every creature untaps: unbounded casts, draws and land drops.
//! - Aluren + Cloudstone Curio + Chulane + two creatures of mana value 3 or less: each free creature returns the other.
//! - Freed from the Real / Pemmin's Aura + Bloom Tender (with the blue Aura out it taps for {G}{U}), + Nyxbloom Ancient
//!   and a creature that taps for {U}, or + Badgermole Cub and a creature that taps for {U}: unbounded mana into
//!   Walking Ballista (or Thrasios drawing the deck to it).

use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{castable, copy_spell, discard_worst, on_cast};
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{Source, Unit, can_pay, cost_of, land_colors_now, mana_units, pay, plan_pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, counter_window, trigger_window};
use crate::engine::values::{epow, etgh, has_type, ptypes, pval, untargetable};
use crate::engine::zones::{bounce, exile_perm, max_by, min_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready};
use crate::impls::partials::{at_once, eot_kw};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

const CHULANE: &str = "Chulane, Teller of Tales";
const CURIO: &str = "Cloudstone Curio";
const ALARM: &str = "Intruder Alarm";
const BREACH: &str = "Mana Breach";
const ALUREN: &str = "Aluren";
const DRAKE: &str = "Shrieking Drake";
const LION: &str = "Whitemane Lion";
const PATHWAY: &str = "Barkchannel Pathway // Tidechannel Pathway";
const KOMA: &str = "Koma, Cosmos Serpent";
const NEZAHAL: &str = "Nezahal, Primal Tide";
const NYX: &str = "Nyxbloom Ancient";
const SKITE: &str = "Spellskite";
const WINNOWER: &str = "Void Winnower";
const ARCHAIC: &str = "Wandering Archaic // Explore the Vastlands";
const FREED: &str = "Freed from the Real";
const PEMMIN: &str = "Pemmin's Aura";
const TENDER: &str = "Bloom Tender";
const CUB: &str = "Badgermole Cub";
const BALLISTA: &str = "Walking Ballista";
const THRASIOS: &str = "Thrasios, Triton Hero";

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, src: PermId) -> PlayerId {
    g.perm(src).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn named(g: &Game, m: PermId, name: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| cname(g, c) == name)
}

/// p's live permanent of this name
fn mine(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && named(g, m, name))
}

fn held(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| cname(g, c) == name)
}

fn creatures_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

/// a permanent's mana value (a token's is 0, a token copy's is its card's)
fn mv(g: &Game, m: PermId) -> u32 {
    if g.perm(m).token && g.perm(m).cd.is_none() { 0 } else { g.perm(m).cd.map_or(0, |c| g.db.get(c).cmc) }
}

/// the permanent types of m that matter to Cloudstone Curio
fn perm_types(g: &Game, m: PermId) -> Vec<Types> {
    let t = ptypes(g, m);
    let mut out: Vec<Types> =
        [Types::ARTIFACT, Types::ENCHANTMENT, Types::PLANESWALKER].into_iter().filter(|&k| t.has(k)).collect();
    if g.is_creature(m) {
        out.push(Types::CREATURE);
    }
    out
}

// ================================================================== Cloudstone Curio
/// what returning creature x to p's hand is worth now: it is recast at once (free under Aluren, or paid for), drawing
/// with Chulane and repeating its enters effect; 0 when it can't be recast now or gains nothing
fn rebuy_value(g: &Game, p: PlayerId, x: PermId) -> f64 {
    let px = g.perm(x);
    let Some(cd) = px.cd else { return 0.0 };
    if px.token || px.is_cmd || !g.is_creature(x) || g.player(p).is_commander(cd) {
        return 0.0;
    }
    let (gn, pips) = cost_of(g, p, cd);
    let free = mine(g, p, ALUREN).is_some() && g.db.get(cd).cmc <= 3; // Aluren: cast without paying its mana cost
    if !free && !can_pay(g, p, gn, &pips, false) {
        return 0.0;
    }
    let chulane = mine(g, p, CHULANE).is_some();
    let etb = crate::dsl::etb_value(g, cd) as f64
        + if g.registry.get(cd).is_some_and(|i| i.etb.is_some()) { 1.0 } else { 0.0 };
    let v = etb + if chulane { 2.0 } else { 0.0 } - if free { 0.0 } else { (gn + pips.len() as u32) as f64 * 0.5 };
    // a creature that taps for mana loses a turn of mana to summoning sickness
    let dork = g.db.get(cd).tags.str(Tag::Dork).is_some() && !px.sick;
    if dork && !free { 0.0 } else { v }
}

/// Whenever a nonartifact permanent you control enters, you may return another permanent you control that shares a
/// permanent type with it to its owner's hand.
fn curio_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if p != o || m == src || !live(g, m) || ptypes(g, m).has(Types::ARTIFACT) || g.combo_spell {
        return Ok(());
    }
    let kinds = perm_types(g, m);
    let cands: Vec<PermId> = g
        .player(o)
        .perms
        .iter()
        .copied()
        .filter(|&x| x != m && live(g, x) && perm_types(g, x).iter().any(|k| kinds.contains(k)))
        .filter(|&x| rebuy_value(g, o, x) >= 2.0)
        .collect();
    let Some(t) = max_by(&cands, |x| rebuy_value(g, o, x)) else { return Ok(()) };
    if !trigger_window(g, o, Some(src), "Cloudstone Curio returns a permanent", None)? || !live(g, t) {
        return Ok(());
    }
    crate::glog!(g, "    {} returns {} to hand (Cloudstone Curio)", pname(g, o), g.perm(t).name);
    bounce(g, t)
}

// ================================================================== Intruder Alarm
/// "Creatures don't untap during their controllers' untap steps": an Intruder Alarm is on the battlefield
pub fn alarm_on(g: &Game) -> bool {
    !g.hooks.is_empty()
        && g.players.iter().filter(|q| q.alive).any(|q| q.perms.iter().any(|&m| live(g, m) && named(g, m, ALARM)))
}

/// Whenever a creature enters, untap all creatures.
fn alarm_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if !live(g, m) || !g.is_creature(m) {
        return Ok(());
    }
    let o = owner(g, src);
    let tapped = g.players.iter().filter(|q| q.alive).flat_map(|q| q.perms.iter().copied());
    let tapped: Vec<PermId> = tapped.filter(|&x| live(g, x) && g.perm(x).tapped && g.is_creature(x)).collect();
    if tapped.is_empty() || !trigger_window(g, o, Some(src), "untap all creatures", None)? {
        return Ok(());
    }
    for x in tapped {
        if live(g, x) && g.is_creature(x) {
            g.perm_mut(x).tapped = false;
        }
    }
    crate::glog!(g, "    Intruder Alarm untaps all creatures");
    Ok(())
}

// ================================================================== Mana Breach
/// the land q returns to hand: a tapped one first, then the one making the fewest colours
fn breach_land(g: &Game, q: PlayerId) -> Option<LandId> {
    min_by(&g.player(q).lands, |l| (!g.land(l).tapped) as i32 as f64 * 10.0 + land_colors_now(g, q, l).count() as f64)
}

/// Whenever a player casts a spell, that player returns a land they control to its owner's hand.
fn breach_cast(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    if !g.player(caster).alive || g.player(caster).lands.is_empty() {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "the caster returns a land to hand", None)? {
        return Ok(());
    }
    let Some(l) = breach_land(g, caster) else { return Ok(()) };
    crate::engine::turn::remove_land(g, caster, l);
    let cd = g.land(l).cd;
    g.player_mut(caster).hand.push(cd);
    crate::glog!(g, "    {} returns {} to hand (Mana Breach)", pname(g, caster), cname(g, cd));
    Ok(())
}

// ================================================================== Barkchannel Pathway
/// the face p would choose for its Pathway now: the colour its other lands make less, then the colour its hand needs
/// more (blue on a tie)
fn pathway_pick(g: &Game, p: PlayerId, l: LandId) -> char {
    let others: Vec<LandId> = g.player(p).lands.iter().copied().filter(|&x| x != l).collect();
    let src = |c: char| others.iter().filter(|&&x| land_colors_now(g, p, x).has(c)).count() as i32;
    let need = |c: char| g.player(p).hand.iter().map(|&x| g.db.get(x).pips.chars().filter(|&y| y == c).count()).sum();
    let (u, gr): (i32, i32) = (src('U'), src('G'));
    let (nu, ng): (usize, usize) = (need('U'), need('G'));
    if u != gr {
        return if u < gr { 'U' } else { 'G' };
    }
    if ng > nu { 'G' } else { 'U' }
}

/// As it is played, the face is chosen: Barkchannel Pathway taps for {G}, Tidechannel Pathway for {U}.
fn pathway_etb(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    let c = pathway_pick(g, p, l);
    g.land_mut(l).data.set(DataKey::ChosenColor, Val::Str(if c == 'U' { "U" } else { "G" }));
    crate::glog!(
        g,
        "    {} plays it as {}",
        pname(g, p),
        if c == 'U' { "Tidechannel Pathway" } else { "Barkchannel Pathway" }
    );
    Ok(())
}

fn pathway_cols(g: &Game, p: PlayerId, l: LandId) -> Colors {
    match g.land(l).data.get(DataKey::ChosenColor) {
        Some(Val::Str(s)) => Colors::from_letters(s),
        // put onto the battlefield some other way: the face it would have been played as
        _ => Colors::from_letters(&pathway_pick(g, p, l).to_string()),
    }
}

// ================================================================== Koma, Cosmos Serpent
/// p's other Serpents, the least valuable first (Koma's Coil tokens)
fn serpents(g: &Game, p: PlayerId, koma: PermId) -> Vec<PermId> {
    let mut v: Vec<PermId> =
        creatures_of(g, p).into_iter().filter(|&m| m != koma && has_type(g, m, "serpent")).collect();
    v.sort_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)));
    v
}

/// sacrifice another Serpent (its cost): false when there is none
fn sac_serpent(g: &mut Game, p: PlayerId, koma: PermId) -> Res<bool> {
    let Some(&s) = serpents(g, p, koma).first() else { return Ok(false) };
    crate::glog!(g, "    {} sacrifices {} (Koma)", pname(g, p), g.perm(s).name);
    crate::engine::zones::die(g, s, "sac")?;
    Ok(true)
}

/// Sacrifice another Serpent: Koma gains indestructible until end of turn, as it would be destroyed (removal, a wipe,
/// lethal combat damage).
fn koma_regen(g: &mut Game, m: PermId) -> Res<bool> {
    let p = owner(g, m);
    if !named(g, m, KOMA) || g.perm(m).eot_kw.contains(&"indestructible") || serpents(g, p, m).is_empty() {
        return Ok(false);
    }
    if !sac_serpent(g, p, m)? {
        return Ok(false);
    }
    if !ability_window(g, p, Some(m), "Koma gains indestructible", Some(5.0), None)? || !live(g, m) {
        return Ok(false);
    }
    eot_kw(g, m, "indestructible");
    crate::glog!(g, "    Koma gains indestructible");
    Ok(true)
}

/// creatures of q worth tapping with Koma (power 4 or more, or toughness 5 or more), untapped, the biggest first
fn koma_taps(g: &Game, q: PlayerId, attackers: bool) -> Vec<PermId> {
    let mut v: Vec<PermId> = creatures_of(g, q)
        .into_iter()
        .filter(|&m| !g.perm(m).tapped && !untargetable(g, m) && (epow(g, m) >= 4 || etgh(g, m) >= 5))
        .filter(|&m| !attackers || (!g.perm(m).sick || crate::engine::combat::has_haste(g, m)) && !g.perm(m).noatk)
        .collect();
    v.sort_by_key(|&m| -epow(g, m));
    v
}

/// Sacrifice another Serpent: tap target permanent. At the beginning of each combat: on an opponent's turn, its biggest
/// would-be attackers; on Koma's controller's turn, the opponents' biggest blockers. One Serpent is always kept back
/// for indestructible, at most two taps a combat.
fn koma_combat(g: &mut Game, src: Src, active: PlayerId) -> Res {
    let o = owner(g, src);
    if !live(g, src) || crate::cardcode::ability_locked(g, src, o) {
        return Ok(());
    }
    let mut targets: Vec<PermId> = if active == o {
        let mut v: Vec<PermId> = g.opps(o).flat_map(|q| koma_taps(g, q, false)).collect();
        v.sort_by_key(|&m| -epow(g, m));
        v
    } else {
        koma_taps(g, active, true)
    };
    targets.truncate(2);
    for t in targets {
        if serpents(g, o, src).len() < 2 || !live(g, t) || g.perm(t).tapped {
            break;
        }
        sac_serpent(g, o, src)?;
        let label = format!("Koma taps {}", g.perm(t).name);
        if ability_window(g, o, Some(src), &label, None, Some(t))? && live(g, t) {
            g.perm_mut(t).tapped = true;
            crate::glog!(g, "    {label}");
        }
    }
    Ok(())
}

// ================================================================== Nezahal, Primal Tide
/// Discard three cards: Exile Nezahal. Return it to the battlefield tapped under its owner's control at the beginning
/// of the next end step. In response to removal aimed at it (or a steal): with three cards in hand (six against a
/// bounce, which costs only the recast).
fn nezahal_dodge(g: &mut Game, p: PlayerId, m: PermId, kind: Sym) -> Res<bool> {
    let hand = g.player(p).hand.len();
    if !named(g, m, NEZAHAL) || !live(g, m) || g.perm(m).owner != p || hand < 3 || (kind == "bounce" && hand < 6) {
        return Ok(false);
    }
    if crate::cardcode::ability_locked(g, m, p) {
        return Ok(false);
    }
    discard_worst(g, p, 3)?;
    if !ability_window(g, p, Some(m), "Nezahal exiles itself", Some(5.0), None)? || !live(g, m) {
        return Ok(false);
    }
    let (cd, orig) = (g.perm(m).cd, g.perm(m).orig);
    exile_perm(g, m)?;
    if let Some(cd) = cd
        && g.player(orig).exile.contains(&cd)
    {
        g.eot_returns.push((orig, cd));
    }
    crate::glog!(g, "    {} exiles Nezahal until the next end step", pname(g, p));
    Ok(true)
}

// ================================================================== Spellskite
/// Spellskite is a legal new target for a spell or ability of this target type aimed at m
fn skite_legal(g: &Game, m: PermId, kind: &str, spell: Option<CardId>) -> bool {
    let tgt = spell.map(|s| g.db.get(s).tags.str(Tag::Tgt).unwrap_or("c"));
    if kind == "zero" || kind.starts_with("lock") {
        return g.is_creature(m); // a creature-only lock
    }
    match tgt {
        Some("cna" | "fe" | "blue") => false,
        Some(_) => true, // creature, artifact or any permanent: an artifact creature fits them all
        None => g.is_creature(m) || ptypes(g, m).has(Types::ARTIFACT),
    }
}

/// {U/P}: Change a target of target spell or ability to Spellskite. Opponents' targeted removal aimed at a better
/// permanent of p's goes to Spellskite instead: {U}, or 2 life when p has more than 10.
fn skite_redirect(
    g: &mut Game,
    p: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    let Some(actor) = actor.filter(|&a| a != p) else { return Ok(false) };
    if matches!(&*kind, "edict" | "wipe" | "sac" | "steal") {
        return Ok(false);
    }
    let Some(s) = mine(g, p, SKITE).filter(|&s| s != m && !crate::cardcode::ability_locked(g, s, p)) else {
        return Ok(false);
    };
    if !(g.perm(m).is_cmd || pval(g, m) > pval(g, s) + 0.5) || !skite_legal(g, m, &kind, spell) {
        return Ok(false);
    }
    if can_pay(g, p, 0, "U", false) {
        pay(g, p, 0, "U", false)?;
    } else if g.player(p).life > 10 {
        lose_life(g, p, 2, Some(p), "other", None)?;
    } else {
        return Ok(false);
    }
    if !ability_window(g, p, Some(s), "Spellskite redirects the removal", Some(5.0), None)? || !live(g, s) {
        return Ok(false);
    }
    crate::glog!(g, "    {} redirects it to Spellskite", pname(g, p));
    apply_removal(g, Some(actor), s, &kind, spell)?;
    Ok(true)
}

/// the pool decks' answers to targeted removal from these cards (ai::decks::protect_main): Spellskite redirects,
/// Nezahal exiles itself. True: m is safe.
pub fn protect(
    g: &mut Game,
    p: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if nezahal_dodge(g, p, m, kind)? {
        return Ok(true);
    }
    skite_redirect(g, p, m, kind, actor, spell)
}

// ================================================================== Nyxbloom Ancient
/// If you tap a permanent for mana, it produces three times as much of that mana instead: 3 for each Nyxbloom Ancient
/// p controls (engine/mana.rs multiplies the units of tapped sources before Kinnan's additional mana)
pub fn nyxbloom_factor(g: &Game, p: PlayerId) -> u32 {
    let Some(nyx) = g.db.id(NYX) else { return 1 };
    let n = g
        .hooks
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            x.cd == Some(nyx) && x.owner == p && x.on_bf && !x.phased && !x.neutered
        })
        .count() as u32;
    3u32.pow(n.min(4))
}

/// a unit made by tapping a permanent (a land, a mana creature or rock, a Treasure)
pub fn tapped_source(s: Source) -> bool {
    matches!(s, Source::Land(_) | Source::Perm(_) | Source::Treasure)
}

fn nyx_hooked(_g: &Game, _src: Src, _p: PlayerId, _u: &[Unit]) -> Vec<Unit> {
    vec![] // keeps the Ancient in `g.hooks` (its rule is `nyxbloom_factor`)
}

// ================================================================== Void Winnower
/// a mana value Void Winnower stops (zero is even)
fn even(n: u32) -> bool {
    n % 2 == 0
}

/// Your opponents can't cast spells with even mana values.
fn winnower_cast(g: &Game, src: Src, caster: PlayerId, c: CardId, _zone: Sym) -> bool {
    caster == owner(g, src) || !live(g, src) || !even(g.db.get(c).cmc)
}

/// Your opponents can't block with creatures with even mana values (the "cant_block" keyword).
fn winnower_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "cant_block" && live(g, src) && g.perm(m).owner != owner(g, src) && g.is_creature(m) && even(mv(g, m))
}

fn winnower_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.dsl_on = true; // the "can't block" grant is read through the keyword checks
    }
    Ok(())
}

// ================================================================== Wandering Archaic // Explore the Vastlands
/// Whenever an opponent casts an instant or sorcery spell, they may pay {2}. If they don't, you may copy that spell.
/// You may choose new targets for the copy.
fn archaic_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    let d = g.db.get(c);
    if caster == o || !(d.instant || d.sorcery) || !g.player(caster).alive || g.in_rtf {
        return Ok(());
    }
    let worth = crate::impls::mine::spell_copy_value(g, o, c);
    if !trigger_window(g, o, Some(src), "pay {2} or Wandering Archaic copies the spell", None)? {
        return Ok(());
    }
    if worth >= 2.0 && can_pay(g, caster, 2, "", false) {
        pay(g, caster, 2, "", false)?;
        crate::glog!(g, "    {} pays {{2}} (Wandering Archaic)", pname(g, caster));
        return Ok(());
    }
    if worth < 1.0 {
        return Ok(());
    }
    let ctx = g.cur_cast.as_ref().filter(|x| x.0 == c).map(|x| x.1.clone());
    crate::glog!(g, "    {} copies {} (Wandering Archaic)", pname(g, o), cname(g, c));
    g.in_rtf = true;
    let r = copy_spell(g, o, c, ctx.as_ref(), false);
    g.in_rtf = false;
    r
}

/// Explore the Vastlands ({3}, sorcery): cast from hand when the front face ({5}) is out of reach and the hand is
/// running low
fn vastlands_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let pl = g.player(p);
    if post.is_none() || g.active != Some(p) || !pl.hand.contains(&c) || pl.hand.len() > 3 {
        return Ok(vec![]);
    }
    if can_pay(g, p, 5, "", false) || !can_pay(g, p, 3, "", false) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.0,
        label: "cast Explore the Vastlands".into(),
        act: Some(Action::Plan { f: vastlands_go, arg: c.0 as i64 }),
    }])
}

fn vastlands_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 3, "", false) {
        return Ok(false);
    }
    crate::ai::decks::remove_from_hand(g, p, c);
    pay(g, p, 3, "", false)?;
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts Explore the Vastlands", pname(g, p));
    on_cast(g, p, c)?;
    let ok = counter_window(g, p, c, 2.0, vec![])?;
    g.player_mut(p).gy.push(c);
    if ok && !g.over {
        explore_vastlands(g, p)?;
    }
    Ok(true)
}

/// Each player looks at the top five cards of their library and may reveal a land card and/or an instant or sorcery
/// card from among them, puts them into their hand and the rest on the bottom in a random order. Each player gains 3.
pub fn explore_vastlands(g: &mut Game, p: PlayerId) -> Res {
    let order: Vec<PlayerId> = std::iter::once(p).chain(g.after(p)).filter(|&q| g.player(q).alive).collect();
    for &q in &order {
        let n = g.player(q).library.len().min(5);
        let lib = &mut g.player_mut(q).library;
        let at = lib.len() - n;
        let mut top: Vec<CardId> = lib.split_off(at);
        let mut take = vec![];
        if let Some(i) = top.iter().rposition(|&x| g.db.get(x).land) {
            take.push(top.remove(i));
        }
        if let Some(i) = top.iter().rposition(|&x| g.db.get(x).instant || g.db.get(x).sorcery) {
            take.push(top.remove(i));
        }
        g.rng.shuffle(&mut top);
        let pl = g.player_mut(q);
        pl.hand.extend(&take);
        for x in top {
            pl.library.insert(0, x);
        }
    }
    for &q in &order {
        gain(g, q, 3)?;
    }
    Ok(())
}

// ================================================================== combos
/// p's untapped creature mana (summoning sickness and Enduring Vitality's grant as the engine counts them)
fn creature_units(g: &Game, p: PlayerId) -> Vec<Unit> {
    mana_units(g, p, false).into_iter().filter(|w| matches!(w.src, Source::Perm(m) if g.is_creature(m))).collect()
}

/// p's creatures can pay this cost again and again (they untap each time, Intruder Alarm)
fn creatures_pay(g: &Game, p: PlayerId, generic: u32, pips: &str) -> bool {
    plan_pay(g, &creature_units(g, p), generic, pips).is_some()
}

/// Intruder Alarm (on the battlefield, or cast from hand) + Shrieking Drake or Whitemane Lion in hand, whose cost p's
/// creatures can pay, + Chulane: the bouncer returns itself, every creature untaps, Chulane draws and plays a land
fn alarm_ready(g: &Game, p: PlayerId) -> Ready {
    let Some(ch) = mine(g, p, CHULANE) else { return (false, vec![], vec![]) };
    let Some(b) = [DRAKE, LION].iter().find_map(|n| held(g, p, n)) else { return (false, vec![], vec![]) };
    let (gn, pips) = cost_of(g, p, b);
    if !creatures_pay(g, p, gn, &pips) || !castable(g, p, b, "hand") {
        return (false, vec![], vec![]);
    }
    if let Some(a) = mine(g, p, ALARM) {
        return (true, vec![a, ch], vec![b]);
    }
    match held(g, p, ALARM) {
        Some(a) if castable(g, p, a, "hand") => (true, vec![ch], vec![a, b]),
        _ => (false, vec![], vec![]),
    }
}

/// a creature card of mana value 3 or less (Aluren casts it free), not a commander
fn cheap_creature(g: &Game, p: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    d.creature && d.cmc <= 3 && !g.player(p).is_commander(c)
}

/// Aluren + Cloudstone Curio + Chulane on the battlefield, a cheap creature on the battlefield and another in hand:
/// each free cast returns the other creature (Curio), Chulane draws and plays a land each time
fn curio_ready(g: &Game, p: PlayerId) -> Ready {
    let (Some(a), Some(cu), Some(ch)) = (mine(g, p, ALUREN), mine(g, p, CURIO), mine(g, p, CHULANE)) else {
        return (false, vec![], vec![]);
    };
    let on = creatures_of(g, p).into_iter().any(|m| {
        let x = g.perm(m);
        !x.token && !x.is_cmd && x.cd.is_some_and(|c| cheap_creature(g, p, c))
    });
    let Some(c) = g.player(p).hand.iter().copied().find(|&c| cheap_creature(g, p, c) && castable(g, p, c, "hand"))
    else {
        return (false, vec![], vec![]);
    };
    if !on {
        return (false, vec![], vec![]);
    }
    (true, vec![a, cu, ch], vec![c])
}

/// unbounded mana wins: Walking Ballista (on the battlefield or in hand), or Thrasios drawing the deck to it
fn mana_outlet(g: &Game, p: PlayerId) -> bool {
    if mine(g, p, BALLISTA).is_some() || held(g, p, BALLISTA).is_some() {
        return true;
    }
    let lib = g.player(p).library.iter().any(|&c| cname(g, c) == BALLISTA);
    lib && mine(g, p, THRASIOS).is_some()
}

/// the untapping Aura: on the battlefield (keys), or castable from hand (casts)
fn aura_piece(g: &Game, p: PlayerId) -> Option<(Vec<PermId>, Vec<CardId>)> {
    if let Some(a) = mine(g, p, FREED).or_else(|| mine(g, p, PEMMIN)) {
        return Some((vec![a], vec![]));
    }
    let c = held(g, p, FREED).or_else(|| held(g, p, PEMMIN))?;
    castable(g, p, c, "hand").then(|| (vec![], vec![c]))
}

/// a creature of p's that taps for {U} now (not summoning sick)
fn blue_creature(g: &Game, p: PlayerId, not: Option<PermId>) -> Option<PermId> {
    creature_units(g, p).into_iter().find_map(|w| match w.src {
        Source::Perm(m) if w.cols.has('U') && Some(m) != not => Some(m),
        _ => None,
    })
}

/// Freed from the Real / Pemmin's Aura on Bloom Tender: with the blue Aura out it taps for {G}{U}, the Aura untaps it
/// for {U}
fn tender_ready(g: &Game, p: PlayerId) -> Ready {
    let Some(t) = mine(g, p, TENDER).filter(|&t| !g.perm(t).sick || crate::dsl::has_kw(g, t, "haste")) else {
        return (false, vec![], vec![]);
    };
    let Some((mut keys, casts)) = aura_piece(g, p) else { return (false, vec![], vec![]) };
    if !mana_outlet(g, p) {
        return (false, vec![], vec![]);
    }
    keys.push(t);
    (true, keys, casts)
}

/// Nyxbloom Ancient + the Aura on a creature that taps for {U}: three {U} a tap, {U} to untap
fn nyx_ready(g: &Game, p: PlayerId) -> Ready {
    let Some(n) = mine(g, p, NYX) else { return (false, vec![], vec![]) };
    let Some(d) = blue_creature(g, p, None) else { return (false, vec![], vec![]) };
    let Some((mut keys, casts)) = aura_piece(g, p) else { return (false, vec![], vec![]) };
    if !mana_outlet(g, p) {
        return (false, vec![], vec![]);
    }
    keys.extend([n, d]);
    (true, keys, casts)
}

/// Badgermole Cub + the Aura on a creature that taps for {U}: {U} and the Cub's {G} a tap, {U} to untap
fn cub_ready(g: &Game, p: PlayerId) -> Ready {
    let Some(c) = mine(g, p, CUB) else { return (false, vec![], vec![]) };
    let Some(d) = blue_creature(g, p, None) else { return (false, vec![], vec![]) };
    let Some((mut keys, casts)) = aura_piece(g, p) else { return (false, vec![], vec![]) };
    if !mana_outlet(g, p) {
        return (false, vec![], vec![]);
    }
    keys.extend([c, d]);
    (true, keys, casts)
}

pub static COMBOS: [Combo; 5] = [
    Combo {
        name: "Intruder Alarm + Shrieking Drake / Whitemane Lion + a mana creature + Chulane",
        groups: &[&[ALARM], &[DRAKE, LION], &[CHULANE]],
        ready: alarm_ready,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Aluren + Cloudstone Curio + Chulane + two cheap creatures",
        groups: &[&[ALUREN], &[CURIO], &[CHULANE]],
        ready: curio_ready,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Freed from the Real / Pemmin's Aura + Bloom Tender",
        groups: &[&[FREED, PEMMIN], &[TENDER], &[BALLISTA]],
        ready: tender_ready,
        mana: (0, ""),
        finish: None,
        xcost: &[(BALLISTA, 1)],
        steps: &[],
    },
    Combo {
        name: "Freed from the Real / Pemmin's Aura + Nyxbloom Ancient + a blue mana creature",
        groups: &[&[FREED, PEMMIN], &[NYX], &[BALLISTA]],
        ready: nyx_ready,
        mana: (0, ""),
        finish: None,
        xcost: &[(BALLISTA, 1)],
        steps: &[],
    },
    Combo {
        name: "Freed from the Real / Pemmin's Aura + Badgermole Cub + a blue mana creature",
        groups: &[&[FREED, PEMMIN], &[CUB], &[BALLISTA]],
        ready: cub_ready,
        mana: (0, ""),
        finish: None,
        xcost: &[(BALLISTA, 1)],
        steps: &[],
    },
];

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, CURIO)?.etb = Some(curio_etb);
    let c = r.card(db, ALARM)?;
    c.etb = Some(alarm_etb);
    c.prio = Some(|_, _, _| 0); // a combo piece only: it locks its own creatures too
    r.card(db, BREACH)?.cast = Some(breach_cast);
    let c = r.card(db, PATHWAY)?;
    c.land_etb = Some(pathway_etb);
    c.land_cols = Some(pathway_cols);
    let c = r.card(db, KOMA)?;
    c.self_regen = Some(koma_regen);
    c.each_combat = Some(koma_combat);
    at_once(c, crate::hooks::Event::EachCombat, true); // an activated ability, not a trigger
    r.card(db, NYX)?.extra_mana = Some(nyx_hooked);
    let c = r.card(db, WINNOWER)?;
    c.can_cast = Some(winnower_cast);
    c.grant_kw = Some(winnower_kw);
    c.etb = Some(winnower_etb);
    at_once(c, crate::hooks::Event::Etb, true);
    let c = r.card(db, ARCHAIC)?;
    c.cast = Some(archaic_cast);
    c.hand_options = Some(vastlands_opts);
    Ok(())
}
