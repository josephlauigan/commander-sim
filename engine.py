"""Abstracted 4-player Commander engine.  Rules are simplified on purpose:
role-tagged cards, greedy mana payment, heuristic AIs, simplified combat."""
import random, re
from collections import defaultdict
from carddb import DB_TEXT

IDENT = {'seph': 'WUBG', 'veyran': 'UR', 'sauron': 'UBR', 'najeela': 'WUBRG'}


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
                 'age', 'neutered', 'is_cmd', 'phys', 'temp', 'loyalty', 'loyalty_used', 'colors')

    def __init__(s, owner, cd=None, pw=1, tg=None, fly=False, warrior=False, name='Token'):
        s.cd, s.owner, s.orig = cd, owner, owner
        s.token = cd is None
        s.tapped = False; s.sick = True; s.plus = 0; s.undying = False; s.army = False
        s.phased = False; s.attached = None; s.age = 0; s.neutered = False; s.is_cmd = False
        s.phys = None; s.temp = False; s.loyalty = None; s.loyalty_used = None; s.colors = ''
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
        s.key = key; s.ident = IDENT[key]
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


class Game:
    def __init__(s, players, rng, goldfish=False):
        s.players = players; s.rng = rng; s.goldfish = goldfish
        s.over = False; s.winner = None; s.wintype = None; s.round = 0
        s.active = None
        s.dsl_on = False; s.eot_pt = {}; s.eot_kw = {}; s.dsl_depth = 0
        s.flutes = []          # (Disruptor Flute permanent, chosen card name)
        s.elim = []
        s.log = None          # list of strings when tracing a game

    def opps(s, p):
        return [q for q in s.players if q.alive and q is not p]

    def after(s, p):
        i = s.players.index(p)
        return [s.players[(i + k) % len(s.players)] for k in range(1, len(s.players))]


# ---------------------------------------------------------------- trace log
CUR_G = None
DSLMOD = None             # the card-ability interpreter (dsl.py) registers itself here
AI_MODE = 'adaptive'     # 'adaptive' (probabilistic, board-reading) or 'rigid' (fixed priorities)
DAMAGE_HOOK = None


def NAME(p):
    return {'seph': 'Sephiroth', 'veyran': 'Veyran', 'sauron': 'Sauron', 'najeela': 'Najeela'}[p.key]


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


def lose_life(g, p, n, src, kind='other'):
    if n <= 0 or not p.alive: return
    p.life -= n
    if src is not None and src is not p:
        p.last_src = src; p.last_kind = kind
        src.stats['dmg_dealt'] += n; src.stats['dmgk_' + kind] += n
        src.hit_turn = src.turns
    elif src is None:
        p.last_src = src
    if DAMAGE_HOOK is not None: DAMAGE_HOOK(p, src, n)


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
    for L in p.lands:
        if not L.tapped:
            U.append([L, land_cols(p, L, anyc), int(L.cd.tags.get('amt', 1))])
    rite = has(p, 'rite')
    for m in p.perms:
        if m.tapped or m.phased: continue
        if m.cd is None:
            if rite and not m.sick: U.append([m, p.ident, 1])
            continue
        t = m.cd.tags
        if 'rock' in t:
            a, c = t['rock'].split(':')
            U.append([m, p.ident if c == 'A' else ('' if c == 'C' else c), int(a)])
        elif 'dork' in t and not m.sick:
            c = t['dork']; U.append([m, p.ident if c == 'A' else c, 1])
        elif rite and m.cd.creature and not m.sick and m.noatk:
            U.append([m, p.ident, 1])
    for _ in range(p.treasures):
        U.append(['T', p.ident, 1])
    for _ in range(p.floatR):
        U.append(['F', 'R', 1])
    for _ in range(p.floatA):
        U.append(['G', p.ident, 1])
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
            k = (u[0] == 'T', waste, len(u[1]))
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
            if u[0] == 'T': p.treasures -= 1
            elif u[0] == 'F': p.floatR -= 1
            elif u[0] == 'G': p.floatA -= 1
            else: u[0].tapped = True
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
    if g is not None and stopped(g, c.name): gen += 3                 # Disruptor Flute tax
    if c is p.cmd: gen += p.tax
    return gen, pips


# ---------------------------------------------------------------- zones
def draw(g, p, n=1, step=False):
    for k in range(n):
        if not p.alive: return
        if not p.library:
            p.decked = True; return
        p.hand.append(p.library.pop())
        p.seen_names.add(p.hand[-1].name)
        p.stats['cards_drawn'] += 1
        if has(p, 'ironman'):
            a = army_of(p)
            if a is not None: a.plus += 1
        extra = not (step and k == 0)
        if DSLMOD is not None and g.dsl_on: DSLMOD.fire(g, 'draw', player=p, extra=extra)
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


