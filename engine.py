"""Abstracted 4-player Commander engine.  Rules are simplified on purpose:
role-tagged cards, greedy mana payment, heuristic AIs, simplified combat."""
import random, re
from collections import defaultdict
from carddb import DB_TEXT

IDENT = {'seph': 'WUBG', 'veyran': 'UR', 'sauron': 'UBR', 'najeela': 'WUBRG'}
# Outside decks (opponent pools) register here: key -> {'ident': 'WU', 'name': 'Brago'}. The four main decks
# keep their hard-wired entries in IDENT / NAME / the AI tables; anything else falls back to generic defaults.
SEATS = {}


def register_seat(key, ident, name):
    SEATS[key] = {'ident': ident, 'name': name}


class CD:
    def __init__(s, name, types, cost, tags):
        s.name, s.types = name, types
        if cost in ('-', ''):
            s.generic, s.pips = 0, ''
        else:
            m = re.match(r'^(\d*)(.*)$', cost)
            s.generic = int(m.group(1)) if m.group(1) else 0
            s.pips = m.group(2)
        s.cmc = s.generic + len(s.pips)
        s.tags = {}
        for t in tags.split():
            if '=' in t:
                k, v = t.split('=', 1); s.tags[k] = v
            else:
                s.tags[t] = True
        s.land = 'L' in types
        s.creature = 'C' in types
        s.instant = 'I' in types
        s.sorcery = 'S' in types
        s.perm = not (s.instant or s.sorcery or s.land)
        s.pow = int(s.tags.get('pow', 0))
        s.tgh = int(s.tags.get('tgh', max(1, s.pow))) if s.creature else 0
        s.bomb = int(s.tags.get('bomb', 0))
        s.source = 'manual'; s.unparsed = []; s.game_changer = None
        s.dsl = None; s.start_loyalty = None
        s.kws = frozenset()      # Scryfall keywords (lower case) for cards built from Scryfall data
        s.subtypes = frozenset() # creature / other subtypes, lower case (Scryfall-built cards)
        s.protfrom = ''          # colours it has protection from (printed)
        s.ward = 0               # ward {N}

    def __repr__(s):
        return s.name


DB = {}
for _line in DB_TEXT.strip().splitlines():
    _n, _t, _c, _g = [x.strip() for x in _line.split('|')]
    DB[_n] = CD(_n, _t, _c, _g)


class Land:
    __slots__ = ('cd', 'tapped')

    def __init__(s, cd, tapped):
        s.cd, s.tapped = cd, tapped


class Perm:
    __slots__ = ('cd', 'owner', 'orig', 'token', 'tapped', 'sick', 'pow', 'tgh', 'fly', 'dt', 'vig',
                 'life', 'plus', 'undying', 'army', 'warrior', 'noatk', 'name', 'phased', 'attached',
                 'age', 'neutered', 'is_cmd', 'phys', 'temp', 'loyalty', 'loyalty_used', 'colors', 'ttypes', 'data')

    def __init__(s, owner, cd=None, pw=1, tg=None, fly=False, warrior=False, name='Token'):
        s.cd, s.owner, s.orig = cd, owner, owner
        s.token = cd is None
        s.tapped = False; s.sick = True; s.plus = 0; s.undying = False; s.army = False
        s.phased = False; s.attached = None; s.age = 0; s.neutered = False; s.is_cmd = False
        s.phys = None; s.temp = False; s.loyalty = None; s.loyalty_used = None; s.colors = ''
        s.ttypes = frozenset(); s.data = None
        if cd:
            t = cd.tags
            s.name = cd.name; s.pow = cd.pow; s.tgh = cd.tgh
            s.fly = 'fly' in t; s.dt = 'dt' in t; s.vig = 'vig' in t; s.life = 'lifelink' in t
            s.warrior = 'warrior' in t; s.noatk = 'noatk' in t
        else:
            s.name = name; s.pow = pw; s.tgh = pw if tg is None else tg
            s.fly = fly; s.dt = False; s.vig = False; s.life = False; s.warrior = warrior; s.noatk = False

    @property
    def creature(s):
        return s.token or s.cd.creature

    def tag(s, k):
        return s.cd is not None and k in s.cd.tags


class Player:
    def __init__(s, key, cards, cmdname):
        s.key = key; s.ident = IDENT[key] if key in IDENT else SEATS[key]['ident']
        s.cmd = DB[cmdname]
        lib = list(cards); lib.remove(cmdname)
        s.library = [DB[n] for n in lib]
        s.hand, s.gy, s.exile, s.lands, s.perms = [], [], [], [], []
        s.floatR = 0          # red mana from Birgi, lasts until end of turn
        s.floatA = 0          # any-colour mana added by rituals / DSL effects, lasts until end of turn
        s.extra_combats = 0
        s.cast_names = set(); s.seen_names = set(); s.lost_names = defaultdict(int)
        s.clues = 0; s.hit_turn = -1; s.jin_turn = None; s.flag_turn = {}
        s.life = 40; s.alive = True; s.treasures = 0; s.turns = 0
        s.cmd_in_zone = True; s.tax = 0
        s.cmd_dmg = defaultdict(int); s.stats = defaultdict(int)
        s.land_played = False; s.yawg = False; s.spells_this_turn = 0
        s.decked = False; s.last_src = None; s.killer = None
        s.pump = 0; s.pumpadd = 0; s.trample = False; s.combo_tried = False
        s.removed_bombs = set(); s.first_bomb = None
        s.milestone = {}
        s.ring_prot = False   # The One Ring: protection from everything until this player's next turn
        s.floatU = 0          # blue mana (Lion's Eye Diamond), lasts until end of turn
        s.draw_st = None; s.draw_n = 0      # cards drawn this turn (Narset, Parter of Veils)
        s.chasm_age = 0       # Glacial Chasm age counters (cumulative upkeep)
        s.impulse = []        # cards exiled with "you may play them this turn" (Jeska's Will), held in hand
        s.land_turn = -1      # turn of this player's last land drop
        s.agent_ids = set()   # cards taken with Opposition Agent (castable with mana of any type)


class Game:
    def __init__(s, players, rng, goldfish=False):
        s.players = players; s.rng = rng; s.goldfish = goldfish
        s.over = False; s.winner = None; s.wintype = None; s.round = 0
        s.active = None
        s.dsl_on = False; s.eot_pt = {}; s.eot_kw = {}; s.dsl_depth = 0
        s.flutes = []          # (Disruptor Flute permanent, chosen card name)
        s.elim = []
        s.log = None          # list of strings when tracing a game
        s.hooks = []          # permanents with hand-written implementations (cardimpl), in entry order

    def opps(s, p):
        return [q for q in s.players if q.alive and q is not p]

    def after(s, p):
        i = s.players.index(p)
        return [s.players[(i + k) % len(s.players)] for k in range(1, len(s.players))]


# ---------------------------------------------------------------- trace log
CUR_G = None
CI = None                # hand-written card implementations (cardimpl.py) register themselves here
POOL_RULES = False       # pool games: rules fixes that would change the original four-deck results
DSLMOD = None             # the card-ability interpreter (dsl.py) registers itself here
AI_MODE = 'adaptive'     # 'adaptive' (probabilistic, board-reading) or 'rigid' (fixed priorities)
DAMAGE_HOOK = None


def NAME(p):
    n = {'seph': 'Sephiroth', 'veyran': 'Veyran', 'sauron': 'Sauron', 'najeela': 'Najeela'}.get(p.key)
    return n if n is not None else SEATS[p.key]['name']


def log(msg, g=None):
    g = g or CUR_G
    if g is not None and g.log is not None:
        g.log.append(f'R{g.round:<2d} {msg}')


# ---------------------------------------------------------------- helpers
def has(p, tag):
    for m in p.perms:
        if m.cd is not None and not m.phased and tag in m.cd.tags and not m.neutered:
            return True
    return False


def find(p, tag):
    return [m for m in p.perms if m.cd is not None and not m.phased and tag in m.cd.tags]


def opp_has(g, p, tag):
    return any(has(q, tag) for q in g.opps(p))


def army_of(p):
    for m in p.perms:
        if m.army and not m.phased:
            return m
    return None


def equipped(m, tag):
    return any(e.attached is m and tag in e.cd.tags for e in m.owner.perms if e.cd)


def norn_vs(g, p):
    return opp_has(g, p, 'normgc')


def static_bonus(g, m):
    """anthems and equipment that change a creature's size"""
    if not m.creature: return 0
    p = m.owner; b = 0
    t = m.cd.tags if m.cd is not None else {}
    if has(p, 'anthem2') and 'anthem2' not in t: b += 2                       # Elesh Norn, Grand Cenobite
    if has(p, 'anthemnh') and 'anthemnh' not in t and 'human' not in t: b += 1   # Mikaeus
    if has(p, 'warleader'): b += 1                                                # Warleader's Call
    for x in p.perms:                                                             # auto-tagged anthems
        if x.cd is not None and 'anth' in x.cd.tags and x is not m and not x.phased: b += int(x.cd.tags['anth'])
    if equipped(m, 'flail'): b += 3                                               # Conqueror's Flail (~3 colors)
    if equipped(m, 'animist'): b += 1
    return b


def epow(g, m):
    p = m.owner
    v = m.pow + m.plus + static_bonus(g, m)
    if m.cd is not None and 'adeline' in m.cd.tags:
        v = sum(1 for x in p.perms if x.creature and not x.phased) + m.plus + static_bonus(g, m)
    if equipped(m, 'sword'): v += 2
    if norn_vs(g, p): v -= 2
    if DSLMOD is not None: v += DSLMOD.pt(g, m)[0]
    if p.pump and m.creature: v = max(v, p.pump)
    v += p.pumpadd if m.creature else 0
    return max(0, v)


def etgh(g, m):
    v = m.tgh + m.plus + static_bonus(g, m)
    if DSLMOD is not None: v += DSLMOD.pt(g, m)[1]
    if equipped(m, 'sword'): v += 2
    if norn_vs(g, m.owner): v -= 2
    return v


def subtypes(m):
    """a permanent's subtypes (lower case): printed for Scryfall-built cards, set on tokens by their maker"""
    if m.cd is None: return m.ttypes
    s = m.cd.subtypes
    if 'changeling' in m.cd.kws: return s | ALL_TYPES
    if not s:
        t = m.cd.tags
        s = frozenset(k for k in ('human', 'warrior', 'shaman', 'wizard') if k in t)
    return s | m.ttypes


ALL_TYPES = frozenset(('human', 'warrior', 'shaman', 'wizard', 'elf', 'goblin', 'ninja', 'rogue', 'zombie', 'angel',
                       'demon', 'dragon', 'soldier', 'knight', 'cleric', 'faerie', 'elemental', 'druid', 'spirit'))


def has_type(m, t):
    return t in subtypes(m)


def indestructible(g, m):
    return DSLMOD is not None and DSLMOD.has_kw(g, m, 'indestructible')


