//! Card code outside the hook table: Python's `cardimpl.py` dispatch helpers and the shared functions of
//! `cards/impl/*.py` that the engine calls by name (Blood Moon, Plaza of Heroes, regeneration, Treasures ...).
//! The engine calls these exactly where the Python does. Each is a `PORT(Mx)` placeholder returning what a table
//! without that card gives, until the card code is ported (M5 for Tier 1 and Sauron, phase 6 for the rest).

use crate::engine::hooks;
use crate::flow::Res;
use crate::hooks::{CardImpl, Event, Opt};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::Game;
use crate::sym::Sym;

/// PORT(M5): Blood Moon / Magus of the Moon in play (nonbasic lands tap for R)
pub fn blood_moon(_g: &Game) -> bool {
    false
}

/// PORT(M5): Dryad of the Ilysian Grove (rules.dryad_colors): p's lands tap for any colour in its identity
pub fn dryad_colors(_g: &Game, _p: PlayerId) -> bool {
    false
}

/// a card's code, if it has any
fn imp_of(g: &Game, c: CardId) -> Option<CardImpl> {
    g.registry.get(c).copied()
}

/// a land with its own colour rules (CI.LAND_COLS: Plaza of Heroes, Unclaimed Territory, Vivid lands, Gemstone
/// Mine ...). None: the land's `c` tag decides.
pub fn land_colors(g: &Game, p: PlayerId, l: LandId) -> Option<crate::cards::Colors> {
    imp_of(g, g.land(l).cd)?.land_cols.map(|f| f(g, p, l))
}

/// CI.DYN_MANA, the mana a land makes now (Gaea's Cradle). None: its tag's amount.
pub fn dyn_mana_land(g: &Game, p: PlayerId, l: LandId) -> Option<u32> {
    imp_of(g, g.land(l).cd)?.dyn_mana_land.map(|f| f(g, p, l))
}

/// CI.DYN_MANA for a nonland permanent (Priest of Titania)
pub fn dyn_mana_perm(g: &Game, p: PlayerId, m: PermId) -> Option<u32> {
    imp_of(g, g.perm(m).cd?)?.dyn_mana_perm.map(|f| f(g, p, m))
}

/// PORT(M5): CI.locked(g, m, kind): an Aura locks m (Arrest: no activated abilities; Pacify: can't attack)
pub fn locked(_g: &Game, _m: PermId, _kind: &str) -> bool {
    false
}

/// PORT(M5): rules.halfling_colors (Delighted Halfling)
pub fn halfling_colors(_g: &Game, _p: PlayerId) -> Option<crate::cards::Colors> {
    None
}

/// mine.is_legendary: a legendary card, or your Ring-bearer (Champion's Helm's hexproof needs a legendary creature)
pub fn is_legendary(g: &Game, m: PermId) -> bool {
    crate::impls::mine::is_legendary(g, m)
}

/// PORT(M5): CI.attached_prot, colours an Aura gives protection from
pub fn attached_prot(_g: &Game, _m: PermId) -> crate::cards::Colors {
    crate::cards::Colors::NONE
}

/// PORT(M5): CI.garland_check, Garland's steals end when a player leaves
pub fn garland_check(_g: &mut Game) {}

/// PORT(M5): your own Auras on m (common.auras_on, excluding locks: CI.LOCKS)
pub fn own_auras_on(_g: &Game, _m: PermId) -> usize {
    0
}

/// PORT(M5): how close a planeswalker is to its ultimate (common.ult_pressure, from its WALKERS entry)
pub fn ult_pressure(_g: &Game, _m: PermId) -> f64 {
    0.0
}

/// cardimpl.threat_value: what a permanent is worth removing beyond its tags (its card code and compiled abilities,
/// and the value of a combo piece)
pub fn threat_value(g: &Game, m: PermId) -> f64 {
    let Some(c) = g.perm(m).cd else { return 0.0 };
    crate::ai::pool::card_threat_value(g, c) + piece_threat(g, m)
}

/// PORT(M5): how much of m's value is left under the Auras locking it (zur.lock_factor)
pub fn lock_factor(_g: &Game, _m: PermId) -> f64 {
    1.0
}

/// PORT(M5): partials.hand_mana, Elvish Spirit Guide and kin: exile from hand for mana
pub fn hand_mana(_g: &Game, _p: PlayerId) -> Vec<crate::engine::mana::Unit> {
    vec![]
}

/// PORT(M5): partials.special_unit_paid, paying with a Scion (sacrifice it) or a card in hand (exile it)
pub fn special_unit_paid(_g: &mut Game, _p: PlayerId, _u: crate::engine::mana::Unit) -> crate::flow::Res {
    Ok(())
}

