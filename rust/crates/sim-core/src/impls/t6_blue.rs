//! Tier 6 batch `blue` (rust/agents/t6-batches.md): the blue and small utility cards of the Kraum & Tymna, Rograkh &
//! Silas and Kinnan decks. Cards only: no engine file is touched; what needs the engine is listed in
//! rust/agents/engine-needs-blue.md.
//!
//! - Borne Upon a Wind: draw, and the one-turn "cast spells as though they had flash" flag (`t6_lands::FLASH_FLAG`).
//! - Mnemonic Betrayal: opponents' graveyards are exiled and their nonland cards held in hand (any mana type) for the turn.
//! - Timetwister: every player's hand and graveyard shuffled into their library, then seven cards drawn.
//! - Demonic Counsel: a Demon search, or any card with delirium (four card types in the graveyard).
//! - Rain of Filth: sets the one-turn flag (`rain_of_filth_active`); the sacrifice-a-land mana source is an engine need.
//! - Sevinne's Reclamation: the best permanent card of mana value 3 or less returns; from the graveyard it is copied.
//! - Exhibition Tidecaller: Opus on the cast hook; the old compiled mill-3 trigger is removed (dsl = []).
//! - Voice of Victory: Mobilize 2 on its attack hook (the tokens carry `EotSac`); opponents' casts refused on its controller's turn (`can_cast`).
//! - Drift of Phantasms: Transmute {1}{U}{U} as a hand option (sorcery speed); defender, flying and the 0/5 body are data.
//! - Emrakul, the Promised End: costs {1} less per card type in your graveyard (`self_cost`); control of an opponent's turn, the extra turn and protection from instants are engine needs.
//! - Silas Renn: the combat damage trigger marks the best artifact card in the graveyard for the turn (a per-player turn flag); an option casts it from the graveyard in a main phase.
//! - Hidden Strings: untaps two of its own tapped mana sources (else taps down two opposing creatures); cipher is an engine need.

use crate::cards::{CardDb, Colors, Types};
use crate::engine::tutors::{card_worth, shuffle_library, tutor};
use crate::engine::cast::{cast_card, castable};
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::stack::trigger_window;
use crate::engine::values::{pval, untargetable};
use crate::engine::zones::{Enter, Tokens, discard_cards, agent_for, agent_take, draw, enter, make_tokens, max_by, mill, searchable};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

fn set_turn_flag(g: &mut Game, p: PlayerId, key: Sym) {
    let stamp = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(key, stamp);
}

/// does p hold the one-turn grant "you may cast spells this turn as though they had flash"? (Borne Upon a Wind;
/// shared with Emergence Zone). The engine's cast-timing check and the AI's end-of-turn window should read this.
pub fn flash_turn_grant(g: &Game, p: PlayerId) -> bool {
    g.player(p).flag_turn.get(crate::impls::t6_lands::FLASH_FLAG) == Some(&g.turn_stamp())
}

// ================================================================== Borne Upon a Wind
/// Borne Upon a Wind: you may cast spells this turn as though they had flash; draw a card
fn borne_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    set_turn_flag(g, p, crate::impls::t6_lands::FLASH_FLAG);
    draw(g, p, 1, false)?;
    Ok("gy")
}

// ================================================================== Mnemonic Betrayal
/// Mnemonic Betrayal: exile every opponent's graveyard; p may cast spells from among those cards this turn with mana
/// of any type (the cards are held in p's hand like Opposition Agent's, `yshtola::take_card`, and listed in
/// `impulse` so the end step clears what was not cast); Betrayal itself is exiled.
/// Missing: what remains goes back to its owner's graveyard (the end-step cleanup exiles it with p's own impulse cards).
fn betrayal_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in g.opps(p).collect::<Vec<_>>() {
        let gy = std::mem::take(&mut g.player_mut(q).gy);
        let n = gy.len();
        for c in gy {
            if g.db.get(c).land {
                g.player_mut(q).exile.push(c); // a land can't be cast
            } else {
                crate::impls::yshtola::take_card(g, p, c, q, "Mnemonic Betrayal");
                g.player_mut(p).impulse.push(c);
            }
        }
        crate::glog!(g, "    Mnemonic Betrayal exiles {} card(s) from {}'s graveyard", n, g.player(q).name);
    }
    Ok("exile")
}

