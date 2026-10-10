//! Hand-written card code (Python's `cardimpl.py` registry and `cards/impl/*.py`).
//!
//! Python registers a function per (card name, event) with `@on(name, event)`, and the engine finds the hooked
//! permanents for an event by walking `g.hooks`. Here each card with code has a `CardImpl`: one typed function slot
//! per event, so a hook with the wrong shape doesn't compile. The `Registry` maps `CardId` to its `CardImpl`; card
//! modules fill it at startup by card name.
//!
//! Slots are added to `CardImpl` event by event, as the engine code that fires each event is ported (M2, M3).
//!
//! Triggered abilities don't run when their event fires: they wait in `Game::trig_queue` as a `Trigger`, then go on
//! the stack. Python stores the hook function and its arguments; here a `Trigger` stores the event and its
//! arguments as data (`Call`), and the hook is found again in the registry when it resolves, so a game copy
//! (which may hold pending triggers) stays plain data.

use crate::cards::CardDb;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::sym::Sym;

/// Every event card code can hook (Python: the second argument of `@on(name, event)`, and the events the engine
/// fires), sorted by name. `TRIGGER_EVENTS` are triggered abilities and go on the stack; the rest are asked for
/// a value at once (cost changes, locks, options).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Event {
    Attack,
    AttackCap,
    AttackTax,
    Blocks,
    BloodMoon,
    CanCast,
    CardsToGy,
    Cast,
    CombatDamage,
    CombatStart,
    Copycast,
    Cost,
    CreatureToGy,
    Crew,
    Defend,
    Dies,
    Discard,
    Draw,
    EndStep,
    Etb,
    ExiledFromBf,
    ExtraLands,
    ExtraMana,
    GainLife,
    GrantKw,
    GyDies,
    GyHate,
    GyLandfall,
    GyOptions,
    HandAttack,
    HandCast,
    HandDefend,
    HandOptions,
    LandDefend,
    LandGy,
    LandMana,
    LandOptions,
    LandPlay,
    LandUpkeep,
    Landfall,
    LandsFromGy,
    LandsFromTop,
    Leaves,
    LoseLife,
    LoyaltyExtra,
    Main1,
    Main2,
    ManaLock,
    ManaTapped,
    MinCost,
    Monarch,
    Ninjutsu,
    NoArtifactMana,
    NoCreatureMana,
    NoGraveyard,
    NoLifegain,
    NonlandManaBonus,
    Options,
    PlayerHexproof,
    PreventDamage,
    ProliferateExtra,
    Proliferated,
    Rebound,
    Regenerate,
    Resolve,
    Sacrifice,
    Sba,
    SearchLimit,
    SelfDies,
    SkipDraw,
    StealDraw,
    TokenCreated,
    TokensEnter,
    TreasureBonus,
    TriggerCopies,
    Uncounterable,
    Upkeep,
}

