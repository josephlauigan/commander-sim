"""The human seat's mana pool: tap sources yourself, pay costs from the pool, and the pool empties between steps.

The AI seats keep the engine's automatic payment (engine.pay). A tapped source goes through engine.spend_unit, so
pain lands, Treasures and on-tap triggers behave exactly as they do for the AI.
"""
from commander_sim import engine as E

COLOURS = 'WUBRGC'
NAMES = {'W': 'white', 'U': 'blue', 'B': 'black', 'R': 'red', 'G': 'green', 'C': 'colourless'}
FLOATS = ('F', 'G', 'FU', 'FC', 'FG')          # engine units that are already floating mana, not sources to tap
SPENDING = None      # what is being paid for now (a card, or a creature whose ability it is): restricted mana's test
RESTRICTED_LANDS = ('Secluded Courtyard',)       # any colour, but only for creature spells / creatures of a named type


class ManaPool:
    """mana you have made and not spent. 'A' is mana of any colour the engine adds for you (rituals and similar)"""
    def __init__(s):
        s.m = dict.fromkeys(COLOURS + 'A', 0)
        s.k = dict.fromkeys(COLOURS, 0)          # restricted mana (Secluded Courtyard): creature spells of type s.ktype
        s.ktype = None

    def add(s, colour, n=1):
        s.m[colour] += n

    def add_restricted(s, colour, n, ctype):
        s.k[colour] += n; s.ktype = ctype

    def total(s):
        return sum(s.m.values()) + sum(s.k.values())

    def empty(s):
        s.m = dict.fromkeys(COLOURS + 'A', 0)
        s.k = dict.fromkeys(COLOURS, 0)

    def text(s):
        """{B}{B}{C} style: coloured mana, then colourless, then any-colour ({3} would read as generic mana)"""
        out = ''.join('{' + c + '}' for c in 'WUBRGC' for _ in range(s.m[c]))
        if s.m['A']: out += f" +{s.m['A']} any"
        if any(s.k.values()):
            out += ' +' + ''.join('{' + c + '}' for c in 'WUBRGC' for _ in range(s.k[c])) + \
                   f" for {(s.ktype or 'the named').title()} creature spells"
        return out or 'empty'

    def k_ok(s):
        """may the restricted mana pay for what is being paid for now"""
        x = SPENDING
        if not any(s.k.values()) or x is None or s.ktype is None: return False
        import importlib
        G = importlib.import_module('commander_sim.cards.impl.galadriel')
        if isinstance(x, E.Perm): return x.creature and E.has_type(x, s.ktype)
        return x.creature and G.card_is(G_OWNER[0], x, s.ktype) if G_OWNER else x.creature and s.ktype in x.subtypes

    def plan(s, generic, pips):
        """the pool left after paying {generic}+pips, or None if it can't: each coloured pip from its own colour
        (any-colour mana last); generic from colourless first, then the most plentiful colour, then any-colour.
        Restricted mana (Secluded Courtyard) is spent first when it may be"""
        if s.k_ok():
            m = dict(s.m); k = dict(s.k)
            rest = ''
            for c in pips:
                if k.get(c, 0) > 0: k[c] -= 1
                else: rest += c
            need = generic
            for c in COLOURS:
                t = min(k[c], need); k[c] -= t; need -= t
            saved = s.m, s.k
            s.m = m; s.k = dict.fromkeys(COLOURS, 0)
            try:
                left = s.plan(need, rest)
            finally:
                s.m, s.k = saved
            if left is None: return None
            return dict(left, _k=k)
        m = dict(s.m)
        for c in pips:
            if c in m and m[c] > 0: m[c] -= 1
            elif m['A'] > 0: m['A'] -= 1
            else: return None
        need = generic
        order = ['C'] + sorted('WUBRG', key=lambda c: -m[c]) + ['A']
        for c in order:
            k = min(m[c], need); m[c] -= k; need -= k
            if need == 0: break
        return m if need == 0 else None

    def can_pay(s, generic, pips):
        return s.plan(generic, pips) is not None

    def pay(s, generic, pips):
        m = s.plan(generic, pips)
        if m is None: return False
        k = m.pop('_k', None)
        s.m = m
        if k is not None: s.k = k
        return True

    def missing(s, generic, pips):
        """in words, what the pool lacks for this cost (for the error message)"""
        m = dict(s.m); short = []
        for c in pips:
            if m.get(c, 0) > 0: m[c] -= 1
            elif m['A'] > 0: m['A'] -= 1
            else: short.append('{' + c + '}')
        left = sum(m.values())
        if generic > left: short.insert(0, '{' + str(generic - left) + '}')
        return ''.join(short)

    def has_text(s):
        """'is empty' / 'has {B}{B}', for messages"""
        return 'is empty' if s.total() == 0 else f'has {s.text()}'


G_OWNER = []         # the player paying now (for Maskwood Nexus: their creature cards are every type)


def pool_of(p):
    """p's mana pool, created on first use (only the human seat has one)"""
    if getattr(p, 'pool', None) is None: p.pool = ManaPool()
    G_OWNER[:] = [p]
    return p.pool