def untargetable(g, m):
    if m.phased: return True
    if DSLMOD is not None and (DSLMOD.has_kw(g, m, 'hexproof') or DSLMOD.has_kw(g, m, 'shroud')): return True
    if m.cd is not None and m.cd.tags.get('sauron'): return True   # ward: sac a legendary
    if equipped(m, 'cloak') or any(e.attached is m and (e.cd.tags.get('prot') == 'boots' or 'spider' in e.cd.tags)
                                   for e in m.owner.perms if e.cd):
        return True
    return False


def colors_of(m):
    """a permanent's colours (cards: coloured mana symbols in the cost; tokens: their token colour)"""
    if m is None: return set()
    if m.cd is not None: return set(m.cd.pips) & set('WUBRG')
    return set(m.colors or '')


def prot_colors(g, m):
    """colours m has protection from: Sword of Feast and Famine (B, G) plus auto-modeled equipment"""
    s = set()
    if equipped(m, 'sword'): s |= {'B', 'G'}
    if DSLMOD is not None: s |= set(DSLMOD.protection(g, m))
    if m.cd is not None and m.cd.protfrom: s |= set(m.cd.protfrom)
    if getattr(g, 'auras', None): s |= set(CI.attached_prot(g, m))
    return s


def protected_from(g, m, source_colors):
    return bool(prot_colors(g, m) & set(source_colors))


def stopped(g, name):
    """Disruptor Flute: is a Flute on the battlefield naming this card?"""
    if g is None or not getattr(g, 'flutes', None): return False
    return any(nm == name and f in f.owner.perms and not f.phased for f, nm in g.flutes)


def silenced(g, q):
    """Conqueror's Flail: while attached, opponents can't cast spells during its controller's turn."""
    a = getattr(g, 'active', None)
    if a is None or a is q or not a.alive: return False
    return any(e.cd is not None and 'flail' in e.cd.tags and e.attached is not None and e.attached in a.perms
               for e in a.perms)


def parse_cost(s):
    m = re.match(r'^(\d*)(.*)$', s)
    return (int(m.group(1)) if m.group(1) else 0), m.group(2)


def once_per_turn(g, p, key):
    stamp = (g.round, getattr(g, 'active', None) and g.active.key)
    if p.flag_turn.get(key) == stamp: return False
    p.flag_turn[key] = stamp
    return True


# lose_life kinds that are damage (prevented by protection); 'drain' and 'other' are life loss.
# 'triggers' is mostly damage (Orcish Bowmasters, Kaervek, DSL damage abilities); Undermine passes damage=False.
DAMAGE_KINDS = ('combat', 'burn', 'aether', 'triggers')


def lose_life(g, p, n, src, kind='other', damage=None):
    if n <= 0 or not p.alive: return
    if damage is None: damage = kind in DAMAGE_KINDS
    if damage and prevents_damage(g, p, src):
        p.stats['dmg_prevented'] += n
        log(f'    {n} damage to {NAME(p)} is prevented', g)
        return
    p.life -= n
    if src is not None and src is not p:
        p.last_src = src; p.last_kind = kind
        src.stats['dmg_dealt'] += n; src.stats['dmgk_' + kind] += n
        src.hit_turn = src.turns
    elif src is None:
        p.last_src = src
    if DAMAGE_HOOK is not None: DAMAGE_HOOK(p, src, n)


def chasm(p):
    return any(L.cd.tags.get('chasm') for L in p.lands)


def prevents_damage(g, p, src):
    """Glacial Chasm prevents all damage to you; The One Ring's protection prevents damage from opponents' sources"""
    if chasm(p): return True
    return p.ring_prot and src is not None and src is not p


def shielded(p):
    """damage to p from an opponent would be prevented (attack and burn AIs look elsewhere)"""
    return p.ring_prot or chasm(p)


def gain(p, n):
    if not p.alive: return
    if CUR_G is not None and any(has(q, 'erebos') for q in CUR_G.opps(p)): return   # Erebos: opponents can't gain life
    if CUR_G is not None and DSLMOD is not None and DSLMOD.no_lifegain(CUR_G, p): return
    p.life += n


def check_state(g):
    if g.over: return
    for p in g.players:
        if p.alive and (p.life <= 0 or p.decked or (p.cmd_dmg and max(p.cmd_dmg.values()) >= 21)):
            eliminate(g, p)
    alive = [p for p in g.players if p.alive]
    if len(alive) <= 1 and not g.goldfish:
        g.over = True
        g.winner = alive[0] if alive else None
        if g.wintype is None: g.wintype = 'damage'


def eliminate(g, p):
    p.alive = False; p.killer = p.last_src
    p.death_kind = 'commander damage' if (p.cmd_dmg and max(p.cmd_dmg.values()) >= 21) else \
        ('decked' if p.decked else getattr(p, 'last_kind', 'other'))
    if p.killer is not None and p.killer is not p:
        p.killer.stats['kills_' + p.death_kind] += 1
    g.elim.append((p, g.round)); p.stats['elim_round'] = g.round
    log(f'*** {NAME(p)} is eliminated (life {p.life}) by {NAME(p.killer) if p.killer else "?"}', g)
    for m in list(p.perms):
        p.perms.remove(m)


# ---------------------------------------------------------------- mana
def land_cols(p, L, anyc):
    if anyc: return p.ident
    c = L.cd.tags.get('c', 'C')
    if c == 'A': return p.ident
    if c == 'C': return ''
    return c


def mana_units(g, p, convoke=False):
    U = []
    anyc = has(p, 'lantern') or (any(L.cd.tags.get('worldtree') for L in p.lands) and len(p.lands) >= 6)
    art = PAY_FOR is not None and 'A' in PAY_FOR.types
    for L in p.lands:
        if not L.tapped:
            if L.cd.tags.get('workshop') and not art: continue          # Mishra's Workshop: artifact spells only
            amt = int(L.cd.tags.get('amt', 1))
            if CI is not None and L.cd.name in CI.DYN_MANA: amt = CI.dyn_mana(g, p, L)
            if g.hooks: amt += CI.total(g, 'land_mana', p, L)
            U.append([L, land_cols(p, L, anyc), amt])
    rite = has(p, 'rite')
    for m in p.perms:
        if m.tapped or m.phased: continue
        if m.cd is None:
            if rite and not m.sick: U.append([m, p.ident, 1])
            continue
        t = m.cd.tags
        if 'chromemox' in t:                      # Chrome Mox: one mana of the imprinted card's colours (none if nothing imprinted)
            if m.colors: U.append([m, m.colors, 1])
            continue
        if 'rock' in t:
            a, c = t['rock'].split(':')
            U.append([m, p.ident if c == 'A' else ('' if c == 'C' else c), int(a)])
        elif 'dork' in t and not m.sick:
            c = t['dork']
            U.append([m, p.ident if c == 'A' else c, CI.dyn_mana(g, p, m) if CI is not None and m.cd.name in CI.DYN_MANA else 1])
        elif rite and m.cd.creature and not m.sick and m.noatk:
            U.append([m, p.ident, 1])
    for _ in range(p.treasures):
        U.append(['T', p.ident, 1])
    for _ in range(p.floatR):
        U.append(['F', 'R', 1])
    for _ in range(p.floatA):
        U.append(['G', p.ident, 1])
    for _ in range(p.floatU):
        U.append(['FU', 'U', 1])
    if convoke:                                   # each untapped creature pays for {1} or one coloured pip
        seen = {id(u[0]) for u in U}
        for m in p.perms:
            if m.creature and not m.tapped and not m.phased and id(m) not in seen:
                U.append([m, p.ident, 1])
    return U


def plan_pay(U, generic, pips):
    rem = [u[2] for u in U]; used = [0] * len(U)
    for c in sorted(pips, key=lambda c: sum(rem[i] for i, u in enumerate(U) if c in u[1])):
        best, bk = None, None
        for i, u in enumerate(U):
            if rem[i] > 0 and c in u[1]:
                k = (u[0] == 'T', len(u[1]))
                if bk is None or k < bk: bk, best = k, i
        if best is None: return None
        rem[best] -= 1; used[best] += 1
    need = generic
    while need > 0:
        best, bk = None, None
        for i, u in enumerate(U):
            if rem[i] <= 0: continue
            waste = 0 if used[i] else max(0, rem[i] - need)
            pain = isinstance(u[0], Land) and bool(u[0].cd.tags.get('tomb'))     # Ancient Tomb last: it deals damage
            k = (u[0] == 'T', pain, waste, len(u[1]))
            if bk is None or k < bk: bk, best = k, i
        if best is None: return None
        take = min(rem[best], need); rem[best] -= take; used[best] += take; need -= take
    return used


def can_pay(g, p, generic, pips, convoke=False):
    return plan_pay(mana_units(g, p, convoke), generic, pips) is not None


def pay(g, p, generic, pips, convoke=False):
    U = mana_units(g, p, convoke)
    used = plan_pay(U, generic, pips)
    if used is None: return False
    for i, u in enumerate(U):
        if used[i]:
            if u[0] == 'T':
                p.treasures -= 1
                if g.hooks: CI.fire(g, 'sacrifice', p, 'Treasure')
            elif u[0] == 'F': p.floatR -= 1
            elif u[0] == 'G': p.floatA -= 1
            elif u[0] == 'FU': p.floatU -= 1
            else:
                u[0].tapped = True
                if CI is not None and getattr(u[0], 'cd', None) is not None and u[0].cd.name in CI.ON_TAP: CI.ON_TAP[u[0].cd.name](g, p, u[0], used[i])
                if isinstance(u[0], Land) and u[0].cd.tags.get('tomb'):    # Ancient Tomb deals 2 damage to you
                    lose_life(g, p, 2, p, damage=True)
    return True


def total_mana(g, p, convoke=False):
    return sum(u[2] for u in mana_units(g, p, convoke))


def cost_of(p, c):
    g = CUR_G
    gen, pips = c.generic, c.pips
    t = c.tags
    if g is not None:
        if 'perCreature' in t:                    # Blasphemous Act, Vanquish the Horde
            n = sum(1 for q in g.players if q.alive for m in q.perms if m.creature and not m.phased)
            gen = max(0, gen - n)
        if 'eris' in t:                           # -2 per distinct MV among instants/sorceries in graveyard
            mvs = {x.cmc for x in p.gy if x.instant or x.sorcery}
            gen = max(0, gen - 2 * len(mvs))
        if 'dawning' in t:
            gen = max(0, gen - sum(1 for x in p.gy if x.instant or x.sorcery))
        if 'spectacle' in t and p.hit_turn == p.turns:
            gen, pips = 0, 'R'
        if DSLMOD is not None: gen = max(0, gen + DSLMOD.cost_delta(g, p, c))
        if g.hooks:
            gen = max(0, gen + CI.total(g, 'cost', p, c))
            floor = max([0] + [fn(g, src, p, c) or 0 for src, fn in CI.hooked(g, 'min_cost')])   # Trinisphere
            gen = max(gen, floor - len(pips))
    if g is not None and stopped(g, c.name): gen += 3                 # Disruptor Flute tax
    if c is p.cmd: gen += p.tax
    if id(c) in p.agent_ids:                      # Opposition Agent: spend mana as though it were mana of any type
        off = [x for x in pips if x not in p.ident]
        if off: gen += len(off); pips = ''.join(x for x in pips if x in p.ident)
    return gen, pips