TOKEN_COLOR = {'najeela': 'W', 'seph': 'B', 'sauron': 'B', 'veyran': 'R'}     # default colour of a deck's tokens


def make_tokens(g, p, n, pw, tg=None, fly=False, warrior=False, attacking=False, lifelink=False, sick=True, dt=False,
                color=None):
    out = []
    if DSLMOD is not None: n *= DSLMOD.token_mult(g, p)
    if n <= 0: return out
    blank = opp_has(g, p, 'mother')
    if has(p, 'jinnie') and pw < 2 and not attacking:     # Jinnie Fay: make 2/2 Cats with haste instead
        pw, tg, sick = 2, 2, False
    for _ in range(n):
        m = Perm(p, None, pw=pw, tg=tg, fly=fly, warrior=warrior)
        m.life = lifelink; m.sick = sick; m.dt = dt
        m.colors = TOKEN_COLOR.get(p.key, '') if color is None else color
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
        if has(p, 'warleader'):
            for q in g.opps(p): lose_life(g, q, k, p, kind='drain')
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
              and not untargetable(g, m)]
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
        if has(p, 'warleader'):
            for q in g.opps(p): lose_life(g, q, 1, p, kind='drain')
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
    if cause == 'destroy' and DSLMOD is not None and DSLMOD.has_kw(g, m, 'indestructible'): return
    leave(g, m)
    if DSLMOD is not None and g.dsl_on: DSLMOD.fire(g, 'dies', perm=m, owner=p, card=m.cd, dying=m)
    # death triggers
    for q in g.players:
        if not q.alive: continue
        if has(q, 'bartist'):
            opps = g.opps(q)
            if opps:
                t = max(opps, key=lambda o: threat(g, q, o)); lose_life(g, t, 1, q, kind='drain'); gain(q, 1)
        if q is p and has(q, 'drain') and m.creature:
            for o in g.opps(q): lose_life(g, o, 1, q, kind='drain')
            gain(q, 1)
    if m.cd is not None and m.cd.tags.get('fill') == 'stitcher': mill(g, p, 3)
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
        n = enter(g, p, m.cd); n.plus = 1; n.undying = True
        p.stats['undying'] += 1
        return
    to_zone_card(g, m, 'gy')


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
    if m.is_cmd: v += 1
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
def on_cast(g, p, c):
    for q in g.opps(p):
        if has(q, 'sauron'): amass(g, q, 1)
        if has(q, 'rhystic') and g.rng.random() < 0.45: draw(g, q, 1)
        if has(q, 'kaervek') and c.cmc > 0: lose_life(g, p, min(c.cmc, 6), q, kind='triggers')
    if c.instant or c.sorcery: magecraft(g, p, c)
    if DSLMOD is not None and g.dsl_on: DSLMOD.fire(g, 'cast', caster=p, spell=c)
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
        p.hand.remove(x); p.gy.append(x)


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
    if c is p.cmd: return {'seph': 8, 'veyran': 5, 'sauron': 6, 'najeela': 6}[p.key], aff
    if c.bomb and p.key == 'seph': return c.bomb, aff
    if 'vkitten' in t: return (9 if has(p, 'vfire') else 4), aff
    if 'vfire' in t: return (9 if has(p, 'vkitten') else 4), aff
    if 'sword' in t: return (9 if has(p, 'assault') else 4), aff
    if 'assault' in t: return (9 if has(p, 'sword') else 4), aff
    if 'skate' in t:
        a = army_of(p); return (7 if a and a.plus >= 6 else 3), aff
    for k, v in (('aether', 6), ('crusade', 6), ('rhystic', 5), ('tithe', 5), ('mirror', 5),
                 ('witchking', 5), ('dragoncaller', 5), ('normgc', 7), ('mother', 7)):
        if k in t: return v, aff
    if 'tokx' in t and ctx.get('x', 0) >= 4: return 5, aff
    return 0, aff


CTHRESH = {'seph': 6, 'veyran': 7, 'sauron': 7, 'najeela': 99}

# Interaction profiles for the AI opponents.
#   conservative: counter only big threats (importance >= 7), hold instant removal for emergencies
#   loose:        counter at importance >= 6, use instant removal as freely as sorcery removal
PROFILES = {
    'conservative': {'cthresh': {'seph': 6, 'veyran': 7, 'sauron': 7, 'najeela': 99}, 'instant_extra': 2},
    'loose':        {'cthresh': {'seph': 6, 'veyran': 6, 'sauron': 6, 'najeela': 99}, 'instant_extra': 0},
}
INSTANT_EXTRA = 2