pub const EVENTS: [(Event, &str); 77] = [
    (Event::Attack, "attack"),
    (Event::AttackCap, "attack_cap"),
    (Event::AttackTax, "attack_tax"),
    (Event::Blocks, "blocks"),
    (Event::BloodMoon, "blood_moon"),
    (Event::CanCast, "can_cast"),
    (Event::CardsToGy, "cards_to_gy"),
    (Event::Cast, "cast"),
    (Event::CombatDamage, "combat_damage"),
    (Event::CombatStart, "combat_start"),
    (Event::Copycast, "copycast"),
    (Event::Cost, "cost"),
    (Event::CreatureToGy, "creature_to_gy"),
    (Event::Crew, "crew"),
    (Event::Defend, "defend"),
    (Event::Dies, "dies"),
    (Event::Discard, "discard"),
    (Event::Draw, "draw"),
    (Event::EndStep, "end_step"),
    (Event::Etb, "etb"),
    (Event::ExiledFromBf, "exiled_from_bf"),
    (Event::ExtraLands, "extra_lands"),
    (Event::ExtraMana, "extra_mana"),
    (Event::GainLife, "gain_life"),
    (Event::GrantKw, "grant_kw"),
    (Event::GyDies, "gy_dies"),
    (Event::GyHate, "gy_hate"),
    (Event::GyLandfall, "gy_landfall"),
    (Event::GyOptions, "gy_options"),
    (Event::HandAttack, "hand_attack"),
    (Event::HandCast, "hand_cast"),
    (Event::HandDefend, "hand_defend"),
    (Event::HandOptions, "hand_options"),
    (Event::LandDefend, "land_defend"),
    (Event::LandGy, "land_gy"),
    (Event::LandMana, "land_mana"),
    (Event::LandOptions, "land_options"),
    (Event::LandPlay, "land_play"),
    (Event::LandUpkeep, "land_upkeep"),
    (Event::Landfall, "landfall"),
    (Event::LandsFromGy, "lands_from_gy"),
    (Event::LandsFromTop, "lands_from_top"),
    (Event::Leaves, "leaves"),
    (Event::LoseLife, "lose_life"),
    (Event::LoyaltyExtra, "loyalty_extra"),
    (Event::Main1, "main1"),
    (Event::Main2, "main2"),
    (Event::ManaLock, "mana_lock"),
    (Event::ManaTapped, "mana_tapped"),
    (Event::MinCost, "min_cost"),
    (Event::Monarch, "monarch"),
    (Event::Ninjutsu, "ninjutsu"),
    (Event::NoArtifactMana, "no_artifact_mana"),
    (Event::NoCreatureMana, "no_creature_mana"),
    (Event::NoGraveyard, "no_graveyard"),
    (Event::NoLifegain, "no_lifegain"),
    (Event::NonlandManaBonus, "nonland_mana_bonus"),
    (Event::Options, "options"),
    (Event::PlayerHexproof, "player_hexproof"),
    (Event::PreventDamage, "prevent_damage"),
    (Event::ProliferateExtra, "proliferate_extra"),
    (Event::Proliferated, "proliferated"),
    (Event::Rebound, "rebound"),
    (Event::Regenerate, "regenerate"),
    (Event::Resolve, "resolve"),
    (Event::Sacrifice, "sacrifice"),
    (Event::Sba, "sba"),
    (Event::SearchLimit, "search_limit"),
    (Event::SelfDies, "self_dies"),
    (Event::SkipDraw, "skip_draw"),
    (Event::StealDraw, "steal_draw"),
    (Event::TokenCreated, "token_created"),
    (Event::TokensEnter, "tokens_enter"),
    (Event::TreasureBonus, "treasure_bonus"),
    (Event::TriggerCopies, "trigger_copies"),
    (Event::Uncounterable, "uncounterable"),
    (Event::Upkeep, "upkeep"),
];

/// Python's `engine.TRIGGER_EVENTS`
pub const TRIGGER_EVENTS: [Event; 24] = [
    Event::Attack,
    Event::Blocks,
    Event::CardsToGy,
    Event::Cast,
    Event::CombatDamage,
    Event::CombatStart,
    Event::Copycast,
    Event::CreatureToGy,
    Event::Dies,
    Event::Discard,
    Event::Draw,
    Event::EndStep,
    Event::Etb,
    Event::ExiledFromBf,
    Event::GainLife,
    Event::LandGy,
    Event::Landfall,
    Event::Leaves,
    Event::LoseLife,
    Event::Proliferated,
    Event::Sacrifice,
    Event::SelfDies,
    Event::TokenCreated,
    Event::Upkeep,
];
impl Event {
    pub fn name(self) -> &'static str {
        EVENTS[self as usize].1
    }

    pub fn from_name(name: &str) -> Option<Event> {
        EVENTS.binary_search_by(|e| e.1.cmp(name)).ok().map(|i| EVENTS[i].0)
    }

    pub fn is_trigger(self) -> bool {
        TRIGGER_EVENTS.contains(&self)
    }
}

// ------------------------------------------------------------------ hook signatures
/// The permanent whose code is running (Python's `src`).
pub type Src = PermId;