PAY_FOR = None           # the card currently being paid for (Mishra's Workshop mana is for artifact spells only)


# ---------------------------------------------------------------- zones
def draw(g, p, n=1, step=False):
    for k in range(n):
        if not p.alive: return
        st = (g.round, g.active.key if getattr(g, 'active', None) else None)
        if p.draw_st != st: p.draw_st, p.draw_n = st, 0
        if p.draw_n >= 1 and any(has(q, 'narset') for q in g.opps(p)):      # Narset: opponents draw at most one card each turn
            p.stats['narset_denied'] += n - k; return
        if not p.library:
            p.decked = True; return
        p.draw_n += 1
        p.hand.append(p.library.pop())
        p.seen_names.add(p.hand[-1].name)
        p.stats['cards_drawn'] += 1
        if has(p, 'ironman'):
            a = army_of(p)
            if a is not None: a.plus += 1
        extra = not (step and k == 0)
        if DSLMOD is not None and g.dsl_on: DSLMOD.fire(g, 'draw', player=p, extra=extra)
        if g.hooks: CI.fire(g, 'draw', p)
        for q in g.opps(p):
            if has(q, 'sheoA'):
                lose_life(g, p, 2, q, kind='drain'); gain(q, 2)
            if has(q, 'tithe') and g.rng.random() < 0.5:
                q.treasures += 1
            if extra and has(q, 'bowmasters'):
                amass(g, q, 1); lose_life(g, p, 1, q, kind='triggers')
        if has(p, 'sheoA'): gain(p, 2)


KEYSPELL = ('rean', 'tut', 'yawg')


def mill(g, p, n):
    for _ in range(n):
        if not p.library: return
        c = p.library.pop(); p.gy.append(c)
        if any(k in c.tags for k in KEYSPELL):
            p.stats['key_milled'] += 1
            p.milled_keys = getattr(p, 'milled_keys', set()); p.milled_keys.add(c.name)


def amass(g, p, n):
    a = army_of(p)
    bonus = 1 if has(p, 'mauhur') else 0
    if a is None:
        a = Perm(p, None, pw=0, tg=0, name='Orc Army'); a.army = True; a.colors = 'B'
        p.perms.append(a); a.sick = True
        if has(p, 'mimic'): a.plus += 1
    a.plus += n + bonus


TOKEN_CAP = 250          # creature tokens per player; beyond this the board is lethal many times over and games crawl


TOKEN_COLOR = {'najeela': 'W', 'seph': 'B', 'sauron': 'B', 'veyran': 'R'}     # default colour of a deck's tokens


def make_tokens(g, p, n, pw, tg=None, fly=False, warrior=False, attacking=False, lifelink=False, sick=True, dt=False,
                color=None, types=None):
    out = []
    if DSLMOD is not None: n *= DSLMOD.token_mult(g, p)
    have = sum(1 for m in p.perms if m.token)
    n = min(n, max(0, TOKEN_CAP - have))          # runaway token growth (e.g. Najeela in a long game) is already lethal
    if n <= 0: return out
    blank = opp_has(g, p, 'mother')
    if has(p, 'jinnie') and pw < 2 and not attacking:     # Jinnie Fay: make 2/2 Cats with haste instead
        pw, tg, sick = 2, 2, False
    for _ in range(n):
        m = Perm(p, None, pw=pw, tg=tg, fly=fly, warrior=warrior)
        m.life = lifelink; m.sick = sick; m.dt = dt
        m.colors = TOKEN_COLOR.get(p.key, '') if color is None else color
        if types: m.ttypes = frozenset(types)
        if attacking: m.tapped = True
        p.perms.append(m)
        if etgh(g, m) <= 0:
            die(g, m, 'sba'); continue
        out.append(m)
    k = len(out)
    if k and not blank:
        if has(p, 'crusade'):
            for x in p.perms:
                if x.creature: x.plus = min(x.plus + k, 60)
        if has(p, 'warleader'):                            # Warleader's Call deals damage
            for q in g.opps(p): lose_life(g, q, k, p, kind='drain', damage=True)
        if has(p, 'wisp') and pw <= 2:                     # Wispdrinker Vampire
            for q in g.opps(p): lose_life(g, q, k, p, kind='drain')
            gain(p, k * len(g.opps(p)))
    shards_trigger(g, p, k)
    if DSLMOD is not None and g.dsl_on:
        for x in out: DSLMOD.fire(g, 'etb', perm=x, owner=p)
    return out


def shards_trigger(g, p, k):
    """Aura Shards: whenever a creature enters under your control, destroy target artifact or enchantment."""
    if k <= 0 or not has(p, 'shards') or opp_has(g, p, 'mother'): return
    reps = k * (2 if has(p, 'mother') else 1)
    for _ in range(reps):
        tg = [m for q in g.opps(p) for m in q.perms
              if m.cd is not None and not m.creature and ('A' in m.cd.types or 'E' in m.cd.types)
              and not untargetable(g, m) and not indestructible(g, m)]
        if not tg: return
        best = max(tg, key=lambda m: pval(g, m))
        if pval(g, best) < 1: return
        p.stats['shards_kill'] += 1
        apply_removal(g, p, best, 'destroy')
        if g.over: return


def creature_entered(g, p, m):
    if etgh(g, m) <= 0 and not m.army:
        die(g, m, 'sba'); return
    if not opp_has(g, p, 'mother'):
        if has(p, 'crusade'):
            for x in p.perms:
                if x.creature: x.plus = min(x.plus + 1, 60)
        if has(p, 'warleader'):                            # Warleader's Call deals damage
            for q in g.opps(p): lose_life(g, q, 1, p, kind='drain', damage=True)
        if has(p, 'wisp') and epow(g, m) <= 2 and not (m.cd is not None and 'wisp' in m.cd.tags):
            for q in g.opps(p): lose_life(g, q, 1, p, kind='drain')
            gain(p, len(g.opps(p)))
        if has(p, 'uprising') and epow(g, m) >= 4: draw(g, p, 1)
    shards_trigger(g, p, 1)


def detach(m):
    for e in m.owner.perms:
        if e.attached is m: e.attached = None


def leave(g, m):
    p = m.owner
    if m.creature and m.plus > 0 and m in p.perms and has(p, 'ozolith'):
        p.ozolith_counters = getattr(p, 'ozolith_counters', 0) + m.plus       # The Ozolith keeps the counters
    if m in p.perms: p.perms.remove(m)
    detach(m)
    if getattr(g, 'auras', None): CI.aura_fall(g, m)
    if g.hooks and m in g.hooks:
        g.hooks.remove(m)
        fn = CI.HOOKS[m.cd.name].get('leaves')
        if fn is not None: fn(g, m)


def to_zone_card(g, m, zone):
    """move a nontoken card to zone ('gy','exile','hand','lib'); commanders go to command zone."""
    if m.token: return
    owner = m.orig
    if m.phys is not None:
        cd = m.phys
        {'gy': owner.gy, 'exile': owner.exile, 'hand': owner.hand}.get(zone, owner.gy).append(cd)
        return
    if m.cd is owner.cmd:
        owner.cmd_in_zone = True; return
    if zone == 'gy': owner.gy.append(m.cd)
    elif zone == 'exile': owner.exile.append(m.cd)
    elif zone == 'hand': owner.hand.append(m.cd)
    elif zone == 'lib': owner.library.insert(g.rng.randrange(len(owner.library) + 1), m.cd)


def die(g, m, cause='destroy'):
    p = m.owner
    if m not in p.perms: return
    if cause == 'destroy' and indestructible(g, m): return
    if cause == 'destroy' and getattr(g, 'auras', None) and CI.umbra_save(g, m): return
    selfdies = CI is not None and m.cd is not None and CI.live(m.cd.name) and CI.HOOKS[m.cd.name].get('self_dies')
    leave(g, m)
    if DSLMOD is not None and g.dsl_on: DSLMOD.fire(g, 'dies', perm=m, owner=p, card=m.cd, dying=m)
    if g.hooks:
        CI.fire(g, 'dies', m, cause)
        if cause == 'sac': CI.fire(g, 'sacrifice', p, m)
    if selfdies:
        for _ in range(1 + (CI.total(g, 'trigger_copies', p, 'dies', m) if g.hooks else 0)): selfdies(g, m, cause)
    # death triggers
    for q in g.players:
        if not q.alive: continue
        for _ in range(1 + (CI.total(g, 'trigger_copies', q, 'dies', m) if g.hooks else 0)):
            _hand_tag_death(g, q, p, m)
    if m.cd is not None and m.cd.tags.get('fill') == 'stitcher': mill(g, p, 3)
    _die_rest(g, m, p, cause, selfdies)


def _hand_tag_death(g, q, p, m):
    if has(q, 'bartist'):
        opps = g.opps(q)
        if opps:
            t = max(opps, key=lambda o: threat(g, q, o)); lose_life(g, t, 1, q, kind='drain'); gain(q, 1)
    if q is p and has(q, 'drain') and m.creature:
        for o in g.opps(q): lose_life(g, o, 1, q, kind='drain')
        gain(q, 1)


def _die_rest(g, m, p, cause, selfdies):
    if m.creature:
        for q in g.opps(p):
            if has(q, 'wurmdrain'): lose_life(g, p, 2, q, kind='drain')      # Massacre Wurm
            for h in find(q, 'heir'): h.plus += 1                              # Sephiroth, Planet's Heir
    if m.cd is not None and 'wurmcoil' in m.cd.tags:
        make_tokens(g, p, 1, 3, dt=True, color=''); make_tokens(g, p, 1, 3, lifelink=True, color='')
    if m.token: return
    if m.cd.bomb and m.orig is p: p.removed_bombs.add(m.cd.name)
    # undying from Mikaeus
    if (has(p, 'mikaeus') and m.cd.creature and 'human' not in m.cd.tags and 'mikaeus' not in m.cd.tags
            and not m.undying and m.plus <= 0 and m.orig is p and m.cd is not p.cmd):
        n = enter(g, p, m.cd, undying=True); n.plus = 1; n.undying = True
        p.stats['undying'] += 1
        return
    k = m.cd.kws
    if k and m.orig is p and m.cd is not p.cmd and not (g.hooks and CI.total(g, 'no_graveyard', p)):
        if 'undying' in k and m.plus <= 0:                     # undying: back with a +1/+1 counter
            enter(g, p, m.cd, undying=True); p.stats['undying'] += 1; return
        if 'persist' in k and m.plus >= 0:                     # persist: back with a -1/-1 counter
            n = enter(g, p, m.cd); n.plus = -1; p.stats['persist'] += 1
            if etgh(g, n) <= 0: die(g, n, 'sba')
            return
    to_zone_card(g, m, 'gy')
    if cause == 'sac': tergrid_steal(g, p, m.phys or m.cd, m.orig)