/// CI.ON_TAP for a land (Vivid lands' charge counters ...)
pub fn on_tap_land(g: &mut Game, p: PlayerId, l: LandId, n: u32) -> crate::flow::Res {
    match imp_of(g, g.land(l).cd).and_then(|i| i.on_tap_land) {
        Some(f) => f(g, p, l, n),
        None => Ok(()),
    }
}

/// CI.ON_TAP for a permanent (Heritage Druid, Mana Vault's damage ...)
pub fn on_tap_perm(g: &mut Game, p: PlayerId, m: PermId, n: u32) -> crate::flow::Res {
    match g.perm(m).cd.and_then(|c| imp_of(g, c)).and_then(|i| i.on_tap_perm) {
        Some(f) => f(g, p, m, n),
        None => Ok(()),
    }
}

/// PORT(M5): engine.SELF_COST, a card's own cost change (Draco's domain, delve)
pub fn self_cost(_g: &Game, _p: PlayerId, _c: crate::ids::CardId) -> i32 {
    0
}

/// mine.add_counters: +1/+1 counters on m; Mauhúr: one more on an Army, Goblin or Orc you control
pub fn add_counters(g: &mut Game, m: PermId, n: i32) {
    use crate::engine::values::{card_tag, has_type};
    let x = g.perm(m);
    let orcish = x.army || (x.cd.is_some() && (has_type(g, m, "orc") || has_type(g, m, "goblin")));
    let mauhur = g.player(x.owner).perms.iter().any(|&y| card_tag(g, y, crate::tag::Tag::Mauhur) && !g.perm(y).phased);
    let n = if n > 0 && orcish && mauhur { n + 1 } else { n };
    g.perm_mut(m).plus += n;
}

/// rules.revolt: a permanent of p's left the battlefield this turn
pub fn revolt(g: &Game, p: PlayerId) -> bool {
    g.player(p).left_turn == Some(g.turn_stamp())
}

/// rules.spare_after: can p pay a tax of n (Rhystic Study, Smothering Tithe) and still cast the most expensive spell
/// it could cast now (any spell on its own turn, an instant or flash card on someone else's)? Cards it can't afford
/// anyway don't stop it paying.
pub fn spare_after(g: &Game, p: PlayerId, n: u32) -> bool {
    use crate::engine::mana::{can_pay, cost_of, total_mana};
    if !can_pay(g, p, n, "", false) {
        return false;
    }
    let avail = total_mana(g, p, false);
    let own = g.active == Some(p);
    let need = g
        .player(p)
        .hand
        .iter()
        .filter(|&&c| {
            let d = g.db.get(c);
            !d.land && (own || d.instant || d.tag(crate::tag::Tag::Flash))
        })
        .map(|&c| {
            let (gn, pips) = cost_of(g, p, c);
            gn + pips.len() as u32
        })
        .filter(|&mv| mv <= avail)
        .max()
        .unwrap_or(0);
    avail >= n + need
}

/// rules.tithe_unpaid: Smothering Tithe: p pays {2} when it can spare the mana; true: unpaid, so the Tithe's owner
/// gets a Treasure. HUMAN(phase 9): a person decides.
pub fn tithe_unpaid(g: &mut Game, p: PlayerId) -> Res<bool> {
    if spare_after(g, p, 2) {
        crate::engine::mana::pay(g, p, 2, "", false)?;
        return Ok(false);
    }
    Ok(true)
}

/// rules.rhystic_unpaid, as tithe_unpaid for Rhystic Study's {1}. HUMAN(phase 9): a person decides.
pub fn rhystic_unpaid(g: &mut Game, p: PlayerId) -> Res<bool> {
    if spare_after(g, p, 1) {
        crate::engine::mana::pay(g, p, 1, "", false)?;
        return Ok(false);
    }
    Ok(true)
}

/// topdeck.scry: scry n, or surveil n (the AI's ordering is ai::topdeck)
pub fn scry(g: &mut Game, p: PlayerId, n: i32, surveil: bool) -> Res {
    crate::ai::topdeck::scry(g, p, n, surveil)
}