// ================================================================== Timetwister
/// Timetwister: each player shuffles their hand and graveyard into their library, then draws seven cards; Timetwister
/// itself goes to the graveyard afterwards (it is not shuffled in). A card held from an opponent (Opposition Agent
/// style) goes into its owner's library.
fn timetwister_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut order = vec![p];
    order.extend(g.after(p).filter(|&q| g.player(q).alive));
    for &q in &order {
        let hand = std::mem::take(&mut g.player_mut(q).hand);
        let gy = std::mem::take(&mut g.player_mut(q).gy);
        for c in hand.into_iter().chain(gy) {
            let owner = g.player_mut(q).stolen.swap_remove(&c).unwrap_or(q);
            g.player_mut(q).agent_ids.retain(|&x| x != c);
            g.player_mut(q).impulse.retain(|&x| x != c);
            g.player_mut(owner).library.push(c);
        }
    }
    for &q in &order {
        shuffle_library(g, q);
    }
    for &q in &order {
        draw(g, q, 7, false)?;
        if g.over {
            break;
        }
    }
    Ok("gy")
}


// ================================================================== Demonic Counsel
/// Demonic Counsel: search for a Demon card (any card with delirium: four or more card types in your graveyard),
/// reveal it, put it into your hand, then shuffle. Delirium searches use the deck's own tutor pick; a plain Demon
/// search takes the Demon the deck would value most (a card taken by Opposition Agent follows its rule).
fn counsel_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if crate::impls::t6_green2::delirium(g, p) {
        tutor(g, p, "any")?;
        return Ok("gy");
    }
    let is_demon = |g: &Game, c: CardId| g.db.get(c).has_subtype("demon");
    if crate::human::is_human(g, p) {
        let found = crate::human::choices::search(g, p, &|g: &Game, c: CardId| is_demon(g, c), 1, "Search your library for a Demon", true)?;
        for c in found {
            let pl = g.player_mut(p);
            pl.hand.push(c);
            pl.stat("tutored", 1);
            pl.seen.insert(c);
        }
        return Ok("gy");
    }
    let cands: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| is_demon(g, c)).collect();
    if let Some(c) = max_by(&cands, |c| card_worth(g, p, c, false)) {
        let lib = &mut g.player_mut(p).library;
        let i = lib.iter().position(|&x| x == c).unwrap();
        lib.remove(i);
        if let Some(a) = agent_for(g, p) {
            agent_take(g, a, p, c);
        } else {
            let pl = g.player_mut(p);
            pl.hand.push(c);
            pl.stat("tutored", 1);
            pl.seen.insert(c);
            crate::glog!(g, "    {} tutors {} (Demonic Counsel)", g.player(p).name, g.db.get(c).name);
        }
    }
    shuffle_library(g, p);
    Ok("gy")
}


// ================================================================== Rain of Filth
const RAIN_FLAG: &str = "rain_of_filth";

/// have p's lands "Sacrifice this land: Add {B}" until end of turn (Rain of Filth)? The mana code (`total_mana`, the
/// unit list `pay` uses) should add one {B} unit per land p controls, tapped or not, paid by sacrificing the land.
pub fn rain_of_filth_active(g: &Game, p: PlayerId) -> bool {
    g.player(p).flag_turn.get(RAIN_FLAG) == Some(&g.turn_stamp())
}

/// Rain of Filth: until end of turn, lands you control gain "Sacrifice this land: Add {B}" (sets the turn flag; the
/// engine has no sacrifice-a-land mana source yet)
fn rain_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    set_turn_flag(g, p, RAIN_FLAG);
    Ok("gy")
}


// ================================================================== Sevinne's Reclamation
/// the permanent card with mana value 3 or less in p's graveyard the AI returns: creatures by power, then other
/// permanents by mana value; a land only when nothing else is there
fn reclaim_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let cs: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&c| (g.db.get(c).perm || g.db.get(c).land) && g.db.get(c).cmc <= 3).collect();
    let key = |c: CardId| {
        let d = g.db.get(c);
        (!d.land) as i32 as f64 * 10000.0 + d.creature as i32 as f64 * 1000.0 + d.pow as f64 * 10.0 + d.cmc as f64
    };
    max_by(&cs, key)
}

/// put the chosen permanent card from p's graveyard onto the battlefield (a land enters like a played one, without
/// using the land drop)
fn reclaim_return(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(x) = reclaim_pick(g, p) else { return Ok(false) };
    let gy = &mut g.player_mut(p).gy;
    let i = gy.iter().position(|&c| c == x).unwrap();
    gy.remove(i);
    if g.db.get(x).land {
        let tapped = crate::engine::turn::land_enters_tapped(g, p, x);
        let l = g.add_land(p, x, tapped);
        crate::cardcode::land_etb(g, p, l)?;
        crate::engine::zones::landfall(g, p)?;
    } else {
        enter(g, p, x, Enter::default())?;
    }
    crate::glog!(g, "    Sevinne's Reclamation returns {}", g.db.get(x).name);
    Ok(true)
}

