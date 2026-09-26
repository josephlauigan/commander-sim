"""Building a game position for rule tests: seat decks, empty their hands, and put chosen cards where a test needs them.

    g = table('veyran', 'seph')                 # two seats (any of your deck keys or pool deck keys), before turn one
    me, opp = g.players
    lands(me, 'Island', 2); hand(me, 'Counterspell'); perm(g, opp, 'Grave Titan')

Games use the heuristic AI and the conservative profile; nothing here searches. Every card must be in one of the
decklists (so the Scryfall cache has it and the tests stay offline).
"""
from commander_sim import engine as E, ais, compare, pools, poolmode


def setup():
    pools.register()
    compare.set_ai('adaptive', 1.0)
    E.set_profile('conservative')


def table(*keys, seed=1, life=40):
    """a game with these decks seated in this order; hands go back into the libraries, the first seat is active in
    its first main phase"""
    setup()
    g = ais.setup_pool_game(seed, [poolmode.seat_spec(k) for k in keys])
    import collections
    for p in g.players:
        p.library += p.hand; p.hand = []
        p.life = life
        p.turns = 1                                    # the per-turn state ais._step_start sets
        p.extra_combats = 0; p.combat_no = 0; p.spells_this_turn = 0
        p.pump = 0; p.pumpadd = 0; p.trample = False; p.combo_tried = False
        p.haste_all = False; p.najeela_boost = False
        p.gy_start = collections.Counter()
    g.round = 1; g.active = g.players[0]; g.step = 'main1'
    return g


def card(name):
    return E.DB[name]


def take(p, name):
    """the card out of p's library (a fresh copy of its definition if the deck doesn't run it)"""
    for c in p.library:
        if c.name == name:
            p.library.remove(c); return c
    return E.DB[name]


def hand(p, *names):
    cs = [take(p, n) for n in names]
    p.hand += cs
    return cs[0] if len(cs) == 1 else cs


def lands(p, name, n=1, tapped=False):
    for _ in range(n): p.lands.append(E.Land(take(p, name), tapped))


def perm(g, p, name, sick=False):
    """a permanent put onto the battlefield (its enter effects happen)"""
    m = E.enter(g, p, take(p, name), sick=sick)
    return m


def token(g, p, power, tough=None, **kw):
    return E.make_tokens(g, p, 1, power, tough, sick=False, **kw)[0]
