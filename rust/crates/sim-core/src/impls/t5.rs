//! Python's `cards/impl/t5.py`: the Tier 5 pool decks (Urza, Kinnan, Winota, Zur, Yawgmoth); the combos themselves
//! are `combos.rs`.

use super::common::{WalkerAb, best_opp_nonland, death_value};
use super::partials::{at_once, eot_kw, first_max, first_min, on, remove_card};
use super::t2::{blink, blink_value};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable, discard_worst};
use crate::engine::life::lose_life;
use crate::engine::mana::{Source, Unit, can_pay, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library, tutor, tutor_to_top};
use crate::engine::values::{epow, etgh, has_type, pval, threat, untargetable};
use crate::engine::zones::{
    Enter, Tokens, die, discard_cards, draw, enter, enter_token_copy, leave, make_tokens, max_by, mill, minus_counter,
    searchable,
};
use crate::flow::Res;
use crate::hooks::{Action, Assign, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, PermData, Val};
use crate::sym::Sym;
use crate::tag::Tag;

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

/// a permanent's card name ("" for a token without a card)
fn cname(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

/// `c.bomb or c.pow`
fn bomb_or_pow(g: &Game, c: CardId) -> i32 {
    let d = g.db.get(c);
    if d.bomb != 0 { d.bomb } else { d.pow }
}

fn is_art_card(g: &Game, c: CardId) -> bool {
    g.db.get(c).types.has(Types::ARTIFACT)
}

/// an activated ability of src for the AI's option list
fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

/// t5.is_artifact: an artifact card, or a token made as an artifact (a Construct, a Thopter)
pub fn is_artifact(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.cd.is_some_and(|c| is_art_card(g, c)) || x.data.truthy(DataKey::Artifact)
}

/// the top n cards of p's library, top first (Python's `[p.library.pop() for _ in range(min(n, len))]`)
fn pop_top(g: &mut Game, p: PlayerId, n: usize) -> Vec<CardId> {
    let lib = &mut g.player_mut(p).library;
    let k = n.min(lib.len());
    (0..k).map(|_| lib.pop().unwrap()).collect()
}

/// the rest go to the bottom in a random order (Python's `g.rng.shuffle(top); p.library[:0] = top`)
fn bottom_random(g: &mut Game, p: PlayerId, mut top: Vec<CardId>) {
    g.rng.shuffle(&mut top);
    g.player_mut(p).library.splice(0..0, top);
}

// ======================================================== Urza, Lord High Artificer
/// Construct token (+1/+1 per artifact)
fn urza(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
        let o = owner(g, src);
        if !trigger_window(g, o, Some(src), "create a Construct", None)? {
            return Ok(());
        }
        let mut data = PermData::default();
        data.set(DataKey::Construct, Val::Bool(true));
        data.set(DataKey::Artifact, Val::Bool(true));
        let spec = Tokens {
            tgh: Some(0),
            color: Some(Colors::from_letters("")),
            types: vec!["construct"],
            data,
            ..Tokens::new(1, 0)
        };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

/// artifacts tap for U
fn urza_mana(g: &Game, _src: Src, p: PlayerId, u: &[Unit]) -> Vec<Unit> {
    let used: Vec<PermId> = u.iter().filter_map(|w| if let Source::Perm(m) = w.src { Some(m) } else { None }).collect();
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| is_artifact(g, m) && !g.perm(m).tapped && !g.perm(m).phased && !used.contains(&m))
        .map(|m| Unit { src: Source::Perm(m), cols: Colors::from_letters("U"), amt: 1 })
        .collect()
}

/// {5}: shuffle, then play the top card for free
fn urza_five(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(post) = post else { return Ok(vec![]) };
    if !can_pay(g, p, 5, "", false) || g.player(p).library.len() < 5 {
        return Ok(vec![]);
    }
    Ok(vec![ability(if post { 1.5 } else { 0.8 }, "Urza: free card".into(), src, urza_go, 0)])
}

fn urza_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 5, "", false) {
        return Ok(false);
    }
    pay(g, p, 5, "", false)?;
    if !ability_window(g, p, Some(src), "shuffle, play the top card free", None, None)?
        || g.player(p).library.is_empty()
    {
        return Ok(true);
    }
    shuffle_library(g, p);
    let c = g.player_mut(p).library.pop().unwrap();
    crate::glog!(g, "  Urza: {} off the top, free", g.db.get(c).name);
    let d = g.db.get(c);
    if d.land {
        if g.player(p).lands_played < 1 {
            crate::engine::turn::play_land_card(g, p, c, "plays")?;
        } else {
            g.player_mut(p).exile.push(c);
        }
    } else if castable(g, p, c, "hand") && !d.tag(Tag::Ctr) {
        g.player_mut(p).hand.push(c);
        cast_card(g, p, c, "hand", Ctx::default())?;
    } else {
        g.player_mut(p).exile.push(c);
    }
    Ok(true)
}