/// Sevinne's Reclamation: return target permanent card with mana value 3 or less from your graveyard to the
/// battlefield; cast from a graveyard (flashback), the AI copies it and the copy returns a second card
fn reclamation_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    reclaim_return(g, p)?;
    if ctx.from_gy {
        reclaim_return(g, p)?;
    }
    Ok("gy")
}


// ================================================================== Exhibition Tidecaller
/// the mana spent on the spell being cast (c): its mana value plus X, or nothing when an alternative cost was paid.
/// The engine records no real payment total, so cost reductions are not seen (see engine-needs-blue.md).
fn mana_spent_on(g: &Game, c: CardId) -> i32 {
    match &g.cur_cast {
        Some((cc, ctx, _)) if *cc == c => {
            if ctx.paid_otherwise {
                0
            } else {
                g.db.get(c).cmc as i32 + ctx.x.max(0)
            }
        }
        _ => g.db.get(c).cmc as i32,
    }
}

/// Exhibition Tidecaller (Opus): whenever you cast an instant or sorcery spell, target player mills three cards; if
/// five or more mana was spent to cast it, that player mills ten instead. The AI aims it at the opposing player it
/// most wants to hurt (the DSL's "target player" rule: highest threat, not hexproof).
fn tidecaller_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != o || !(d.instant || d.sorcery) {
        return Ok(());
    }
    let n = if mana_spent_on(g, c) >= 5 { 10 } else { 3 };
    if !trigger_window(g, o, Some(src), &format!("mill {n}"), Some(2.0))? {
        return Ok(());
    }
    let Some(&t) = crate::dsl::players(g, o, Some("target_player"), &crate::dsl::DslCtx::default()).first() else {
        return Ok(());
    };
    crate::glog!(g, "    Exhibition Tidecaller: {} mills {}", g.player(t).name, n);
    mill(g, t, n)
}


// ================================================================== Voice of Victory
/// Voice of Victory (Mobilize 2): whenever it attacks, two tapped and attacking 1/1 red Warrior tokens, sacrificed at
/// the beginning of the next end step (`DataKey::EotSac`, as Zurgo's, `t0_zurgo::zurgo_attack`)
fn voice_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p || !atk.contains(&src) || !g.perm(src).on_bf {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "mobilize 2: two tapped and attacking Warriors", Some(4.0))? {
        return Ok(vec![]);
    }
    let dd = g.atk_targets.iter().find(|x| x.0 == src).map_or(d, |x| x.1);
    let spec = Tokens {
        attacking: true,
        sick: false,
        warrior: true,
        color: Some(Colors::from_letters("R")),
        types: vec!["warrior"],
        ..Tokens::new(2, 1)
    };
    crate::glog!(g, "    {} mobilizes: two tapped and attacking Warriors", g.perm(src).name);
    let toks = make_tokens(g, p, spec)?;
    for &t in &toks {
        g.perm_mut(t).data.set(DataKey::EotSac, Val::Bool(true));
        g.atk_targets.push((t, dd));
    }
    Ok(toks)
}

/// Voice of Victory: your opponents can't cast spells during your turn
fn voice_can_cast(g: &Game, src: Src, p: PlayerId, _c: CardId, _zone: Sym) -> bool {
    let o = g.perm(src).owner;
    !(p != o && g.active == Some(o) && g.perm(src).on_bf && !g.perm(src).phased)
}


// ================================================================== High Fae Trickster / Valley Floodcaller: flash
/// may p cast a spell as though it had flash because of a permanent (High Fae Trickster: every spell; Valley
/// Floodcaller: noncreature spells), or the one-turn grant of Borne Upon a Wind? `creature_spell` says whether the
/// spell is a creature. The engine's cast-timing check and the AI's end-of-turn / in-response casting should read
/// this (engine-needs-blue.md).
pub fn static_flash(g: &Game, p: PlayerId, creature_spell: bool) -> bool {
    flash_turn_grant(g, p)
        || g.player(p).perms.iter().any(|&m| {
            let x = g.perm(m);
            x.on_bf
                && !x.phased
                && match x.cd.map(|c| &*g.db.get(c).name) {
                    Some("High Fae Trickster") => true,
                    Some("Valley Floodcaller") => !creature_spell,
                    _ => false,
                }
        })
}


// ================================================================== Drift of Phantasms
/// the card with mana value 3 in p's library the AI transmutes for: the best by `card_worth` (no land: a land has
/// mana value 0); None if there is none
fn transmute_pick(g: &Game, p: PlayerId, mv: u32) -> Option<CardId> {
    let cs: Vec<CardId> =
        searchable(g, p).into_iter().filter(|&c| !g.db.get(c).land && g.db.get(c).cmc == mv).collect();
    max_by(&cs, |c| card_worth(g, p, c, false))
}

