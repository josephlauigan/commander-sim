"""Card rules for cards in the main decks (decklists/mine/) that need more than their hand tags: equipment, lands with
abilities, and cards whose full text the tag model left out. Hooks here are live in every game (pool rules).
"""
from engine import *
import engine as E
import cardimpl as CI
from cardimpl import on
from pool_cards import card, note


def full(name, text): note(name, 'Full', text)


# ================================================================== Sephiroth
# ------------------------------------------------------------------ Displacer Kitten
ETB_VALUE = {'atraxa': 8.0, 'rsd': 4.0, 'witness': 2.5, 'wall': 2.0, 'wurm': 4.0, 'bowmasters': 2.5, 'skate': 3.0}


def etb_value(g, p, m):
    """what re-entering the battlefield is worth for permanent m (its enters-the-battlefield effects)"""
    if m.cd is None: return 0.0
    t = m.cd.tags
    v = sum(x for k, x in ETB_VALUE.items() if k in t)
    if 'draw' in t and m.cd.perm: v += 1.5 * int(t['draw'])
    if CI.live(m.cd.name) and 'etb' in CI.HOOKS[m.cd.name]: v = max(v, 2.0)
    if m.cd.start_loyalty and m.loyalty is not None: v = max(v, 0.6 * (int(m.cd.start_loyalty) - m.loyalty))
    v -= 0.5 * max(0, m.plus)                                  # counters are lost
    if getattr(g, 'auras', None) and m.creature: v -= 2.0 * len(CI.auras_on(g, m))
    return v


@on('Displacer Kitten', 'cast')
def _kitten(g, src, caster, c):
    """whenever you cast a noncreature spell: exile up to one other nonland permanent you control, return it"""
    p = src.owner
    if caster is not p or c.creature or c.land or src not in p.perms or src.phased: return
    cands = [m for m in p.perms if m is not src and not m.token and not m.phased and m.cd is not None and m.orig is p]
    if not cands: return
    m = max(cands, key=lambda x: etb_value(g, p, x))
    if etb_value(g, p, m) < 2.0: return
    cd, was_cmd = m.cd, m.is_cmd
    log(f'    Displacer Kitten flickers {m.name}', g)
    leave(g, m)
    n = enter(g, p, cd, orig=p)
    n.is_cmd = was_cmd
card('Displacer Kitten', 'pow=2 tgh=2 noatk', types='C', dsl=[])
full('Displacer Kitten', 'each noncreature spell you cast flickers your permanent with the best enters-the-battlefield '
     'value (Atraxa, Grand Unifier, Eternal Witness ...); stolen permanents are left alone')


# ------------------------------------------------------------------ Nim Deathmantle
@on('Nim Deathmantle', 'creature_to_gy')
def _nim_return(g, src, m):
    """whenever a nontoken creature is put into your graveyard from the battlefield, you may pay {4}: return it and
    attach this Equipment to it"""
    p = src.owner
    if src not in p.perms or src.phased or m.token or m.orig is not p or m.cd not in p.gy: return
    if m.cd is p.cmd or not can_pay(g, p, 4, ''): return
    if pval(g, m) < 3 and etb_value(g, p, m) < 2.5: return
    pay(g, p, 4, '')
    p.gy.remove(m.cd)
    n = enter(g, p, m.cd, orig=p)
    src.attached = n
    log(f'    {NAME(p)} pays 4: Nim Deathmantle returns {n.name}', g)


@on('Nim Deathmantle', 'options')
def _nim_equip(g, src, p, s, post):
    """equip {4}: onto your best creature (a bomb first)"""
    if p is not src.owner or post is None or not can_pay(g, p, 4, ''): return []
    if src.attached is not None and src.attached in p.perms and not src.attached.phased: return []
    cre = [m for m in p.perms if m.creature and not m.phased and m.cd is not None]
    if not cre: return []
    t = max(cre, key=lambda m: pval(g, m))

    def go():
        if not can_pay(g, p, 4, '') or t not in p.perms: return False
        pay(g, p, 4, ''); src.attached = t
        log(f'  {NAME(p)} equips Nim Deathmantle to {t.name}', g); return True
    return [(1.0 + 0.2 * pval(g, t), 'equip Nim Deathmantle', go)]
card('Nim Deathmantle', 'nim', types='A', dsl=[])
full('Nim Deathmantle', 'equipped creature +2/+2, intimidate, black Zombie; returns your nontoken creatures that die '
     'for {4} (worth-it check) and re-attaches; equip {4}')


# ------------------------------------------------------------------ Triskelion
@on('Triskelion', 'etb')
def _trisk_etb(g, src, p, m):
    if m is src: src.plus += 3


@on('Triskelion', 'options')
def _trisk_ping(g, src, p, s, post):
    """remove a +1/+1 counter: 1 damage to any target (lethal players, X/1 creatures, or its own last counters when
    Nim Deathmantle can bring it back)"""
    if p is not src.owner or src.plus <= 0 or post is None: return []
    opps = g.opps(p)
    lethal = [q for q in opps if q.life <= src.plus]
    tg = [m for q in opps for m in q.perms if m.creature and etgh(g, m) <= 1 and pval(g, m) >= 2.5 and not untargetable(g, m)]
    if not lethal and not tg: return []

    def go():
        if src.plus <= 0 or src not in p.perms: return False
        src.plus -= 1
        if lethal and lethal[0].alive: lose_life(g, lethal[0], 1, p, kind='triggers')
        elif tg and tg[0] in tg[0].owner.perms: apply_removal(g, p, tg[0], 'dmg1')
        if etgh(g, src) <= 0: die(g, src, 'sba')
        return True
    return [(8.0 if lethal else 2.0, 'Triskelion ping', go)]