/// common.TOKEN_PT entry: Urza's Construct gets +1/+1 per artifact its controller has
fn construct_bonus(g: &Game, m: PermId) -> (i32, i32) {
    if g.perm(m).data.truthy(DataKey::Construct) {
        let o = owner(g, m);
        let n = g.player(o).perms.iter().filter(|&&x| is_artifact(g, x) && !g.perm(x).phased).count() as i32;
        return (n, n);
    }
    (0, 0)
}

/// opponents' artifacts can't activate (mana rocks, Treasures)
fn karn_static(g: &Game, src: Src, p: PlayerId) -> i32 {
    (p != owner(g, src)) as i32
}

/// the Lattice lock: opponents make no mana
fn karn_lattice(g: &Game, src: Src, p: PlayerId) -> i32 {
    let o = owner(g, src);
    (p != o
        && g.lattice_lock == Some(o)
        && g.players.iter().any(|q| q.perms.iter().any(|&m| cname(g, m) == "Mycosynth Lattice"))) as i32
}

/// -2: an artifact card from exile to hand (the wish from outside the game is modeled as from exile)
fn karn_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(c) = g.player(p).exile.iter().copied().find(|&c| is_art_card(g, c)) {
        let pl = g.player_mut(p);
        remove_card(&mut pl.exile, c);
        pl.hand.push(c);
    }
    Ok(())
}

/// opponents' artifacts can't activate (mana rocks, Treasures); the wish from outside the game is modeled as from
/// exile; the Lattice lock is a combo
pub const KARN: [WalkerAb; 2] = [
    WalkerAb { delta: 1, label: "animate an artifact", val: |_, _, _| Some(1.0), eff: |_, _, _| Ok(()) },
    WalkerAb {
        delta: -2,
        label: "artifact from exile",
        val: |g, p, _| if g.player(p).exile.iter().any(|&c| is_art_card(g, c)) { Some(2.0) } else { None },
        eff: karn_minus,
    },
];
static KARN_ABS: [WalkerAb; 2] = KARN;

/// t5.IC_COMBO_ART: the combo artifacts tutors prefer (rules2's and fixes' tutors read it too)
pub const IC_COMBO_ART: [&str; 6] = [
    "Isochron Scepter",
    "Power Artifact",
    "Basalt Monolith",
    "Grim Monolith",
    "Mycosynth Lattice",
    "Rings of Brighthearth",
];

/// t5._tezz_minus: -X (all its loyalty): an artifact with mana value X or less onto the battlefield
fn tezz_minus(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let x = g.perm(src).loyalty.unwrap_or(0);
    let cs: Vec<CardId> =
        searchable(g, p).into_iter().filter(|&c| is_art_card(g, c) && g.db.get(c).cmc as i32 <= x).collect();
    let pref = ["Isochron Scepter", "Power Artifact", "Basalt Monolith", "Grim Monolith", "Mycosynth Lattice"];
    if let Some(c) =
        first_max(&cs, |c| (pref.contains(&&*g.db.get(c).name), card_worth(g, p, c, false), g.db.get(c).cmc))
    {
        remove_card(&mut g.player_mut(p).library, c);
        shuffle_library(g, p);
        enter(g, p, c, Enter::default())?;
    }
    Ok(())
}

/// +1: untap two artifacts
fn tezz_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let ms: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| is_artifact(g, m) && g.perm(m).tapped).take(2).collect();
    for m in ms {
        g.perm_mut(m).tapped = false;
    }
    Ok(())
}

/// -X uses all its loyalty as X; the ultimate is not used (rules.py adds one: register the whole list there)
pub const TEZZERET: [WalkerAb; 2] = [
    WalkerAb { delta: 1, label: "untap two artifacts", val: |_, _, _| Some(2.0), eff: tezz_plus },
    WalkerAb {
        delta: -2,
        label: "artifact onto the battlefield",
        val: |g, p, src| {
            let l = g.perm(src).loyalty.unwrap_or(0);
            let any = g.player(p).library.iter().any(|&c| is_art_card(g, c) && g.db.get(c).cmc as i32 <= l);
            if any { Some(3.0) } else { None }
        },
        eff: tezz_minus,
    },
];
static TEZZERET_ABS: [WalkerAb; 2] = TEZZERET;

