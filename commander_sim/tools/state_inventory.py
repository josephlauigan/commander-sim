"""Inventory of the game state Python sets at run time, for the Rust port (documents/rust-design.md, section 10).

Plays seeded games (heuristic AI, every deck of yours against every tier) and records every attribute that appears
on the game and the players, the keys of each permanent's and land's data dict (by card), and stack items.

    python3 -m commander_sim.tools.state_inventory OUT.json [games per deck per tier, default 20]
"""
import collections, json, sys
from commander_sim import poolmode as PM, ais, engine as E
from commander_sim.decks import DECKS
N = int(sys.argv[2]) if len(sys.argv) > 2 else 20
PM._setup('loose', 'adaptive', 1.0)
G = collections.defaultdict(set); P = collections.defaultdict(set)
D = collections.defaultdict(lambda: collections.defaultdict(set))   # data key -> card -> types
LD = collections.defaultdict(lambda: collections.defaultdict(set))
DICTKEYS = collections.defaultdict(set)
ST = collections.defaultdict(set)
def tn(v):
    t = type(v).__name__
    if isinstance(v, (list, tuple, set, frozenset)) and v:
        t += '[' + type(next(iter(v))).__name__ + ']'
    if isinstance(v, dict) and v:
        k, x = next(iter(v.items())); t += '{' + type(k).__name__ + ':' + type(x).__name__ + '}'
    return t
def snap(g):
    for k, v in vars(g).items(): G[k].add(tn(v))
    for p in g.players:
        for k, v in vars(p).items(): P[k].add(tn(v))
        for m in p.perms:
            if m.data:
                for k, v in m.data.items(): D[k][m.name].add(tn(v))
        for L in p.lands:
            if L.data:
                for k, v in L.data.items(): LD[k][L.cd.name].add(tn(v))
    for it in g.stack:
        for k, v in vars(it).items(): ST[k].add(tn(v))
orig = ais.take_turn
def tt(g, p):
    snap(g); r = orig(g, p); snap(g); return r
ais.take_turn = tt
n = 0
for tier in ('t1', 't2', 't3', 't4', 't5'):
    keys = PM.pool_keys(tier)
    for deck in DECKS:
        for s in range(700000, 700000 + N):
            g = PM.play(s, deck, None, keys); snap(g); n += 1
out = {'games': n,
       'game': {k: sorted(v) for k, v in sorted(G.items())},
       'player': {k: sorted(v) for k, v in sorted(P.items())},
       'perm_data': {k: {c: sorted(t) for c, t in sorted(v.items())} for k, v in sorted(D.items())},
       'land_data': {k: {c: sorted(t) for c, t in sorted(v.items())} for k, v in sorted(LD.items())},
       'stack_item': {k: sorted(v) for k, v in sorted(ST.items())}}
json.dump(out, open(sys.argv[1], 'w'), indent=1)
print(n, 'games;', len(G), 'game attrs;', len(P), 'player attrs;', len(D), 'perm data keys;', len(LD), 'land data keys')
