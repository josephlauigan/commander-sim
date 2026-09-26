"""Completing the cards the audit listed as Partial or Unmodeled: the abilities that were missing, each implemented
from its rules text. Loaded last, so these hooks and card data replace earlier partial versions.
"""
from engine import *
import engine as E
import cardimpl as CI
from cardimpl import on, _eot, count_type
from pool_cards import card, note
import impl_common as IC
from impl_common import best_opp_creature, best_opp_nonland, make_artifact_tokens


def spare(g, p, reserve=0):
    return total_mana(g, p) - reserve


def turn_now(g):
    return turn_stamp(g)


def deal_noncombat(g, src_owner, m, n):
    """n damage to creature m from a non-combat source (Tajic prevents it for the other creatures of its owner)"""
    if m not in m.owner.perms: return
    if tajic_protects(g, m): log(f'    damage to {m.name} is prevented (Tajic)', g); return
    if etgh(g, m) <= n and not indestructible(g, m): die(g, m, 'destroy')


def tajic_protects(g, m):
    return any(x.cd is not None and x.cd.name == 'Tajic, Legion\'s Edge' and x is not m and not x.phased
                                for x in m.owner.perms)


# ------------------------------------------------------------------ Archangel Avacyn
@on('Archangel Avacyn // Avacyn, the Purifier', 'dies')
def _avacyn_flag(g, src, m, cause):
    if m is not src and m.owner is src.owner and m.creature and not E.has_type(m, 'angel') and not (src.data or {}).get('flipped'):
        if src.data is None: src.data = {}
        src.data['flip'] = True


@on('Archangel Avacyn // Avacyn, the Purifier', 'upkeep')
def _avacyn_flip(g, src, p):
    if not src.data or not src.data.get('flip') or src.data.get('flipped'): return
    src.data['flip'] = False; src.data['flipped'] = True
    src.pow, src.tgh = 6, 5
    o = src.owner
    log(f'    Archangel Avacyn transforms: 3 damage to each other creature and each opponent', g)
    for q in g.players:
        for m in list(q.perms):
            if m is not src and m.creature: deal_noncombat(g, o, m, 3)
    for q in g.opps(o): lose_life(g, q, 3, o, kind='burn', damage=True)
    check_state(g)
note('Archangel Avacyn // Avacyn, the Purifier', 'Full', 'flash; team indestructible on entry; transforms at the next '
     'upkeep after a non-Angel creature of yours dies (3 damage to each other creature and each opponent; 6/5)')


# ------------------------------------------------------------------ Archfiend of Sorrows: unearth
@on('Archfiend of Sorrows', 'gy_options')
def _archfiend_unearth(g, c, p, s, post):
    if post is not False or not can_pay(g, p, 3, 'BB'): return []
    small = sum(pval(g, m) for q in g.opps(p) for m in q.perms if m.creature and etgh(g, m) <= 2)
    if small < 5: return []

    def go():
        if c not in p.gy or not can_pay(g, p, 3, 'BB'): return False
        p.gy.remove(c); pay(g, p, 3, 'BB')
        m = enter(g, p, c); m.sick = False
        if m.data is None: m.data = {}
        m.data['unearth'] = True
        log(f'  {NAME(p)} unearths Archfiend of Sorrows', g); return True
    return [(small / 3.0, 'unearth Archfiend of Sorrows', go)]


@on('Archfiend of Sorrows', 'end_step')
def _archfiend_exile(g, src, p):
    if src.data and src.data.get('unearth') and src in src.owner.perms:
        leave(g, src)
        if src.cd in src.owner.gy: src.owner.gy.remove(src.cd)
        src.owner.exile.append(src.cd)
note('Archfiend of Sorrows', 'Full', 'flying; opposing creatures -2/-2 on entry; unearth {3}{B}{B} when it would wipe '
     'small creatures (hasty, exiled at end of turn)')


# ------------------------------------------------------------------ Aven Mindcensor: searches see the top four
@on('Aven Mindcensor', 'search_limit')
def _mindcensor(g, src, searcher):
    return 4 if searcher is not src.owner else None
card('Aven Mindcensor', 'pow=2 tgh=1 fly flash', dsl=[])
note('Aven Mindcensor', 'Full', 'flash, flying; opponents searching their library search only the top four cards')


# ------------------------------------------------------------------ Bloodline Keeper // Lord of Lineage
@on('Bloodline Keeper // Lord of Lineage', 'options')
def _keeper(g, src, p, s, post):
    if p is not src.owner or post is None: return []
    o = []
    if not src.tapped and not src.sick:
        def tok():
            if src.tapped: return False
            src.tapped = True; make_tokens(g, p, 1, 2, fly=True, color='B', types=('vampire',)); return True
        o.append((2.0, 'Bloodline Keeper token', tok))
    if not (src.data or {}).get('lord') and count_type(g, p, 'vampire') >= 5 and can_pay(g, p, 0, 'B'):
        def flip():
            if not can_pay(g, p, 0, 'B'): return False
            pay(g, p, 0, 'B')
            if src.data is None: src.data = {}
            src.data['lord'] = True; src.pow, src.tgh = 5, 5
            log(f'  {NAME(p)} transforms Bloodline Keeper into Lord of Lineage', g); return True
        o.append((4.0, 'transform Bloodline Keeper', flip))
    return o


IC.SELF_PT['Bloodline Keeper // Lord of Lineage'] = lambda g, p, m: (0, 0)


@on('Bloodline Keeper // Lord of Lineage', 'etb')
def _keeper_on(g, src, p, m):
    if m is src: g.selfpt = True; g.lineage = True


def lineage_bonus(g, m):
    """Lord of Lineage: other Vampires its controller controls get +2/+2"""
    if not getattr(g, 'lineage', False) or not m.creature: return 0
    n = sum(1 for x in m.owner.perms if x is not m and x.cd is not None and x.cd.name == 'Bloodline Keeper // Lord of Lineage'
            and x.data and x.data.get('lord') and not x.phased)
    return 2 * n if n and E.has_type(m, 'vampire') else 0


card('Bloodline Keeper // Lord of Lineage', 'pow=3 tgh=3 fly', dsl=[])
note('Bloodline Keeper // Lord of Lineage', 'Full', '{T}: 2/2 flying Vampire; {B}: transform with five Vampires '
     '(Lord of Lineage: other Vampires +2/+2, still makes Vampires)')


# ------------------------------------------------------------------ Brutal Hordechief: you choose how they block
@on('Brutal Hordechief', 'options')
def _hordechief_act(g, src, p, s, post):
    if p is not src.owner or post is not False or not can_pay(g, p, 3, 'RR') and not can_pay(g, p, 3, 'WW') \
            and not can_pay(g, p, 3, 'RW'): return []
    ready = [m for m in p.perms if m.creature and not m.sick and not m.tapped and not m.noatk]
    theirs = [m for q in g.opps(p) for m in q.perms if m.creature and not m.tapped]
    if len(ready) < 3 or not theirs or (src.data or {}).get('lure') == turn_now(g): return []
    pips = next(x for x in ('RR', 'RW', 'WW') if can_pay(g, p, 3, x))

    def go():
        if not can_pay(g, p, 3, pips): return False
        pay(g, p, 3, pips)
        if src.data is None: src.data = {}
        src.data['lure'] = turn_now(g)
        log(f'  {NAME(p)} activates Brutal Hordechief: opponents block as {NAME(p)} chooses', g); return True
    return [(0.5 * len(theirs), 'Brutal Hordechief (choose blocks)', go)]


@on('Brutal Hordechief', 'blocks')
def _hordechief_blocks(g, src, p, atk, d, assign):
    """after the activation, every untapped creature of the defender blocks, assigned where it dies without killing"""
    if p is not src.owner or not src.data or src.data.get('lure') != turn_now(g): return
    assign.clear()
    for b in [b for b in d.perms if b.creature and not b.tapped and not b.phased]:
        good = [a for a in atk if a in p.perms and epow(g, a) >= etgh(g, b) and epow(g, b) < etgh(g, a)]
        a = max(good, key=lambda a: epow(g, a)) if good else None
        if a is not None and a not in assign: assign[a] = b
note('Brutal Hordechief', 'Full', 'each attacker drains 1; {3}{R/W}{R/W}: opponents block as you choose (their '
     'creatures block where they die without killing)')


# ------------------------------------------------------------------ Celestial Mantle / Noble Hierarch (already complete)
note('Celestial Mantle', 'Full', '+3/+3; combat damage to a player doubles its controller\'s life total')
note('Noble Hierarch', 'Full', 'exalted; taps for {G}, {W} or {U}')


# ------------------------------------------------------------------ Chain of Vapor
@IC.spell('Chain of Vapor', prio=0, tags='rem=bounce tgt=nl', types='I',
          status=('Full', 'bounces a nonland permanent; its controller may sacrifice a land to copy it back at you'))
def _chain(g, p, c, ctx):
    t = ctx.get('target')
    if t is None or t not in t.owner.perms: return
    q = t.owner
    apply_removal(g, p, t, 'bounce')
    if q is p or len(q.lands) < 5: return
    back = best_opp_nonland(g, q)
    if back is not None and pval(g, back) >= 5:
        L = min(q.lands, key=lambda L: (len(L.cd.tags.get('c', '')), not L.tapped))
        q.lands.remove(L); q.gy.append(L.cd)
        log(f'    {NAME(q)} sacrifices {L.cd.name} to copy Chain of Vapor', g)
        apply_removal(g, q, back, 'bounce')