def exile_perm(g, m):
    leave(g, m); to_zone_card(g, m, 'exile')


def bounce(g, m):
    leave(g, m); to_zone_card(g, m, 'hand')


def tuck(g, m):
    leave(g, m); to_zone_card(g, m, 'lib')


# ---------------------------------------------------------------- values / threat
def pval(g, m):
    p = m.owner
    if m.army:
        v = 2 + m.plus * 0.5
        if equipped(m, 'sword') and has(p, 'assault'): v += 5
        return v
    if m.token:
        return 0.3 + epow(g, m) * 0.25
    t = m.cd.tags; cd = m.cd
    v = cd.bomb
    if cd.creature and not v: v = 1 + epow(g, m) * 0.5
    if 'vkitten' in t: v = 5 + (5 if has(p, 'vfire') else 0)
    if 'vfire' in t: v = 4 + (5 if has(p, 'vkitten') else 0)
    if 'aether' in t: v = 5
    if 'ping' in t: v = max(v, 4)
    if 'veyran' in t: v = 5
    if 'dragoncaller' in t: v = 5
    if 'sauron' in t: v = 7
    if 'witchking' in t: v = 5
    if 'rhystic' in t or 'tithe' in t: v = 4
    if 'assault' in t: v = 3 + (5 if has(p, 'sword') else 0)
    if 'sword' in t: v = 3 + (5 if has(p, 'assault') else 0)
    if 'cloak' in t: v = 3
    if 'crusade' in t: v = 6
    if 'warleader' in t: v = 4
    if 'najeela' in t: v = 6
    if 'mirror' in t: v = 5
    if 'tokup' in t or 'tokatk' in t: v = max(v, 3)
    if 'normgc' in t and any(q.key == 'najeela' for q in g.opps(p)): v += 2
    if 'mikaeus' in t: v = 5
    if 'eng' in t: v = max(v, 2)
    if 'onering' in t: v = max(v, 5)
    if 'sphinx' in t: v = max(v, 6)
    if 'narset' in t: v = max(v, 4)
    if 'panoptic' in t: v = max(v, 2 + 2 * len(getattr(g, 'imprint', {}).get(id(m), [])))
    if m.is_cmd: v += 1
    if getattr(g, 'auras', None) and cd.creature: v += 1.5 * len(CI.auras_on(g, m))     # removing it takes the Auras too
    return v


def threat(g, me, q):
    s = sum(pval(g, m) for m in q.perms if not m.phased)
    if q.key == 'veyran' and has(q, 'vkitten') and has(q, 'vfire'): s += 10
    if q.key == 'sauron':
        a = army_of(q)
        if a and equipped(a, 'sword') and has(q, 'assault'): s += 10
    s += (q.life - 40) * 0.05
    return s


def board_power(g, p):
    return sum(epow(g, m) for m in p.perms if m.creature and not m.phased)


# ---------------------------------------------------------------- casting
def sac_fodder(g, p, what, exclude=None):
    """the cheapest permanent p would sacrifice for a cost ('creature', 'artifact', 'artifact or creature',
    'green creature', 'permanent'); 'Treasure' for a Treasure token; None if there is none"""
    art = 'artifact' in what or what == 'permanent'
    cre = 'creature' in what or what == 'permanent'
    cands = [m for m in p.perms if m is not exclude and not m.phased and not m.is_cmd and
             ((cre and m.creature) or (art and m.cd is not None and 'A' in m.cd.types) or what == 'permanent')]
    if 'green' in what: cands = [m for m in cands if 'G' in colors_of(m)]
    best = min(cands, key=lambda m: pval(g, m)) if cands else None
    if art and p.treasures and (best is None or pval(g, best) > 0.6): return 'Treasure'
    return best


def additional_cost(g, p, c, dry=False):
    """'As an additional cost to cast this spell, sacrifice ... / discard a card' (compiled cards).
    dry: can it be paid? Otherwise pay it. Returns False when it can't be paid."""
    for a in (c.dsl or ()):
        if a.get('type') != 'additional_cost': continue
        t = a['text']
        choice = None
        if 'sacrifice a land' in t:
            if p.lands: choice = ('land', min(p.lands, key=lambda L: (not L.tapped, L.cd.name not in ('Forest', 'Island', 'Plains', 'Swamp', 'Mountain'))))
        else:
            m = re.search(r'sacrifice (?:an? |another )((?:green )?(?:artifact or creature|creature or artifact|creature|artifact|permanent))', t)
            if m:
                fod = sac_fodder(g, p, m.group(1), exclude=None)
                if fod is not None and (fod == 'Treasure' or pval(g, fod) < 4 or 'discard' not in t): choice = ('sac', fod)
        if choice is None and 'discard a card' in t:
            others = [x for x in p.hand if x is not c]
            if others: choice = ('discard', None)
        if choice is None: return False
        if dry: continue
        kind, x = choice
        if kind == 'land':
            p.lands.remove(x); p.gy.append(x.cd)
            if g.hooks: CI.fire(g, 'sacrifice', p, x.cd)
        elif kind == 'sac':
            if x == 'Treasure':
                p.treasures -= 1
                if g.hooks: CI.fire(g, 'sacrifice', p, 'Treasure')
            else:
                c.sac_snapshot = {'mv': x.cd.cmc if x.cd is not None else 0, 'tgh': etgh(g, x)}
                die(g, x, 'sac')
        else:
            others = [x for x in p.hand if x is not c]
            lands = [x for x in others if x.land]
            x = lands[0] if len(lands) > 2 or not [y for y in others if not y.land] else \
                min([y for y in others if not y.land], key=lambda y: card_worth(g, p, y))
            discard_cards(g, p, [x])
    return True


def castable(g, p, c, zone='hand'):
    """can p cast c right now as far as locks go (Rule of Law, Drannith Magistrate, Grand Abolisher ...)"""
    return not g.hooks or CI.allowed(g, p, c, zone)


def turn_stamp(g):
    return (g.round, g.active.key if g.active is not None else None)


def casts_this_turn(g, p, pred=None):
    """spells p has cast during the current turn (anyone's turn), optionally only those matching pred"""
    log_ = getattr(p, 'turn_casts', None)
    if not log_ or log_[0] != turn_stamp(g): return 0
    return sum(1 for c in log_[1] if pred is None or pred(c))


def on_cast(g, p, c):
    st = turn_stamp(g)
    if getattr(p, 'turn_casts', None) is None or p.turn_casts[0] != st: p.turn_casts = (st, [])
    p.turn_casts[1].append(c)
    for q in g.opps(p):
        if has(q, 'sauron'): amass(g, q, 1)
        if has(q, 'rhystic') and g.rng.random() < 0.45: draw(g, q, 1)
        if has(q, 'kaervek') and c.cmc > 0: lose_life(g, p, min(c.cmc, 6), q, kind='triggers')
    if c.instant or c.sorcery: magecraft(g, p, c)
    if DSLMOD is not None and g.dsl_on: DSLMOD.fire(g, 'cast', caster=p, spell=c)
    if g.hooks: CI.fire(g, 'cast', p, c)
    if (c.instant or c.sorcery) and has(p, 'jin') and ('A' in c.types or c.instant or c.sorcery) and once_per_turn(g, p, 'jincopy'):
        magecraft(g, p, c, copy=True)                # Jin-Gitaxias copies your first spell each turn
        if 'draw' in c.tags: draw(g, p, int(c.tags['draw']))
    if has(p, 'conflict') and p.spells_this_turn == 3 and once_per_turn(g, p, 'conflict'):
        cast_copy(g, p, lambda: bolt_something(g, p, 3))     # Emeritus of Conflict: third spell -> Lightning Bolt copy
    if has(p, 'eris') and p.spells_this_turn == 2:
        n = 2 if (has(p, 'veyran') and (c.instant or c.sorcery)) else 1
        make_tokens(g, p, n, 4, fly=True)                     # Eris: second spell -> 4/4 flying Dragon
    if has(p, 'prolifall') and c.creature:
        a = army_of(p)
        if a: a.plus += len(find(p, 'prolifall'))
    if has(p, 'birgi'):
        # Birgi: add R whenever you cast a spell; an instant/sorcery trigger is doubled by Veyran
        p.floatR += 1 + (1 if (has(p, 'veyran') and (c.instant or c.sorcery)) else 0)
    if not c.creature:
        n = len(find(p, 'prolif'))
        a = army_of(p)
        if n and a: a.plus += n
    check_state(g)


def magecraft(g, p, c=None, copy=False):
    # each magecraft ability triggers once, +1 with Veyran, +1 more with Harmonic Prodigy
    # if the source is a Shaman or Wizard
    vey = 1 if has(p, 'veyran') else 0
    prod = 1 if has(p, 'prodigy') else 0
    base_mult = 1 + vey
    thor = 1 if has(p, 'thor') else 0
    opps = g.opps(p)
    if not opps: return
    for m in list(p.perms):
        if m.cd is None or m.phased: continue
        t = m.cd.tags
        # copies only trigger "cast or copy" abilities (Archmage Emeritus, Storm-Kiln Artist, Ral)
        if copy and not any(k in t for k in ('spelldraw', 'kiln', 'ral')): continue
        mult = base_mult + (prod if ('shaman' in t or 'wizard' in t) else 0)
        if 'ping' in t:
            base = int(t['ping'])
            if 'opus3' in t and c is not None and c.cmc >= 5: base = 3        # Thunderdrum Soloist, 5+ mana spell
            d = (base + thor) * mult
            if 'ral' in t:
                tgt = max(opps, key=lambda o: threat(g, p, o)); lose_life(g, tgt, d, p, kind='burn')
            else:
                for q in opps: lose_life(g, q, d, p, kind='burn')
        if 'dragoncaller' in t: make_tokens(g, p, mult, 5, fly=True, color='R')
        if 'mystic' in t: make_tokens(g, p, mult, 1, fly=True, color='U')
        if 'spelltok' in t:                          # Talrand, Young Pyromancer, Third Path Iconoclast
            make_tokens(g, p, mult, int(t['spelltok']), fly='spelltokfly' in t,
                        color='U' if 'spelltokfly' in t else ('' if 'Iconoclast' in m.cd.name else 'R'))
        if 'spelldraw' in t: draw(g, p, mult)
        if 'kiln' in t: p.treasures += mult          # Storm-Kiln Artist: a Treasure per trigger
        if 'spellloot' in t:                         # Muse Seeker: draw, then discard unless 5+ mana spent
            draw(g, p, mult)
            if not (c is not None and c.cmc >= 5): discard_worst(g, p, mult)
        if 'sanar' in t and not copy and not m.tapped:   # Sanar: tap for a Treasure once you've cast an I/S
            m.tapped = True; p.treasures += 1
    if has(p, 'aether') and not copy: gain(p, p.spells_this_turn * base_mult)


