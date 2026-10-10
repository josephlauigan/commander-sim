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

/// rules.dryad_colors: Dryad of the Ilysian Grove: p's lands tap for any colour in its identity
pub fn dryad_colors(g: &Game, p: PlayerId) -> bool {
    crate::impls::rules::dryad_colors(g, p)
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

/// PORT(M5): CI.locked(g, m, kind) (zur.locked): an Aura locks m (Arrest: no activated abilities; Pacify: can't
/// attack). Only zur.py's lock Auras (Arrest, Prison Sentence, Luminous Bonds, Bound in Silence, Encrust) use it, and
/// no pilot deck runs one.
pub fn locked(_g: &Game, _m: PermId, _kind: &str) -> bool {
    false
}

/// rules.halfling_colors: Delighted Halfling's coloured mana only pays for legendary spells (None: colourless)
pub fn halfling_colors(g: &Game, p: PlayerId) -> Option<crate::cards::Colors> {
    crate::impls::rules::halfling_colors(g, p)
}

/// mine.is_legendary: a legendary card, or your Ring-bearer (Champion's Helm's hexproof needs a legendary creature)
pub fn is_legendary(g: &Game, m: PermId) -> bool {
    crate::impls::mine::is_legendary(g, m)
}

/// CI.attached_prot (common.attached_prot): colours m's Auras give it protection from
pub fn attached_prot(g: &Game, m: PermId) -> crate::cards::Colors {
    crate::impls::common::attached_prot(g, m)
}

/// PORT(M5): CI.garland_check, Garland's steals end when a player leaves
pub fn garland_check(_g: &mut Game) {}

/// your own Auras on m (common.auras_on, excluding locks: CI.LOCKS)
pub fn own_auras_on(g: &Game, m: PermId) -> usize {
    crate::impls::common::own_auras_on(g, m)
}

/// how close a planeswalker is to its ultimate (common.ult_pressure, from its WALKERS entry)
pub fn ult_pressure(g: &Game, m: PermId) -> f64 {
    crate::impls::common::ult_pressure(g, m)
}

/// cardimpl.threat_value: what a permanent is worth removing beyond its tags (its card code and compiled abilities,
/// and the value of a combo piece)
pub fn threat_value(g: &Game, m: PermId) -> f64 {
    let Some(c) = g.perm(m).cd else { return 0.0 };
    crate::ai::pool::card_threat_value(g, c) + piece_threat(g, m)
}

/// PORT(M5): how much of m's value is left under the Auras locking it (zur.lock_factor; zur.py's lock Auras only,
/// which no pilot deck runs)
pub fn lock_factor(_g: &Game, _m: PermId) -> f64 {
    1.0
}

/// partials.hand_mana, Elvish Spirit Guide and kin: exile from hand for mana
pub fn hand_mana(g: &Game, p: PlayerId) -> Vec<crate::engine::mana::Unit> {
    crate::impls::partials::hand_mana(g, p)
}

/// partials.special_unit_paid, paying with a Scion (sacrifice it) or a card in hand (exile it)
pub fn special_unit_paid(g: &mut Game, p: PlayerId, u: crate::engine::mana::Unit) -> crate::flow::Res {
    crate::impls::partials::special_unit_paid(g, p, u)
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

port_res! {
    /// PORT(M5): marchesa.marchesa_dies
    fn marchesa_dies(p: PlayerId, m: PermId);
    /// PORT(M5): CI.muld_mark (Muldrotha's permanent types used this turn)
    fn muld_mark(p: PlayerId, kind: Sym);
    /// PORT(M5): CI.apply_lock (zur.apply_lock: Arrest, Encrust ...; no pilot deck runs a lock Aura)
    fn apply_lock(actor: Option<PlayerId>, m: PermId, kind: Sym);
}

/// rules.emblem_draw: Teferi's emblem exiles an opposing permanent whenever p draws
pub fn emblem_draw(g: &mut Game, p: PlayerId) -> Res {
    crate::impls::rules::emblem_draw(g, p)
}

/// rules2.chatterfang_squirrels: Chatterfang makes that many Squirrels whenever p makes tokens
pub fn chatterfang_squirrels(g: &mut Game, p: PlayerId, n: i32) -> Res {
    crate::impls::rules2::chatterfang_squirrels(g, p, n)
}

/// partials.bestow_fall: a bestowed Eidolon on m becomes a creature when m leaves
pub fn bestow_fall(g: &mut Game, m: PermId) -> Res {
    crate::impls::partials::bestow_fall(g, m)
}

/// partials.answer_ability: counter an opponent's important ability on top of the stack (Tishana's Tidebinder,
/// Azorius Guildmage); `item` is the stack item's id
pub fn answer_ability(g: &mut Game, q: PlayerId, item: u32) -> Res {
    crate::impls::partials::answer_ability(g, q, item)
}

/// rules.emblem_cast: Chandra's and Venser's emblems on each spell p casts
pub fn emblem_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    crate::impls::rules::emblem_cast(g, p, c)
}

/// rules2.glimpse_draw: Glimpse of Nature draws for each creature spell
pub fn glimpse_draw(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    crate::impls::rules2::glimpse_draw(g, p, c)
}

/// rules.transform_away: becomes a 3/3 Elk, a 0/1 Insect (mutate) or a Forest, with no abilities
pub fn transform_away(g: &mut Game, m: PermId, kind: Sym) -> Res {
    crate::impls::rules::transform_away(g, m, kind)
}

/// rules.ezuri_regen: Ezuri regenerates another valuable Elf for {G}
pub fn ezuri_regen(g: &mut Game, m: PermId) -> Res<bool> {
    crate::impls::rules::ezuri_regen(g, m)
}

/// partials.tidebinder_response: an outside deck flashes in Tishana's Tidebinder to counter m's enters trigger
pub fn tidebinder_response(g: &mut Game, p: PlayerId, m: PermId) -> Res<bool> {
    crate::impls::partials::tidebinder_response(g, p, m)
}

/// rules2.hullbreaker_counter: Hullbreaker Horror bounces spell c when q casts a cheap instant
pub fn hullbreaker_counter(g: &mut Game, q: PlayerId, c: CardId) -> Res<bool> {
    crate::impls::rules2::hullbreaker_counter(g, q, c)
}

/// rules.veil_response: p answers q's counterspell ctr with Veil of Summer
pub fn veil_response(g: &mut Game, p: PlayerId, q: PlayerId, ctr: CardId) -> Res<bool> {
    crate::impls::rules::veil_response(g, p, q, ctr)
}

/// t1.tutor_named for Auras (Three Dreams): k Auras with different names to hand
pub fn tutor_auras(g: &mut Game, p: PlayerId, k: usize) -> Res {
    crate::impls::t1::tutor_named(g, p, &|g, c| g.db.get(c).has_subtype("aura"), k, "hand")?;
    Ok(())
}

/// common.AURA, for t1's Light-Paws and Flickering Ward: c has an entry in the Aura table (`c.name in IC.AURA`)
pub fn aura_known(g: &Game, c: CardId) -> bool {
    crate::impls::common::aura_spec(g, c).is_some()
}

/// common.AURA, for t1's Light-Paws: c's entry enchants your own creatures (`AURA[c]['target'] == 'own'`) and its
/// `host_ok` (if any) accepts host for player p
pub fn aura_own_fits(g: &Game, p: PlayerId, c: CardId, host: PermId) -> bool {
    crate::impls::common::aura_spec(g, c)
        .is_some_and(|a| a.target == "own" && a.host_ok.is_none_or(|ok| ok(g, p, host)))
}

/// common.SELF_PT for m (Kor Spiritdancer, Eidolon of Countless Battles ...): its own power/toughness rule, while
/// `g.selfpt` is on and its controller is in the game; (0, 0) without one. For common.attached_bonus.
pub fn self_pt(g: &Game, m: PermId) -> (i32, i32) {
    let x = g.perm(m);
    if !g.selfpt || !g.player(x.owner).alive {
        return (0, 0);
    }
    match x.cd.and_then(|c| imp_of(g, c)).and_then(|i| i.self_pt) {
        Some(f) => f(g, x.owner, m),
        None => (0, 0),
    }
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

/// common.aura_fall: Auras on a permanent that left go to the graveyard (or back to hand)
pub fn aura_fall(g: &mut Game, m: PermId) -> Res {
    crate::impls::common::aura_fall(g, m)
}

/// common.umbra_save: umbra armor: an umbra on m is destroyed instead of m
pub fn umbra_save(g: &mut Game, m: PermId) -> Res<bool> {
    crate::impls::common::umbra_save(g, m)
}

/// lands.try_regenerate: a land that regenerates (Yavimaya Hollow) saves m from destruction
pub fn try_regenerate(g: &mut Game, m: PermId) -> Res<bool> {
    crate::impls::lands::try_regenerate(g, m)
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

/// rules2.aura_ward: ward {2} from Sheltered by Ghosts
pub fn aura_ward(g: &Game, m: PermId) -> u32 {
    crate::impls::rules2::aura_ward(g, m)
}

/// rules.removal_taxes (Terror of the Peaks, Phyrexian Obliterator): false stops the removal
pub fn removal_taxes(g: &mut Game, actor: Option<PlayerId>, m: PermId, kind: Sym) -> Res<bool> {
    crate::impls::rules::removal_taxes(g, actor, m, kind)
}

/// partials.tajic_protects: Tajic prevents noncombat damage to its controller's other creatures
pub fn tajic_protects(g: &Game, m: PermId) -> bool {
    crate::impls::partials::tajic_protects(g, m)
}

/// PORT(M5): Skyclave Apparition remembers what it exiled
pub fn apparition_note(_g: &mut Game, _src: PermId, _cd: Option<CardId>, _owner: PlayerId) {}

/// combos: a modeled combo is ready in p's hand and battlefield (Python: `any(cmb.ready(g, p)[0] for cmb in COMBOS)`)
pub fn combo_ready(g: &Game, p: PlayerId) -> bool {
    crate::impls::combos::combo_ready(g, p)
}

// ------------------------------------------------------------------ M3: combat and the turn
/// rules.evasion_blocked: fear, intimidate, Signal Pest and kin: b can't block a
pub fn evasion_blocked(g: &Game, b: PermId, a: PermId) -> bool {
    crate::impls::rules::evasion_blocked(g, b, a)
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

/// rules.dovin_blocked: Dovin's -1: damage to and from m is prevented
pub fn dovin_blocked(g: &Game, m: PermId) -> bool {
    crate::impls::rules::dovin_blocked(g, m)
}

/// rules2.prot_vs (protection from creatures, from Demons and Dragons)
pub fn prot_vs(g: &Game, m: PermId, from: PermId) -> bool {
    crate::impls::rules2::prot_vs(g, m, from)
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
pub fn defend_hooks(
    g: &mut Game,
    p: PlayerId,
    atk: &mut Vec<PermId>,
    d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
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

/// partials.insight_draw (Hunter's Insight) and rules.emblem_combat (Vraska's and Kaito's emblems) on combat damage
pub fn combat_damage_cards(g: &mut Game, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    crate::impls::partials::insight_draw(g, p, a, dmg)?;
    if !g.player(p).emblems.is_empty() {
        crate::impls::rules::emblem_combat(g, p, a, d, dmg)?;
    }
    Ok(())
}

/// CI.become_monarch: p becomes the monarch; Garland's steals from the old monarch end; the 'monarch' hooks run at
/// once (Palace Jailer's exiled creature returns)
pub fn become_monarch(g: &mut Game, p: PlayerId) -> Res {
    if g.monarch == Some(p) || !g.player(p).alive {
        return Ok(());
    }
    g.monarch = Some(p);
    crate::glog!(g, "    {} becomes the monarch", g.player(p).name);
    if g.garland {
        garland_check(g);
    }
    if !g.hooks.is_empty() {
        for (src, imp) in hooks::hooked(g, Event::Monarch) {
            (imp.monarch.unwrap())(g, src, p)?;
            if g.over {
                break;
            }
        }
    }
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

/// rules2.forced_attackers: Goblin Rabblemaster's Goblins and Legion Warboss's tokens attack if able
pub fn forced_attackers(g: &mut Game, p: PlayerId, xs: Vec<PermId>, cand0: &[PermId]) -> Res<Vec<PermId>> {
    crate::impls::rules2::forced_attackers(g, p, xs, cand0)
}

/// rules2.annex_life: Norn's Annex: each attacker at d costs {W} or 2 life
pub fn annex_life(g: &mut Game, p: PlayerId, d: PlayerId, xs: Vec<PermId>) -> Res<Vec<PermId>> {
    crate::impls::rules2::annex_life(g, p, d, xs)
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

/// lands.dakmor_dredge: Dakmor Salvage dredges 2 instead of the draw when p has no land in hand
pub fn dakmor_dredge(g: &mut Game, p: PlayerId) -> Res<bool> {
    crate::impls::lands::dakmor_dredge(g, p)
}

/// rules.REPLACED_TAG_ENGINES (engines whose card code replaces the tag's draw)
pub fn replaced_tag_engine(name: &str) -> bool {
    crate::impls::rules::replaced_tag_engine(name)
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
/// CI.attached_bonus (common.attached_bonus): Auras' and Elspeth's emblem's power and toughness, and the card
/// code's own rules (TOKEN_PT, SELF_PT, CREATURE_PT)
pub fn attached_bonus(g: &Game, m: PermId) -> (i32, i32) {
    crate::impls::common::attached_bonus(g, m)
}

/// CI.attached_kw (common.attached_kw): keywords from Auras
pub fn attached_kw(g: &Game, m: PermId, kw: &str) -> bool {
    crate::impls::common::attached_kw(g, m, kw)
}

/// rules.ability_locked (Collector Ouphe, Cursed Totem, Grand Abolisher, Arrest ...)
pub fn ability_locked(g: &Game, src: PermId, p: PlayerId) -> bool {
    crate::impls::rules::ability_locked(g, src, p)
}

// ------------------------------------------------------------------ M4: what the AI reads from card code
/// combos.combo_imp: a combo piece's importance for counter decisions (9: completes a combo, 7: one short)
pub fn combo_imp(g: &Game, p: PlayerId, c: CardId) -> f64 {
    crate::impls::combos::combo_imp(g, p, c)
}

/// combos.PIECES: every card that is a piece of a modeled combo
pub fn combo_pieces(g: &Game) -> Vec<CardId> {
    crate::impls::combos::pieces(g)
}

/// c is a piece of a modeled combo (Python: `c.name in impl_combos.PIECES`)
pub fn is_combo_piece(g: &Game, c: CardId) -> bool {
    crate::impls::combos::is_piece_name(&g.db.get(c).name)
}

/// combos.missing_pieces: the pieces of p's closest combo it doesn't have yet (for tutors)
pub fn missing_pieces(g: &Game, p: PlayerId) -> Vec<CardId> {
    crate::impls::combos::missing_pieces(g, p)
}

/// search.combo_progress: the share of q's closest combo that q holds, squared
pub fn combo_progress(g: &Game, q: PlayerId) -> f64 {
    crate::impls::combos::combo_progress(g, q)
}

/// combos.piece_threat: extra value of m if it's a piece of a combo its controller nearly has
pub fn piece_threat(g: &Game, m: PermId) -> f64 {
    crate::impls::combos::piece_threat(g, m)
}

/// combos.combo_options: go for a ready combo (ctr_risk: the chance a spell in it is countered)
pub fn combo_options(g: &mut Game, p: PlayerId, ctr_risk: f64, post: Option<bool>) -> Res<Vec<Opt>> {
    crate::impls::combos::combo_options(g, p, ctr_risk, post)
}

/// common.aristocrat_options (sacrifice outlets with a payoff out: Grave Pact, Blood Artist ...)
pub fn aristocrat_options(g: &mut Game, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    crate::impls::common::aristocrat_options(g, p, post)
}

/// lands.land_options (lands' activated abilities: Barad-dûr, Urza's Saga ...): each land's `land_options` slot
pub fn land_options(g: &mut Game, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    crate::impls::lands::land_options(g, p, post)
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

/// common.sac_in_response: sacrifice the permanent for value before removal resolves
pub fn sac_in_response(g: &mut Game, owner: PlayerId, m: PermId, kind: Sym) -> Res<bool> {
    crate::impls::common::sac_in_response(g, owner, m, kind)
}

/// partials.regen_wipe: regenerate the board against a destroy wipe (Golgari Charm)
pub fn regen_wipe(g: &mut Game, q: PlayerId) -> Res<bool> {
    crate::impls::partials::regen_wipe(g, q)
}

/// the outside decks' card plays: common.adventure_options, t2.evoke_options,
/// common.aristocrat_options, common.food_options, partials.miracle_options and incubator_options (PORT(M5):
/// partials, wired in impls::common::pool_card_options)
pub fn pool_card_options(g: &mut Game, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    crate::impls::common::pool_card_options(g, p, post)
}

/// partials.aid_active (Sigarda's Aid: Auras at instant speed)
pub fn aid_active(g: &Game, p: PlayerId) -> bool {
    crate::impls::partials::aid_active(g, p)
}

/// PORT(M5): t4.NINJUTSU, ninjutsu_cost: the ninjutsu costs of the Ninjas in p's hand
pub fn ninjutsu_costs(_g: &Game, _p: PlayerId) -> Vec<(u32, String)> {
    vec![]
}

/// common.SAC_OUTLET
pub fn is_sac_outlet(name: &str) -> bool {
    crate::impls::common::is_sac_outlet(name)
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

/// t2.kaalia_prio
pub fn kaalia_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    crate::impls::t2::kaalia_prio(g, p, c)
}

/// PORT(phase 6): mine.flicker_worth (what flickering Sephiroth's creature m is worth: the commander Atraxa, Summon:
/// Bahamut's restart, counters and Equipment lost); t2.flicker_worth calls it for Sephiroth's deck. Until then
/// t2.blink_value.
pub fn seph_flicker_worth(g: &Game, p: PlayerId, m: PermId) -> f64 {
    crate::impls::t2::blink_value(g, p, m) as f64
}

/// PORT(phase 6): zur.zur_fetch (CI.zur_fetch: Zur attacking in your Y'shtola deck searches for an enchantment with
/// mana value 3 or less and puts it onto the battlefield)
pub fn zur_fetch(_g: &mut Game, _src: PermId, _p: PlayerId) -> Res {
    Ok(())
}
