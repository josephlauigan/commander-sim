//! Card code outside the hook table: Python's `cardimpl.py` dispatch helpers and the shared functions of
//! `cards/impl/*.py` that the engine calls by name (Blood Moon, Plaza of Heroes, regeneration, Treasures ...).
//! The engine calls these exactly where the Python does. Each is a `PORT(Mx)` placeholder returning what a table
//! without that card gives, until the card code is ported (M5 for Tier 1 and Sauron, phase 6 for the rest).

use crate::ids::{LandId, PermId, PlayerId};
use crate::state::Game;

/// PORT(M5): Blood Moon / Magus of the Moon in play (nonbasic lands tap for R)
pub fn blood_moon(_g: &Game) -> bool {
    false
}

/// PORT(M5): Dryad of the Ilysian Grove (rules.dryad_colors): p's lands tap for any colour in its identity
pub fn dryad_colors(_g: &Game, _p: PlayerId) -> bool {
    false
}

/// PORT(M5): a land with its own colour rules (Plaza of Heroes, Unclaimed Territory, CI.LAND_COLS: Vivid lands,
/// Gemstone Mine ...). None: the land's `c` tag decides.
pub fn land_colors(_g: &Game, _p: PlayerId, _l: LandId) -> Option<crate::cards::Colors> {
    None
}

/// PORT(M5): CI.DYN_MANA, mana a source makes now (Gaea's Cradle, Priest of Titania). None: its tag's amount.
pub fn dyn_mana_land(_g: &Game, _p: PlayerId, _l: LandId) -> Option<u32> {
    None
}

/// PORT(M5): CI.DYN_MANA for a nonland permanent
pub fn dyn_mana_perm(_g: &Game, _p: PlayerId, _m: PermId) -> Option<u32> {
    None
}

/// PORT(M5): CI.locked(g, m, kind): an Aura locks m (Arrest: no activated abilities; Pacify: can't attack)
pub fn locked(_g: &Game, _m: PermId, _kind: &str) -> bool {
    false
}

/// PORT(M5): rules.halfling_colors (Delighted Halfling)
pub fn halfling_colors(_g: &Game, _p: PlayerId) -> Option<crate::cards::Colors> {
    None
}

/// PORT(M5): mine.is_legendary (Champion's Helm's hexproof needs a legendary creature)
pub fn is_legendary(g: &Game, m: PermId) -> bool {
    use crate::tag::Tag;
    g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Leg))
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