macro_rules! port_res {
    ($($(#[$doc:meta])* fn $name:ident($($arg:ident: $ty:ty),*);)*) => {$(
        $(#[$doc])*
        pub fn $name(_g: &mut Game, $($arg: $ty),*) -> Res {
            $(let _ = $arg;)*
            Ok(())
        }
    )*};
}

macro_rules! port_bool {
    ($($(#[$doc:meta])* fn $name:ident($($arg:ident: $ty:ty),*);)*) => {$(
        $(#[$doc])*
        pub fn $name(_g: &mut Game, $($arg: $ty),*) -> Res<bool> {
            $(let _ = $arg;)*
            Ok(false)
        }
    )*};
}

port_res! {
    /// PORT(M5): rules.emblem_draw (emblems that trigger on draws)
    fn emblem_draw(p: PlayerId);
    /// PORT(M5): rules2.chatterfang_squirrels
    fn chatterfang_squirrels(p: PlayerId, n: i32);
    /// PORT(M5): common.aura_fall (Auras on a permanent that left)
    fn aura_fall(m: PermId);
    /// PORT(M5): partials.bestow_fall
    fn bestow_fall(m: PermId);
    /// PORT(M5): marchesa.marchesa_dies
    fn marchesa_dies(p: PlayerId, m: PermId);
    /// PORT(M5): partials.answer_ability (Tishana's Tidebinder, Azorius Guildmage)
    fn answer_ability(q: PlayerId, item: u32);
    /// PORT(M5): rules.emblem_cast
    fn emblem_cast(p: PlayerId, c: CardId);
    /// PORT(M5): rules2.glimpse_draw
    fn glimpse_draw(p: PlayerId, c: CardId);
    /// PORT(M5): CI.muld_mark (Muldrotha's permanent types used this turn)
    fn muld_mark(p: PlayerId, kind: Sym);
    /// PORT(M5): t1.tutor_named for Auras (Three Dreams)
    fn tutor_auras(p: PlayerId, k: usize);
    /// PORT(M5): rules.transform_away (Elk, mutate, Forest Dryad)
    fn transform_away(m: PermId, kind: Sym);
    /// PORT(M5): CI.apply_lock (Arrest, Encrust ...)
    fn apply_lock(actor: Option<PlayerId>, m: PermId, kind: Sym);
}

port_bool! {
    /// PORT(M5): common.umbra_save (umbra armor)
    fn umbra_save(m: PermId);
    /// PORT(M5): lands.try_regenerate (Yavimaya Hollow)
    fn try_regenerate(m: PermId);
    /// PORT(M5): rules.ezuri_regen
    fn ezuri_regen(m: PermId);
    /// PORT(M5): partials.tidebinder_response
    fn tidebinder_response(p: PlayerId, m: PermId);
    /// PORT(M5): rules2.hullbreaker_counter
    fn hullbreaker_counter(q: PlayerId, c: CardId);
    /// PORT(M5): rules.veil_response
    fn veil_response(p: PlayerId, q: PlayerId, ctr: CardId);
}

/// mine.kindred_enter: Kindred Discovery draws when a creature of the named type entered without `enter` (an Orc
/// Army)
pub fn kindred_enter(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    crate::impls::mine::kindred_enter(g, p, m)
}

/// CI.kaervek (mine.kaervek): Kaervek the Merciless's damage, q's, for p's spell c
pub fn kaervek(g: &mut Game, q: PlayerId, p: PlayerId, c: CardId) -> Res {
    crate::impls::mine::kaervek(g, q, p, c)
}

/// CI.ring_tempt (mine.ring_tempt): the Ring tempts p
pub fn ring_tempt(g: &mut Game, p: PlayerId) -> Res {
    crate::impls::mine::ring_tempt(g, p)
}

/// mine.proliferate_all: proliferate everything worth it (your counters and loyalty, opponents' poison and -1/-1)
pub fn proliferate_all(g: &mut Game, p: PlayerId) -> Res {
    crate::impls::mine::proliferate_all(g, p)
}

/// CI.gy_cards(owner, 'gy_dies'): cards in owner's graveyard see owner's creature m die (Nether Traitor)
pub fn gy_dies(g: &mut Game, owner: PlayerId, m: PermId) -> Res {
    let own = g.perm(m).cd;
    for c in g.player(owner).gy.clone() {
        if Some(c) != own
            && let Some(f) = imp_of(g, c).and_then(|i| i.gy_dies)
        {
            f(g, c, owner, m)?;
        }
    }
    Ok(())
}

/// CI.AS_ENTERS: as m enters, before any trigger (naming a creature type)
pub fn as_enters(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    match g.perm(m).cd.and_then(|c| imp_of(g, c)).and_then(|i| i.as_enters) {
        Some(f) => f(g, p, m),
        None => Ok(()),
    }
}

/// CI.gy_cards(p, 'gy_landfall'): cards in p's graveyard see a land enter under p's control
pub fn gy_landfall(g: &mut Game, p: PlayerId) -> Res {
    for c in g.player(p).gy.clone() {
        if let Some(f) = imp_of(g, c).and_then(|i| i.gy_landfall) {
            f(g, c, p)?;
        }
    }
    Ok(())
}

/// CI.SELF_CAST: "when you cast this spell" (cascade)
pub fn self_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    match imp_of(g, c).and_then(|i| i.self_cast) {
        Some(f) => f(g, p, c),
        None => Ok(()),
    }
}

/// CI.hand_cards(p, 'hand_cast'): cards in p's hand see p cast c (Return the Favor)
pub fn hand_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    for x in g.player(p).hand.clone() {
        if let Some(f) = imp_of(g, x).and_then(|i| i.hand_cast) {
            f(g, x, p, c)?;
        }
    }
    Ok(())
}

/// CI.hand_cards(q, 'hand_opp_cast'): cards in q's hand see an opponent cast c (Dualcaster Mage)
pub fn hand_opp_cast(g: &mut Game, q: PlayerId, c: CardId) -> Res {
    for x in g.player(q).hand.clone() {
        if let Some(f) = imp_of(g, x).and_then(|i| i.hand_opp_cast) {
            f(g, x, q, c)?;
        }
    }
    Ok(())
}

/// CI.SELF_REGEN: m regenerates instead of being destroyed (the caller checks the cause and Rest in Peace-style
/// `noregen`)
pub fn self_regen(g: &mut Game, m: PermId) -> Res<bool> {
    match g.perm(m).cd.and_then(|c| imp_of(g, c)).and_then(|i| i.self_regen) {
        Some(f) => f(g, m),
        None => Ok(false),
    }
}

/// PORT(M5): CI.marchesa_sac_worth
pub fn marchesa_sac_worth(_g: &Game, _m: PermId, v: f64) -> f64 {
    v
}

/// PORT(M5): CI.PROWESS (cards with prowess the keywords don't show)
pub fn is_prowess_card(_g: &Game, _c: CardId) -> bool {
    false
}

/// PORT(M5): rules2.aura_ward
pub fn aura_ward(_g: &Game, _m: PermId) -> u32 {
    0
}

/// PORT(M5): rules.removal_taxes (Karmic Justice ...): false stops the removal
pub fn removal_taxes(_g: &mut Game, _actor: Option<PlayerId>, _m: PermId, _kind: Sym) -> Res<bool> {
    Ok(true)
}

/// PORT(M5): partials.tajic_protects
pub fn tajic_protects(_g: &Game, _m: PermId) -> bool {
    false
}

/// PORT(M5): Skyclave Apparition remembers what it exiled
pub fn apparition_note(_g: &mut Game, _src: PermId, _cd: Option<CardId>, _owner: PlayerId) {}

/// PORT(M5): combos: a modeled combo is ready in p's hand and battlefield
pub fn combo_ready(_g: &Game, _p: PlayerId) -> bool {
    false
}

// ------------------------------------------------------------------ M3: combat and the turn
/// PORT(M5): rules.evasion_blocked (menace-style and type-based evasion from card code)
pub fn evasion_blocked(_g: &Game, _b: PermId, _a: PermId) -> bool {
    false
}

/// CI.granted_kw: a static keyword grant from a hooked permanent (Teysa: tokens have vigilance and lifelink)
pub fn granted_kw(g: &Game, m: PermId, kw: &str) -> bool {
    if g.hooks.is_empty() {
        return false;
    }
    let kw = crate::sym::intern(kw);
    hooks::hooked(g, Event::GrantKw).iter().any(|(src, imp)| (imp.grant_kw.unwrap())(g, *src, m, kw))
}

/// mine.ring_unblockable (the Ring, level 1)
pub fn ring_unblockable(g: &Game, b: PermId, a: PermId) -> bool {
    crate::impls::mine::ring_unblockable(g, b, a)
}

/// mine.ring_blocked (the Ring, level 3)
pub fn ring_blocked(g: &Game, p: PlayerId, a: PermId, b: PermId) -> bool {
    crate::impls::mine::ring_blocked(g, p, a, b)
}

/// mine.damage_prevented (Old Fat Spider's chapter II)
pub fn damage_prevented(g: &Game, m: PermId) -> bool {
    crate::impls::mine::damage_prevented(g, m)
}

/// PORT(M5): rules.dovin_blocked
pub fn dovin_blocked(_g: &Game, _m: PermId) -> bool {
    false
}

/// PORT(M5): rules2.prot_vs (protection from creatures, from Demons and Dragons)
pub fn prot_vs(_g: &Game, _m: PermId, _from: PermId) -> bool {
    false
}

/// PORT(M5): CI.kaldra_exile (Sword of Kaldra exiles what it damages): true if it exiled
pub fn kaldra_exile(_g: &mut Game, _src: PermId, _m: PermId) -> Res<bool> {
    Ok(false)
}

/// mine.necromancer_attack (an attacking token copy of a graveyard creature)
pub fn necromancer_attack(g: &mut Game, p: PlayerId, m: PermId) -> Res<Vec<PermId>> {
    crate::impls::mine::necromancer_attack(g, p, m)
}

/// PORT(M5): CI.keyword_attack (keyword attack triggers: annihilator, myriad ...)
pub fn keyword_attack(_g: &mut Game, _p: PlayerId, _atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    Ok(vec![])
}

/// CI.fire(g, 'blocks', p, atk, d, assign): hooks that see or change the blocks (Brimaz's blocking token). They
/// run here rather than through the trigger queue, since they edit `assign`; one with a `trigger_window` still gets
/// its own stack window.
pub fn blocks_hooks(g: &mut Game, p: PlayerId, atk: &[PermId], d: PlayerId, assign: &mut Vec<(PermId, PermId)>) -> Res {
    if g.hooks.is_empty() {
        return Ok(());
    }
    for (src, imp) in hooks::hooked(g, Event::Blocks) {
        (imp.blocks.unwrap())(g, src, p, atk, d, assign)?;
        if g.over {
            break;
        }
    }
    Ok(())
}

/// After blocks (ais.py): cards in the attacker's hand ('hand_blocks'); your Veyran's Aetherize; an outside
/// defender's 'defend' hooks (Yawgmoth), hand answers ('hand_defend') and lands ('land_defend'); an outside attacker's
/// ninjutsu.
pub fn defend_hooks(g: &mut Game, p: PlayerId, atk: &[PermId], d: PlayerId, assign: &mut Vec<(PermId, PermId)>) -> Res {
    for c in g.player(p).hand.clone() {
        if let Some(f) = imp_of(g, c).and_then(|i| i.hand_blocks) {
            f(g, c, p, atk, d, assign)?;
        }
    }
    let dk = g.player(d).key;
    if dk == "veyran" {
        // your instant-speed answers to an attack (Aetherize)
        for c in g.player(d).hand.clone() {
            if &*g.db.get(c).name == "Aetherize"
                && let Some(f) = imp_of(g, c).and_then(|i| i.hand_defend)
            {
                f(g, c, d, p, atk, assign)?;
            }
        }
    }
    if !crate::ai::is_main(dk) {
        if !g.hooks.is_empty() {
            for (src, imp) in hooks::hooked(g, Event::Defend) {
                (imp.defend.unwrap())(g, src, d, p, atk, assign)?;
                if g.over {
                    return Ok(());
                }
            }
        }
        for c in g.player(d).hand.clone() {
            if let Some(f) = imp_of(g, c).and_then(|i| i.hand_defend) {
                f(g, c, d, p, atk, assign)?;
            }
        }
        for l in g.player(d).lands.clone() {
            if let Some(f) = imp_of(g, g.land(l).cd).and_then(|i| i.land_defend) {
                f(g, l, d, p, atk, assign)?;
            }
        }
    }
    if !crate::ai::is_main(g.player(p).key) {
        ninjutsu(g, p, atk, d, assign)?;
    }
    Ok(())
}

/// PORT(phase 6): t4.ninjutsu (Yuriko's Ninjas swap in for unblocked attackers)
pub fn ninjutsu(_g: &mut Game, _p: PlayerId, _atk: &[PermId], _d: PlayerId, _assign: &mut [(PermId, PermId)]) -> Res {
    Ok(())
}

/// PORT(M5): Professional's Insight-style draws and emblems on combat damage
pub fn combat_damage_cards(_g: &mut Game, _p: PlayerId, _a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    Ok(())
}

/// PORT(M5): CI.become_monarch
pub fn become_monarch(g: &mut Game, p: PlayerId) -> Res {
    g.monarch = Some(p);
    Ok(())
}

/// mine.ring_damage (the Ring, level 4)
pub fn ring_damage(g: &mut Game, p: PlayerId, a: PermId, d: PlayerId) -> Res {
    crate::impls::mine::ring_damage(g, p, a, d)
}

/// PORT(M5): CI.vanguard_blocks (Defiant Vanguard)
pub fn vanguard_blocks(_g: &mut Game, _assign: &[(PermId, PermId)]) -> Res {
    Ok(())
}

/// PORT(M5): rules2.forced_attackers
pub fn forced_attackers(_g: &mut Game, _p: PlayerId, xs: Vec<PermId>, _cand0: &[PermId]) -> Res<Vec<PermId>> {
    Ok(xs)
}

/// PORT(M5): rules2.annex_life
pub fn annex_life(_g: &mut Game, _p: PlayerId, _d: PlayerId, xs: Vec<PermId>) -> Res<Vec<PermId>> {
    Ok(xs)
}

/// PORT(M5): CI.fire(g, 'combat_start', p) (Helm of the Host): new attackers
pub fn combat_start(_g: &mut Game, _p: PlayerId) -> Res<Vec<PermId>> {
    Ok(vec![])
}

/// mine.ring_attack (the Ring, level 2)
pub fn ring_attack(g: &mut Game, p: PlayerId, atk: &[PermId]) -> Res {
    crate::impls::mine::ring_attack(g, p, atk)
}

/// CI.hand_cards(p, 'hand_attack'): cards in p's hand as p attacks d
pub fn hand_attack(g: &mut Game, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res {
    for c in g.player(p).hand.clone() {
        if let Some(f) = imp_of(g, c).and_then(|i| i.hand_attack) {
            f(g, c, p, atk, d)?;
        }
    }
    Ok(())
}

/// PORT(M5): CI.muldrotha_on
pub fn muldrotha_on(_g: &Game, _p: PlayerId) -> bool {
    false
}

/// PORT(M5): CI.muld_types (a permanent type Muldrotha still allows this turn)
pub fn muld_types(_g: &Game, _p: PlayerId, _c: CardId) -> bool {
    false
}

/// CI.LAND_ETB: a land played as a land drop enters (Bojuka Bog ...)
pub fn land_etb(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    match imp_of(g, g.land(l).cd).and_then(|i| i.land_etb) {
        Some(f) => f(g, p, l),
        None => Ok(()),
    }
}

/// CI.total(g, 'extra_lands', p): extra land drops (Exploration, Azusa ...)
pub fn extra_lands(g: &Game, p: PlayerId) -> u32 {
    if g.hooks.is_empty() { 0 } else { hooks::total_count(g, Event::ExtraLands, p).max(0) as u32 }
}

/// CI.total(g, 'lands_from_top', p) > 0 (Oracle of Mul Daya)
pub fn lands_from_top(g: &Game, p: PlayerId) -> bool {
    !g.hooks.is_empty() && hooks::total_count(g, Event::LandsFromTop, p) > 0
}

/// CI.total(g, 'lands_from_gy', p) > 0 (Crucible of Worlds)
pub fn lands_from_gy(g: &Game, p: PlayerId) -> bool {
    !g.hooks.is_empty() && hooks::total_count(g, Event::LandsFromGy, p) > 0
}

port_res! {
    /// PORT(M5): CI.suspend_upkeep
    fn suspend_upkeep(p: PlayerId);
    /// PORT(M5): ais.braids_sacrifice
    fn braids_sacrifice(p: PlayerId);
    /// PORT(M5): ais.necro_deliver (Necropotence's cards at the end step)
    fn necro_deliver(p: PlayerId);
    /// PORT(M5): ais.necro_pay
    fn necro_pay(p: PlayerId);
    /// PORT(M5): the delayed end-step effects: Marchesa's returns, The Eternal Wanderer, Eerie Interlude, Memory Jar
    fn delayed_end_step(p: PlayerId);
    /// PORT(M5): Galadriel's precombat taps
    fn galadriel_precombat(q: PlayerId);
    /// PORT(M5): Opposition's precombat taps
    fn opposition_precombat(q: PlayerId);
}

/// ais.mirror_upkeep (Panoptic Mirror)
pub fn mirror_upkeep(g: &mut Game, p: PlayerId) -> Res {
    crate::impls::mine::mirror_upkeep(g, p)
}

/// mine.spider_chapter2
pub fn spider_chapter2(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    crate::impls::mine::spider_chapter2(g, p, m)
}

port_bool! {
    /// PORT(M5): lands.dakmor_dredge
    fn dakmor_dredge(p: PlayerId);
}

/// PORT(M5): rules.REPLACED_TAG_ENGINES (engines whose card code replaces the tag's draw)
pub fn replaced_tag_engine(_name: &str) -> bool {
    false
}

/// PORT(M5): CI.crackdown_on
pub fn crackdown_on(_g: &Game) -> bool {
    false
}

/// PORT(M5): CI.crackdown_holds
pub fn crackdown_holds(_g: &Game, _m: PermId) -> bool {
    false
}

/// CI.total(g, 'skip_draw', p) (Solitary Confinement)
pub fn skip_draw(g: &Game, p: PlayerId) -> bool {
    !g.hooks.is_empty() && hooks::total_count(g, Event::SkipDraw, p) > 0
}

/// cardimpl.turn_start: the start of p's turn: animated lands revert, vehicles stop being crewed, Mana Drain's mana,
/// Baubles' delayed draws, lands' upkeep abilities, pact payments, Sagas' chapters, rebound spells
pub fn turn_start(g: &mut Game, p: PlayerId) -> Res {
    crate::impls::lands::revert_animated(g)?;
    crate::impls::rules2::uncrew(g, p);
    let drain = std::mem::take(&mut g.player_mut(p).drain_mana);
    g.player_mut(p).floating.c += drain; // Mana Drain: the countered spell's mana value, colourless
    for q in 0..g.players.len() {
        let q = PlayerId(q as u8);
        let n = g.player(q).delayed_draws;
        if n > 0 && g.player(q).alive {
            g.player_mut(q).delayed_draws = 0; // Baubles: draw at the beginning of the next upkeep
            crate::engine::zones::draw(g, q, n, false)?;
        }
    }
    for l in g.player(p).lands.clone() {
        if g.player(p).lands.contains(&l)
            && let Some(f) = imp_of(g, g.land(l).cd).and_then(|i| i.land_upkeep)
        {
            f(g, l, p)?;
        }
    }
    use crate::engine::mana::{can_pay, pay};
    let pacts = std::mem::take(&mut g.player_mut(p).pacts);
    for _ in 0..pacts {
        if can_pay(g, p, 3, "UU", false) {
            pay(g, p, 3, "UU", false)?;
        } else {
            crate::glog!(g, "  {} can't pay for Pact of Negation and loses", g.player(p).name);
            lose_to_pact(g, p)?;
            break;
        }
    }
    let debts = std::mem::take(&mut g.player_mut(p).pact_debts);
    for (generic, pips) in debts {
        if can_pay(g, p, generic, &pips, false) {
            pay(g, p, generic, &pips, false)?;
        } else {
            crate::glog!(g, "  {} can't pay for a pact and loses", g.player(p).name);
            lose_to_pact(g, p)?;
            break;
        }
    }
    crate::impls::common::saga_step(g, p)?;
    let reb = std::mem::take(&mut g.player_mut(p).rebound);
    for c in reb {
        if g.player(p).exile.contains(&c)
            && let Some(f) = imp_of(g, c).and_then(|i| i.rebound)
        {
            f(g, p, c)?;
        }
    }
    Ok(())
}

fn lose_to_pact(g: &mut Game, p: PlayerId) -> Res {
    let pl = g.player_mut(p);
    pl.life = 0;
    pl.last_src = None;
    crate::engine::life::check_state(g)
}

// ------------------------------------------------------------------ M3: the ability language's card-code calls
/// PORT(M5): CI.attached_bonus (Auras' and Elspeth's emblem's power and toughness)
pub fn attached_bonus(_g: &Game, _m: PermId) -> (i32, i32) {
    (0, 0)
}

/// PORT(M5): CI.attached_kw (keywords from Auras)
pub fn attached_kw(_g: &Game, _m: PermId, _kw: &str) -> bool {
    false
}

/// PORT(M5): rules.ability_locked (Pithing Needle, Linvala ...)
pub fn ability_locked(_g: &Game, _src: PermId, _p: PlayerId) -> bool {
    false
}

// ------------------------------------------------------------------ M4: what the AI reads from card code
/// PORT(M5): combos.combo_imp: a combo piece's importance for counter decisions (9: completes a combo, 7: one short)
pub fn combo_imp(_g: &Game, _p: PlayerId, _c: CardId) -> f64 {
    0.0
}

/// PORT(M5): combos.PIECES: every card that is a piece of a modeled combo
pub fn combo_pieces(_g: &Game) -> Vec<CardId> {
    vec![]
}

/// PORT(M5): c is a piece of a modeled combo (Python: `c.name in impl_combos.PIECES`)
pub fn is_combo_piece(_g: &Game, _c: CardId) -> bool {
    false
}

/// PORT(M5): combos.missing_pieces: the pieces of p's closest combo it doesn't have yet (for tutors)
pub fn missing_pieces(_g: &Game, _p: PlayerId) -> Vec<CardId> {
    vec![]
}

/// PORT(M5): search.combo_progress: the share of q's closest combo that q holds, squared
pub fn combo_progress(_g: &Game, _q: PlayerId) -> f64 {
    0.0
}

/// PORT(M5): combos.piece_threat: extra value of m if it's a piece of a combo its controller nearly has
pub fn piece_threat(_g: &Game, _m: PermId) -> f64 {
    0.0
}

/// PORT(M5): combos.combo_options: go for a ready combo (ctr_risk: the chance a spell in it is countered)
pub fn combo_options(_g: &mut Game, _p: PlayerId, _ctr_risk: f64, _post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(vec![])
}

/// PORT(M5): common.aristocrat_options (sacrifice outlets with a payoff out: Grave Pact, Blood Artist ...)
pub fn aristocrat_options(_g: &mut Game, _p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(vec![])
}

/// PORT(M5): lands.land_options (lands' activated abilities: Barad-dûr, Urza's Saga ...)
pub fn land_options(_g: &mut Game, _p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(vec![])
}

/// ais.breach_gc_options: Underworld Breach escapes, Panoptic Mirror imprints, Lion's Eye Diamond, Bolas's Citadel
pub fn breach_gc_options(g: &mut Game, p: PlayerId) -> Res<Vec<Opt>> {
    crate::impls::mine::breach_gc_options(g, p)
}

/// mine.breach_options: Sauron's Underworld Breach line
pub fn breach_options(g: &mut Game, p: PlayerId, post: bool) -> Res<Vec<Opt>> {
    crate::impls::mine::breach_options(g, p, post)
}

/// PORT(phase 6): mine.loop_need: the creature cards that complete one of Sephiroth's loops
pub fn loop_need(_g: &Game, _p: PlayerId) -> Vec<CardId> {
    vec![]
}

/// PORT(M5): common.sac_in_response: sacrifice the permanent for value before removal resolves
pub fn sac_in_response(_g: &mut Game, _owner: PlayerId, _m: PermId, _kind: Sym) -> Res<bool> {
    Ok(false)
}

/// PORT(M5): partials.regen_wipe: regenerate the board against a destroy wipe
pub fn regen_wipe(_g: &mut Game, _q: PlayerId) -> Res<bool> {
    Ok(false)
}

/// PORT(M5): the outside decks' card plays: common.adventure_options, t2.evoke_options,
/// common.aristocrat_options, common.food_options, partials.miracle_options and incubator_options
pub fn pool_card_options(_g: &mut Game, _p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(vec![])
}

/// PORT(M5): partials.aid_active (Sigarda's Aid: Auras at instant speed)
pub fn aid_active(_g: &Game, _p: PlayerId) -> bool {
    false
}

/// PORT(M5): t4.NINJUTSU, ninjutsu_cost: the ninjutsu costs of the Ninjas in p's hand
pub fn ninjutsu_costs(_g: &Game, _p: PlayerId) -> Vec<(u32, String)> {
    vec![]
}

/// PORT(M5): common.SAC_OUTLET
pub fn is_sac_outlet(_name: &str) -> bool {
    false
}

/// PORT(M5): CI.gy_response: an opponent answers a reanimation spell aimed at src's graveyard (Scavenger Grounds)
pub fn gy_response(_g: &mut Game, _p: PlayerId, _value: f64, _src: PlayerId) -> Res<bool> {
    Ok(false)
}

/// rules.tithe_prio: Smothering Tithe by the Treasures it will make over the next three rounds (likelier early,
/// while opponents have few lands), worth more when your hand holds more spells than your mana can cast
pub fn tithe_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let lands = if opps.is_empty() {
        0.0
    } else {
        opps.iter().map(|&q| g.player(q).lands.len()).sum::<usize>() as f64 / opps.len() as f64
    };
    let unpaid = if lands <= 4.0 {
        0.65
    } else if lands <= 6.0 {
        0.45
    } else {
        0.3
    };
    let treasures = 3.0 * opps.len() as f64 * unpaid;
    let mana = crate::engine::mana::total_mana(g, p, false).max(1) as f64;
    let backlog: f64 =
        g.player(p).hand.iter().filter(|&&x| x != c && !g.db.get(x).land).map(|&x| g.db.get(x).cmc as f64).psum()
            / mana;
    let need = if backlog >= 2.0 {
        1.2
    } else if backlog >= 1.0 {
        1.0
    } else {
        0.7
    };
    (35.0 + 4.0 * treasures * need).clamp(15.0, 75.0) as i32
}

/// PORT(M5): t4.yuriko_wish
pub fn yuriko_wish(_g: &Game, _p: PlayerId) -> Vec<CardId> {
    vec![]
}

/// PORT(M5): t4.yuriko_prio
pub fn yuriko_prio(_g: &Game, _p: PlayerId, _c: CardId) -> Option<i32> {
    None
}

/// PORT(M5): t2.kaalia_prio
pub fn kaalia_prio(_g: &Game, _p: PlayerId, _c: CardId) -> Option<i32> {
    None
}