def discard_worst(g, p, n):
    for _ in range(n):
        if not p.hand: return
        lands = [x for x in p.hand if x.land]
        nonl = [x for x in p.hand if not x.land]
        if len(lands) > 2 or not nonl: x = lands[0]
        else: x = max(nonl, key=lambda c: c.cmc)
        discard_cards(g, p, [x])


def cast_copy(g, p, effect=None):
    """'You may cast a copy of ...': a real cast (magecraft, opponents' cast triggers), no card moves."""
    p.spells_this_turn += 1; p.stats['spells_cast'] += 1
    if p.key == 'veyran': magecraft(g, p)
    for q in g.opps(p):
        if has(q, 'sauron'): amass(g, q, 1)
        if has(q, 'rhystic') and g.rng.random() < 0.45: draw(g, q, 1)
    if effect: effect()
    check_state(g)


def bolt_something(g, p, dmg=3):
    """burn: kill a worthwhile creature if possible, otherwise go face on the weakest opponent"""
    tg = [m for q in g.opps(p) for m in q.perms if m.creature and not untargetable(g, m) and etgh(g, m) <= dmg]
    best = max(tg, key=lambda m: pval(g, m)) if tg else None
    opps = g.opps(p)
    if best is not None and pval(g, best) >= 3:
        apply_removal(g, p, best, f'dmg{dmg}')
    elif opps:
        lose_life(g, min(opps, key=lambda o: o.life), dmg, p, kind='burn')


def spell_imp(g, p, c, ctx):
    """returns (general importance, per-player importance dict)"""
    t = c.tags; aff = {}
    if 'target' in ctx and ctx['target'] is not None:
        tg = ctx['target']; aff[tg.owner] = pval(g, tg); return 0, aff
    if 'wipe' in t:
        for q in g.opps(p):
            aff[q] = 0.8 * sum(pval(g, m) for m in q.perms if m.creature or t['wipe'] in ('rift', 'rebuke'))
        return 0, aff
    if 'rean_target' in ctx: return ctx['rean_value'], aff
    if c is p.cmd: return {'seph': 8, 'veyran': 5, 'sauron': 6, 'najeela': 6}.get(p.key, CMD_IMP), aff
    if c.bomb and p.key == 'seph': return c.bomb, aff
    if 'vkitten' in t: return (9 if has(p, 'vfire') else 4), aff
    if 'vfire' in t: return (9 if has(p, 'vkitten') else 4), aff
    if 'sword' in t: return (9 if has(p, 'assault') else 4), aff
    if 'assault' in t: return (9 if has(p, 'sword') else 4), aff
    if 'skate' in t:
        a = army_of(p); return (7 if a and a.plus >= 6 else 3), aff
    for k, v in (('aether', 6), ('crusade', 6), ('rhystic', 5), ('tithe', 5), ('mirror', 5),
                 ('witchking', 5), ('dragoncaller', 5), ('normgc', 7), ('mother', 7), ('onering', 6),
                 ('sphinx', 6), ('breach', 5), ('panoptic', 5)):
        if k in t: return v, aff
    if 'tokx' in t and ctx.get('x', 0) >= 4: return 5, aff
    return 0, aff


CTHRESH = {'seph': 6, 'veyran': 7, 'sauron': 7, 'najeela': 99}
CMD_IMP = 6              # importance of an outside deck's commander spell (counter decisions)

# Interaction profiles for the AI opponents.
#   conservative: counter only big threats (importance >= 7), hold instant removal for emergencies
#   loose:        counter at importance >= 6, use instant removal as freely as sorcery removal
PROFILES = {
    'conservative': {'cthresh': {'seph': 6, 'veyran': 7, 'sauron': 7, 'najeela': 99}, 'instant_extra': 2, 'default': 7},
    'loose':        {'cthresh': {'seph': 6, 'veyran': 6, 'sauron': 6, 'najeela': 99}, 'instant_extra': 0, 'default': 6},
}
INSTANT_EXTRA = 2
CTHRESH_DEFAULT = 7      # outside decks: counter threshold under the current profile


def set_profile(name):
    global INSTANT_EXTRA, CTHRESH_DEFAULT
    prof = PROFILES[name]
    CTHRESH_DEFAULT = prof['default']
    CTHRESH.clear(); CTHRESH.update(prof['cthresh'])
    INSTANT_EXTRA = prof['instant_extra']


def counter_ok(ctr, c):
    s = ctr.tags.get('ctr')
    if s == 'any': return True
    if s == 'nc': return not c.creature
    if s == 'ise': return c.instant or c.sorcery or ('E' in c.types and not c.creature)
    if s == 'mv4': return c.cmc >= 4
    if s == 'cre': return c.creature
    return False


def pick_counter(g, q, c):
    best = None
    for ctr in q.hand:
        if 'ctr' not in ctr.tags or not counter_ok(ctr, c): continue
        if g.hooks and not castable(g, q, ctr): continue
        if 'fierce' in ctr.tags and commander_out(q): return ctr        # Fierce Guardianship: free with your commander out
        if 'free' in ctr.tags:
            if any(x is not ctr and 'U' in x.pips for x in q.hand) or can_pay(g, q, ctr.generic, ctr.pips):
                if best is None: best = ctr
            continue
        if can_pay(g, q, ctr.generic, ctr.pips):
            if best is None or ctr.cmc < best.cmc or 'free' in best.tags: best = ctr
    return best


def commander_out(p):
    return any(m.is_cmd and not m.phased for m in p.perms)


def cast_counter(g, q, ctr):
    if 'fierce' in ctr.tags and commander_out(q):
        pass                                     # cast without paying its mana cost
    elif 'free' in ctr.tags and not can_pay(g, q, ctr.generic, ctr.pips):
        blues = [x for x in q.hand if x is not ctr and 'U' in x.pips]
        if not blues: return False
        q.hand.remove(blues[0]); q.exile.append(blues[0]); lose_life(g, q, 1, q)
    elif not pay(g, q, ctr.generic, ctr.pips):
        return False
    q.hand.remove(ctr); q.gy.append(ctr)
    q.spells_this_turn += 1
    on_cast(g, q, ctr)
    q.stats['counters_cast'] += 1
    q.cast_names.add(ctr.name)
    return True


LAST_COUNTER = None


def counter_window(g, p, c, imp, aff):
    global LAST_COUNTER
    if 'unc' in c.tags: return True
    for q in g.after(p):
        if not q.alive or g.over or silenced(g, q): continue
        val = aff.get(q, imp)
        if AI_MODE == 'adaptive':
            import brain
            nc = sum(1 for x in q.hand if 'ctr' in x.tags)
            if q.key == 'veyran' and has(q, 'veyran'): val += 1.5   # every counter is also a doubled magecraft trigger
            if not nc or g.rng.random() > brain.wants_counter(g, q, val, CTHRESH.get(q.key, CTHRESH_DEFAULT), nc): continue
        else:
            if val < CTHRESH.get(q.key, CTHRESH_DEFAULT): continue
            if g.rng.random() > 0.9: continue
        ctr = pick_counter(g, q, c)
        if ctr is None: continue
        if not cast_counter(g, q, ctr): continue
        log(f'    {NAME(q)} counters {c.name} with {ctr.name}', g)
        soft = int(ctr.tags.get('soft', 0))
        if soft and can_pay(g, p, soft, ''):           # Spell Pierce / Mystic Confluence: pay and it resolves
            pay(g, p, soft, ''); log(f'    {NAME(p)} pays {soft}', g); continue
        counter_side_effects(g, q, p, ctr)
        # original caster may fight back
        if max(imp, 7) >= 7 and imp >= 6:
            back = pick_counter(g, p, ctr)
            if back is not None and cast_counter(g, p, back):
                p.stats['counterwar_won'] += 1
                log(f'    {NAME(p)} counters back with {back.name}', g)
                continue
        if p.key == 'seph': p.stats['seph_spell_countered'] += 1
        p.stats['spells_countered'] += 1
        LAST_COUNTER = ctr
        return False
    return True


def counter_side_effects(g, q, p, ctr):
    """q countered p's spell with ctr"""
    t = ctr.tags
    if 'offer' in t: p.treasures += 2                          # An Offer You Can't Refuse
    if 'denial' in t: draw(g, p, 2); draw(g, q, 1)             # Arcane Denial
    if 'undermine' in t: lose_life(g, p, 3, q, kind='triggers', damage=False)   # life loss, not damage
    if 'swan' in t: make_tokens(g, p, 1, 2, fly=True)          # Swan Song gives the caster a Bird


def cast_card(g, p, c, zone='hand', ctx=None, paid=True):
    """card already paid for.  zone: hand/gy/cmd"""
    ctx = ctx or {}
    if zone == 'hand': p.hand.remove(c)
    elif zone == 'gy': p.gy.remove(c)
    elif zone == 'cmd': p.cmd_in_zone = False; p.tax += 2
    elif zone == 'escape': p.gy.remove(c)
    elif zone == 'lib': pass                      # Bolas's Citadel: already taken off the top of the library       # Underworld Breach: cast from the graveyard, resolves back to it
    p.spells_this_turn += 1; p.stats['spells_cast'] += 1
    p.cast_names.add(c.name)
    tgt = ctx.get('target')
    log(f'  {NAME(p)} casts {c.name}' + (f' -> {tgt.name} ({NAME(tgt.owner)})' if tgt is not None else ''), g)
    on_cast(g, p, c)
    if g.over or not p.alive: return False
    imp, aff = spell_imp(g, p, c, ctx)
    jin = [q for q in g.opps(p) if has(q, 'jin') and ('A' in c.types or c.instant or c.sorcery)]
    if jin and once_per_turn(g, jin[0], 'jincounter'):
        log(f'    Jin-Gitaxias counters {c.name}', g)
        (p.exile if zone == 'gy' else p.gy).append(c)
        return False
    global LAST_COUNTER
    LAST_COUNTER = None
    if (imp > 0 or aff) and not counter_window(g, p, c, imp, aff):
        if c is p.cmd: p.cmd_in_zone = True
        elif zone in ('gy',) or ctx.get('exile_after'): p.exile.append(c)
        elif LAST_COUNTER is not None and 'lapse' in LAST_COUNTER.tags and not c.land: p.library.append(c)
        elif not c.land: p.gy.append(c)
        return False
    resolve(g, p, c, ctx, zone)
    check_state(g)
    return True


def flashback_grant(g, p):
    """Flashback (the card): recast the best affordable instant/sorcery from your graveyard"""
    cs = [x for x in p.gy if (x.instant or x.sorcery) and 'ctr' not in x.tags and 'fbgrant' not in x.tags
          and 'rem' not in x.tags and 'wipe' not in x.tags]
    cs = [x for x in cs if can_pay(g, p, x.generic, x.pips)]
    if not cs: return
    x = max(cs, key=lambda c: (int(c.tags.get('draw', 0)), c.cmc))
    pay(g, p, x.generic, x.pips); cast_card(g, p, x, 'gy')