/// +1/+1 counter and can't be blocked whenever another artifact enters under your control
fn kappa(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if owner(g, m) == o && is_artifact(g, m) && m != src {
        if !trigger_window(g, o, Some(src), "a +1/+1 counter, can't be blocked", Some(2.0))? {
            return Ok(());
        }
        g.perm_mut(src).plus += 1;
        eot_kw(g, src, "unblockable");
    }
    Ok(())
}

/// Thopter per artifact spell; the sacrifice-for-cards ability is not used
fn sai(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster == owner(g, src)
        && is_art_card(g, c)
        && trigger_window(g, caster, Some(src), "create a 1/1 Thopter", None)?
    {
        let spec =
            Tokens { fly: true, color: Some(Colors::from_letters("")), types: vec!["thopter"], ..Tokens::new(1, 1) };
        for t in make_tokens(g, caster, spec)? {
            let mut data = PermData::default();
            data.set(DataKey::Artifact, Val::Bool(true));
            g.perm_mut(t).data = data;
        }
    }
    Ok(())
}

/// common._reducer for Foundry Inspector and Etherium Sculptor: your artifact spells cost {1} less
fn art_reducer(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    if caster == owner(g, src) && is_art_card(g, c) { -1 } else { 0 }
}

/// common._reducer for Jet Medallion: your black spells cost {1} less
fn jet(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    if caster == owner(g, src) && g.db.get(c).pips.contains('B') { -1 } else { 0 }
}

/// common._tutor_etb for Trophy Mage and Tribute Mage: on entry, search for an artifact of the right mana value
fn mage_tutor(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m == src && trigger_window(g, o, Some(src), "search for a card", None)? {
        let cmc = if cname(g, src) == "Trophy Mage" { 3 } else { 2 };
        super::t1::tutor_named(g, o, &|g, c| is_art_card(g, c) && g.db.get(c).cmc == cmc, 1, "hand")?;
    }
    Ok(())
}

/// creatures' mana abilities are off for everyone; other creature activations (Krenko, Kinnan, Yawgmoth) are not
/// blocked
fn totem(_g: &Game, _src: Src, _p: PlayerId) -> i32 {
    1
}

/// X = spare mana; the best artifact onto the battlefield (improvise not used).
/// Not registered: in Python rules2.py's Whir of Invention (its resolve and priority) replaces this one.
#[allow(dead_code)]
fn whir(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x;
    let cs: Vec<CardId> =
        searchable(g, p).into_iter().filter(|&y| is_art_card(g, y) && g.db.get(y).cmc as i32 <= x).collect();
    if let Some(y) =
        first_max(&cs, |y| (IC_COMBO_ART.contains(&&*g.db.get(y).name), card_worth(g, p, y, false), g.db.get(y).cmc))
    {
        remove_card(&mut g.player_mut(p).library, y);
        shuffle_library(g, p);
        enter(g, p, y, Enter::default())?;
    }
    Ok("gy")
}

#[allow(dead_code)]
fn whir_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if total_mana(g, p, false) >= 5 { 55 } else { 0 }
}

/// each player discards their hand and draws as many as the largest hand
fn windfall(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let qs: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    let n = qs.iter().map(|&q| g.player(q).hand.len() as i64 - if q == p { 1 } else { 0 }).max().unwrap_or(0);
    for &q in &qs {
        let cards: Vec<CardId> = g.player(q).hand.iter().copied().filter(|&x| x != c).collect();
        discard_cards(g, q, &cards)?;
    }
    for &q in &qs {
        draw(g, q, n.max(0) as u32, false)?;
    }
    Ok("gy")
}

fn windfall_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).hand.len() <= 2 { 50 } else { 0 }
}

// ======================================================== Kinnan, Bonder Prodigy
/// nonland mana sources make one extra
fn kinnan(g: &Game, src: Src, p: PlayerId) -> i32 {
    (p == owner(g, src)) as i32
}

/// {5}{G}{U}: a non-Human creature from the top five onto the battlefield
fn kinnan_act(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(post) = post else { return Ok(vec![]) };
    if !can_pay(g, p, 5, "GU", false) {
        return Ok(vec![]);
    }
    let u = 3.0 + if post { 2.0 } else { 0.0 };
    Ok(vec![ability(u, "Kinnan: dig five".into(), src, kinnan_go, 0)])
}

fn kinnan_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 5, "GU", false) {
        return Ok(false);
    }
    pay(g, p, 5, "GU", false)?;
    if !ability_window(g, p, Some(src), "look at the top five", None, None)? {
        return Ok(true);
    }
    let mut top = pop_top(g, p, 5);
    let hits: Vec<CardId> =
        top.iter().copied().filter(|&c| g.db.get(c).creature && !g.db.get(c).has_subtype("human")).collect();
    if let Some(c) = first_max(&hits, |c| (bomb_or_pow(g, c), g.db.get(c).cmc)) {
        remove_card(&mut top, c);
        enter(g, p, c, Enter::default())?;
    }
    bottom_random(g, p, top);
    Ok(true)
}