# ------------------------------------------------------------------ Chaos Warp: the revealed permanent comes in
@on('Chaos Warp', 'resolve')
def _chaos_warp(g, p, c, ctx):
    t = ctx.get('target')
    if t is None or t not in t.owner.perms: return
    q = t.owner
    apply_removal(g, p, t, 'tuck', c)
    if q.library:
        top = q.library[-1]
        log(f'    {NAME(q)} reveals {top.name}', g)
        if top.perm and not top.land:
            q.library.pop(); enter(g, q, top)
        elif top.land:
            q.library.pop(); q.lands.append(Land(top, False))
note('Chaos Warp', 'Full', 'the target is shuffled away; a revealed permanent card enters for its owner')


# ------------------------------------------------------------------ Coat of Arms
@on('Coat of Arms', 'etb')
def _coat_on(g, src, p, m):
    if m is src: g.coat = True; g.selfpt = True


def _ctypes(m):
    if m.cd is not None:
        return {t for t in m.cd.subtypes if t not in ('aura', 'equipment')} if m.cd.subtypes else set()
    return set(m.ttypes) - {'land', 'artifact'}


def coat_bonus(g, m):
    """Coat of Arms: +1/+1 per other creature sharing a creature type (cached per battlefield version)"""
    if not getattr(g, 'coat', False) or not m.creature: return 0
    ver = getattr(g, 'bf_ver', 0)
    cache = getattr(g, 'coat_cache', None)
    if cache is None or cache[0] != ver:
        coats = sum(1 for q in g.players for x in q.perms if x.cd is not None and x.cd.name == 'Coat of Arms' and not x.phased)
        cs = [x for q in g.players if q.alive for x in q.perms if x.creature and not x.phased]
        types = {id(x): _ctypes(x) for x in cs}
        bonus = {}
        if coats:
            for x in cs:
                tx = types[id(x)]
                bonus[id(x)] = coats * sum(1 for y in cs if y is not x and tx & types[id(y)]) if tx else 0
        cache = g.coat_cache = (ver, bonus)
    return cache[1].get(id(m), 0)
card('Coat of Arms', '', types='A', dsl=[])
note('Coat of Arms', 'Full', 'each creature +1/+1 per other creature sharing a creature type with it (everyone\'s)')


# ------------------------------------------------------------------ Conduit of Worlds
@on('Conduit of Worlds', 'options')
def _conduit(g, src, p, s, post):
    """{T}: cast a nonland permanent card from your graveyard if you haven't cast a spell this turn (sorcery)"""
    if p is not src.owner or post is not False or src.tapped or casts_this_turn(g, p): return []
    cs = [c for c in p.gy if c.perm and not c.land and can_pay(g, p, *cost_of(p, c))]
    if not cs: return []
    c = max(cs, key=lambda c: (card_worth(g, p, c), c.cmc))

    def go():
        if src.tapped or c not in p.gy or not can_pay(g, p, *cost_of(p, c)): return False
        src.tapped = True; pay(g, p, *cost_of(p, c))
        log(f'  {NAME(p)} casts {c.name} from the graveyard with Conduit of Worlds', g)
        cast_card(g, p, c, 'gy', {})
        p.conduit_lock = turn_now(g)
        return True
    return [(1.0 + card_worth(g, p, c) / 20.0, f'Conduit of Worlds ({c.name})', go)]


@on('Conduit of Worlds', 'can_cast')
def _conduit_lock(g, src, caster, c, zone):
    return not (caster is src.owner and getattr(caster, 'conduit_lock', None) == turn_now(g))
note('Conduit of Worlds', 'Full', 'lands from the graveyard; {T}: casts the best permanent card from the graveyard '
     '(then no more spells that turn)')


# ------------------------------------------------------------------ Conspicuous Snoop
@on('Conspicuous Snoop', 'options')
def _snoop(g, src, p, s, post):
    """cast Goblin spells from the top of the library; has the activated abilities of a Goblin on top"""
    if p is not src.owner or not p.library: return []
    top = p.library[-1]
    o = []
    if post is False and 'goblin' in top.subtypes and top.creature and can_pay(g, p, *cost_of(p, top)) and castable(g, p, top):
        def go(top=top):
            if not p.library or p.library[-1] is not top or not can_pay(g, p, *cost_of(p, top)): return False
            p.library.pop(); pay(g, p, *cost_of(p, top))
            log(f'  {NAME(p)} casts {top.name} from the top of the library (Snoop)', g)
            cast_card(g, p, top, 'lib', {}); return True
        o.append((2.0 + card_worth(g, p, top) / 20.0, f'Snoop: cast {top.name}', go))
    h = CI.HOOKS.get(top.name, {})
    if 'goblin' in top.subtypes and 'options' in h:
        o += h['options'](g, src, p, s, post) or []
    return o
card('Conspicuous Snoop', 'pow=2 tgh=2 rogue', dsl=[])
note('Conspicuous Snoop', 'Full', 'casts Goblins from the top of the library; gains the activated abilities of a Goblin on top')


# ------------------------------------------------------------------ Crypt Ghast: extort
def extort(name):
    @on(name, 'cast')
    def _x(g, src, caster, c):
        if caster is not src.owner or src.phased: return
        if total_mana(g, caster) >= 3 and can_pay(g, caster, 0, 'B'):
            pay(g, caster, 0, 'B')
            n = 0
            for q in g.opps(caster): lose_life(g, q, 1, caster, kind='drain'); n += 1
            gain(caster, n)


extort('Crypt Ghast')
note('Crypt Ghast', 'Full', 'Swamps tap for an extra {B}; extort (paid with spare mana)')


# ------------------------------------------------------------------ delve: Dig Through Time, Treasure Cruise
DELVE = {'Dig Through Time', 'Treasure Cruise'}


def delve_count(g, p, c):
    fodder = [x for x in p.gy if card_worth(g, p, x, True) < 20 and x is not c]
    return min(c.generic, len(fodder))


def delve_exile(g, p, c, n):
    fodder = sorted([x for x in p.gy if x is not c], key=lambda x: card_worth(g, p, x, True))[:n]
    for x in fodder: p.gy.remove(x); p.exile.append(x)


@IC.spell('Treasure Cruise', prio=lambda g, p, c: 48, types='S', status=('Full', 'delve (exiles the least useful '
          'graveyard cards); draw three'))
def _cruise(g, p, c, ctx):
    delve_exile(g, p, c, delve_count(g, p, c))
    draw(g, p, 3)


@IC.spell('Dig Through Time', prio=lambda g, p, c: 50, types='I', status=('Full', 'delve; look at the top seven, '
          'keep the best two, the rest on the bottom'))
def _dig(g, p, c, ctx):
    delve_exile(g, p, c, delve_count(g, p, c))
    import impl_topdeck as T
    top = T.look(p, 7)
    top.sort(key=lambda x: -T.desire(g, p, x))
    for x in top[:2]: p.hand.append(x); p.seen_names.add(x.name)
    p.library[:0] = top[2:]


# ------------------------------------------------------------------ Draco
def domain(p):
    types = set()
    for L in p.lands:
        st = getattr(L.cd, 'subtypes', ()) or ()
        for t, n in (('plains', 'Plains'), ('island', 'Island'), ('swamp', 'Swamp'), ('mountain', 'Mountain'), ('forest', 'Forest')):
            if L.cd.name == n or t in st: types.add(t)
    return len(types)


E.SELF_COST['Draco'] = lambda g, p, c: -2 * domain(p)
E.SELF_COST['Treasure Cruise'] = lambda g, p, c: -delve_count(g, p, c)
E.SELF_COST['Dig Through Time'] = lambda g, p, c: -delve_count(g, p, c)


@on('Draco', 'upkeep')
def _draco_upkeep(g, src, p):
    if p is not src.owner: return
    n = max(0, 10 - 2 * domain(p))
    if can_pay(g, p, n, ''): pay(g, p, n, '')
    else: log(f'    {NAME(p)} sacrifices Draco', g); die(g, src, 'sac')
CI.SPELL_PRIO['Draco'] = lambda g, p, c: 0 if __import__('impl_topdeck').mv_mode(p) else 50
card('Draco', 'pow=9 tgh=9 fly bomb=9', types='AC', dsl=[])
note('Draco', 'Full', 'costs {2} less per basic land type; upkeep: pay {10} minus {2} per type or sacrifice it '
     '(a Yuriko deck keeps it for the reveal)')


# ------------------------------------------------------------------ Druid Class level 3
@on('Druid Class', 'options')
def _druid3(g, src, p, s, post):
    if p is not src.owner or post is not False or (src.data or {}).get('level', 1) != 2 or not can_pay(g, p, 4, 'G'): return []
    if len(p.lands) < 7: return []

    def go():
        if not can_pay(g, p, 4, 'G') or not p.lands: return False
        pay(g, p, 4, 'G'); src.data['level'] = 3
        L = min(p.lands, key=lambda L: (len(L.cd.tags.get('c', '')), L.cd.name not in ('Forest', 'Island')))
        p.lands.remove(L)
        m = Perm(p, None, pw=len(p.lands) + 1, tg=len(p.lands) + 1, name=L.cd.name)
        m.sick = False; m.ttypes = frozenset(('land',)); m.data = {'land': L, 'druidclass': True}
        p.perms.append(m)
        log(f'  {NAME(p)} levels Druid Class to 3: {L.cd.name} becomes a {m.pow}/{m.tgh} creature', g); return True
    return [(1.0 + len(p.lands) / 4.0, 'Druid Class level 3', go)]


@on('Druid Class', 'upkeep')
def _druid_size(g, src, p):
    if p is not src.owner: return
    for m in p.perms:
        if m.data and m.data.get('druidclass'): m.pow = m.tgh = len(p.lands) + 1
note('Druid Class', 'Full', 'landfall: 1 life; level 2: an extra land each turn; level 3: a land becomes a hasty '
     'creature with P/T equal to your lands')