/// Transmute {1}{U}{U} (sorcery speed, from the hand): the AI does it in its own main phase when a card with the same
/// mana value is worth more than the 0/5 wall
fn drift_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.stack.is_empty() || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 1, "UU", false) {
        return Ok(vec![]);
    }
    let mv = g.db.get(c).cmc;
    let Some(t) = transmute_pick(g, p, mv) else { return Ok(vec![]) };
    let v = card_worth(g, p, t, false) / 10.0 + 0.2;
    if v < 0.6 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: v,
        label: "transmute Drift of Phantasms".into(),
        act: Some(Action::Plan { f: drift_transmute, arg: c.0 as i64 }),
    }])
}

fn drift_transmute(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "UU", false) {
        return Ok(false);
    }
    pay(g, p, 1, "UU", false)?;
    crate::glog!(g, "  {} transmutes Drift of Phantasms", g.player(p).name);
    discard_cards(g, p, &[c])?;
    let mv = g.db.get(c).cmc;
    if let Some(t) = transmute_pick(g, p, mv) {
        let lib = &mut g.player_mut(p).library;
        let i = lib.iter().position(|&x| x == t).unwrap();
        lib.remove(i);
        if let Some(a) = agent_for(g, p) {
            agent_take(g, a, p, t);
        } else {
            let pl = g.player_mut(p);
            pl.hand.push(t);
            pl.stat("tutored", 1);
            pl.seen.insert(t);
            crate::glog!(g, "    {} reveals {} (transmute)", g.player(p).name, g.db.get(t).name);
        }
    }
    shuffle_library(g, p);
    Ok(true)
}


// ================================================================== Emrakul, the Promised End
/// the number of card types among the cards in p's graveyard (land, creature, instant, sorcery, artifact,
/// enchantment, planeswalker: the engine's card data has no battle or kindred type)
pub fn graveyard_types(g: &Game, p: PlayerId) -> u32 {
    let mut seen = 0u8;
    for &c in &g.player(p).gy {
        let d = g.db.get(c);
        for (on, bit) in [
            (d.land, 1u8),
            (d.creature, 2),
            (d.instant, 4),
            (d.sorcery, 8),
            (d.types.has(Types::ARTIFACT), 16),
            (d.types.has(Types::ENCHANTMENT), 32),
            (d.types.has(Types::PLANESWALKER), 64),
        ] {
            if on {
                seen |= bit;
            }
        }
    }
    seen.count_ones()
}

/// Emrakul costs {1} less to cast for each card type among cards in your graveyard
fn emrakul_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    -(graveyard_types(g, p) as i32)
}


// ================================================================== Silas Renn, Seeker Adept
fn silas_key(c: CardId) -> Sym {
    intern(&format!("silas_{}", c.0))
}

/// has Silas Renn chosen artifact card c in p's graveyard this turn ("you may cast that card this turn")?
pub fn silas_chosen(g: &Game, p: PlayerId, c: CardId) -> bool {
    g.player(p).flag_turn.get(silas_key(c)) == Some(&g.turn_stamp()) && g.player(p).gy.contains(&c)
}

/// how much the AI wants an artifact card from p's graveyard cast again (the deck's priority, then the card's worth)
fn silas_value(g: &Game, p: PlayerId, c: CardId) -> f64 {
    crate::ai::decks::deck_prio_f(g, p, c) / 10.0 + 0.3 * card_worth(g, p, c, true) / 10.0
}

/// Silas Renn: whenever it deals combat damage to a player, choose target artifact card in your graveyard; you may
/// cast that card this turn. The AI picks the artifact card it values most and marks it for the turn; `silas_options`
/// casts it from the graveyard in a main phase (artifacts are sorcery speed).
fn silas_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a != src || g.perm(src).owner != p {
        return Ok(());
    }
    let cands: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).types.has(Types::ARTIFACT) && !g.db.get(c).land)
        .collect();
    let Some(c) = max_by(&cands, |c| silas_value(g, p, c)) else { return Ok(()) };
    if !trigger_window(g, p, Some(src), "choose an artifact card in your graveyard to cast", Some(3.0))? {
        return Ok(());
    }
    let st = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(silas_key(c), st);
    crate::glog!(g, "    Silas Renn: {} may cast {} from the graveyard this turn", g.player(p).name, g.db.get(c).name);
    Ok(())
}