/// one mana per colour among your permanents
fn bloom_tender(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    let mut cols = Colors::NONE;
    for &x in &g.player(p).perms {
        if let Some(c) = g.perm(x).cd {
            cols = cols.union(Colors::from_letters(&g.db.get(c).pips));
        }
    }
    (cols.count() as u32).max(1)
}

/// three mana once it has a counter (adapt not activated)
fn incubation(g: &Game, _p: PlayerId, m: PermId) -> u32 {
    if g.perm(m).plus > 0 { 3 } else { 1 }
}

/// untaps your permanents in each other player's untap step ("doesn't untap during your untap step" (Grim Monolith)
/// doesn't stop it here)
fn seedborn(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if p == o {
        return Ok(());
    }
    for l in g.player(o).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    for m in g.player(o).perms.clone() {
        g.perm_mut(m).tapped = false;
    }
    Ok(())
}

/// a creature on top of the library: the wish list first, cast at the end of the turn before yours
fn worldly(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    tutor_to_top(g, p, "cre", 0)?;
    Ok("gy")
}

fn worldly_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    crate::ai::pool::tutor_prio(g, p, "cre", 0)
}

/// X = spare mana, creature onto the battlefield; harmonize not used
fn rhythm(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x;
    super::t3::put_creature(g, p, |g, y| g.db.get(y).cmc as i32 <= x, true)?;
    Ok("gy")
}

fn rhythm_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if total_mana(g, p, false) >= 5 { 55 } else { 0 }
}

// ======================================================== Winota, Joiner of Forces
/// each non-Human attacker: a Human from the top six onto the battlefield attacking and indestructible (ETBs fire)
fn winota(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p {
        return Ok(vec![]);
    }
    if !atk.iter().any(|&m| !has_type(g, m, "human")) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "look at the top six for a Human", Some(6.0))? {
        return Ok(vec![]);
    }
    let mut new = vec![];
    let non: Vec<PermId> = atk.iter().copied().filter(|&m| !has_type(g, m, "human")).collect();
    for _a in non {
        let mut top = pop_top(g, p, 6);
        let hum: Vec<CardId> =
            top.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).has_subtype("human")).collect();
        if let Some(c) = first_max(&hum, |c| (bomb_or_pow(g, c), g.db.get(c).cmc)) {
            remove_card(&mut top, c);
            let m = enter(g, p, c, Enter::default())?;
            let x = g.perm_mut(m);
            x.tapped = true;
            x.sick = false;
            eot_kw(g, m, "indestructible");
            new.push(m);
            crate::glog!(g, "    Winota puts {} onto the battlefield attacking", g.db.get(c).name);
        }
        bottom_random(g, p, top);
        if g.over {
            break;
        }
    }
    Ok(new)
}

/// hasty copy of your best nonlegendary creature each turn (sacrificed at end); the infinite loops are combos
fn kiki(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).tapped || post != Some(false) {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && g.perm(m).cd.is_some_and(|c| !g.db.get(c).tag(Tag::Leg)) && m != src)
        .collect();
    let Some(t) = max_by(&cands, |m| blink_value(g, p, m) as f64 + epow(g, m) as f64 * 0.5) else {
        return Ok(vec![]);
    };
    let u = 1.5 + 0.5 * blink_value(g, p, t) as f64 + 0.3 * epow(g, t) as f64;
    Ok(vec![ability(u, format!("Kiki-Jiki copies {}", g.perm(t).name), src, kiki_go, t.0 as i64)])
}

fn kiki_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if g.perm(src).tapped || !on(g, p, t) {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    let label = format!("copy {}", g.perm(t).name);
    if !ability_window(g, p, Some(src), &label, None, Some(t))? || !on(g, p, t) {
        return Ok(true);
    }
    let cd = g.perm(t).cd.unwrap();
    if let Some(c) = enter_token_copy(g, p, cd)? {
        let x = g.perm_mut(c);
        x.sick = false;
        x.temp = true;
    }
    Ok(true)
}

/// blinks your best ETB permanent (Kiki loop is a combo)
fn felidar(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let mut cands: Vec<PermId> = g
        .player(o)
        .perms
        .iter()
        .copied()
        .filter(|&x| {
            // the loop is a combo, not a value blink
            x != src
                && !g.perm(x).token
                && g.perm(x).cd.is_some()
                && blink_value(g, o, x) > 0
                && !["Felidar Guardian", "Restoration Angel"].contains(&cname(g, x))
        })
        .collect();
    if cands.is_empty() || !trigger_window(g, o, Some(src), "blink a permanent", None)? {
        return Ok(());
    }
    cands.retain(|&x| on(g, o, x));
    if let Some(t) = max_by(&cands, |x| blink_value(g, o, x) as f64) {
        blink(g, o, t)?;
    }
    Ok(())
}