/// etb(g, src, p, m): m entered under p's control
pub type EtbFn = fn(&mut Game, Src, PlayerId, PermId) -> Res;
/// leaves(g, src, m): m left the battlefield (any way)
pub type LeavesFn = fn(&mut Game, Src, PermId) -> Res;
/// dies(g, src, m, cause): any permanent m died
pub type DiesFn = fn(&mut Game, Src, PermId, Sym) -> Res;
/// upkeep, end_step, main1, main2, draw, landfall, monarch (g, src, p): something happened to or for player p
pub type PlayerFn = fn(&mut Game, Src, PlayerId) -> Res;
/// cast(g, src, caster, c): any player cast c
pub type CastFn = fn(&mut Game, Src, PlayerId, CardId) -> Res;
/// attack(g, src, p, atk, d): p attacked d with atk; returns new attacking creatures
pub type AttackFn = fn(&mut Game, Src, PlayerId, &[PermId], PlayerId) -> Res<Vec<PermId>>;
/// combat_damage(g, src, p, a, d, dmg): attacker a dealt dmg combat damage to player d
pub type CombatDamageFn = fn(&mut Game, Src, PlayerId, PermId, PlayerId, i32) -> Res;
/// options(g, src, p, post): activated abilities the AI may use (post: None at the end-of-turn window)
pub type OptionsFn = fn(&mut Game, Src, PlayerId, Option<bool>) -> Res<Vec<Opt>>;
/// cost(g, src, caster, c): generic mana added to (+) or removed from (-) c's cost
pub type CostFn = fn(&Game, Src, PlayerId, CardId) -> i32;
/// can_cast(g, src, caster, c, zone): false forbids casting c
pub type CanCastFn = fn(&Game, Src, PlayerId, CardId, Sym) -> bool;
/// attack_tax / attack_cap (g, src, attacker, d): mana per attacker / the most creatures that can attack d
pub type AttackLimitFn = fn(&Game, Src, PlayerId, PlayerId) -> Option<i32>;
/// grant_kw(g, src, m, kw): a static keyword grant
pub type GrantKwFn = fn(&Game, Src, PermId, Sym) -> bool;
/// prevent_damage, no_lifegain, player_hexproof (g, src, p): a static that applies to player p
pub type PlayerStaticFn = fn(&Game, Src, PlayerId) -> bool;
/// lose_life, gain_life (g, src, p, n): player p lost or gained n life
pub type LifeFn = fn(&mut Game, Src, PlayerId, i32) -> Res;
/// sba(g, src): state-based checks of a card's own (run at once)
pub type SbaFn = fn(&mut Game, Src) -> Res;
/// mana_lock, no_creature_mana, no_artifact_mana, nonland_mana_bonus, treasure_bonus (g, src, p): a count summed
/// over the hooked permanents (Python's `CI.total`)
pub type PlayerCountFn = fn(&Game, Src, PlayerId) -> i32;
/// land_mana(g, src, p, land): extra mana a land of p's makes
pub type LandManaFn = fn(&Game, Src, PlayerId, crate::ids::LandId) -> i32;
/// extra_mana(g, src, p, units): mana sources a card adds (Urza, Kinnan)
pub type ExtraManaFn = fn(&Game, Src, PlayerId, &[crate::engine::mana::Unit]) -> Vec<crate::engine::mana::Unit>;
/// steal_draw(g, src, p): Notion Thief takes p's extra draw
pub type StealDrawFn = fn(&Game, Src, PlayerId) -> bool;
/// trigger_copies(g, src, p, kind, x): extra copies of p's triggered abilities ('etb', 'dies', 'landfall')
pub type TriggerCopiesFn = fn(&Game, Src, PlayerId, Sym, Option<PermId>) -> i32;
/// search_limit(g, src, p): p may only search the top n cards (Aven Mindcensor)
pub type SearchLimitFn = fn(&Game, Src, PlayerId) -> Option<u32>;
/// uncounterable(g, src, caster, c): c can't be countered
pub type UncounterableFn = fn(&Game, Src, PlayerId, CardId) -> bool;
/// resolve(g, p, c, ctx): a hand-written instant or sorcery resolves; returns where the card goes ('gy', 'exile',
/// 'handled' when its code moved it)
pub type ResolveFn = fn(&mut Game, PlayerId, CardId, &crate::state::Ctx) -> Res<Sym>;
/// mana_tapped(g, src, p, m, n): p tapped permanent m for n mana (run at once)
pub type ManaTappedFn = fn(&mut Game, Src, PlayerId, PermId, u32) -> Res;
/// gy_options / hand_options (g, c, p, post): plays from a card in p's graveyard or hand (post: None at the end-of-turn
/// window)
pub type CardOptionsFn = fn(&mut Game, CardId, PlayerId, Option<bool>) -> Res<Vec<Opt>>;
/// sacrifice(g, src, p, what): p sacrificed a permanent, or a Treasure / Food / Clue
pub type SacrificeFn = fn(&mut Game, Src, PlayerId, Sacrificed) -> Res;
/// token_created(g, src, p, kinds, n): p made n Treasure / Food / Clue tokens of these kinds
pub type TokenCreatedFn = fn(&mut Game, Src, PlayerId, &[Sym], i32) -> Res;
/// The attackers' blockers: (attacker, blocker) pairs.
pub type Assign = Vec<(PermId, PermId)>;
/// blocks(g, src, p, atk, d, assign): d's blockers are declared against p's attackers; may change the blocks
pub type BlocksFn = fn(&mut Game, Src, PlayerId, &[PermId], PlayerId, &mut Assign) -> Res;
/// hand_defend(g, c, d, p, atk, assign): a card in defender d's hand answers p's attack (Aetherize)
pub type HandDefendFn = fn(&mut Game, CardId, PlayerId, PlayerId, &[PermId], &mut Assign) -> Res;
/// land_defend(g, land, d, p, atk, assign): a land of defender d's answers p's attack (Kor Haven)
pub type LandDefendFn = fn(&mut Game, crate::ids::LandId, PlayerId, PlayerId, &[PermId], &mut Assign) -> Res;
/// land_options(g, land, p, post): a land's activated abilities the AI may use (post: None at the end-of-turn window)
pub type LandOptionsFn = fn(&mut Game, crate::ids::LandId, PlayerId, Option<bool>) -> Res<Vec<Opt>>;
/// land_upkeep(g, land, p): p's upkeep, for a land of p's (Emeria, the Sky Ruin)
pub type LandUpkeepFn = fn(&mut Game, crate::ids::LandId, PlayerId) -> Res;
/// gy_dies(g, c, owner, m): card c in owner's graveyard sees owner's creature m die (Nether Traitor)
pub type GyDiesFn = fn(&mut Game, CardId, PlayerId, PermId) -> Res;
/// gy_landfall(g, c, p): card c in p's graveyard sees a land enter under p's control
pub type GyPlayerFn = fn(&mut Game, CardId, PlayerId) -> Res;
/// hand_cast / hand_opp_cast (g, c, p, spell): card c in p's hand sees spell cast (Return the Favor, Dualcaster Mage)
pub type HandCastFn = fn(&mut Game, CardId, PlayerId, CardId) -> Res;
/// hand_blocks(g, c, p, atk, d, assign): card c in attacker p's hand after blocks (ninjutsu-style)
pub type HandBlocksFn = fn(&mut Game, CardId, PlayerId, &[PermId], PlayerId, &mut Assign) -> Res;
/// hand_attack(g, c, p, atk, d): card c in p's hand as p attacks d
pub type HandAttackFn = fn(&mut Game, CardId, PlayerId, &[PermId], PlayerId) -> Res;
/// defend(g, src, d, p, atk, assign): the defender's permanents after blocks (Yawgmoth)
pub type DefendFn = fn(&mut Game, Src, PlayerId, PlayerId, &[PermId], &mut Assign) -> Res;
/// SELF_CAST(g, p, c): "when you cast this spell" (cascade)
pub type SelfCastFn = fn(&mut Game, PlayerId, CardId) -> Res;
/// AS_ENTERS(g, p, m): as m enters, before any trigger (naming a creature type)
pub type AsEntersFn = fn(&mut Game, PlayerId, PermId) -> Res;
/// SELF_REGEN(g, m): true if m regenerates instead of being destroyed
pub type SelfRegenFn = fn(&mut Game, PermId) -> Res<bool>;
/// ON_TAP for a land / a permanent (g, p, source, n): after it's tapped for n mana (Vivid lands, Heritage Druid)
pub type OnTapLandFn = fn(&mut Game, PlayerId, crate::ids::LandId, u32) -> Res;
pub type OnTapPermFn = fn(&mut Game, PlayerId, PermId, u32) -> Res;
/// DYN_MANA for a land / a permanent (g, p, source): the mana its tap ability makes now (Gaea's Cradle, Priest of
/// Titania)
pub type DynManaLandFn = fn(&Game, PlayerId, crate::ids::LandId) -> u32;
pub type DynManaPermFn = fn(&Game, PlayerId, PermId) -> u32;
/// LAND_ETB(g, p, land): a land played as a land drop enters (Bojuka Bog)
pub type LandEtbFn = fn(&mut Game, PlayerId, crate::ids::LandId) -> Res;
/// LAND_COLS(g, p, land): the colours a land can make now (Vivid lands, Gemstone Mine)
pub type LandColsFn = fn(&Game, PlayerId, crate::ids::LandId) -> crate::cards::Colors;
/// a card's own cast priority (0-90, 0: not now): Python's `CI.SPELL_PRIO`
pub type PrioFn = fn(&Game, PlayerId, CardId) -> i32;
/// a spell's importance to counter (0-9): Python's `CI.SPELL_IMP`
pub type SpellImpFn = fn(&Game, PlayerId, CardId) -> f64;
/// SELF_PT(g, p, m): a creature's own power/toughness rule, read while `g.selfpt` is on (common.SELF_PT: Kor
/// Spiritdancer ...)
pub type SelfPtFn = fn(&Game, PlayerId, PermId) -> (i32, i32);
/// an entry of common.TOKEN_PT (tokens with data: Urza's Construct) or common.CREATURE_PT (any creature: the
/// Banners): (g, m) -> (dp, dt)
pub type PtFn = fn(&Game, PermId) -> (i32, i32);