def resolve(g, p, c, ctx, zone):
    t = c.tags
    if c.dsl and not c.perm:                     # interpreter-driven instant / sorcery
        if DSLMOD is not None: DSLMOD.resolve_spell(g, p, c, ctx)
        if zone == 'gy' or ctx.get('exile_after'): p.exile.append(c)
        else: p.gy.append(c)
        return
    if c.perm:
        if 'moxd' in t:                          # Mox Diamond: discard a land card instead, or it goes to the graveyard
            lands = [x for x in p.hand if x.land]
            if not lands:
                p.gy.append(c); log('    Mox Diamond goes to the graveyard (no land to discard)', g); return
            x = min(lands, key=lambda L: (len(L.tags.get('c', '')), -('t' in L.tags))); p.hand.remove(x); p.gy.append(x)
        m = enter(g, p, c, was_cast=True)
        if c is p.cmd: m.is_cmd = True
        if 'lr' in t: land_ramp(g, p, int(t['lr']), 'lrt' in t)
        return
    if 'draw' in t: draw(g, p, int(t['fbdraw']) if (zone == 'gy' and 'fbdraw' in t) else int(t['draw']))
    if 'treas' in t: p.treasures += int(t['treas'])
    if 'lr' in t:
        land_ramp(g, p, int(t['lr']), 'lrt' in t)
        if 'lh' in t: land_to_hand(g, p)
    if 'tut' in t: tutor(g, p, t['tut'])
    if 'gifts' in t: pile_tutor(g, p, 4, 2)     # Gifts Ungiven: four cards, opponent puts two in the graveyard
    if 'intuition' in t: pile_tutor(g, p, 3, 1) # Intuition: three cards, opponent picks the one you keep
    if 'jeska' in t: jeskas_will(g, p)
    if 'explore' in t: p.extra_land_now = getattr(p, 'extra_land_now', 0) + 1
    if 'loam' in t:
        for x in sorted([x for x in p.gy if x.land], key=lambda x: -len(x.tags.get('c', '')))[:3]: p.gy.remove(x); p.hand.append(x)
    if 'rishkar' in t:
        cr = [m for m in p.perms if m.creature and not m.phased]
        draw(g, p, max((epow(g, m) for m in cr), default=0))
        free = [x for x in p.hand if not x.land and x.cmc <= 5 and not any(k in x.tags for k in ('ctr', 'rem', 'wipe'))]
        if free:
            x = max(free, key=lambda x: (x.cmc, card_worth(g, p, x))); cast_card(g, p, x, 'hand', {})
    if 'threedreams' in t and CI is not None:
        import impl_t1; impl_t1.tutor_named(g, p, lambda x: 'aura' in x.subtypes, k=3)
    if 'seal' in t: tutor_to_top(g, p)          # Imperial Seal / Vampiric Tutor: card on top, lose 2 life
    if 'adnaus' in t: ad_nauseam(g, p)
    if 'lose' in t: lose_life(g, p, int(t['lose']), p)
    if 'selfdmg' in t: lose_life(g, p, int(t['selfdmg']), p)
    if 'discard1' in t: discard_worst(g, p, 1)
    if ctx.get('face') is not None:
        lose_life(g, ctx['face'], int(t['rem'][3:]) + (1 if has(p, 'thor') else 0), p, kind='burn')
    elif 'rem' in t and ctx.get('target') is not None: apply_removal(g, p, ctx['target'], t['rem'], c)
    if 'wipe' in t and not ctx.get('target') and not ctx.get('face'):
        ctx = dict(ctx); ctx['tags'] = t
        apply_wipe(g, p, t['wipe'], ctx)
    if 'fill' in t: import ais; ais.seph_fill_resolve(g, p, t['fill'], ctx)
    if 'rean' in t: import ais; ais.seph_rean_resolve(g, p, c, ctx)
    if 'burn' in t:
        opps = g.opps(p)
        if opps: lose_life(g, min(opps, key=lambda o: o.life), 5 + total_mana(g, p), p, kind='burn')
    if 'tokx' in t: make_tokens(g, p, ctx.get('x', 0), 1, warrior='warrior' in t, sick=False, lifelink='toklife' in t)
    if 'clue' in t: p.clues += 1
    if 'mktok' in t:                            # auto-tagged token spells  n:power:flying
        n_, pw_, fl_ = (t['mktok'].split(':') + ['1', '1', '0'])[:3]
        make_tokens(g, p, int(n_), int(pw_), fly=fl_ == '1', sick=False)
    if 'drainetb' in t:
        for q in g.opps(p): lose_life(g, q, int(t['drainetb']), p, kind='drain')
    if 'edictetb' in t:
        for q in g.opps(p): edict(g, q)
    if 'pumpall' in t: p.pumpadd += int(t['pumpall'])
    if 'lh' in t and 'lr' not in t: land_to_hand(g, p)
    if 'crackle' in t:                          # X = (mana spent - 2)/3; 5X to each of X targets
        x = max(1, ctx.get('x', 1))
        for q in sorted(g.opps(p), key=lambda o: o.life)[:x]: lose_life(g, q, 5 * x, p, kind='burn')
    if 'drawcre' in t:                          # Shamanic Revelation
        cr = [m for m in p.perms if m.creature and not m.phased]
        draw(g, p, len(cr)); gain(p, 4 * sum(1 for m in cr if epow(g, m) >= 4))
    if 'stampede' in t:                         # Overwhelming Stampede: +X/+X, X = greatest power
        cr = [m for m in p.perms if m.creature]
        if cr: p.pumpadd += max(epow(g, m) for m in cr); p.trample = True
    if 'prolif1' in t:
        a = army_of(p)
        if a: a.plus += 1
    if 'unearth' in t:
        cs = [x for x in p.gy if x.creature and x.cmc <= 3]
        if cs:
            x = max(cs, key=lambda c: (('bowmasters' in c.tags) * 5 + c.pow)); p.gy.remove(x); enter(g, p, x)
    if 'fbgrant' in t: flashback_grant(g, p)
    if zone == 'gy' and 'fbnib' in t:           # Nibelheim Aflame from graveyard: discard hand, draw four
        discard_cards(g, p, list(p.hand)); draw(g, p, 4)
    if 'yawg' in t: p.yawg = True
    if 'mastery' in t and ctx.get('overload'):
        # Mizzix's Mastery overloaded: cast a copy of every instant/sorcery in the graveyard
        copies = [x for x in p.gy if (x.instant or x.sorcery) and x is not c and 'ctr' not in x.tags]
        log(f'    Mizzix\'s Mastery recasts {len(copies)} spells', g)
        for x in copies:
            p.spells_this_turn += 1; p.stats['spells_cast'] += 1
            on_cast(g, p, x)
            if 'draw' in x.tags: draw(g, p, int(x.tags['draw']))
            if 'burn' in x.tags:
                opps = g.opps(p)
                if opps: lose_life(g, min(opps, key=lambda o: o.life), 5, p, kind='burn')
            if g.over: return
        for x in copies: p.gy.remove(x); p.exile.append(x)
    if zone == 'gy' or ctx.get('exile_after'): p.exile.append(c)
    else: p.gy.append(c)


def enter(g, p, cd, orig=None, sick=True, was_cast=False, undying=False):
    phys = None
    if 'clone' in cd.tags:                      # Phyrexian Metamorph: copy the best creature or artifact on the battlefield
        cands = [x for q in g.players if q.alive for x in q.perms if x.cd is not None and x.cd is not q.cmd
                 and (x.creature or 'A' in x.cd.types) and 'clone' not in x.cd.tags]
        if cands:
            phys, cd = cd, max(cands, key=lambda x: pval(g, x)).cd
    m = Perm(p, cd); m.orig = orig or p; m.sick = sick; m.phys = phys
    if 'haste' in cd.tags: m.sick = False
    if cd.dsl: g.dsl_on = True
    if cd.start_loyalty: m.loyalty = int(cd.start_loyalty)
    if undying: m.plus = 1; m.undying = True       # returns with its +1/+1 counter (so it survives -X/-X effects)
    p.perms.append(m)
    if CI is not None and CI.live(cd.name): g.hooks.append(m)
    if cd.creature: creature_entered(g, p, m)
    if m in p.perms: do_etb(g, p, m)
    if DSLMOD is not None and g.dsl_on and m in p.perms: DSLMOD.fire(g, 'etb', perm=m, owner=p, was_cast=was_cast)
    if g.hooks and m in p.perms:
        g.last_cast_etb = was_cast
        CI.fire(g, 'etb', p, m)
        g.last_cast_etb = False
    return m


def enter_token_copy(g, p, cd):
    """a token that's a copy of card cd (Scute Swarm, Kiki-Jiki, Helm of the Host ...)"""
    have = sum(1 for m in p.perms if m.token)
    if have >= TOKEN_CAP: return None
    m = enter(g, p, cd)
    m.token = True
    return m


def do_etb(g, p, m):
    if m.cd.dsl: return                         # interpreter handles this card's abilities
    if opp_has(g, p, 'mother'): return
    reps = 2 if (has(p, 'mother') and m.cd.creature) else 1
    for _ in range(reps):
        etb_once(g, p, m)
        if g.over: return