fn silas_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || g.perm(src).owner != p || !g.stack.is_empty() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for c in g.player(p).gy.clone() {
        if !silas_chosen(g, p, c) || !castable(g, p, c, "gy") {
            continue;
        }
        let (gn, pips) = cost_of(g, p, c);
        if !can_pay(g, p, gn, &pips, false) {
            continue;
        }
        let v = silas_value(g, p, c);
        if v <= 0.4 {
            continue;
        }
        out.push(Opt {
            utility: v,
            label: format!("Silas Renn: cast {} from the graveyard", g.db.get(c).name),
            act: Some(Action::Plan { f: silas_cast, arg: c.0 as i64 }),
        });
    }
    Ok(out)
}

fn silas_cast(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (gn, pips) = cost_of(g, p, c);
    if !silas_chosen(g, p, c) || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    g.player_mut(p).flag_turn.shift_remove(silas_key(c));
    g.pay_for = Some(c);
    let r = pay(g, p, gn, &pips, false);
    g.pay_for = None;
    r?;
    crate::glog!(g, "  {} casts {} from the graveyard (Silas Renn)", g.player(p).name, g.db.get(c).name);
    cast_card(g, p, c, "mgy", Ctx::default())?;
    Ok(true)
}


// ================================================================== Hidden Strings
/// p's tapped mana sources, the best first: a rock (more mana first), a dork, then lands
fn tapped_mana_sources(g: &Game, p: PlayerId) -> Vec<(Option<PermId>, Option<crate::ids::LandId>)> {
    let mut out = vec![];
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        let Some(cd) = x.cd else { continue };
        let d = g.db.get(cd);
        if x.on_bf && !x.phased && x.tapped && (d.tags.has(Tag::Rock) || d.tags.has(Tag::Dork)) {
            out.push((Some(m), None));
        }
    }
    for &l in &g.player(p).lands {
        if g.land(l).tapped {
            out.push((None, Some(l)));
        }
    }
    out
}

/// The AI casts it when two of its mana sources are tapped (untapping both pays back the {1}{U} and then some)
fn strings_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if tapped_mana_sources(g, p).len() >= 3 { 40 } else { 0 }
}

/// Hidden Strings: you may tap or untap target permanent, then you may tap or untap another target permanent. The
/// AI untaps two of its own tapped mana sources; with none tapped it taps down the two best untapped creatures
/// opponents control. Cipher (encoding the card on a creature, a free copy on combat damage) is an engine need.
fn strings_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mine = tapped_mana_sources(g, p);
    if !mine.is_empty() {
        for (m, l) in mine.into_iter().take(2) {
            if let Some(m) = m {
                g.perm_mut(m).tapped = false;
                crate::glog!(g, "    Hidden Strings untaps {}", g.perm(m).name);
            }
            if let Some(l) = l {
                g.land_mut(l).tapped = false;
                crate::glog!(g, "    Hidden Strings untaps {}", g.db.get(g.land(l).cd).name);
            }
        }
        g.bf_ver += 1;
        return Ok("gy");
    }
    let mut theirs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&m| g.perm(m).on_bf && !g.perm(m).tapped && g.is_creature(m) && !untargetable(g, m))
        .collect();
    theirs.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
    for m in theirs.into_iter().take(2) {
        g.perm_mut(m).tapped = true;
        crate::glog!(g, "    Hidden Strings taps {}", g.perm(m).name);
    }
    Ok("gy")
}


// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Hidden Strings")?;
    c.resolve = Some(strings_resolve);
    c.prio = Some(strings_prio);
    let c = r.card(db, "Silas Renn, Seeker Adept")?;
    c.combat_damage = Some(silas_damage);
    c.options = Some(silas_options);
    r.card(db, "Emrakul, the Promised End")?.self_cost = Some(emrakul_cost);
    r.card(db, "Drift of Phantasms")?.hand_options = Some(drift_options);
    let c = r.card(db, "Voice of Victory")?;
    c.attack = Some(voice_attack);
    c.can_cast = Some(voice_can_cast);
    r.card(db, "Exhibition Tidecaller")?.cast = Some(tidecaller_cast);
    r.card(db, "Sevinne's Reclamation")?.resolve = Some(reclamation_resolve);
    r.card(db, "Rain of Filth")?.resolve = Some(rain_resolve);
    r.card(db, "Demonic Counsel")?.resolve = Some(counsel_resolve);
    r.card(db, "Timetwister")?.resolve = Some(timetwister_resolve);
    r.card(db, "Borne Upon a Wind")?.resolve = Some(borne_resolve);
    r.card(db, "Mnemonic Betrayal")?.resolve = Some(betrayal_resolve);
    Ok(())
}