/// One card's code: a slot per event it handles.
#[derive(Debug, Clone, Copy, Default)]
pub struct CardImpl {
    pub etb: Option<EtbFn>,
    pub leaves: Option<LeavesFn>,
    pub dies: Option<DiesFn>,
    pub self_dies: Option<DiesFn>,
    pub upkeep: Option<PlayerFn>,
    pub end_step: Option<PlayerFn>,
    pub main1: Option<PlayerFn>,
    pub main2: Option<PlayerFn>,
    pub draw: Option<PlayerFn>,
    pub landfall: Option<PlayerFn>,
    pub cast: Option<CastFn>,
    pub attack: Option<AttackFn>,
    pub combat_damage: Option<CombatDamageFn>,
    pub options: Option<OptionsFn>,
    pub cost: Option<CostFn>,
    pub can_cast: Option<CanCastFn>,
    pub attack_tax: Option<AttackLimitFn>,
    pub attack_cap: Option<AttackLimitFn>,
    pub grant_kw: Option<GrantKwFn>,
    pub prevent_damage: Option<PlayerStaticFn>,
    pub no_lifegain: Option<PlayerStaticFn>,
    pub player_hexproof: Option<PlayerStaticFn>,
    pub lose_life: Option<LifeFn>,
    pub gain_life: Option<LifeFn>,
    pub sba: Option<SbaFn>,
    pub min_cost: Option<CostFn>,
    pub mana_lock: Option<PlayerCountFn>,
    pub no_creature_mana: Option<PlayerCountFn>,
    pub no_artifact_mana: Option<PlayerCountFn>,
    pub nonland_mana_bonus: Option<PlayerCountFn>,
    pub treasure_bonus: Option<PlayerCountFn>,
    pub land_mana: Option<LandManaFn>,
    pub extra_mana: Option<ExtraManaFn>,
    pub mana_tapped: Option<ManaTappedFn>,
    pub steal_draw: Option<StealDrawFn>,
    pub trigger_copies: Option<TriggerCopiesFn>,
    pub search_limit: Option<SearchLimitFn>,
    pub uncounterable: Option<UncounterableFn>,
    /// no_graveyard (Rest in Peace): a count over the hooked permanents
    pub no_graveyard: Option<PlayerCountFn>,
    pub resolve: Option<ResolveFn>,
    pub gy_options: Option<CardOptionsFn>,
    pub hand_options: Option<CardOptionsFn>,
    /// extra land drops for p (Exploration, Azusa): a count over the hooked permanents
    pub extra_lands: Option<PlayerCountFn>,
    /// p may play lands from the graveyard (Crucible of Worlds) / the top of the library (Oracle of Mul Daya)
    pub lands_from_gy: Option<PlayerCountFn>,
    pub lands_from_top: Option<PlayerCountFn>,
    pub blocks: Option<BlocksFn>,
    pub sacrifice: Option<SacrificeFn>,
    /// land_play(g, src, p, c): p played land c
    pub land_play: Option<CastFn>,
    pub token_created: Option<TokenCreatedFn>,
    /// discard(g, src, q, c): q discarded c
    pub discard: Option<CastFn>,
    /// land_gy(g, src, p, c): land card c of p's went to the graveyard from the battlefield
    pub land_gy: Option<CastFn>,
    pub hand_defend: Option<HandDefendFn>,
    pub land_defend: Option<LandDefendFn>,
    pub land_options: Option<LandOptionsFn>,
    pub land_upkeep: Option<LandUpkeepFn>,
    pub gy_dies: Option<GyDiesFn>,
    pub gy_landfall: Option<GyPlayerFn>,
    pub hand_cast: Option<HandCastFn>,
    pub hand_opp_cast: Option<HandCastFn>,
    pub hand_blocks: Option<HandBlocksFn>,
    pub hand_attack: Option<HandAttackFn>,
    pub defend: Option<DefendFn>,
    /// skip_draw(g, src, p): p skips its draw step (Solitary Confinement): a count over the hooked permanents
    pub skip_draw: Option<PlayerCountFn>,
    // Python's per-card tables (CI.SELF_CAST, AS_ENTERS, SELF_REGEN, ON_TAP, DYN_MANA, LAND_ETB, LAND_COLS)
    pub self_cast: Option<SelfCastFn>,
    pub as_enters: Option<AsEntersFn>,
    pub self_regen: Option<SelfRegenFn>,
    pub on_tap_land: Option<OnTapLandFn>,
    pub on_tap_perm: Option<OnTapPermFn>,
    pub dyn_mana_land: Option<DynManaLandFn>,
    pub dyn_mana_perm: Option<DynManaPermFn>,
    pub land_etb: Option<LandEtbFn>,
    pub land_cols: Option<LandColsFn>,
    /// rebound(g, p, c): a rebound spell in p's exile is cast again at p's upkeep
    pub rebound: Option<SelfCastFn>,
    /// the AI's cast priority for the card (Python's `CI.SPELL_PRIO`, a number or a function)
    pub prio: Option<PrioFn>,
    /// how much opponents want to counter it (Python's `CI.SPELL_IMP`)
    pub spell_imp: Option<SpellImpFn>,
    /// a fixed value of removing it (Python's `CI.PVAL`)
    pub pval: Option<f64>,
    /// an Aura's bonus, keywords and host rule (common.AURA; set by `impls::common::aura`)
    pub aura: Option<&'static crate::impls::common::AuraSpec>,
    /// a planeswalker's loyalty abilities for the walker framework (common.WALKERS; set by `impls::common::walker`)
    pub walker: Option<&'static [crate::impls::common::WalkerAb]>,
    /// common.SELF_PT
    pub self_pt: Option<SelfPtFn>,
    /// triggered-event hooks that run at once instead of going on the stack: Python's hooks without a
    /// `trigger_window` call (cardimpl `converted` is false). A bit per `Event`.
    pub at_once: u128,
}