def etb_once(g, p, m):
    t = m.cd.tags
    if 'chromemox' in t: chrome_imprint(g, p, m)
    if 'flute' in t:                             # Disruptor Flute: choose a card name
        import ais
        name, score = ais.flute_pick(g, p)
        if name:
            g.flutes.append((m, name)); p.stats['flute_named'] += 1
            log(f'    Disruptor Flute names {name}', g)
    if 'prepare' in t and p.key == 'veyran':
        # prepared: cast a copy of the back-face instant/sorcery -> a real spell cast
        if 'sanar' in t: cast_copy(g, p, lambda: tutor(g, p, 'is'))       # Wild Idea: find an instant/sorcery
        else: cast_copy(g, p)
    if 'recruit' in t: tutor(g, p, 'cre2')     # Imperial Recruiter: creature with power 2 or less
    opps = g.opps(p)
    if 'titan' in t: make_tokens(g, p, 2, 2, color='B')
    if 'archon' in t and opps: archon_trig(g, p)
    if 'gray' in t:
        dev = 2 + sum(x.cd.pips.count('B') for x in p.perms if x.cd and x.cd.perm and x is not m)
        dev = min(dev, 10)
        for q in opps: lose_life(g, q, dev, p, kind='drain')
        gain(p, dev * len(opps))
    if 'wurm' in t:
        for q in opps:
            for x in list(q.perms):
                if x.creature and etgh(g, x) <= 2:
                    die(g, x, 'destroy'); lose_life(g, q, 2, p, kind='drain')
    if 'rsd' in t: tutor(g, p, 'any')
    if 'witness' in t: import ais; ais.regrow(g, p, False)
    if 'wall' in t: import ais; ais.regrow(g, p, True)
    if 'atraxa' in t: draw(g, p, 4)
    if 'draw' in t and not ('I' in m.cd.types or 'S' in m.cd.types): draw(g, p, int(t['draw']))
    if 'skate' in t:
        for x in p.perms:
            if x.plus > 0: x.plus = min(x.plus * 2, 200)
    if 'bowmasters' in t and opps:
        x1 = [x for q in opps for x in q.perms if x.creature and etgh(g, x) <= 1 and pval(g, x) >= 2 and not untargetable(g, x)]
        if x1: die(g, max(x1, key=lambda x: pval(g, x)), 'destroy')
        else:
            q = max(opps, key=lambda o: threat(g, p, o)); lose_life(g, q, 1, p, kind='triggers')
        amass(g, p, 1)
    if 'rem' in t and 'etb' in t: etb_removal(g, p, m)
    if 'heir' in t:                               # Sephiroth, Planet's Heir: opponents' creatures get -2/-2
        for q in opps:
            for x in list(q.perms):
                if x.creature and etgh(g, x) <= 2: die(g, x, 'destroy')
    if 'suntitan' in t: sun_titan(g, p)
    if 'mycoloth' in t:                           # devour 2: eat up to three tokens
        toks = [x for x in p.perms if x.token and x.creature][:3]
        for x in toks: die(g, x, 'sac')
        m.plus += 2 * len(toks)
    if 'endraze' in t: p.pumpadd += 2; p.trample = True
    if 'spider' in t:                             # Old Fat Spider: hexproof on your best creature (like Boots)
        cr = [x for x in p.perms if x.creature and x.cd is not None]
        if cr: m.attached = max(cr, key=lambda x: (('veyran' in x.cd.tags) * 10 + pval(g, x)))
    if 'dualcaster' in t:                         # copy a spell: approximated as copying your last instant/sorcery
        last = next((x for x in reversed(p.gy) if x.instant or x.sorcery), None)
        if last is not None:
            if p.key == 'veyran': magecraft(g, p, last, copy=True)
            if 'draw' in last.tags: draw(g, p, int(last.tags['draw']))
    if 'uprising' in t and any(x.creature and epow(g, x) >= 4 for x in p.perms): draw(g, p, 1)
    if 'lidless' in t and opps:                   # threaten: steal their best creature to attack with this turn
        pass
    if 'tok' in t: make_tokens(g, p, int(t['tok']), int(t.get('tokp', 1)), fly='tokfly' in t,
                               lifelink='tokdt' not in t and 'tokp' not in t, dt='tokdt' in t,
                               color='G' if 'tokdt' in t else None)
    if m.cd.source == 'scryfall':               # generic ETB effects from auto-tagged cards
        if 'tut' in t: tutor(g, p, t['tut'])
        if 'treas' in t: p.treasures += int(t['treas'])
        if 'drainetb' in t:
            for q in opps: lose_life(g, q, int(t['drainetb']), p, kind='drain')
        if 'edictetb' in t:
            for q in opps: edict(g, q)
    if 'tokbig' in t:
        for sz in (2, 3, 4): make_tokens(g, p, 1, sz)      # Trostani's Summoner: 2/2, 3/3, 4/4
    f = t.get('fill')
    if f == 'stitcher': mill(g, p, 3)
    if f == 'wayfinder':
        top = [p.library.pop() for _ in range(min(4, len(p.library)))]
        lands = [c for c in top if c.land]
        if lands: top.remove(lands[0]); p.hand.append(lands[0])
        for c in top:
            p.gy.append(c)
            if any(k in c.tags for k in KEYSPELL): p.stats['key_milled'] += 1


def archon_trig(g, p):
    opps = g.opps(p)
    q = max(opps, key=lambda o: threat(g, p, o))
    edict(g, q)
    if q.hand: discard_index(g, q, g.rng.randrange(len(q.hand)))
    lose_life(g, q, 3, p, kind='drain'); gain(p, 3); draw(g, p, 1)


def edict(g, q):
    cr = [m for m in q.perms if m.creature and not m.phased]
    if cr: die(g, min(cr, key=lambda x: pval(g, x)), 'sac')


def land_ramp(g, p, n, tapped):
    for _ in range(n):
        basics = [c for c in p.library if c.land and c.name in ('Forest', 'Island', 'Plains', 'Swamp', 'Mountain')]
        if not basics: return
        c = g.rng.choice(basics); p.library.remove(c)
        a = agent_for(g, p)
        if a is not None: agent_take(g, a, p, c); continue
        p.lands.append(Land(c, tapped))
        landfall(g, p)


def landfall(g, p):
    copies = 1 + (CI.total(g, 'trigger_copies', p, 'landfall', None) if g.hooks else 0)     # Ancient Greenwarden
    for _ in range(copies):
        _landfall_once(g, p)


def _landfall_once(g, p):
    for _ in find(p, 'landfall2'): make_tokens(g, p, 1, 2)      # Felidar Retreat: 2/2 Cat per land
    fields = [L for L in p.lands if L.cd.tags.get('fotd')]
    if fields and len({L.cd.name for L in p.lands}) >= 7:         # Field of the Dead: 7+ lands with different names
        for _ in fields: make_tokens(g, p, 1, 2, color='B')
    if DSLMOD is not None and g.dsl_on: DSLMOD.fire(g, 'landfall', player=p)
    if g.hooks: CI.fire(g, 'landfall', p)


def land_to_hand(g, p):
    basics = [c for c in p.library if c.land and c.name in ('Forest', 'Island', 'Plains', 'Swamp', 'Mountain')]
    if basics:
        c = g.rng.choice(basics); p.library.remove(c)
        a = agent_for(g, p)
        if a is not None: agent_take(g, a, p, c)
        else: p.hand.append(c)


def tutor(g, p, kind):
    import ais
    name = ais.tutor_pick(g, p, kind)
    if name is None: return
    for c in p.library:
        if c.name == name:
            a = agent_for(g, p)
            if a is not None:
                p.library.remove(c); agent_take(g, a, p, c); g.rng.shuffle(p.library); return
            p.library.remove(c); p.hand.append(c); p.stats['tutored'] += 1
            p.seen_names.add(c.name)
            log(f'    {NAME(p)} tutors {c.name}')
            g.rng.shuffle(p.library); return


# ---------------------------------------------------------------- removal
def legal_targets(g, p, kind, tgt, mv4=False, spell=None):
    res = []
    for q in g.opps(p):
        for m in q.perms:
            if untargetable(g, m): continue
            is_c = m.creature
            cd = m.cd
            ty = cd.types if cd else 'C'
            ok = {'c': is_c, 'cp': is_c or 'P' in ty, 'cap': is_c or 'A' in ty or 'P' in ty,
                  'ce': is_c or 'E' in ty, 'nl': True, 'p': True, 'a': 'A' in ty,
                  'cna': is_c and 'A' not in ty, 'ae': 'A' in ty or 'E' in ty}.get(tgt, is_c)
            if spell is not None and 'alsoart' in spell.tags and 'A' in ty and not is_c:
                res.append(m); continue                  # Abrade: destroy target artifact mode
            if not ok: continue
            if mv4 and cd is not None and cd.cmc > 4: continue
            if cd is not None and cd.ward and spell is not None and not can_pay(g, p, spell.generic + cd.ward, spell.pips): continue
            if spell is not None and protected_from(g, m, spell.pips): continue
            if kind.startswith('dmg'):
                if not is_c or etgh(g, m) > int(kind[3:]): continue
            if (kind == 'destroy' or kind.startswith('dmg')) and indestructible(g, m): continue
            res.append(m)
    return res


def apply_removal(g, actor, m, kind, spell=None):
    owner = m.owner
    if m not in owner.perms or untargetable(g, m): return
    if (kind == 'destroy' or kind.startswith('dmg')) and indestructible(g, m):
        log(f'    {m.name} ({NAME(owner)}) is indestructible', g); return
    import ais
    if ais.protect_response(g, owner, m, kind, actor, spell):
        log(f'    {NAME(owner)} protects {m.name}', g); return
    if m not in owner.perms: return
    if m.cd is not None and m.cd.ward and actor is not owner and spell is not None:     # ward {N}: pay or it's countered
        if not can_pay(g, actor, m.cd.ward, ''):
            log(f'    ward counters the removal on {m.name}', g); return
        pay(g, actor, m.cd.ward, '')
    log(f'    {m.name} ({NAME(owner)}) is removed: {kind}', g)
    owner.lost_names[m.name] += 1
    owner.stats['threats_lost'] += 1 if pval(g, m) >= 5 else 0
    if owner.key == 'seph' and m.cd is not None and m.cd.bomb:
        owner.stats['bomb_removed'] += 1; owner.stats['bomb_removed_' + kind[:5]] += 1
    power, mv = epow(g, m), (m.cd.cmc if m.cd is not None else 0)
    if kind in ('destroy',) or kind.startswith('dmg'): die(g, m, 'destroy')
    elif kind == 'exile': exile_perm(g, m)
    elif kind == 'bounce': bounce(g, m)
    elif kind == 'tuck': tuck(g, m)
    if spell is not None:
        t = spell.tags
        if 'rgain' in t: gain(owner, power)                      # Swords to Plowshares
        if 'rland' in t: land_ramp(g, owner, 1, True)            # Path to Exile
        if 'rtok' in t: make_tokens(g, owner, 1, int(t['rtok']), color='' if 'Reality' in spell.name else 'G')
        if 'losemv' in t: lose_life(g, actor, mv, actor)         # Feed the Swarm
        if 'gaintgh' in t: gain(actor, m.tgh)                    # Noxious Gearhulk
    check_state(g)


def sun_titan(g, p):
    """return the best permanent card with mana value 3 or less from graveyard to the battlefield"""
    cs = [x for x in p.gy if (x.perm or x.land) and x.cmc <= 3]
    if not cs: return
    x = max(cs, key=lambda c: (c.creature, c.pow, c.cmc))
    p.gy.remove(x)
    if x.land: p.lands.append(Land(x, True))
    else: enter(g, p, x)


def etb_removal(g, p, m):
    t = m.cd.tags
    tg = legal_targets(g, p, t['rem'], t.get('tgt', 'c'), 'mv4' in t, spell=m.cd)
    if not tg: return
    best = max(tg, key=lambda x: pval(g, x))
    if pval(g, best) < 2: return
    # Tidebinder can counter the trigger
    import ais
    if ais.tide_response(g, p, 'etbrem', pval(g, best), victim=best.owner): return
    apply_removal(g, p, best, t['rem'], m.cd)