# ------------------------------------------------------------------ Eidolon of Countless Battles: bestow
def eidolon_x(g, p):
    return sum(1 for x in p.perms if x.creature and not x.phased) + \
        sum(1 for x in p.perms if x.cd is not None and 'aura' in x.cd.subtypes and not x.phased) + \
        sum(len((x.data or {}).get('bestow', ())) for x in p.perms)


@on('Eidolon of Countless Battles', 'hand_options')
def _bestow(g, c, p, s, post):
    """bestow {2}{W}{W}: an Aura on the best creature (the commander in Light-Paws), +X/+X for creatures and Auras"""
    if post is not False or not can_pay(g, p, 2, 'WW'): return []
    import impl_common
    host = impl_common.own_host(g, p, {'host_ok': None})
    if host is None: return []

    def go():
        if c not in p.hand or host not in p.perms or not can_pay(g, p, 2, 'WW'): return False
        p.hand.remove(c); pay(g, p, 2, 'WW')
        log(f'  {NAME(p)} bestows Eidolon of Countless Battles on {host.name}', g)
        on_cast(g, p, c)
        if not counter_window(g, p, c, 4, {}): p.gy.append(c); return True
        if host.data is None: host.data = {}
        host.data.setdefault('bestow', []).append(c)
        g.selfpt = True
        return True
    return [(3.0 + 0.5 * eidolon_x(g, p), 'bestow Eidolon', go)]


def bestow_bonus(g, m):
    if not m.data or not m.data.get('bestow'): return 0
    return len(m.data['bestow']) * eidolon_x(g, m.owner)


def bestow_fall(g, m):
    """the host left the battlefield: a bestowed Eidolon becomes a creature"""
    for c in (m.data or {}).get('bestow', ()):
        if m.owner.alive: enter(g, m.owner, c)
    m.data['bestow'] = []
note('Eidolon of Countless Battles', 'Full', 'creature or bestow {2}{W}{W} on the best creature: +1/+1 per creature and '
     'Aura you control; falls off as a creature')


# ------------------------------------------------------------------ Elvish Spirit Guide: exile from hand for {G}
def hand_mana(g, p):
    """mana from cards in hand (Elvish Spirit Guide): units ['H', color, 1, card]"""
    return [['H', 'G', 1, c] for c in p.hand if c.name == 'Elvish Spirit Guide']
CI.SPELL_PRIO['Elvish Spirit Guide'] = 0
note('Elvish Spirit Guide', 'Full', 'exiled from hand for {G} when that mana is needed (never cast)')


# ------------------------------------------------------------------ Enter the Infinite
@IC.spell('Enter the Infinite', prio=lambda g, p, c: 70, types='S',
          status=('Full', 'draws the library, puts the best card for next turn on top, no maximum hand size until '
                          'your next turn'))
def _eti(g, p, c, ctx):
    n = len(p.library)
    draw(g, p, max(0, n))
    if p.hand:
        x = max(p.hand, key=lambda x: card_worth(g, p, x)); p.hand.remove(x); p.library.append(x)
    p.nomax_turn = p.turns


# ------------------------------------------------------------------ Faerie Mastermind
@on('Faerie Mastermind', 'draw')
def _mastermind(g, src, p):
    if p is src.owner or not p.alive: return
    if getattr(p, 'draw_n', 0) == 2 and once_per_turn(g, src.owner, f'mastermind{id(src)}{p.key}'):
        draw(g, src.owner, 1)
card('Faerie Mastermind', 'pow=2 tgh=1 fly flash', dsl=[])
note('Faerie Mastermind', 'Full', 'flash, flying; draws when an opponent draws their second card each turn (the '
     'symmetric draw ability is never worth it and is not used)')


# ------------------------------------------------------------------ Gitaxian Probe: Phyrexian mana
@IC.spell('Gitaxian Probe', prio=lambda g, p, c: 40, types='S',
          status=('Full', '{U} or 2 life; draw a card (looking at a hand has no effect in the sim)'))
def _probe(g, p, c, ctx):
    draw(g, p, 1)
card('Gitaxian Probe', '', types='S', cost='0', dsl=[])


@on('Gitaxian Probe', 'resolve')
def _probe_pay(g, p, c, ctx):
    if can_pay(g, p, 0, 'U') and total_mana(g, p) >= 3: pay(g, p, 0, 'U')
    else: lose_life(g, p, 2, p)
    draw(g, p, 1)


# ------------------------------------------------------------------ Gix, Yawgmoth Praetor
@on('Gix, Yawgmoth Praetor', 'options')
def _gix(g, src, p, s, post):
    if p is not src.owner or post is not False or not can_pay(g, p, 4, 'BBB'): return []
    junk = [c for c in p.hand if card_worth(g, p, c) < 35]
    if len(junk) < 2: return []
    q = max(g.opps(p), key=lambda q: len(q.library)) if g.opps(p) else None
    if q is None or len(q.library) < len(junk): return []

    def go():
        if not can_pay(g, p, 4, 'BBB'): return False
        pay(g, p, 4, 'BBB')
        for c in junk:
            if c in p.hand: p.hand.remove(c); p.gy.append(c)
        top = [q.library.pop() for _ in range(len(junk))]
        log(f'  {NAME(p)} activates Gix: exiles {len(top)} cards of {NAME(q)}', g)
        for c in top:
            if c.land and len(p.lands) < 20: p.lands.append(Land(c, False))
            elif not c.land and castable(g, p, c): cast_card(g, p, c, 'lib', {})
            else: q.exile.append(c)
        return True
    return [(1.0 + 0.8 * len(junk), 'Gix (discard X)', go)]
note('Gix, Yawgmoth Praetor', 'Full', 'combat damage draws; {4}{B}{B}{B}, discard X junk cards: plays the top X of an '
     'opponent\'s library free')


# ------------------------------------------------------------------ Goblin Cratermaker
@on('Goblin Cratermaker', 'options')
def _cratermaker(g, src, p, s, post):
    if p is not src.owner or not can_pay(g, p, 1, ''): return []
    t1 = best_opp_creature(g, p, lambda m: etgh(g, m) <= 2)
    t2 = best_opp_nonland(g, p, lambda m: m.cd is not None and not m.cd.pips)
    t = max([x for x in (t1, t2) if x is not None], key=lambda m: pval(g, m), default=None)
    if t is None or pval(g, t) < 3: return []

    def go():
        if src not in p.perms or t not in t.owner.perms or not can_pay(g, p, 1, ''): return False
        pay(g, p, 1, ''); die(g, src, 'sac')
        apply_removal(g, p, t, 'dmg2' if t is t1 else 'destroy'); return True
    return [(pval(g, t) - 2.0, f'Goblin Cratermaker -> {t.name}', go)]
card('Goblin Cratermaker', 'pow=2 tgh=2 warrior', dsl=[])
note('Goblin Cratermaker', 'Full', '{1}, sacrifice: 2 damage to a creature or destroy a colorless nonland permanent')


# ------------------------------------------------------------------ Golgari Charm
@IC.spell('Golgari Charm', prio=0, tags='', types='I',
          status=('Full', 'modes: -1/-1 to all creatures (against tokens), destroy an enchantment, or regenerate your '
                          'creatures in response to a destroy wipe'))
def _golgari_charm(g, p, c, ctx):
    mode = ctx.get('mode')
    if mode == 'regen':
        p.regen_turn = turn_now(g); return
    if mode == 'shrink':
        for q in g.players:
            for m in list(q.perms):
                if m.creature:
                    _eot(g, m, -1, -1)
                    if etgh(g, m) <= 0: die(g, m, 'sba')
        return
    t = best_opp_nonland(g, p, lambda m: m.cd is not None and 'E' in m.cd.types)
    if t is not None: apply_removal(g, p, t, 'destroy', c)


@on('Golgari Charm', 'hand_options')
def _golgari_opts(g, c, p, s, post):
    if not can_pay(g, p, 0, 'BG'): return []
    o = []
    toks = sum(pval(g, m) for q in g.opps(p) for m in q.perms if m.creature and etgh(g, m) <= 1)
    mine = sum(pval(g, m) for m in p.perms if m.creature and etgh(g, m) <= 1)
    if toks - mine >= 5:
        o.append((toks - mine - 2, 'Golgari Charm (-1/-1)', lambda: _cast_mode(g, p, c, 'shrink')))
    t = best_opp_nonland(g, p, lambda m: m.cd is not None and 'E' in m.cd.types)
    if t is not None and pval(g, t) >= 4:
        o.append((pval(g, t) - 2.5, 'Golgari Charm (enchantment)', lambda: _cast_mode(g, p, c, 'ench')))
    return o


def _cast_mode(g, p, c, mode):
    if c not in p.hand or not can_pay(g, p, 0, 'BG'): return False
    p.hand.remove(c); pay(g, p, 0, 'BG')
    cast_card(g, p, c, 'lib', {'mode': mode}); p.gy.append(c) if c not in p.gy else None
    return True


def regen_wipe(g, p):
    """a destroy wipe is coming: Golgari Charm regenerates p's creatures"""
    c = next((x for x in p.hand if x.name == 'Golgari Charm'), None)
    if c is None or not can_pay(g, p, 0, 'BG') or not castable(g, p, c): return False
    if sum(pval(g, m) for m in p.perms if m.creature) < 8: return False
    return _cast_mode(g, p, c, 'regen')