impl CardImpl {
    /// Mark a triggered-event hook as running at once (not through the stack).
    pub fn at_once(mut self, e: Event) -> CardImpl {
        self.at_once |= 1u128 << (e as u32);
        self
    }

    pub fn runs_at_once(&self, e: Event) -> bool {
        self.at_once >> (e as u32) & 1 == 1
    }

    /// Python's `CI.live(name)` (`name in HOOKS`): the card handles an event, rather than only having a table entry
    /// (PVAL, SPELL_PRIO, DYN_MANA, LAND_ETB ...). A live card's permanents go in `g.hooks`.
    pub fn live(&self) -> bool {
        EVENTS.iter().any(|&(e, _)| self.handles(e)) || self.hand_opp_cast.is_some() || self.hand_blocks.is_some()
    }

    /// Does the card handle `e`? (only for the events with a slot so far)
    pub fn handles(&self, e: Event) -> bool {
        match e {
            Event::Etb => self.etb.is_some(),
            Event::Leaves => self.leaves.is_some(),
            Event::Dies => self.dies.is_some(),
            Event::SelfDies => self.self_dies.is_some(),
            Event::Upkeep => self.upkeep.is_some(),
            Event::EndStep => self.end_step.is_some(),
            Event::Main1 => self.main1.is_some(),
            Event::Main2 => self.main2.is_some(),
            Event::Draw => self.draw.is_some(),
            Event::Landfall => self.landfall.is_some(),
            Event::Cast => self.cast.is_some(),
            Event::Attack => self.attack.is_some(),
            Event::CombatDamage => self.combat_damage.is_some(),
            Event::Options => self.options.is_some(),
            Event::Cost => self.cost.is_some(),
            Event::CanCast => self.can_cast.is_some(),
            Event::AttackTax => self.attack_tax.is_some(),
            Event::AttackCap => self.attack_cap.is_some(),
            Event::GrantKw => self.grant_kw.is_some(),
            Event::PreventDamage => self.prevent_damage.is_some(),
            Event::NoLifegain => self.no_lifegain.is_some(),
            Event::PlayerHexproof => self.player_hexproof.is_some(),
            Event::LoseLife => self.lose_life.is_some(),
            Event::GainLife => self.gain_life.is_some(),
            Event::Sba => self.sba.is_some(),
            Event::MinCost => self.min_cost.is_some(),
            Event::ManaLock => self.mana_lock.is_some(),
            Event::NoCreatureMana => self.no_creature_mana.is_some(),
            Event::NoArtifactMana => self.no_artifact_mana.is_some(),
            Event::NonlandManaBonus => self.nonland_mana_bonus.is_some(),
            Event::TreasureBonus => self.treasure_bonus.is_some(),
            Event::LandMana => self.land_mana.is_some(),
            Event::ExtraMana => self.extra_mana.is_some(),
            Event::ManaTapped => self.mana_tapped.is_some(),
            Event::StealDraw => self.steal_draw.is_some(),
            Event::TriggerCopies => self.trigger_copies.is_some(),
            Event::SearchLimit => self.search_limit.is_some(),
            Event::Uncounterable => self.uncounterable.is_some(),
            Event::NoGraveyard => self.no_graveyard.is_some(),
            Event::Resolve => self.resolve.is_some(),
            Event::GyOptions => self.gy_options.is_some(),
            Event::HandOptions => self.hand_options.is_some(),
            Event::ExtraLands => self.extra_lands.is_some(),
            Event::LandsFromGy => self.lands_from_gy.is_some(),
            Event::LandsFromTop => self.lands_from_top.is_some(),
            Event::Blocks => self.blocks.is_some(),
            Event::Sacrifice => self.sacrifice.is_some(),
            Event::LandPlay => self.land_play.is_some(),
            Event::TokenCreated => self.token_created.is_some(),
            Event::Discard => self.discard.is_some(),
            Event::LandGy => self.land_gy.is_some(),
            Event::HandDefend => self.hand_defend.is_some(),
            Event::LandDefend => self.land_defend.is_some(),
            Event::LandOptions => self.land_options.is_some(),
            Event::LandUpkeep => self.land_upkeep.is_some(),
            Event::GyDies => self.gy_dies.is_some(),
            Event::GyLandfall => self.gy_landfall.is_some(),
            Event::HandCast => self.hand_cast.is_some(),
            Event::HandAttack => self.hand_attack.is_some(),
            Event::Defend => self.defend.is_some(),
            Event::SkipDraw => self.skip_draw.is_some(),
            Event::Rebound => self.rebound.is_some(),
            _ => false,
        }
    }
}