def apply_wipe(g, p, kind, ctx):
    import ais
    log(f'    board wipe resolves ({kind})', g)
    if kind == 'rift':
        victims = g.opps(p)
    elif kind == 'rebuke':
        victims = [ctx['victim']] if ctx.get('victim') and ctx['victim'].alive else []
    else:
        victims = [q for q in g.players if q.alive]
    if kind == 'minus' and 'deluge' in ctx.get('tags', {}):
        xs = [etgh(g, m) for q in g.opps(p) for m in q.perms if m.creature]
        lose_life(g, p, min(max(xs) if xs else 1, 10), p)          # Toxic Deluge: pay X life
    modes = set()
    if kind in ('farewell', 'austere2'):
        modes = ais.wipe_modes(g, p, kind)
        log(f'      modes: {sorted(modes)}', g)
    if kind == 'vandal': victims = g.opps(p)
    for q in victims:
        if g.over: return
        prot = ais.wipe_response(g, q, kind, p)
        if prot == 'all': continue
        biggest = None
        if kind == 'nib':
            cr = [m for m in p.perms if m.creature]
            biggest = max(cr, key=lambda x: epow(g, x)) if cr else None
            x_dmg = epow(g, biggest) if biggest else 0
        for m in list(q.perms):
            if m.phased or (prot == 'indes' and kind in ('destroy', 'dmg13', 'austere', 'nib', 'austere2', 'vandal')): continue
            ty = m.cd.types if m.cd is not None else 'C'
            if kind == 'vandal':
                if 'A' in ty: die(g, m, 'destroy')
                continue
            if kind in ('farewell', 'austere2'):
                hit = (('art' in modes and 'A' in ty) or ('ench' in modes and 'E' in ty) or
                       ('cre' in modes and m.creature) or
                       ('le3' in modes and m.creature and (m.cd is None or m.cd.cmc <= 3)) or
                       ('ge4' in modes and m.creature and m.cd is not None and m.cd.cmc >= 4))
                if hit:
                    if kind == 'farewell': exile_perm(g, m)
                    else: die(g, m, 'destroy')
                continue
            if kind in ('rift', 'rebuke'):
                if m.token: leave(g, m)
                else: bounce(g, m)
                continue
            if not m.creature: continue
            if kind == 'evac':                   # Evacuation: all creatures to owners' hands
                if m.token: leave(g, m)
                else: bounce(g, m)
                continue
            if kind == 'destroy' or kind == 'minus': die(g, m, 'destroy')
            elif kind == 'exile': exile_perm(g, m)
            elif kind == 'dmg13':
                if etgh(g, m) <= 13: die(g, m, 'destroy')
            elif kind == 'austere':
                if m.cd is not None and m.cd.cmc >= 4: die(g, m, 'destroy')
            elif kind == 'nib':
                if m is not biggest and etgh(g, m) <= x_dmg: die(g, m, 'destroy')
    check_state(g)

# ---------------------------------------------------------------- Game Changer mechanics
def card_worth(g, p, c, in_gy=False):
    """how much p's AI wants card c in hand (or, in_gy, in the graveyard: flashback cards keep most of their value)"""
    import ais
    v = ais.deck_prio(g, p, c)
    if v <= 0 and c.dsl and DSLMOD is not None: v = DSLMOD.card_value(g, p, c) * 10
    if v <= 0:                                   # held interaction has no cast priority but is worth keeping
        t = c.tags
        v = (60 if t.get('wipe') == 'rift' else 55 if 'ctr' in t else 50 if ('mastery' in t or 'crackle' in t)
             else 45 if ('rem' in t or 'wipe' in t) else 40 if (t.get('prot') or 'x' in t) else 0)
    if c.land: v = 25 if len(p.lands) + sum(1 for x in p.hand if x.land) < 7 else 5
    if in_gy: return v * 0.8 if ('fb' in c.tags or 'fbnib' in c.tags) else 0.0
    return v


def pile_tutor(g, p, n, keep):
    """search for n cards with different names; an opponent chooses which `keep` of them go to your hand, the rest
    go to your graveyard (Gifts Ungiven, Intuition). You pick the pile that is best against their choice."""
    from itertools import combinations
    opp = max(g.opps(p), key=lambda q: threat(g, p, q)) if g.opps(p) else None
    uniq = {}
    for c in p.library: uniq.setdefault(c.name, c)
    cs = list(uniq.values())
    if not cs: return
    hv = {c.name: card_worth(g, p, c) for c in cs}; gv = {c.name: card_worth(g, p, c, True) for c in cs}
    pool = sorted(cs, key=lambda c: -hv[c.name])[:8] + sorted(cs, key=lambda c: -gv[c.name])[:3]
    pool = list({c.name: c for c in pool}.values())
    k = min(n, len(pool))

    def result(pile):              # the opponent's choice: the split that leaves you the least value
        best = None
        for hand in combinations(pile, min(keep, len(pile))):
            v = sum(hv[c.name] for c in hand) + sum(gv[c.name] for c in pile if c not in hand)
            if best is None or v < best[0]: best = (v, hand)
        return best
    best = None
    for pl in combinations(pool, k):
        r = result(pl)
        if best is None or r[0] > best[0]: best = (r[0], pl, r[1])
    _, pile, hand = best
    a = agent_for(g, p)
    if a is not None:
        for c in pile: p.library.remove(c); agent_take(g, a, p, c)
        g.rng.shuffle(p.library); return
    for c in pile:
        p.library.remove(c)
        if c in hand: p.hand.append(c); p.seen_names.add(c.name)
        else: p.gy.append(c)
    p.stats['tutored'] += 1
    g.rng.shuffle(p.library)
    log(f'    {NAME(p)} gets {", ".join(c.name for c in hand)}'
        + (f'; {NAME(opp)} bins {", ".join(c.name for c in pile if c not in hand)}' if opp else ''), g)


def jeskas_will(g, p):
    """Choose one (both if you control your commander): add {R} for each card in target opponent's hand;
    exile the top three cards of your library, you may play them this turn."""
    opps = g.opps(p)
    most = max((len(q.hand) for q in opps), default=0)
    both = commander_out(p)
    need = sum(x.cmc for x in p.hand if not x.land) - total_mana(g, p)
    mana = both or (most >= 4 and need >= 3)
    if mana:
        p.floatR += most; log(f'    Jeska\'s Will adds {most} red mana', g)
    if both or not mana:
        top = [p.library.pop() for _ in range(min(3, len(p.library)))]
        for c in top: p.hand.append(c); p.seen_names.add(c.name)
        p.impulse += top
        log(f'    Jeska\'s Will exiles {", ".join(c.name for c in top)} (playable this turn)', g)
        lands = [c for c in top if c.land]
        if lands and p.land_turn != p.turns:
            L = lands[0]; p.hand.remove(L); p.impulse.remove(L); p.land_turn = p.turns
            import ais
            p.lands.append(Land(L, ais.land_enters_tapped(p, L))); landfall(g, p)


def chrome_imprint(g, p, m):
    """Chrome Mox: you may exile a nonartifact, nonland card from your hand; it taps for that card's colours"""
    cs = [c for c in p.hand if not c.land and 'A' not in c.types and set(c.pips) & set(p.ident)]
    if not cs: return
    c = min(cs, key=lambda c: card_worth(g, p, c))
    p.hand.remove(c); p.exile.append(c)
    m.colors = ''.join(sorted(set(c.pips) & set(p.ident)))
    log(f'    Chrome Mox imprints {c.name}', g)


def cast_spell_copy(g, p, c, ctx=None):
    """cast a copy of instant/sorcery card c (Panoptic Mirror): a real cast that can be countered; no card moves"""
    ctx = dict(ctx or {})
    p.spells_this_turn += 1; p.stats['spells_cast'] += 1
    log(f'  {NAME(p)} casts a copy of {c.name}', g)
    on_cast(g, p, c)
    if g.over or not p.alive: return False
    imp, aff = spell_imp(g, p, c, ctx)
    if (imp > 0 or aff) and not counter_window(g, p, c, imp, aff): return False
    n_gy, n_ex = len(p.gy), len(p.exile)
    resolve(g, p, c, ctx, 'hand')
    for zone, n0 in ((p.gy, n_gy), (p.exile, n_ex)):          # the copy ceases to exist instead of going to a zone
        for i in range(len(zone) - 1, n0 - 1, -1):
            if zone[i] is c: del zone[i]; break
    check_state(g)
    return True


def discard_cards(g, q, cards):
    """q discards these cards from hand: to the graveyard (exiled instead under Necropotence), then Tergrid"""
    necro = has(q, 'necro')
    for c in cards:
        q.hand.remove(c)
        (q.exile if necro else q.gy).append(c)
        if g.hooks: CI.fire(g, 'discard', q, c)
    if not necro:
        for c in cards: tergrid_steal(g, q, c, q)


def discard_index(g, q, i):
    """q discards the card at position i in hand (random discards)"""
    c = q.hand.pop(i)
    if g.hooks: CI.fire(g, 'discard', q, c)
    if has(q, 'necro'): q.exile.append(c); return
    q.gy.append(c); tergrid_steal(g, q, c, q)


def tergrid_steal(g, loser, c, gy_owner):
    """Tergrid, God of Fright: whenever an opponent sacrifices a nontoken permanent or discards a permanent card,
    you may put that card from a graveyard onto the battlefield under your control"""
    if c is None or not (c.perm or c.land) or c not in gy_owner.gy: return
    for t in g.players:
        if not t.alive or t is loser or not has(t, 'tergrid'): continue
        gy_owner.gy.remove(c)
        if c.land:
            t.lands.append(Land(c, False))
        else:
            enter(g, t, c, orig=gy_owner)
        t.stats['tergrid_steals'] += 1
        log(f'    Tergrid puts {c.name} onto the battlefield under {NAME(t)}\'s control', g)
        return


def agent_for(g, p):
    """the opponent controlling Opposition Agent while p searches, if any"""
    for q in g.opps(p):
        if any(m.cd is not None and 'agent' in m.cd.tags and not m.phased for m in q.perms): return q
    return None


def agent_take(g, a, p, c):
    """Opposition Agent: the searching player exiles the card; a may play it (held in a's hand here)"""
    a.hand.append(c); a.agent_ids.add(id(c)); a.stats['agent_takes'] += 1
    log(f'    Opposition Agent: {NAME(a)} takes {c.name} from {NAME(p)}\'s search', g)


def tutor_to_top(g, p):
    import ais
    name = ais.tutor_pick(g, p, 'any')
    lose_life(g, p, 2, p)
    if name is None: return
    c = next((x for x in p.library if x.name == name), None)
    if c is None: return
    p.library.remove(c)
    a = agent_for(g, p)
    if a is not None: agent_take(g, a, p, c); g.rng.shuffle(p.library); return
    g.rng.shuffle(p.library); p.library.append(c); p.stats['tutored'] += 1
    log(f'    {NAME(p)} puts {c.name} on top of their library', g)


def ad_nauseam(g, p, floor=18):
    """reveal the top card, put it in hand, lose life equal to its mana value; repeat while it is safe"""
    n = 0
    while p.library and p.alive and p.life - 7 > floor and n < 15:
        c = p.library.pop(); p.hand.append(c); p.seen_names.add(c.name); n += 1
        lose_life(g, p, c.cmc, p)
    p.stats['adnaus_cards'] += n
    log(f'    Ad Nauseam: {n} cards, life now {p.life}', g)