# ------------------------------------------------------------------ Grim Hireling: -X/-X for Treasures
@on('Grim Hireling', 'options')
def _hireling(g, src, p, s, post):
    if p is not src.owner or post is not False or not can_pay(g, p, 0, 'B') or p.treasures < 1: return []
    t = best_opp_creature(g, p, lambda m: etgh(g, m) <= p.treasures)
    if t is None or pval(g, t) < 4: return []
    x = etgh(g, t)

    def go():
        if t not in t.owner.perms or p.treasures < x or not can_pay(g, p, 0, 'B'): return False
        pay(g, p, 0, 'B'); p.treasures -= x
        for _ in range(x): CI.fire(g, 'sacrifice', p, 'Treasure') if g.hooks else None
        log(f'  {NAME(p)} sacrifices {x} Treasures: {t.name} gets -{x}/-{x}', g)
        if not untargetable(g, t): _eot(g, t, -x, -x); die(g, t, 'sba') if etgh(g, t) <= 0 else None
        return True
    return [(pval(g, t) - 1.0 - 0.6 * x, f'Grim Hireling -> {t.name}', go)]
note('Grim Hireling', 'Full', 'two Treasures when your creatures connect; {B}, sacrifice X Treasures: -X/-X to a creature')


# ------------------------------------------------------------------ Gryff's Boon: recursion
@on("Gryff's Boon", 'gy_options')
def _gryff(g, c, p, s, post):
    if post is not False or not can_pay(g, p, 3, 'W') or not any(m.creature for m in p.perms): return []

    def go():
        if c not in p.gy or not can_pay(g, p, 3, 'W'): return False
        p.gy.remove(c); pay(g, p, 3, 'W'); enter(g, p, c)
        log(f"  {NAME(p)} returns Gryff's Boon from the graveyard", g); return True
    return [(1.8, "Gryff's Boon (graveyard)", go)]
note("Gryff's Boon", 'Full', '+1/+0 and flying; {3}{W}: returns from the graveyard onto a creature')


# ------------------------------------------------------------------ Heartless Act
@on('Heartless Act', 'resolve')
def _heartless(g, p, c, ctx):
    t = ctx.get('target')
    if t is None or t not in t.owner.perms: return
    if t.plus > 0:
        t.plus = max(0, t.plus - 3)
        log(f'    Heartless Act removes counters from {t.name}', g)
        if etgh(g, t) <= 0: die(g, t, 'sba')
    else:
        apply_removal(g, p, t, 'destroy', c)
note('Heartless Act', 'Full', 'destroys a creature with no counters, or removes up to three counters (and kills it if '
     'that is lethal)')


# ------------------------------------------------------------------ Hero of Iroas: heroic
@on('Hero of Iroas', 'etb')
def _hero_heroic(g, src, p, m):
    if m.cd is not None and 'aura' in m.cd.subtypes and m.owner is src.owner and m.attached is src: src.plus += 1
note('Hero of Iroas', 'Full', 'Aura spells cost {1} less; heroic: +1/+1 counter when an Aura targets it')


# ------------------------------------------------------------------ Hope of Ghirapur
@on('Hope of Ghirapur', 'combat_damage')
def _hope_hit(g, src, p, a, d, dmg):
    if a is src:
        if src.data is None: src.data = {}
        src.data['hit'] = (turn_now(g), d)


@on('Hope of Ghirapur', 'options')
def _hope_sac(g, src, p, s, post):
    if p is not src.owner or post is not True or not src.data or 'hit' not in src.data: return []
    st, d = src.data['hit']
    if st != turn_now(g) or not d.alive: return []
    if len(d.hand) < 3: return []

    def go():
        if src not in p.perms: return False
        die(g, src, 'sac'); d.hope_lock = (p, p.turns)
        log(f'  {NAME(p)} sacrifices Hope of Ghirapur: {NAME(d)} can\'t cast noncreature spells', g); return True
    return [(1.0 + 0.2 * len(d.hand), 'Hope of Ghirapur', go)]
note('Hope of Ghirapur', 'Full', 'flying; after it connects, sacrificed to stop that player casting noncreature '
     'spells until your next turn')


# ------------------------------------------------------------------ Hunter's Insight
@IC.spell("Hunter's Insight", prio=0, types='I', status=('Full', 'cast before combat on the creature most likely to '
          'connect: its combat damage to a player draws that many cards'))
def _insight(g, p, c, ctx):
    t = ctx.get('host')
    if t is not None: p.insight = (turn_now(g), t)


@on("Hunter's Insight", 'hand_options')
def _insight_opt(g, c, p, s, post):
    if post is not False or not can_pay(g, p, 2, 'G'): return []
    import impl_lands
    cs = [m for m in p.perms if m.creature and not m.sick and not m.tapped and not m.noatk]
    good = [m for m in cs if impl_lands.open_attack(g, p, epow(g, m), m.fly)]
    if not good: return []
    t = max(good, key=lambda m: epow(g, m))
    if epow(g, t) < 3: return []

    def go():
        if c not in p.hand or t not in p.perms or not can_pay(g, p, 2, 'G'): return False
        p.hand.remove(c); pay(g, p, 2, 'G')
        cast_card(g, p, c, 'lib', {'host': t})
        if c not in p.gy: p.gy.append(c)
        return True
    return [(1.0 + 0.5 * epow(g, t), f"Hunter's Insight on {t.name}", go)]


def insight_draw(g, p, a, dmg):
    ins = getattr(p, 'insight', None)
    if ins and ins[0] == turn_now(g) and ins[1] is a and dmg > 0:
        draw(g, p, dmg); log(f"    Hunter's Insight: {NAME(p)} draws {dmg}", g)


# ------------------------------------------------------------------ Hushbringer
card('Hushbringer', 'pow=1 tgh=2 fly lifelink', dsl=[])
note('Hushbringer', 'Full', 'flying, lifelink; creatures entering or dying trigger nothing (everyone\'s)')


def hushed(g):
    return any(m.cd is not None and m.cd.name == 'Hushbringer' and not m.phased and not m.neutered
                                for q in g.players if q.alive for m in q.perms)


# ------------------------------------------------------------------ Jagged-Scar Archers
@on('Jagged-Scar Archers', 'options')
def _archers(g, src, p, s, post):
    if p is not src.owner or src.tapped or src.sick: return []
    n = epow(g, src)
    t = best_opp_creature(g, p, lambda m: (m.fly or (E.DSLMOD and E.DSLMOD.has_kw(g, m, 'flying'))) and etgh(g, m) <= n)
    if t is None or pval(g, t) < 2.5: return []

    def go():
        if src.tapped or t not in t.owner.perms: return False
        src.tapped = True; apply_removal(g, p, t, f'dmg{n}'); return True
    return [(pval(g, t) - 1.0, f'Jagged-Scar Archers -> {t.name}', go)]
note('Jagged-Scar Archers', 'Full', 'P/T = Elves you control; {T}: damage equal to its power to a flier')


# ------------------------------------------------------------------ Jaxis, the Troublemaker
@on('Jaxis, the Troublemaker', 'hand_options')
def _jaxis_blitz(g, c, p, s, post):
    """blitz {1}{R}: haste, draw when it dies, sacrificed at end of turn"""
    if post is not False or not can_pay(g, p, 1, 'R') or not castable(g, p, c): return []

    def go():
        if c not in p.hand or not can_pay(g, p, 1, 'R'): return False
        p.hand.remove(c); pay(g, p, 1, 'R')
        log(f'  {NAME(p)} casts Jaxis for its blitz cost', g)
        on_cast(g, p, c)
        if not counter_window(g, p, c, 3, {}): p.gy.append(c); return True
        m = enter(g, p, c, was_cast=True); m.sick = False
        if m.data is None: m.data = {}
        m.data['blitz'] = True
        return True
    return [(2.2, 'blitz Jaxis', go)]


@on('Jaxis, the Troublemaker', 'self_dies')
def _jaxis_draw(g, m, cause):
    if m.data and m.data.get('blitz'): draw(g, m.owner, 1)


@on('Jaxis, the Troublemaker', 'end_step')
def _jaxis_end(g, src, p):
    if p is src.owner and src.data and src.data.get('blitz') and src in p.perms: die(g, src, 'sac')
    for m in [m for m in src.owner.perms if m.data and m.data.get('jaxis_copy')]:
        if p is src.owner: draw(g, src.owner, 1); leave(g, m)


@on('Jaxis, the Troublemaker', 'options')
def _jaxis_copy(g, src, p, s, post):
    """{R}, {T}, discard a card: a hasty token copy of another creature (draw when it dies, sacrificed at end step)"""
    if p is not src.owner or post is not False or src.tapped or src.sick or not can_pay(g, p, 0, 'R'): return []
    junk = [c for c in p.hand if card_worth(g, p, c) < 30]
    cs = [m for m in p.perms if m.creature and m is not src and m.cd is not None and not m.cd.tags.get('leg')]
    if not junk or not cs: return []
    t = max(cs, key=lambda m: (etb_val(m) + epow(g, m) / 2))

    def go():
        if src.tapped or t not in p.perms or not can_pay(g, p, 0, 'R'): return False
        pay(g, p, 0, 'R'); src.tapped = True
        x = min(junk, key=lambda c: card_worth(g, p, c)); p.hand.remove(x); p.gy.append(x)
        tok = enter_token_copy(g, p, t.cd)
        if tok is not None:
            tok.sick = False
            if tok.data is None: tok.data = {}
            tok.data['jaxis_copy'] = True
        return True
    return [(1.5 + etb_val(t) / 2, f'Jaxis copies {t.name}', go)]


def etb_val(m):
    return __import__('dsl').etb_value(m.cd) if m.cd is not None and m.cd.dsl else (2.0 if m.cd is not None and 'etb' in m.cd.tags else 0.0)