/// Every card's code, by card id.
#[derive(Debug, Default)]
pub struct Registry {
    by_card: Vec<Option<CardImpl>>,
    /// common.TOKEN_PT: bonuses for tokens with data, summed (modules push theirs in `register`)
    pub token_pt: Vec<PtFn>,
    /// common.CREATURE_PT: bonuses for any creature while `g.selfpt` is on, summed
    pub creature_pt: Vec<PtFn>,
}

impl Registry {
    pub fn new(db: &CardDb) -> Registry {
        Registry { by_card: vec![None; db.len()], ..Registry::default() }
    }

    /// Register a card's code by name (Python's `@on(name, ...)`). A name not in the card database is an error, so
    /// a typo can't silently leave a card unimplemented.
    pub fn on(&mut self, db: &CardDb, name: &str, imp: CardImpl) -> Result<(), String> {
        let id = db.id(name).ok_or_else(|| format!("card code for {name:?}, which isn't in the card database"))?;
        let slot = &mut self.by_card[id.index()];
        if slot.is_some() {
            return Err(format!("{name}: card code registered twice"));
        }
        *slot = Some(imp);
        Ok(())
    }

    /// A card's code to fill in, by name: Python modules add events to the same card (`@on` in t1.py and
    /// partials.py), so modules set the slots they implement. A name not in the card database is an error.
    pub fn card(&mut self, db: &CardDb, name: &str) -> Result<&mut CardImpl, String> {
        let id = db.id(name).ok_or_else(|| format!("card code for {name:?}, which isn't in the card database"))?;
        Ok(self.by_card[id.index()].get_or_insert_with(CardImpl::default))
    }