/// steals the best opposing creature until end of turn (untapped, hasty)
fn conscripts(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let Some(t) = best_opp_nonland(g, o, |x| g.is_creature(x)) else { return Ok(()) };
    if pval(g, t) >= 3.0 && !g.perm(t).is_cmd {
        let label = format!("gain control of {}", g.perm(t).name);
        if !trigger_window(g, o, Some(src), &label, Some(6.0))? || !g.perm(t).on_bf {
            return Ok(());
        }
        let (q, cd) = (owner(g, t), g.perm(src).cd);
        // targeted: can be answered
        if crate::ai::protect_response(g, q, t, "steal", Some(o), cd)? || !g.perm(t).on_bf {
            return Ok(());
        }
        let q = owner(g, t);
        g.player_mut(q).perms.retain(|&x| x != t);
        let x = g.perm_mut(t);
        x.owner = o;
        x.tapped = false;
        x.sick = false;
        g.player_mut(o).perms.push(t);
        g.bf_ver += 1;
        g.player_mut(o).borrowed.push(t);
    }
    Ok(())
}

/// the defending player reveals the top card: a land goes to their hand
fn guide(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let top_land = |g: &Game| g.player(d).library.last().is_some_and(|&c| g.db.get(c).land);
    if atk.contains(&src)
        && top_land(g)
        && trigger_window(g, p, Some(src), "defending player reveals the top card", Some(2.0))?
        && top_land(g)
    {
        let pl = g.player_mut(d);
        let c = pl.library.pop().unwrap();
        pl.hand.push(c);
    }
    Ok(vec![])
}

/// destroys an artifact/enchantment on entry; the draw ability is not used
fn loran(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let t = best_opp_nonland(g, o, |x| {
        g.perm(x)
            .cd
            .is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT) || g.db.get(c).types.has(Types::ENCHANTMENT))
    });
    let Some(t) = t else { return Ok(()) };
    if pval(g, t) < 2.0 {
        return Ok(());
    }
    let label = format!("destroy {}", g.perm(t).name);
    if !trigger_window(g, o, Some(src), &label, Some(5.0))? || !g.perm(t).on_bf {
        return Ok(());
    }
    apply_removal(g, Some(o), t, "destroy", None)
}

// ======================================================== Zur the Enchanter
const ZUR_PREF: [&str; 11] = [
    "Necropotence",
    "Ethereal Armor",
    "Empyrial Armor",
    "Ghostly Prison",
    "Propaganda",
    "Solitary Confinement",
    "Rest in Peace",
    "Mystic Remora",
    "Oblivion Ring",
    "Aura of Silence",
    "Copy Enchantment",
];

/// attacks: an enchantment with MV 3 or less onto the battlefield (Necropotence and armor Auras on Zur first)
fn zur(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if !atk.contains(&src) {
        return Ok(vec![]);
    }
    if g.player(p).key == "yshtola" {
        // Zur in your Y'shtola deck (cards/impl/zur.py)
        if !trigger_window(g, p, Some(src), "search for an enchantment", Some(5.0))? {
            return Ok(vec![]);
        }
        crate::cardcode::zur_fetch(g, src, p)?;
        return Ok(vec![]);
    }
    let have = |g: &Game| -> Vec<CardId> { g.player(p).perms.iter().filter_map(|&m| g.perm(m).cd).collect() };
    let fits = |g: &Game, c: CardId, have: &[CardId]| {
        let d = g.db.get(c);
        d.types.has(Types::ENCHANTMENT) && d.cmc <= 3 && !have.contains(&c)
    };
    let h = have(g);
    if !searchable(g, p).into_iter().any(|c| fits(g, c, &h)) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "search for an enchantment", Some(5.0))? {
        return Ok(vec![]);
    }
    let h = have(g);
    let cs: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&c| fits(g, c, &h) && !(g.db.get(c).tag(Tag::Necro) && crate::ai::decks::necro_prio(g, p, c) == 0))
        .collect();
    let Some(c) = first_min(&cs, |c| {
        let name = &*g.db.get(c).name;
        (ZUR_PREF.iter().position(|&n| n == name).unwrap_or(99), -card_worth(g, p, c, false))
    }) else {
        return Ok(vec![]);
    };
    remove_card(&mut g.player_mut(p).library, c);
    shuffle_library(g, p);
    g.attach_to = if g.db.get(c).has_subtype("aura") { Some(src) } else { None };
    let r = enter(g, p, c, Enter::default());
    g.attach_to = None;
    r?;
    crate::glog!(g, "    Zur fetches {}", g.db.get(c).name);
    Ok(vec![])
}