/// PORT(M4): what a permanent is worth removing beyond its tags (cardimpl.threat_value: the per-card table and
/// the combo pieces' value)
pub fn threat_value(_g: &Game, _m: PermId) -> f64 {
    0.0
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

/// PORT(M5): CI.ON_TAP for a land (Vivid lands' charge counters ...)
pub fn on_tap_land(_g: &mut Game, _p: PlayerId, _l: LandId, _n: u32) -> crate::flow::Res {
    Ok(())
}

/// PORT(M5): CI.ON_TAP for a permanent (Heritage Druid, Mana Vault's damage ...)
pub fn on_tap_perm(_g: &mut Game, _p: PlayerId, _m: PermId, _n: u32) -> crate::flow::Res {
    Ok(())
}

/// PORT(M5): engine.SELF_COST, a card's own cost change (Draco's domain, delve)
pub fn self_cost(_g: &Game, _p: PlayerId, _c: crate::ids::CardId) -> i32 {
    0
}

use crate::flow::Res;
use crate::ids::CardId;
use crate::sym::Sym;

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

/// PORT(M4): rules.tithe_unpaid, p pays Smothering Tithe's {2} only when it can spare it (ais.spare_after); true:
/// unpaid, so the Tithe's owner gets a Treasure. Until then: pays whenever it can.
pub fn tithe_unpaid(g: &mut Game, p: PlayerId) -> Res<bool> {
    if crate::engine::mana::can_pay(g, p, 2, "", false) {
        crate::engine::mana::pay(g, p, 2, "", false)?;
        return Ok(false);
    }
    Ok(true)
}

/// PORT(M4): rules.rhystic_unpaid, as tithe_unpaid for Rhystic Study's {1}
pub fn rhystic_unpaid(g: &mut Game, p: PlayerId) -> Res<bool> {
    if crate::engine::mana::can_pay(g, p, 1, "", false) {
        crate::engine::mana::pay(g, p, 1, "", false)?;
        return Ok(false);
    }
    Ok(true)
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
    /// PORT(M5): mine.kindred_enter (Kindred Discovery draws)
    fn kindred_enter(p: PlayerId, m: PermId);
    /// PORT(M5): rules2.chatterfang_squirrels
    fn chatterfang_squirrels(p: PlayerId, n: i32);
    /// PORT(M5): common.aura_fall (Auras on a permanent that left)
    fn aura_fall(m: PermId);
    /// PORT(M5): partials.bestow_fall
    fn bestow_fall(m: PermId);
    /// PORT(M5): marchesa.marchesa_dies
    fn marchesa_dies(p: PlayerId, m: PermId);
    /// PORT(M5): CI.gy_cards(owner, 'gy_dies') hooks of cards in the graveyard
    fn gy_dies(owner: PlayerId, m: PermId);
    /// PORT(M5): CI.AS_ENTERS (naming a creature type as it enters)
    fn as_enters(p: PlayerId, m: PermId);
    /// PORT(M4): topdeck.scry (scry n, or surveil n): until then, the order stays
    fn scry(p: PlayerId, n: i32, surveil: bool);
    /// PORT(M5): CI.gy_cards(p, 'gy_landfall')
    fn gy_landfall(p: PlayerId);
    /// PORT(M5): partials.answer_ability (Tishana's Tidebinder, Azorius Guildmage)
    fn answer_ability(q: PlayerId, item: u32);
    /// PORT(M5): rules.emblem_cast
    fn emblem_cast(p: PlayerId, c: CardId);
    /// PORT(M5): rules2.glimpse_draw
    fn glimpse_draw(p: PlayerId, c: CardId);
    /// PORT(M5): CI.kaervek
    fn kaervek(q: PlayerId, p: PlayerId, c: CardId);
    /// PORT(M5): CI.SELF_CAST ("when you cast this spell": cascade)
    fn self_cast(p: PlayerId, c: CardId);
    /// PORT(M5): CI.hand_cards(p, 'hand_cast') (Return the Favor)
    fn hand_cast(p: PlayerId, c: CardId);
    /// PORT(M5): CI.hand_cards(q, 'hand_opp_cast') (Dualcaster Mage)
    fn hand_opp_cast(q: PlayerId, c: CardId);
    /// PORT(M5): CI.muld_mark (Muldrotha's permanent types used this turn)
    fn muld_mark(p: PlayerId, kind: Sym);
    /// PORT(M5): CI.ring_tempt (The Ring tempts you)
    fn ring_tempt(p: PlayerId);
    /// PORT(M5): t1.tutor_named for Auras (Three Dreams)
    fn tutor_auras(p: PlayerId, k: usize);
    /// PORT(M5): mine.proliferate_all
    fn proliferate_all(p: PlayerId);
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
    /// PORT(M5): CI.SELF_REGEN
    fn self_regen(m: PermId);
    /// PORT(M5): partials.tidebinder_response
    fn tidebinder_response(p: PlayerId, m: PermId);
    /// PORT(M5): rules2.hullbreaker_counter
    fn hullbreaker_counter(q: PlayerId, c: CardId);
    /// PORT(M5): rules.veil_response
    fn veil_response(p: PlayerId, q: PlayerId, ctr: CardId);
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

/// PORT(M4): combos: a modeled combo is ready in p's hand and battlefield
pub fn combo_ready(_g: &Game, _p: PlayerId) -> bool {
    false
}

// ------------------------------------------------------------------ M3: combat and the turn
/// PORT(M5): rules.evasion_blocked (menace-style and type-based evasion from card code)
pub fn evasion_blocked(_g: &Game, _b: PermId, _a: PermId) -> bool {
    false
}

/// PORT(M5): CI.granted_kw (keywords card code grants)
pub fn granted_kw(_g: &Game, _m: PermId, _kw: &str) -> bool {
    false
}

/// PORT(M5): mine.ring_unblockable (the Ring, level 1)
pub fn ring_unblockable(_g: &Game, _b: PermId, _a: PermId) -> bool {
    false
}

/// PORT(M5): mine.ring_blocked (the Ring, level 3)
pub fn ring_blocked(_g: &Game, _p: PlayerId, _a: PermId, _b: PermId) -> bool {
    false
}

/// PORT(M5): mine.damage_prevented (Old Fat Spider's chapter II)
pub fn damage_prevented(_g: &Game, _m: PermId) -> bool {
    false
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

/// PORT(M5): mine.necromancer_attack (an attacking token copy of a graveyard creature)
pub fn necromancer_attack(_g: &mut Game, _p: PlayerId, _m: PermId) -> Res<Vec<PermId>> {
    Ok(vec![])
}

/// PORT(M5): CI.keyword_attack (keyword attack triggers: annihilator, myriad ...)
pub fn keyword_attack(_g: &mut Game, _p: PlayerId, _atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    Ok(vec![])
}

/// PORT(M5): CI.fire(g, 'blocks', ...) hooks that change blocks
pub fn blocks_hooks(
    _g: &mut Game,
    _p: PlayerId,
    _atk: &[PermId],
    _d: PlayerId,
    _assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    Ok(())
}

/// PORT(M5): hand_blocks, Aetherize, the defender's 'defend' hooks, land_defend, ninjutsu
pub fn defend_hooks(
    _g: &mut Game,
    _p: PlayerId,
    _atk: &[PermId],
    _d: PlayerId,
    _assign: &mut Vec<(PermId, PermId)>,
) -> Res {
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

/// PORT(M5): mine.ring_damage (the Ring, level 4)
pub fn ring_damage(_g: &mut Game, _p: PlayerId, _a: PermId, _d: PlayerId) -> Res {
    Ok(())
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

/// PORT(M5): mine.ring_attack (the Ring, level 2)
pub fn ring_attack(_g: &mut Game, _p: PlayerId, _atk: &[PermId]) -> Res {
    Ok(())
}

/// PORT(M5): CI.hand_cards(p, 'hand_attack')
pub fn hand_attack(_g: &mut Game, _p: PlayerId, _atk: &[PermId], _d: PlayerId) -> Res {
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

/// PORT(M5): CI.LAND_ETB (Bojuka Bog ...)
pub fn land_etb(_g: &mut Game, _p: PlayerId, _l: LandId) -> Res {
    Ok(())
}

/// PORT(M5): CI.total(g, 'extra_lands', p) (Exploration, Azusa ...)
pub fn extra_lands(_g: &Game, _p: PlayerId) -> u32 {
    0
}

/// PORT(M5): lands_from_top (Oracle of Mul Daya)
pub fn lands_from_top(_g: &Game, _p: PlayerId) -> bool {
    false
}

/// PORT(M5): lands_from_gy (Crucible of Worlds)
pub fn lands_from_gy(_g: &Game, _p: PlayerId) -> bool {
    false
}

port_res! {
    /// PORT(M5): CI.turn_start
    fn turn_start(p: PlayerId);
    /// PORT(M5): CI.suspend_upkeep
    fn suspend_upkeep(p: PlayerId);
    /// PORT(M5): ais.mirror_upkeep (Panoptic Mirror)
    fn mirror_upkeep(p: PlayerId);
    /// PORT(M5): ais.braids_sacrifice
    fn braids_sacrifice(p: PlayerId);
    /// PORT(M5): mine.spider_chapter2
    fn spider_chapter2(p: PlayerId, m: PermId);
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

/// PORT(M5): CI.total(g, 'skip_draw', p) (Solitary Confinement)
pub fn skip_draw(_g: &Game, _p: PlayerId) -> bool {
    false
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