card('Triskelion', 'pow=1 tgh=1', types='AC', dsl=[])
full('Triskelion', 'enters with three +1/+1 counters; removes counters to ping lethal players or X/1 creatures')


# ------------------------------------------------------------------ Strip Mine
@on('Strip Mine', 'land_options')
def _strip(g, L, p, s, post):
    """{T}, sacrifice: destroy target land (an opponent's key land: Cradle, Coffers, Urborg, Tomb ...)"""
    if L.tapped: return []
    import impl_lands, impl_fixes
    tg = [(q, x) for q in g.opps(p) for x in q.lands if x.cd.name in impl_lands.KEY_LANDS]
    if not tg: return []
    q, x = max(tg, key=lambda t: (t[1].cd.name in ("Gaea's Cradle", 'Cabal Coffers', 'Serra\'s Sanctum', 'Tolarian Academy',
                                                   'Itlimoc, Cradle of the Sun', 'Nykthos, Shrine to Nyx'), threat(g, p, t[0])))

    def go():
        if L not in p.lands or x not in q.lands: return False
        p.lands.remove(L); p.gy.append(L.cd)
        impl_fixes.destroy_land(g, q, x)
        log(f'  {NAME(p)} sacrifices Strip Mine: destroys {x.cd.name} ({NAME(q)})', g); return True
    return [(3.0, f'Strip Mine -> {x.cd.name}', go)]
full('Strip Mine', '{T}: {C}; {T}, sacrifice: destroy an opponent\'s key land (Cradle, Coffers, Urborg, Ancient Tomb ...)')


# ------------------------------------------------------------------ Galadriel's Dismissal
card("Galadriel's Dismissal", 'prot=phase kicker', types='I', dsl=[])
full("Galadriel's Dismissal", '{W}: your threatened bomb phases out; kicked ({2}{W} more): all your creatures phase out '
     'in answer to a sweeper')


# ------------------------------------------------------------------ Necromancy: the flash mode
def card_etb_value(c):
    """enters-the-battlefield value of a creature card (for a flashed-in Necromancy, which keeps nothing else)"""
    t = c.tags
    v = sum(x for k, x in ETB_VALUE.items() if k in t)
    if 'draw' in t: v += 1.5 * int(t['draw'])
    return v


@on('Necromancy', 'hand_options')
def _necro_flash(g, c, p, s, post):
    """cast at instant speed at the end of an opponent's turn for an enters-the-battlefield creature (Atraxa, Grand
    Unifier): the creature is sacrificed at that turn's cleanup and goes back to the graveyard"""
    if post is not None or p.key != 'seph' or g.active is p or c not in p.hand or not can_pay(g, p, 2, 'B'): return []
    import ais as A
    tg = [(card_etb_value(x), x, q) for q in g.players if q.alive for x in q.gy if x.creature]
    tg = [t for t in tg if t[0] >= 4.0]
    if not tg: return []
    val, cd, src = max(tg, key=lambda t: t[0])

    def go():
        if c not in p.hand or cd not in src.gy or not can_pay(g, p, 2, 'B'): return False
        p.hand.remove(c); pay(g, p, 2, 'B')
        p.spells_this_turn += 1; p.cast_names.add(c.name)
        on_cast(g, p, c)
        log(f'  {NAME(p)} flashes in Necromancy for {cd.name}', g)
        if not counter_window(g, p, c, 6, {}) or (g.hooks and CI.gy_response(g, p, 6, src)):
            p.gy.append(c); return True
        n0 = len(p.perms)
        A.seph_rean_resolve(g, p, c, {'rean_target': cd, 'rean_src': src})
        new = [m for m in p.perms[n0:] if m.cd is cd]
        for m in new: die(g, m, 'sac')                      # cast when a sorcery couldn't be: sacrificed at cleanup
        p.gy.append(c); check_state(g)
        return True
    return [(1.5 + 0.5 * val, f'Necromancy (flash) -> {cd.name}', go)]
note('Necromancy', 'Full', 'reanimates at sorcery speed (the aura is abstracted: the creature stays); flashed in at '
     'the end of an opponent\'s turn for an enters-the-battlefield creature, which is sacrificed at cleanup')


note('Stinkweed Imp', 'Full', 'flying; combat damage to a creature destroys it (the same as deathtouch for a 1-power '
     'creature with no other damage); dredge 5 replaces the draw-step draw when a reanimation spell is available')
note('Atraxa, Grand Unifier', 'Full', 'flying, vigilance, deathtouch, lifelink; enters: reveal ten, one card of each '
     'card type to hand (best first; a multi-type card fills its scarcest type), the rest on the bottom')
note('Farewell', 'Full', 'modes by net value: artifacts, creatures, enchantments, and graveyards (theirs against '
     'yours, since your reanimation targets are there)')
note("Yawgmoth's Will", 'Full', 'until end of turn: cast spells and play lands from the graveyard as it was when it '
     'resolved (reanimation spells included); cards that reach your graveyard this turn are exiled')
note('Altar of Dementia', 'Full', 'free sacrifice outlet; each sacrifice mills you for the creature\'s power while the '
     'library can spare it (reanimation targets)')
note('Tortured Existence', 'Full', '{B}, discard a creature: return a creature from your graveyard (bins a bomb for '
     'the reanimation spells and returns the best creature that isn\'t a reanimation target), once per turn')