/// prevent all damage to you
fn conf_dmg(g: &Game, src: Src, p: PlayerId) -> bool {
    p == owner(g, src)
}

/// skip your draw step
fn conf_draw(g: &Game, src: Src, p: PlayerId) -> i32 {
    (p == owner(g, src)) as i32
}

/// upkeep: sacrifice it unless you discard a card
fn conf_up(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "sacrifice unless you discard", None)? || !on(g, p, src) {
        return Ok(());
    }
    if !g.player(p).hand.is_empty() { discard_worst(g, p, 1) } else { die(g, src, "sac") }
}

/// opponents' extra draws become yours
fn thief(g: &Game, src: Src, p: PlayerId) -> bool {
    p != owner(g, src)
}

/// copies the best enchantment on the battlefield
fn copy_ench(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let cs: Vec<PermId> = g
        .players
        .iter()
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&x| {
            g.perm(x).cd.is_some_and(|c| {
                let d = g.db.get(c);
                d.types.has(Types::ENCHANTMENT) && !d.has_subtype("aura")
            }) && x != src
        })
        .collect();
    if let Some(best) = max_by(&cs, |x| pval(g, x) + if cname(g, x) == "Necropotence" { 3.0 } else { 0.0 }) {
        let (cd, me, o) = (g.perm(best).cd.unwrap(), g.perm(src).cd, owner(g, src));
        leave(g, src)?;
        let n = enter(g, o, cd, Enter::default())?;
        g.perm_mut(n).phys = me;
    }
    Ok(())
}

/// +1: mill two (the opponent with the most cards), draw
fn jace_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let q = if opps.is_empty() { p } else { max_by(&opps, |q| g.player(q).library.len() as f64).unwrap() };
    mill(g, q, 2)?;
    draw(g, p, 1, false)
}

/// drawing from an empty library wins (with Consultation / Pact that is a combo); the -8 is not used (rules.py adds
/// one: register the whole list there)
pub const JACE_WOM: [WalkerAb; 1] =
    [WalkerAb { delta: 1, label: "mill two, draw", val: |_, _, _| Some(2.5), eff: jace_plus }];
static JACE_WOM_ABS: [WalkerAb; 1] = JACE_WOM;

// ======================================================== Yawgmoth, Thran Physician
fn undying_card(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).kws.iter().any(|k| &**k == "undying"))
}

/// Yawgmoth's fodder: tokens, cheap creatures, and undying creatures without their counter
fn yawg_fodder(g: &Game, src: PermId, m: PermId) -> bool {
    g.is_creature(m)
        && m != src
        && !g.perm(m).is_cmd
        && (g.perm(m).token || pval(g, m) < 2.0 || undying_card(g, m) && g.perm(m).plus <= 0)
}

fn opp_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).filter(|&m| g.is_creature(m)).collect()
}

/// pay 1 life, sacrifice: -1/-1 counter on an X/1 (tokens included) and draw, in your main phase; BB discard:
/// proliferate (the undying loop is a combo); protection from Humans not modeled
fn yawg(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    let fod: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| yawg_fodder(g, src, m)).collect();
    let oc = opp_creatures(g, p);
    let tgt: Vec<PermId> =
        oc.iter().copied().filter(|&m| etgh(g, m) <= 1 && pval(g, m) >= 2.0 && !untargetable(g, m)).collect();
    let swarm: Vec<PermId> =
        oc.iter().copied().filter(|&m| etgh(g, m) <= 1 && epow(g, m) >= 1 && !untargetable(g, m)).collect();
    let dv = death_value(g, p, None);
    if !fod.is_empty()
        && g.player(p).life > 8
        && (!tgt.is_empty() || !swarm.is_empty() || dv >= 2.0 || g.player(p).hand.len() <= 2)
    {
        let m = first_min(&fod, |x| (!undying_card(g, x), pval(g, x))).unwrap();
        let kill = if !tgt.is_empty() {
            1.5
        } else if !swarm.is_empty() {
            0.4 + 0.3 * swarm.iter().map(|&x| epow(g, x)).max().unwrap() as f64 // an X/1 attacker (a Goblin token)
        } else {
            0.0
        };
        let label = format!("Yawgmoth: sacrifice {}", g.perm(m).name);
        out.push(ability(1.5 + dv / 2.0 + kill, label, src, yawg_go, m.0 as i64));
    }
    let counters = g.player(p).perms.iter().any(|&m| g.perm(m).plus > 0 || g.perm(m).loyalty.is_some_and(|l| l != 0));
    if can_pay(g, p, 0, "BB", false) && !g.player(p).hand.is_empty() && counters && post == Some(true) {
        out.push(ability(1.0, "Yawgmoth: proliferate".into(), src, yawg_prol, 0));
    }
    Ok(out)
}