note('Jaxis, the Troublemaker', 'Full', 'blitz {1}{R}; {R},{T}, discard: a hasty token copy of another creature '
     '(draws when it leaves at end of turn)')


# ------------------------------------------------------------------ Kaya's Wrath
@on("Kaya's Wrath", 'resolve')
def _kayas(g, p, c, ctx):
    mine = [m for m in p.perms if m.creature and not m.phased]
    for q in g.players:
        for m in list(q.perms):
            if m.creature and not m.phased: die(g, m, 'destroy')
    gain(p, sum(1 for m in mine if m not in p.perms))
note("Kaya's Wrath", 'Full', 'destroys all creatures; gain life for each of yours destroyed')


# ------------------------------------------------------------------ Lim-Dûl's Vault
@IC.spell("Lim-Dûl's Vault", prio=lambda g, p, c: 45 if __import__('pool_ai').wish_list(g, p) else 0, types='I',
          status=('Full', 'digs five at a time for 1 life each until a wanted card turns up, then stacks it on top'))
def _vault(g, p, c, ctx):
    import impl_topdeck as T, pool_ai
    wish = set(pool_ai.wish_list(g, p))
    for i in range(6):
        top = T.look(p, 5)
        if not top: return
        if any(x.name in wish for x in top) or i == 5 or p.life <= 15 or len(p.library) < 10:
            g.rng.shuffle(p.library); T.arrange(g, p, top); return
        p.library[:0] = top; lose_life(g, p, 1, p)


# ------------------------------------------------------------------ Loran of the Third Path
@on('Loran of the Third Path', 'options')
def _loran(g, src, p, s, post):
    if p is not src.owner or post is not None or src.tapped or src.sick: return []
    q = min(g.opps(p), key=lambda q: threat(g, p, q), default=None)
    if q is None: return []

    def go():
        if src.tapped: return False
        src.tapped = True; draw(g, p, 1); draw(g, q, 1); return True
    return [(0.6, 'Loran draw', go)]
note('Loran of the Third Path', 'Full', 'destroys an artifact or enchantment on entry; {T}: you and the least '
     'threatening opponent each draw (at end of turn)')


# ------------------------------------------------------------------ Mardu Charm
@on('Mardu Charm', 'resolve')
def _mardu(g, p, c, ctx):
    t = ctx.get('target')
    if t is not None and t in t.owner.perms: apply_removal(g, p, t, 'dmg4', c); return
    if ctx.get('mode') == 'discard':
        q = max(g.opps(p), key=lambda q: threat(g, p, q))
        cs = [x for x in q.hand if not x.creature and not x.land]
        if cs:
            x = max(cs, key=lambda x: card_worth(g, q, x)); q.hand.remove(x); q.gy.append(x)
            log(f'    {NAME(q)} discards {x.name}', g)
        return
    for m in make_tokens(g, p, 2, 1, warrior=True, color='W', types=('warrior',)):
        g.eot_kw.setdefault(id(m), set()).add('first strike')


@on('Mardu Charm', 'hand_options')
def _mardu_eot(g, c, p, s, post):
    if post is not None or not can_pay(g, p, 0, 'RWB'): return []

    def go():
        if c not in p.hand or not can_pay(g, p, 0, 'RWB'): return False
        p.hand.remove(c); pay(g, p, 0, 'RWB'); cast_card(g, p, c, 'lib', {})
        return True
    return [(1.2, 'Mardu Charm (two Warriors)', go)]
note('Mardu Charm', 'Full', '4 damage to a creature (removal), two first-strike Warriors at end of turn, or a '
     'noncreature card discarded')


# ------------------------------------------------------------------ Massacre Girl
@on('Massacre Girl', 'etb')
def _massacre(g, src, p, m):
    if m is not src: return
    log(f'    Massacre Girl: each other creature gets -1/-1, again for every creature that dies', g)
    shrink = 1
    applied = {}
    while shrink:
        before = sum(1 for q in g.players for x in q.perms if x.creature)
        for q in g.players:
            for x in list(q.perms):
                if x is src or not x.creature: continue
                _eot(g, x, -shrink, -shrink); applied[id(x)] = applied.get(id(x), 0) + shrink
        for q in g.players:
            for x in list(q.perms):
                if x is not src and x.creature and etgh(g, x) <= 0: die(g, x, 'sba')
        after = sum(1 for q in g.players for x in q.perms if x.creature)
        shrink = before - after
        if shrink <= 0 or after <= 1: break
note('Massacre Girl', 'Full', 'menace; -1/-1 to every other creature on entry, repeated for each creature that dies')
CI.SPELL_PRIO['Massacre Girl'] = lambda g, p, c: 60 if sum(pval(g, m) for q in g.opps(p) for m in q.perms if m.creature and etgh(g, m) <= 2) >= \
    8 + sum(pval(g, m) for m in p.perms if m.creature) else 0


# ------------------------------------------------------------------ Mishra's Bauble / Urza's Bauble
def bauble(name):
    @on(name, 'options')
    def _b(g, src, p, s, post):
        if p is not src.owner or src.tapped or post is not None: return []
        if any(m.cd is not None and m.cd.name == 'Urza, Lord High Artificer' for m in p.perms): return []

        def go():
            if src not in p.perms: return False
            die(g, src, 'sac'); p.delayed_draws = getattr(p, 'delayed_draws', 0) + 1
            log(f'  {NAME(p)} sacrifices {name} (draws at the next upkeep)', g); return True
        return [(0.8, name, go)]
    card(name, '', types='A', cost='0', dsl=[])
    note(name, 'Full', '{T}, sacrifice: draw at the beginning of the next upkeep (kept as an artifact for Urza)')


bauble("Mishra's Bauble")
bauble("Urza's Bauble")


# ------------------------------------------------------------------ Mogg Fanatic
@on('Mogg Fanatic', 'options')
def _fanatic(g, src, p, s, post):
    if p is not src.owner: return []
    t = best_opp_creature(g, p, lambda m: etgh(g, m) <= 1)
    lethal = [q for q in g.opps(p) if q.life <= 1]
    if lethal:
        return [(9.0, 'Mogg Fanatic (lethal)', lambda: (die(g, src, 'sac'), lose_life(g, lethal[0], 1, p, kind='burn'), True)[2])]
    if t is None or pval(g, t) < 3: return []
    return [(pval(g, t) - 1.5, f'Mogg Fanatic -> {t.name}', lambda: (die(g, src, 'sac'), apply_removal(g, p, t, 'dmg1'), True)[2])]
card('Mogg Fanatic', 'pow=1 tgh=1', dsl=[])
note('Mogg Fanatic', 'Full', 'sacrifice: 1 damage to a valuable X/1 or a player at 1 life')


# ------------------------------------------------------------------ Multani: return from the graveyard
@on("Multani, Yavimaya's Avatar", 'gy_options')
def _multani(g, c, p, s, post):
    if post is not False or not can_pay(g, p, 1, 'G') or len(p.lands) < 7: return []
    if p.land_turn == p.turns and not any(x.land for x in p.hand): return []

    def go():
        if c not in p.gy or not can_pay(g, p, 1, 'G') or len(p.lands) < 2: return False
        pay(g, p, 1, 'G')
        for L in sorted(p.lands, key=lambda L: (not L.tapped, len(L.cd.tags.get('c', ''))))[:2]:
            p.lands.remove(L); p.hand.append(L.cd)
        p.gy.remove(c); p.hand.append(c)
        log(f"  {NAME(p)} returns Multani to hand", g); return True
    return [(1.5, 'Multani (graveyard)', go)]
note("Multani, Yavimaya's Avatar", 'Full', '+1/+1 per land you control and land card in your graveyard; {1}{G}, return '
     'two lands: back to hand from the graveyard')


# ------------------------------------------------------------------ Muxus: attack pump
@on('Muxus, Goblin Grandee', 'attack')
def _muxus_attack(g, src, p, atk, d):
    if p is src.owner and src in atk:
        n = sum(1 for m in p.perms if m is not src and m.creature and E.has_type(m, 'goblin'))
        _eot(g, src, n, n)
note('Muxus, Goblin Grandee', 'Full', 'Goblins from the top six onto the battlefield; attacks with +1/+1 per other Goblin')


# ------------------------------------------------------------------ Oath of Teferi
@on('Oath of Teferi', 'etb')
def _oath(g, src, p, m):
    if m is not src: return
    o = src.owner
    cs = [x for x in o.perms if x is not src and x.cd is not None and not x.token and
          (etb_val(x) > 0 or (x.cd.start_loyalty and x.loyalty is not None and x.loyalty < int(x.cd.start_loyalty)))]
    if cs:
        t = max(cs, key=lambda x: etb_val(x) + (3 if x.cd.start_loyalty else 0))
        leave(g, t); o.oath_return = getattr(o, 'oath_return', []) + [t.cd]
        log(f'    Oath of Teferi exiles {t.name} until the end step', g)


@on('Oath of Teferi', 'end_step')
def _oath_back(g, src, p):
    o = src.owner
    for cd in getattr(o, 'oath_return', []): enter(g, o, cd)
    o.oath_return = []
note('Oath of Teferi', 'Full', 'blinks a permanent with an ETB or a used planeswalker until the end step; '
     'planeswalkers activate twice each turn')


def walker_uses(p):
    return 2 if any(m.cd is not None and m.cd.name == 'Oath of Teferi' and not m.phased for m in p.perms) else 1


# ------------------------------------------------------------------ Paradise Druid
@on('Paradise Druid', 'grant_kw')
def _pdruid(g, src, m, kw):
    return kw == 'hexproof' and m is src and not src.tapped
note('Paradise Druid', 'Full', 'hexproof while untapped; taps for any color')