def _colours(u):
    """the colours unit u can make; painlands and Talismans can also make colourless, without the pain"""
    cols = ''.join(c for c in u[1] if c in 'WUBRG') or 'C'
    cd = getattr(u[0], 'cd', None)
    if cd is not None and 'pain' in cd.tags and 'C' not in cols: cols += 'C'
    return cols


def sources(g, p):
    """the untapped things p can tap (or sacrifice) for mana now: [{'id', 'name', 'colours', 'amount'}]; ids index the
    engine's mana units and are only valid until the next change to the board"""
    out = []
    for i, u in enumerate(E.mana_units(g, p)):
        if u[0] in FLOATS: continue
        src = u[0]
        name = 'Treasure' if src == 'T' else (getattr(src, 'name', None) or getattr(getattr(src, 'cd', None), 'name', None)
                                             or (getattr(u[3], 'name', 'Token') if len(u) > 3 else str(src)))
        d = {'id': i, 'name': name, 'colours': _colours(u), 'amount': u[2]}
        kt = _restricted_type(src)
        if kt is not None:                                       # Secluded Courtyard: any colour, restricted
            d['colours'] = 'C' + ''.join(c for c in p.ident if c in 'WUBRG'); d['restricted'] = kt
        if src == 'T': d['treasure'] = True                      # where it is on the table, for the browser
        elif any(src is L for L in p.lands): d['land'] = next(k for k, L in enumerate(p.lands) if L is src)
        else:
            m = u[3] if len(u) > 3 and isinstance(src, str) else src
            k = next((k for k, x in enumerate(p.perms) if x is m), None)
            if k is not None: d['perm'] = k
        out.append(d)
    return out


def tap(g, p, source_id, colour=None):
    """tap (or sacrifice) source `source_id` from sources() for its mana, into p's pool; colour: which colour a
    multi-colour source makes (default: its first). Returns an error string, or None when it worked"""
    units = E.mana_units(g, p)
    if not 0 <= source_id < len(units) or units[source_id][0] in FLOATS: return 'That source is gone or already tapped.'
    u = units[source_id]
    kt = _restricted_type(u[0])
    if kt is not None and colour and colour != 'C':
        if colour not in p.ident: return f"That source can't make {NAMES.get(colour, colour)} mana."
        E.spend_unit(g, p, u, 1, '')
        pool_of(p).add_restricted(colour, 1, kt)
        E.log(f'  {E.NAME(p)} taps {sources_name(u)} for {{{colour}}} (only for {kt.title()} creature spells)', g)
        return None
    cols = _colours(u)
    colour = colour or cols[0]
    if colour not in cols: return f"That source can't make {NAMES.get(colour, colour)} mana."
    E.spend_unit(g, p, u, u[2], colour if colour != 'C' else '')
    pool_of(p).add(colour, u[2])
    E.log(f'  {E.NAME(p)} taps {sources_name(u)} for ' + ('{' + colour + '}') * u[2], g)
    return None


def _restricted_type(src):
    cd = getattr(src, 'cd', None)
    if isinstance(src, E.Land) and cd is not None and cd.name in RESTRICTED_LANDS:
        return (src.data or {}).get('ctype')
    return None


def sources_name(u):
    src = u[0]
    if src == 'T': return 'a Treasure'
    return getattr(src, 'name', None) or getattr(getattr(src, 'cd', None), 'name', None) or 'a mana source'


def _gather_floating(p):
    """the engine's floating mana (rituals, Birgi, Lion's Eye Diamond) joins the pool"""
    pool = pool_of(p)
    for attr, c in (('floatR', 'R'), ('floatU', 'U'), ('floatC', 'C'), ('floatA', 'A'), ('floatG', 'G'), ('floatB', 'B')):
        n = getattr(p, attr, 0)
        if n: pool.add(c, n); setattr(p, attr, 0)
    return pool


def pool_can_pay(p, generic, pips):
    return _gather_floating(p).can_pay(generic, pips)


def cost_problem(g, p, generic, pips):
    """None if p's pool can pay this now, else the reason (nothing is spent)"""
    pool = _gather_floating(p)
    if pool.can_pay(generic, pips): return None
    return f'It costs {cost_text(generic, pips)}; your mana pool {pool.has_text()}. Missing {pool.missing(generic, pips)}.'


def pay_from_pool(g, p, generic, pips):
    """pay a cost from p's pool and the engine's floating mana (rituals). Returns None, or the reason it can't"""
    pool = _gather_floating(p)
    if pool.pay(generic, pips): return None
    return f'It costs {cost_text(generic, pips)}; your mana pool {pool.has_text()}. Missing {pool.missing(generic, pips)}.'


def cost_text(generic, pips):
    return (('{' + str(generic) + '}') if generic else '') + ''.join('{' + c + '}' for c in pips) or '{0}'


def empty_pools(g):
    """a step ends: every mana pool empties"""
    for p in g.players:
        if getattr(p, 'pool', None) is not None: p.pool.empty()