fn yawg_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !on(g, p, m) || g.player(p).life <= 8 {
        return Ok(false);
    }
    lose_life(g, p, 1, Some(p), "other", None)?;
    die(g, m, "sac")?;
    if !ability_window(g, p, Some(src), "a -1/-1 counter, draw a card", None, None)? {
        return Ok(true);
    }
    let t: Vec<PermId> = opp_creatures(g, p).into_iter().filter(|&x| etgh(g, x) <= 1 && !untargetable(g, x)).collect();
    if let Some(x) = first_max(&t, |x| (pval(g, x), epow(g, x))) {
        minus_counter(g, x, 1);
        if etgh(g, x) <= 0 {
            die(g, x, "sba")?;
        }
    }
    draw(g, p, 1, false)?;
    Ok(true)
}

fn yawg_prol(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 0, "BB", false) || g.player(p).hand.is_empty() {
        return Ok(false);
    }
    pay(g, p, 0, "BB", false)?;
    discard_worst(g, p, 1)?;
    if ability_window(g, p, Some(src), "proliferate", None, None)? {
        crate::impls::common::proliferate(g, p, 1)?;
    }
    Ok(true)
}

/// at instant speed, after blockers: pay 1 life and sacrifice fodder to put a -1/-1 counter on each unblocked X/1
/// attacker (a Goblin token), drawing a card each time; while the fodder lasts and life stays above 8
fn yawg_defend(g: &mut Game, src: Src, d: PlayerId, p: PlayerId, atk: &[PermId], assign: &mut Assign) -> Res {
    if owner(g, src) != d || !on(g, d, src) {
        return Ok(());
    }
    while g.player(d).life > 8 {
        let tgt: Vec<PermId> = atk
            .iter()
            .copied()
            .filter(|&a| {
                on(g, p, a)
                    && !assign.iter().any(|&(x, _)| x == a)
                    && g.is_creature(a)
                    && etgh(g, a) <= 1
                    && epow(g, a) >= 1
                    && !untargetable(g, a)
            })
            .collect();
        let fod: Vec<PermId> = g
            .player(d)
            .perms
            .iter()
            .copied()
            .filter(|&m| yawg_fodder(g, src, m) && !assign.iter().any(|&(_, b)| b == m))
            .collect();
        if tgt.is_empty() || fod.is_empty() {
            return Ok(());
        }
        let m = first_min(&fod, |x| (!undying_card(g, x), pval(g, x))).unwrap();
        let x = first_max(&tgt, |a| (epow(g, a), pval(g, a))).unwrap();
        crate::glog!(
            g,
            "    {} sacrifices {} to Yawgmoth: -1/-1 counter on attacking {}",
            g.player(d).name,
            g.perm(m).name,
            g.perm(x).name
        );
        lose_life(g, d, 1, Some(d), "other", None)?;
        die(g, m, "sac")?;
        if on(g, p, x) && minus_counter(g, x, 1) && etgh(g, x) <= 0 {
            die(g, x, "sba")?;
        }
        draw(g, d, 1, false)?;
        if g.over || !g.player(d).alive {
            return Ok(());
        }
    }
    Ok(())
}

/// shadow; returns for B when another creature of yours dies
fn traitor(g: &mut Game, c: CardId, p: PlayerId, _m: PermId) -> Res {
    if g.player(p).gy.contains(&c) && can_pay(g, p, 0, "B", false) {
        remove_card(&mut g.player_mut(p).gy, c);
        pay(g, p, 0, "B", false)?;
        enter(g, p, c, Enter::default())?;
    }
    Ok(())
}

/// tutor to hand (bargain free-cast not used).
/// Not registered: in Python rules2.py's Beseech the Mirror (its resolve and priority) replaces this one.
#[allow(dead_code)]
fn beseech(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    tutor(g, p, "any")?;
    Ok("gy")
}

/// tutor, then the most threatening opponent gets it (they don't use it back).
/// Not registered: in Python rules2.py's options hook replaces this one.
#[allow(dead_code)]
fn wishclaw(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.perm(src).tapped || !can_pay(g, p, 1, "", false) || g.active != Some(p) {
        return Ok(vec![]);
    }
    Ok(vec![ability(3.5, "Wishclaw Talisman".into(), src, wishclaw_go, 0)])
}