def set_profile(name):
    global INSTANT_EXTRA
    prof = PROFILES[name]
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
        if 'free' in ctr.tags:
            if any(x is not ctr and 'U' in x.pips for x in q.hand) or can_pay(g, q, ctr.generic, ctr.pips):
                if best is None: best = ctr
            continue
        if can_pay(g, q, ctr.generic, ctr.pips):
            if best is None or ctr.cmc < best.cmc or 'free' in best.tags: best = ctr
    return best


def cast_counter(g, q, ctr):
    if 'free' in ctr.tags and not can_pay(g, q, ctr.generic, ctr.pips):
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
            if not nc or g.rng.random() > brain.wants_counter(g, q, val, CTHRESH[q.key], nc): continue
        else:
            if val < CTHRESH[q.key]: continue
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
    if 'undermine' in t: lose_life(g, p, 3, q, kind='triggers')
    if 'swan' in t: make_tokens(g, p, 1, 2, fly=True)          # Swan Song gives the caster a Bird


def cast_card(g, p, c, zone='hand', ctx=None, paid=True):
    """card already paid for.  zone: hand/gy/cmd"""
    ctx = ctx or {}
    if zone == 'hand': p.hand.remove(c)
    elif zone == 'gy': p.gy.remove(c)
    elif zone == 'cmd': p.cmd_in_zone = False; p.tax += 2
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
        m = enter(g, p, c)
        if c is p.cmd: m.is_cmd = True
        if 'lr' in t: land_ramp(g, p, int(t['lr']), 'lrt' in t)
        return
    if 'draw' in t: draw(g, p, int(t['fbdraw']) if (zone == 'gy' and 'fbdraw' in t) else int(t['draw']))
    if 'treas' in t: p.treasures += int(t['treas'])
    if 'lr' in t:
        land_ramp(g, p, int(t['lr']), 'lrt' in t)
        if 'lh' in t: land_to_hand(g, p)
    if 'tut' in t: tutor(g, p, t['tut'])
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
        p.gy.extend(p.hand); p.hand = []; draw(g, p, 4)
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


def enter(g, p, cd, orig=None, sick=True):
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
    p.perms.append(m)
    if cd.creature: creature_entered(g, p, m)
    if m in p.perms: do_etb(g, p, m)
    if DSLMOD is not None and g.dsl_on and m in p.perms: DSLMOD.fire(g, 'etb', perm=m, owner=p)
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
    if q.hand: q.gy.append(q.hand.pop(g.rng.randrange(len(q.hand))))
    lose_life(g, q, 3, p, kind='drain'); gain(p, 3); draw(g, p, 1)


def edict(g, q):
    cr = [m for m in q.perms if m.creature and not m.phased]
    if cr: die(g, min(cr, key=lambda x: pval(g, x)), 'sac')


def land_ramp(g, p, n, tapped):
    for _ in range(n):
        basics = [c for c in p.library if c.land and c.name in ('Forest', 'Island', 'Plains', 'Swamp', 'Mountain')]
        if not basics: return
        c = g.rng.choice(basics); p.library.remove(c); p.lands.append(Land(c, tapped))
        landfall(g, p)


def landfall(g, p):
    for _ in find(p, 'landfall2'): make_tokens(g, p, 1, 2)      # Felidar Retreat: 2/2 Cat per land
    if DSLMOD is not None and g.dsl_on: DSLMOD.fire(g, 'landfall', player=p)


def land_to_hand(g, p):
    basics = [c for c in p.library if c.land and c.name in ('Forest', 'Island', 'Plains', 'Swamp', 'Mountain')]
    if basics:
        c = g.rng.choice(basics); p.library.remove(c); p.hand.append(c)


def tutor(g, p, kind):
    import ais
    name = ais.tutor_pick(g, p, kind)
    if name is None: return
    for c in p.library:
        if c.name == name:
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
                  'cna': is_c and 'A' not in ty}.get(tgt, is_c)
            if spell is not None and 'alsoart' in spell.tags and 'A' in ty and not is_c:
                res.append(m); continue                  # Abrade: destroy target artifact mode
            if not ok: continue
            if mv4 and cd is not None and cd.cmc > 4: continue
            if spell is not None and protected_from(g, m, spell.pips): continue
            if kind.startswith('dmg'):
                if not is_c or etgh(g, m) > int(kind[3:]): continue
            res.append(m)
    return res


def apply_removal(g, actor, m, kind, spell=None):
    owner = m.owner
    if m not in owner.perms or untargetable(g, m): return
    import ais
    if ais.protect_response(g, owner, m, kind, actor):
        log(f'    {NAME(owner)} protects {m.name}', g); return
    if m not in owner.perms: return
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