# ------------------------------------------------------------------ Pashalik Mons: Goblin tokens
@on('Pashalik Mons', 'options')
def _mons_make(g, src, p, s, post):
    if p is not src.owner or not can_pay(g, p, 3, 'R'): return []
    fod = [m for m in p.perms if m.creature and m.token and E.has_type(m, 'goblin')] or \
          [m for m in p.perms if m.creature and E.has_type(m, 'goblin') and m is not src and pval(g, m) < 2]
    if not fod: return []

    def go():
        if not can_pay(g, p, 3, 'R') or fod[0] not in p.perms: return False
        pay(g, p, 3, 'R'); die(g, fod[0], 'sac')
        make_tokens(g, p, 2, 1, color='R', types=('goblin',)); return True
    return [(1.2 if post is None else 0.8, 'Pashalik Mons tokens', go)]
note('Pashalik Mons', 'Full', 'a Goblin of yours dying deals 1; {3}{R}, sacrifice a Goblin: two Goblin tokens')


# ------------------------------------------------------------------ Pia Nalaar
@on('Pia Nalaar', 'etb')
def _pia_etb(g, src, p, m):
    if m is src: make_tokens(g, src.owner, 1, 1, fly=True, types=('thopter', 'artifact'))


@on('Pia Nalaar', 'options')
def _pia(g, src, p, s, post):
    """{1}, sacrifice an artifact: target creature can't block (the biggest blocker, before a big attack)"""
    if p is not src.owner or post is not False or not can_pay(g, p, 1, ''): return []
    arts = [m for m in p.perms if (m.token and 'artifact' in m.ttypes) or (m.cd is not None and 'A' in m.cd.types and pval(g, m) < 2)]
    if not arts and p.treasures < 1: return []
    ready = sum(epow(g, m) for m in p.perms if m.creature and not m.sick and not m.tapped)
    b = best_opp_creature(g, p, lambda m: not m.tapped and epow(g, m) >= 3)
    if b is None or ready < 6: return []

    def go():
        if not can_pay(g, p, 1, '') or b not in b.owner.perms: return False
        pay(g, p, 1, '')
        if arts and arts[0] in p.perms: die(g, arts[0], 'sac')
        elif p.treasures: p.treasures -= 1
        else: return False
        g.eot_kw.setdefault(id(b), set()).add('cant_block')
        log(f'  {NAME(p)} uses Pia Nalaar: {b.name} can\'t block', g); return True
    return [(1.0, 'Pia Nalaar (can\'t block)', go)]
card('Pia Nalaar', 'leg human pow=2 tgh=2', dsl=[])
note('Pia Nalaar', 'Full', 'Thopter on entry; {1}, sacrifice an artifact: the biggest blocker can\'t block ({1}{R} '
     'pump for artifact creatures unused: rarely better)')


# ------------------------------------------------------------------ Quirion Ranger / Wirewood Symbiote
@on('Quirion Ranger', 'extra_mana')
def _quirion(g, src, p, U):
    """return a Forest to hand to untap your best mana creature (once per turn): the Forest is replayed as the land drop"""
    if src.owner is not p or (src.data or {}).get('used') == turn_now(g): return []
    if not any(L.cd.name == 'Forest' for L in p.lands) or p.land_turn == p.turns: return []
    dorks = [m for m in p.perms if m.creature and m.tapped and m.cd is not None and 'dork' in m.cd.tags and not m.sick]
    if not dorks: return []
    d = max(dorks, key=lambda m: CI.dyn_mana(g, p, m) if m.cd.name in CI.DYN_MANA else 1)
    amt = CI.dyn_mana(g, p, d) if d.cd.name in CI.DYN_MANA else 1
    if amt < 2: return []
    return [['QR', 'G', amt, src, d]]


@on('Wirewood Symbiote', 'extra_mana')
def _symbiote(g, src, p, U):
    """return an Elf to hand to untap your best mana creature (once per turn)"""
    if src.owner is not p or (src.data or {}).get('used') == turn_now(g): return []
    cheap = [m for m in p.perms if m.creature and m is not src and E.has_type(m, 'elf') and m.cd is not None and m.cd.cmc <= 2
             and 'dork' not in m.cd.tags]
    dorks = [m for m in p.perms if m.creature and m.tapped and m.cd is not None and 'dork' in m.cd.tags and not m.sick]
    if not cheap or not dorks: return []
    d = max(dorks, key=lambda m: CI.dyn_mana(g, p, m) if m.cd.name in CI.DYN_MANA else 1)
    amt = CI.dyn_mana(g, p, d) if d.cd.name in CI.DYN_MANA else 1
    if amt < 3: return []
    return [['WS', 'G', amt, src, d, cheap[0]]]


def special_unit_paid(g, p, u):
    """a special mana unit was used: pay its cost"""
    kind = u[0]
    if kind == 'H':
        c = u[3]
        if c in p.hand: p.hand.remove(c); p.exile.append(c)
    elif kind == 'SC':
        m = u[3]
        if m in p.perms: leave(g, m); CI.fire(g, 'sacrifice', p, m) if g.hooks else None
    elif kind == 'QR':
        src = u[3]
        if src.data is None: src.data = {}
        src.data['used'] = turn_now(g)
        forest = next((L for L in p.lands if L.cd.name == 'Forest'), None)
        if forest is not None: p.lands.remove(forest); p.hand.append(forest.cd)
    elif kind == 'WS':
        src, elf = u[3], u[5]
        if src.data is None: src.data = {}
        src.data['used'] = turn_now(g)
        if elf in p.perms: bounce(g, elf)
note('Quirion Ranger', 'Full', 'returns a Forest (replayed as the land drop) to untap the best mana creature, once a turn')
note('Wirewood Symbiote', 'Full', 'returns a cheap Elf to untap the best mana creature, once a turn')


# ------------------------------------------------------------------ Rabble Rousing: hideaway 5
@on('Rabble Rousing', 'etb')
def _rabble_hide(g, src, p, m):
    if m is not src: return
    o = src.owner
    top = [o.library.pop() for _ in range(min(5, len(o.library)))]
    if not top: return
    c = max(top, key=lambda c: (not c.land, c.cmc, card_worth(g, o, c)))
    top.remove(c); g.rng.shuffle(top); o.library[:0] = top
    if src.data is None: src.data = {}
    src.data['hidden'] = c


@on('Rabble Rousing', 'attack')
def _rabble_play(g, src, p, atk, d):
    if p is not src.owner or not src.data or src.data.get('hidden') is None: return
    if sum(1 for m in p.perms if m.creature) >= 10:
        c = src.data.pop('hidden')
        log(f'    Rabble Rousing plays the hidden {c.name}', g)
        if c.land: p.lands.append(Land(c, False))
        else: cast_card(g, p, c, 'lib', {})
note('Rabble Rousing', 'Full', 'a Citizen per attacker; hideaway 5, played free once you control ten creatures')


# ------------------------------------------------------------------ Ranger-Captain of Eos: silence for a combo turn
def ranger_silence(g, p):
    """before a combo or a big spell: sacrifice Ranger-Captain so opponents can't cast noncreature spells"""
    m = next((m for m in p.perms if m.cd is not None and m.cd.name == 'Ranger-Captain of Eos' and not m.phased), None)
    if m is None: return False
    die(g, m, 'sac'); g.silence = (turn_now(g), p)
    log(f'  {NAME(p)} sacrifices Ranger-Captain of Eos: opponents can\'t cast noncreature spells this turn', g)
    return True


def silence_active(g, q):
    s = getattr(g, 'silence', None)
    return bool(s and s[0] == turn_now(g) and s[1] is not q)
note('Ranger-Captain of Eos', 'Full', 'fetches a 1-drop creature; sacrificed before a combo to stop noncreature spells')


# ------------------------------------------------------------------ Rapid Hybridization
@on('Rapid Hybridization', 'resolve')
def _hybrid(g, p, c, ctx):
    t = ctx.get('target')
    if t is None or t not in t.owner.perms: return
    q = t.owner
    apply_removal(g, p, t, 'destroy', c)
    if t not in q.perms: make_tokens(g, q, 1, 3, color='G', types=('frog', 'lizard'))
note('Rapid Hybridization', 'Full', 'destroys a creature; its controller gets a 3/3 Frog Lizard')


# ------------------------------------------------------------------ wheels: Wheel of Fortune, Reforge the Soul
def wheel(g, p):
    for q in [q for q in g.players if q.alive]:
        q.gy.extend(q.hand); q.hand = []
        draw(g, q, 7)


def wheel_prio(g, p, c):
    import impl_combos
    if any(x.name in impl_combos.PIECES for x in p.hand if x is not c): return 0     # don't wheel away a combo piece
    mine = len(p.hand) - 1
    theirs = max((len(q.hand) for q in g.opps(p)), default=0)
    return 55 if mine <= 2 and theirs <= 5 else 0


@IC.spell('Wheel of Fortune', prio=wheel_prio, types='S', status=('Full', 'each player discards their hand and draws seven '
          '(cast when your hand is nearly empty)'))
def _wheel(g, p, c, ctx): wheel(g, p)


@IC.spell('Reforge the Soul', prio=wheel_prio, types='S', status=('Full', 'wheel; miracle {1}{R} when it is the first '
          'card drawn in a turn'))
def _reforge(g, p, c, ctx): wheel(g, p)


def miracle_options(g, p, s, post):
    mc = getattr(p, 'miracle', None)
    if not mc or mc[0] != turn_now(g) or mc[1] not in p.hand or mc[1].name != 'Reforge the Soul' or not can_pay(g, p, 1, 'R'):
        return []
    c = mc[1]
    if len(p.hand) > 4: return []

    def go():
        if c not in p.hand or not can_pay(g, p, 1, 'R'): return False
        pay(g, p, 1, 'R'); p.miracle = None
        log(f'  {NAME(p)} casts Reforge the Soul for its miracle cost', g)
        cast_card(g, p, c, 'hand', {}); return True
    return [(3.0, 'Reforge the Soul (miracle)', go)]