    /// how many cards have code
    pub fn len(&self) -> usize {
        self.by_card.iter().filter(|x| x.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, c: CardId) -> Option<&CardImpl> {
        self.by_card.get(c.index())?.as_ref()
    }
}

// ------------------------------------------------------------------ triggers
/// A triggered ability's event and arguments, as data.
#[derive(Debug, Clone, PartialEq)]
pub enum Call {
    Etb {
        p: PlayerId,
        m: PermId,
    },
    Leaves {
        m: PermId,
    },
    Dies {
        m: PermId,
        cause: Sym,
    },
    Player {
        p: PlayerId,
    },
    Cast {
        caster: PlayerId,
        c: CardId,
    },
    Life {
        p: PlayerId,
        n: i32,
    },
    Sacrifice {
        p: PlayerId,
        what: Sacrificed,
    },
    /// cards_to_gy: these cards of p's are going to the graveyard
    Cards {
        p: PlayerId,
        cards: Vec<CardId>,
    },
    /// tokens_enter: these tokens entered under p's control
    Tokens {
        p: PlayerId,
        toks: Vec<PermId>,
    },
    /// discard: p discarded c
    Discard {
        p: PlayerId,
        c: CardId,
    },
    /// token_created: p made n Treasure / Food / Clue tokens of these kinds
    ArtifactTokens {
        p: PlayerId,
        kinds: Vec<Sym>,
        n: i32,
    },
    Attack {
        p: PlayerId,
        atk: Vec<PermId>,
        d: PlayerId,
    },
    CombatDamage {
        p: PlayerId,
        a: PermId,
        d: PlayerId,
        dmg: i32,
    },
}

/// What was sacrificed (Python passes a permanent, or 'Treasure' / 'Food' / 'Clue', or a land's card).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sacrificed {
    Perm(PermId),
    Token(Sym),
    Card(CardId),
}