#[allow(dead_code)]
fn wishclaw_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "search for a card", Some(7.0), None)? {
        return Ok(true);
    }
    tutor(g, p, "any")?;
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let opp = max_by(&opps, |q| threat(g, p, q)).unwrap();
    g.player_mut(p).perms.retain(|&x| x != src);
    g.perm_mut(src).owner = opp;
    g.player_mut(opp).perms.push(src);
    g.bf_ver += 1;
    Ok(true)
}

/// adds BBB (BBBBB with threshold) when mana is short
fn cabal(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = if g.player(p).gy.len() >= 7 { 5 } else { 3 };
    g.player_mut(p).floating.any += n;
    Ok("gy")
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

// ======================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // Urza, Lord High Artificer
    let c = r.card(db, "Urza, Lord High Artificer")?;
    c.etb = Some(urza);
    c.extra_mana = Some(urza_mana);
    c.options = Some(urza_five);
    r.token_pt.push(construct_bonus);
    let c = r.card(db, "Karn, the Great Creator")?;
    c.no_artifact_mana = Some(karn_static);
    c.mana_lock = Some(karn_lattice);
    super::common::walker(r, db, "Karn, the Great Creator", &KARN_ABS)?;
    super::common::walker(r, db, "Tezzeret the Seeker", &TEZZERET_ABS)?;
    r.card(db, "Kappa Cannoneer")?.etb = Some(kappa);
    r.card(db, "Sai, Master Thopterist")?.cast = Some(sai);
    for name in ["Foundry Inspector", "Etherium Sculptor"] {
        r.card(db, name)?.cost = Some(art_reducer);
    }
    r.card(db, "Jet Medallion")?.cost = Some(jet);
    // common._tutor_etb (t1.rs's tutor_pred has their TUTOR_PRED entries)
    r.card(db, "Trophy Mage")?.etb = Some(mage_tutor);
    r.card(db, "Tribute Mage")?.etb = Some(mage_tutor);
    r.card(db, "Cursed Totem")?.no_creature_mana = Some(totem);
    // Whir of Invention: rules2's resolve and priority replace t5's (whir, whir_prio)
    let c = r.card(db, "Windfall")?;
    c.resolve = Some(windfall);
    c.prio = Some(windfall_prio);
    // Kinnan, Bonder Prodigy
    let c = r.card(db, "Kinnan, Bonder Prodigy")?;
    c.nonland_mana_bonus = Some(kinnan);
    c.options = Some(kinnan_act);
    r.card(db, "Bloom Tender")?.dyn_mana_perm = Some(bloom_tender);
    r.card(db, "Incubation Druid")?.dyn_mana_perm = Some(incubation);
    let c = r.card(db, "Seedborn Muse")?;
    c.upkeep = Some(seedborn);
    at_once(c, Event::Upkeep, true);
    let c = r.card(db, "Worldly Tutor")?;
    c.resolve = Some(worldly);
    c.prio = Some(worldly_prio);
    let c = r.card(db, "Nature's Rhythm")?;
    c.resolve = Some(rhythm);
    c.prio = Some(rhythm_prio);
    // Winota, Joiner of Forces
    r.card(db, "Winota, Joiner of Forces")?.attack = Some(winota);
    r.card(db, "Kiki-Jiki, Mirror Breaker")?.options = Some(kiki);
    r.card(db, "Felidar Guardian")?.etb = Some(felidar);
    r.card(db, "Zealous Conscripts")?.etb = Some(conscripts);
    r.card(db, "Goblin Guide")?.attack = Some(guide);
    r.card(db, "Loran of the Third Path")?.etb = Some(loran);
    // Zur the Enchanter
    r.card(db, "Zur the Enchanter")?.attack = Some(zur);
    let c = r.card(db, "Solitary Confinement")?;
    c.prevent_damage = Some(conf_dmg);
    c.skip_draw = Some(conf_draw);
    c.upkeep = Some(conf_up);
    r.card(db, "Notion Thief")?.steal_draw = Some(thief);
    let c = r.card(db, "Copy Enchantment")?;
    c.etb = Some(copy_ench);
    at_once(c, Event::Etb, true);
    super::common::walker(r, db, "Jace, Wielder of Mysteries", &JACE_WOM_ABS)?;
    // Yawgmoth, Thran Physician
    let c = r.card(db, "Yawgmoth, Thran Physician")?;
    c.options = Some(yawg);
    c.defend = Some(yawg_defend);
    r.card(db, "Nether Traitor")?.gy_dies = Some(traitor);
    // Beseech the Mirror: rules2's resolve and priority replace t5's (beseech); Wishclaw Talisman: rules2's options
    // hook replaces t5's (wishclaw)
    let c = r.card(db, "Cabal Ritual")?;
    c.resolve = Some(cabal);
    c.prio = Some(zero_prio);
    Ok(())
}