# ------------------------------------------------------------------ Retreat to Coralhelm
@on('Retreat to Coralhelm', 'landfall')
def _coralhelm(g, src, p):
    if p is not src.owner: return
    tapped = [m for m in p.perms if m.creature and m.tapped and m.cd is not None and 'dork' in m.cd.tags]
    if tapped:
        m = max(tapped, key=lambda m: CI.dyn_mana(g, p, m) if m.cd.name in CI.DYN_MANA else 1); m.tapped = False
    else:
        import impl_topdeck; impl_topdeck.scry(g, p, 1)
card('Retreat to Coralhelm', '', types='E', dsl=[])
note('Retreat to Coralhelm', 'Full', 'landfall: untaps a tapped mana creature, else scry 1')


# ------------------------------------------------------------------ Sai, Master Thopterist: sacrifice for cards
@on('Sai, Master Thopterist', 'options')
def _sai_draw(g, src, p, s, post):
    if p is not src.owner or post is not None or not can_pay(g, p, 1, 'U'): return []
    thop = [m for m in p.perms if m.token and ('thopter' in m.ttypes or 'artifact' in m.ttypes)]
    urza = any(m.cd is not None and m.cd.name == 'Urza, Lord High Artificer' for m in p.perms)
    if len(thop) < (4 if urza else 3): return []

    def go():
        if not can_pay(g, p, 1, 'U') or len([m for m in thop if m in p.perms]) < 2: return False
        pay(g, p, 1, 'U')
        for m in [m for m in thop if m in p.perms][:2]: die(g, m, 'sac')
        draw(g, p, 1); return True
    return [(1.0, 'Sai: sacrifice two for a card', go)]
note('Sai, Master Thopterist', 'Full', 'a Thopter per artifact spell; {1}{U}, sacrifice two artifacts: draw (spare Thopters)')


# ------------------------------------------------------------------ Sakashima's Protege
@on("Sakashima's Protege", 'etb')
def _protege(g, src, p, m):
    if m is not src: return
    o = src.owner
    if g.last_cast_etb:                            # cascade (the spell was cast)
        cs = []
        while o.library:
            c = o.library.pop()
            if not c.land and c.cmc < 6:
                log(f'    cascade: {c.name}', g); cs.append(c)
                cast_card(g, o, c, 'lib', {})
                break
            cs.append(c)
        o.library[:0] = [c for c in cs if c not in o.hand and c not in o.gy and not any(x.cd is c for x in o.perms)
                         and c not in o.exile][:len(cs)]
    ent = [x for x in getattr(g, 'entered', []) if x[0] == turn_now(g) and x[1] is not src and x[1].cd is not None
           and x[1] in x[1].owner.perms]
    if ent:
        t = max(ent, key=lambda x: pval(g, x[1]))[1]
        if pval(g, t) >= 4:
            log(f"    Sakashima's Protege enters as a copy of {t.name}", g)
            leave(g, src)
            if src.cd in o.gy: o.gy.remove(src.cd)
            x = enter_token_copy(g, o, t.cd)
note("Sakashima's Protege", 'Full', 'flash; cascade; enters as a copy of the best permanent that entered this turn '
     '(the copy is made as a token copy)')


# ------------------------------------------------------------------ Sentinel's Eyes: escape
@on("Sentinel's Eyes", 'gy_options')
def _eyes(g, c, p, s, post):
    if post is not False or not can_pay(g, p, 0, 'W') or not any(m.creature for m in p.perms): return []
    fod = [x for x in p.gy if x is not c]
    if len(fod) < 2: return []

    def go():
        if c not in p.gy or not can_pay(g, p, 0, 'W'): return False
        pay(g, p, 0, 'W')
        for x in sorted([x for x in p.gy if x is not c], key=lambda x: card_worth(g, p, x, True))[:2]:
            p.gy.remove(x); p.exile.append(x)
        p.gy.remove(c); log(f"  {NAME(p)} escapes Sentinel's Eyes", g); enter(g, p, c); return True
    return [(1.5, "escape Sentinel's Eyes", go)]
note("Sentinel's Eyes", 'Full', '+1/+1 and vigilance; escape {W} and two graveyard cards')


# ------------------------------------------------------------------ Sigarda's Aid
@on("Sigarda's Aid", 'etb')
def _aid(g, src, p, m):
    if m.owner is src.owner and m.cd is not None and 'equipment' in m.cd.subtypes and m is not src:
        cr = [x for x in src.owner.perms if x.creature and not x.phased]
        if cr: m.attached = max(cr, key=lambda x: (x.is_cmd, pval(g, x)))
card("Sigarda's Aid", '', types='E', dsl=[])
note("Sigarda's Aid", 'Full', 'Auras and Equipment at instant speed (Auras cast at end of turn); Equipment attaches free')


def aid_active(p):
    return any(m.cd is not None and m.cd.name == "Sigarda's Aid" and not m.phased for m in p.perms)


# ------------------------------------------------------------------ Skyclave Apparition: the token when it leaves
@on('Skyclave Apparition', 'etb')
def _skyclave_rec(g, src, p, m):
    if m is src:
        lr = getattr(g, 'last_removed', None)
        if lr and lr[3] is src.owner and lr[2] == 'exile':
            if src.data is None: src.data = {}
            src.data['exiled'] = (lr[0], lr[1])


@on('Skyclave Apparition', 'leaves')
def _skyclave_leave(g, m):
    ex = (m.data or {}).get('exiled')
    if ex and ex[0] is not None and ex[1].alive:
        cd, owner = ex
        make_tokens(g, owner, 1, cd.cmc, color='U', types=('illusion',))
        log(f'    {NAME(owner)} gets a {cd.cmc}/{cd.cmc} Illusion', g)
note('Skyclave Apparition', 'Full', 'exiles a permanent with MV 4 or less; its owner gets an X/X Illusion when the '
     'Apparition leaves')


# ------------------------------------------------------------------ Snap: untap two lands
@on('Snap', 'resolve')
def _snap(g, p, c, ctx):
    t = ctx.get('target')
    if t is not None and t in t.owner.perms: apply_removal(g, p, t, 'bounce', c)
    n = 0
    for L in p.lands:
        if L.tapped and n < 2: L.tapped = False; n += 1
note('Snap', 'Full', 'bounces a creature and untaps two lands')


# ------------------------------------------------------------------ Starfield Mystic
@on('Starfield Mystic', 'dies')
def _mystic_grow(g, src, m, cause):
    if m is not src and m.owner is src.owner and m.cd is not None and 'E' in m.cd.types: src.plus += 1
note('Starfield Mystic', 'Full', 'enchantments cost {1} less; +1/+1 counter when one of your enchantments goes to the graveyard')


# ------------------------------------------------------------------ Starfield of Nyx: enchantments become creatures
def starfield_update(g, p):
    on = any(m.cd is not None and m.cd.name == 'Starfield of Nyx' and not m.phased for m in p.perms) and \
        sum(1 for m in p.perms if m.cd is not None and 'E' in m.cd.types and not m.phased) >= 5
    for m in p.perms:
        if m.cd is None or 'E' not in m.cd.types or m.cd.creature or 'aura' in m.cd.subtypes: continue
        if m.cd.name == 'Starfield of Nyx': continue
        was = bool(m.data and m.data.get('anim'))
        if on and not was:
            if m.data is None: m.data = {}
            m.data['anim'] = True; m.pow = m.tgh = m.cd.cmc
        elif not on and was:
            m.data['anim'] = False; m.pow = m.tgh = 0
    g.bf_ver = getattr(g, 'bf_ver', 0) + 1


for _ev in ('etb', 'upkeep'):
    @on('Starfield of Nyx', _ev)
    def _sf(g, src, *a): starfield_update(g, src.owner)


@on('Starfield of Nyx', 'dies')
def _sf_d(g, src, m, cause):
    if m.owner is src.owner and m.cd is not None and 'E' in m.cd.types: starfield_update(g, src.owner)


@on('Starfield of Nyx', 'leaves')
def _sf_gone(g, m):
    for x in m.owner.perms:
        if x.data and x.data.get('anim'): x.data['anim'] = False; x.pow = x.tgh = 0
note('Starfield of Nyx', 'Full', 'returns an enchantment each upkeep; with five enchantments the other non-Aura '
     'enchantments are creatures with P/T equal to their mana value')


# ------------------------------------------------------------------ Sunfall
@on('Sunfall', 'resolve')
def _sunfall(g, p, c, ctx):
    n = 0
    for q in g.players:
        for m in list(q.perms):
            if m.creature and not m.phased: exile_perm(g, m); n += 1
    if n:
        p.incubator = getattr(p, 'incubator', 0) + n
        log(f'    {NAME(p)} incubates {n}', g)


def incubator_options(g, p, s, post):
    n = getattr(p, 'incubator', 0)
    if not n or not can_pay(g, p, 2, ''): return []

    def go():
        if not can_pay(g, p, 2, ''): return False
        pay(g, p, 2, ''); p.incubator = 0
        make_tokens(g, p, 1, n, types=('phyrexian', 'artifact')); log(f'  {NAME(p)} transforms the Incubator ({n}/{n})', g)
        return True
    return [(1.0 + n / 2.0, 'transform Incubator', go)]
note('Sunfall', 'Full', 'exiles all creatures; an Incubator with that many counters ({2}: an X/X artifact creature)')