/// A triggered ability waiting to go on the stack (Python's `Trigger`).
#[derive(Debug, Clone, PartialEq)]
pub struct Trigger {
    pub controller: PlayerId,
    /// its source permanent (None: a land's or the engine's own)
    pub src: Option<PermId>,
    pub act: TrigAct,
    pub name: Option<Sym>,
    /// it triggers for sure (the engine's and the ability language's), so no probe is needed
    pub known: bool,
    pub imp: Option<f64>,
    /// an enters trigger of a permanent that was cast
    pub cast_etb: bool,
}

/// What a trigger does when it resolves.
#[derive(Debug, Clone, PartialEq)]
pub enum TrigAct {
    /// a card's hook for this event (found again in the registry by its source's card)
    Hook { event: Event, call: Call },
    /// the engine's tag-driven enter effects (Python's `etb_once`)
    EtbOnce { p: PlayerId, m: PermId },
    /// a compiled triggered ability: ability `idx` of `card` on permanent `src`, with what triggered it
    Dsl { p: PlayerId, src: PermId, card: CardId, idx: u16, ctx: Box<crate::dsl::DslCtx> },
}

// ------------------------------------------------------------------ AI options
/// A play the AI may make: Python's `(utility, label, fn)`. The look-ahead finds the same option again in a copy of
/// the game by its label (and which occurrence of that label it is), so the action is data, re-created in each copy.
#[derive(Debug, Clone)]
pub struct Opt {
    pub utility: f64,
    pub label: String,
    /// None: "stop" (hold the mana, or move on)
    pub act: Option<Action>,
}

/// What an option does. Engine verbs are variants; card abilities name their card's function and arguments.
#[derive(Debug, Clone)]
pub enum Action {
    /// brain.do_cast: zone None (hand or command zone), 'gy' (flashback), 'yawg' (Yawgmoth's Will)
    Cast {
        card: CardId,
        zone: Option<Sym>,
    },
    /// brain.cast_removal: at one of `targets` (sampled by value as it's cast), kicked for `kick` more
    Removal {
        card: CardId,
        targets: Vec<PermId>,
        kick: u32,
        kind: Option<Sym>,
    },
    /// a board wipe (victim: the player a one-player wipe hits)
    Wipe {
        card: CardId,
        victim: Option<PlayerId>,
    },
    /// burn at player q's face (kick: the kicker paid)
    Face {
        card: CardId,
        q: PlayerId,
        kick: u32,
    },
    /// Crackle with Power for X
    Crackle {
        card: CardId,
        x: u32,
    },
    /// crack a Clue
    Clue,
    /// a deck plan's play (its function and an argument: a card or permanent id, or 0)
    Plan {
        f: PlanFn,
        arg: i64,
    },
    Ability {
        src: PermId,
        f: AbilityFn,
        arg: i64,
    },
    /// a compiled activated ability (dsl::activate)
    Dsl {
        src: PermId,
        idx: u16,
    },
    /// a compiled loyalty ability (dsl::use_loyalty)
    Loyalty {
        src: PermId,
        idx: u16,
    },
    /// equip src to target for {n} (dsl::equip)
    Equip {
        src: PermId,
        target: PermId,
        n: u32,
    },
}

/// A card ability chosen from `options`: returns whether it did anything (Python's `fn()` returning a truth value).
pub type AbilityFn = fn(&mut Game, PermId, PlayerId, i64) -> Res<bool>;
/// A deck plan's play (Python's closures in the AI's option lists): returns whether it did anything.
pub type PlanFn = fn(&mut Game, PlayerId, i64) -> Res<bool>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::tests::db;

    #[test]
    fn every_event_in_the_export_is_known() {
        for c in db().iter() {
            for e in c.python_hooks.iter() {
                assert!(Event::from_name(e).is_some(), "{}: event {e:?}", c.name);
            }
        }
        assert_eq!(Event::from_name("end_step"), Some(Event::EndStep));
        assert!(Event::EndStep.is_trigger() && !Event::Cost.is_trigger());
        assert_eq!(Event::Main1.name(), "main1");
    }

    fn noop(_: &mut Game, _: Src, _: PlayerId) -> Res {
        Ok(())
    }

    #[test]
    fn the_registry_finds_card_code_by_card() {
        let db = db();
        let mut r = Registry::new(&db);
        r.on(&db, "Conjurer's Closet", CardImpl { end_step: Some(noop), ..Default::default() }).unwrap();
        let closet = db.id("Conjurer's Closet").unwrap();
        assert!(r.get(closet).unwrap().handles(Event::EndStep));
        assert!(!r.get(closet).unwrap().handles(Event::Etb));
        assert!(r.get(db.id("Sol Ring").unwrap()).is_none());
        assert!(r.on(&db, "Conjurers Closet", CardImpl::default()).is_err(), "a misspelt name");
        assert!(r.on(&db, "Conjurer's Closet", CardImpl::default()).is_err(), "registered twice");
    }
}