# ------------------------------------------------------------------ Tajic, Legion's Edge
@on("Tajic, Legion's Edge", 'attack')
def _tajic_fs(g, src, p, atk, d):
    if p is src.owner and src in atk and can_pay(g, p, 0, 'RW') and total_mana(g, p) >= 4:
        pay(g, p, 0, 'RW'); g.eot_kw.setdefault(id(src), set()).add('first strike')
card("Tajic, Legion's Edge", 'leg human pow=3 tgh=2 haste', dsl=[], kws={'haste', 'mentor'})
note("Tajic, Legion's Edge", 'Full', 'haste, mentor; prevents noncombat damage to your other creatures; {R}{W}: first strike')


# ------------------------------------------------------------------ Tekuthal: indestructible counter
@on('Tekuthal, Inquiry Dominus', 'options')
def _tekuthal_ind(g, src, p, s, post):
    if p is not src.owner or (src.data or {}).get('indestr') or post is None: return []
    pool = [m for m in p.perms if m is not src and (m.plus > 0 or (m.loyalty or 0) > 3)]
    have = sum(m.plus for m in pool) + sum(max(0, (m.loyalty or 0) - 3) for m in pool)
    if have < 3 or not can_pay(g, p, 1, '') or (not can_pay(g, p, 1, 'UU') and p.life < 20): return []

    def go():
        if not can_pay(g, p, 1, ''): return False
        if can_pay(g, p, 1, 'UU'): pay(g, p, 1, 'UU')
        else: pay(g, p, 1, ''); lose_life(g, p, 4, p)
        left = 3
        for m in sorted(pool, key=lambda m: pval(g, m)):
            while left and m.plus > 0: m.plus -= 1; left -= 1
            while left and (m.loyalty or 0) > 3: m.loyalty -= 1; left -= 1
        if src.data is None: src.data = {}
        src.data['indestr'] = True
        log(f'  {NAME(p)} puts an indestructible counter on Tekuthal', g); return True
    return [(1.5 + 0.2 * pval(g, src), 'Tekuthal indestructible', go)]


@on('Tekuthal, Inquiry Dominus', 'grant_kw')
def _tekuthal_kw(g, src, m, kw):
    return kw == 'indestructible' and m is src and bool(src.data and src.data.get('indestr'))
note('Tekuthal, Inquiry Dominus', 'Full', 'flying; proliferate twice; removes three counters for an indestructible counter')


# ------------------------------------------------------------------ Temur Sabertooth
@on('Temur Sabertooth', 'options')
def _sabertooth(g, src, p, s, post):
    """{1}{G}: return another creature to hand (and gain indestructible): rebuys an ETB creature when there is mana
    to recast it this turn (with Chulane: another draw and land)"""
    if p is not src.owner or post is not False or not can_pay(g, p, 1, 'G'): return []
    chulane = any(m.cd is not None and m.cd.name == 'Chulane, Teller of Tales' for m in p.perms)
    cs = []
    for m in p.perms:
        if not m.creature or m is src or m.token or m.cd is None or m.is_cmd: continue
        v = etb_val(m) + (2.0 if chulane else 0)
        if v <= 0: continue
        gen, pips = cost_of(p, m.cd)
        if not can_pay(g, p, 2 + gen, 'G' + pips): continue
        cs.append((v, m))
    if not cs: return []
    v, m = max(cs, key=lambda x: x[0])

    def go():
        if m not in p.perms or not can_pay(g, p, 1, 'G'): return False
        pay(g, p, 1, 'G'); bounce(g, m); g.eot_kw.setdefault(id(src), set()).add('indestructible')
        log(f'  {NAME(p)} returns {m.name} with Temur Sabertooth', g); return True
    return [(1.0 + v, f'Temur Sabertooth rebuys {m.name}', go)]
card('Temur Sabertooth', 'pow=4 tgh=3', dsl=[])
note('Temur Sabertooth', 'Full', '{1}{G}: returns an ETB creature to recast it (Chulane: another trigger), gaining indestructible')


# ------------------------------------------------------------------ The One Ring
@on('The One Ring', 'etb')
def _ring(g, src, p, m):
    if m is src and g.last_cast_etb:
        src.owner.ring_prot = True
        log(f'    {NAME(src.owner)} gains protection from everything until their next turn', g)


@on('The One Ring', 'upkeep')
def _ring_burden(g, src, p):
    if p is src.owner and src.data and src.data.get('burden'): lose_life(g, p, src.data['burden'], p)


@on('The One Ring', 'options')
def _ring_draw(g, src, p, s, post):
    if p is not src.owner or src.tapped: return []
    b = (src.data or {}).get('burden', 0)
    if p.life - (b + 1) * 2 < 12 and b >= 2: return []

    def go():
        if src.tapped: return False
        src.tapped = True
        if src.data is None: src.data = {}
        src.data['burden'] = src.data.get('burden', 0) + 1
        draw(g, p, src.data['burden']); return True
    return [(2.0 + b, 'The One Ring', go)]
card('The One Ring', 'leg', types='A', dsl=[], kws={'indestructible'})
note('The One Ring', 'Full', 'indestructible; protection from everything until your next turn when cast; {T}: burden '
     'counter, draw that many; upkeep life loss per burden')
CI.SPELL_PRIO['The One Ring'] = 62


# ------------------------------------------------------------------ Tishana's Tidebinder
def tidebinder_response(g, p, m):
    """an opponent's creature with a strong ETB enters: a pool deck holding Tidebinder counters the trigger (the
    creature loses its abilities while Tidebinder stays)"""
    v = etb_val(m) if m.cd is not None else 0
    if v < 3 and not (m.cd is not None and m.cd.bomb >= 7): return False
    for q in g.after(p):
        if q is p or not q.alive or q.key in __import__('decks').DECKS: continue
        c = next((x for x in q.hand if x.name == "Tishana's Tidebinder"), None)
        if c is None or not castable(g, q, c) or not can_pay(g, q, *cost_of(q, c)): continue
        pay(g, q, *cost_of(q, c)); q.hand.remove(c)
        log(f"    {NAME(q)} flashes in Tishana's Tidebinder: {m.name}'s trigger is countered", g)
        on_cast(g, q, c)
        if not counter_window(g, q, c, 4, {}): q.gy.append(c); return False
        tb = enter(g, q, c, was_cast=True)
        if tb.data is None: tb.data = {}
        tb.data['bound'] = m; m.neutered = True
        return True
    return False


@on("Tishana's Tidebinder", 'leaves')
def _tide_free(g, m):
    b = (m.data or {}).get('bound')
    if b is not None: b.neutered = False
card("Tishana's Tidebinder", 'pow=3 tgh=2 flash', dsl=[])
note("Tishana's Tidebinder", 'Full', 'flash; counters an opponent\'s strong ETB trigger (that creature loses its '
     'abilities while Tidebinder stays) or a combo piece\'s ability')


# ------------------------------------------------------------------ Tyvar the Bellicose: counters on mana creatures
@on('Tyvar the Bellicose', 'mana_tapped')
def _tyvar(g, src, p, m, amt):
    if p is src.owner and m.creature and once_per_turn(g, p, f'tyvar{id(m)}'): m.plus += amt
note('Tyvar the Bellicose', 'Full', 'attacking Elves gain deathtouch; creatures that tap for mana get that many +1/+1 '
     'counters (once each turn)')


# ------------------------------------------------------------------ Venser: bounce a spell
note('Venser, Shaper Savant', 'Full', 'flash; bounces a spell (as a counter, the spell returns to hand) or a permanent')


# ------------------------------------------------------------------ Voltaic Key
@on('Voltaic Key', 'extra_mana')
def _key(g, src, p, U):
    """{1}, {T}: untap an artifact: counted as the net mana of re-tapping the best mana rock (Monoliths, Sol Ring)"""
    if src.owner is not p or src.tapped: return []
    best = max((int(m.cd.tags['rock'].split(':')[0]) for m in p.perms if m.cd is not None and 'rock' in m.cd.tags
                and 'A' in m.cd.types and m is not src), default=0)
    if best < 2: return []
    return [[src, 'C', best - 1]]
card('Voltaic Key', '', types='A', dsl=[])
note('Voltaic Key', 'Full', '{1},{T}: untap an artifact (the best mana rock: net mana), also used in the Monolith loops')


# ------------------------------------------------------------------ Whirler Rogue
@on('Whirler Rogue', 'attack')
def _whirler(g, src, p, atk, d):
    if p is not src.owner: return
    arts = [m for m in p.perms if not m.tapped and m not in atk and ((m.token and 'artifact' in m.ttypes) or
                                                                   (m.cd is not None and 'A' in m.cd.types))]
    if len(arts) < 2: return
    blockers = [b for b in d.perms if b.creature and not b.tapped]
    if not blockers: return
    import impl_t4
    cands = [a for a in atk if a in p.perms and not (E.DSLMOD and E.DSLMOD.has_kw(g, a, 'unblockable'))]
    if not cands: return
    a = max(cands, key=lambda a: (impl_t4._ninja(a) or a.is_cmd, epow(g, a)))
    for m in arts[:2]: m.tapped = True
    g.eot_kw.setdefault(id(a), set()).add('unblockable')
    log(f'    Whirler Rogue: {a.name} can\'t be blocked', g)
card('Whirler Rogue', 'human rogue pow=2 tgh=2', dsl=[])


@on('Whirler Rogue', 'etb')
def _whirler_etb(g, src, p, m):
    if m is src: make_tokens(g, src.owner, 2, 1, fly=True, types=('thopter', 'artifact'))
note('Whirler Rogue', 'Full', 'two flying Thopters; taps two artifacts to make an attacker (a Ninja first) unblockable')
