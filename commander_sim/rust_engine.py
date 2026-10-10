"""Pool games played by the Rust engine (python3 -m commander_sim ... --engine rust).

The Rust module rust_sim (rust/build-py.sh builds it into commander_sim/rust_sim.so) plays the game from the seats
poolmode draws; this module turns its summary into the objects poolmode's tallies read: the game's winner, wintype
and round, and each player's key, turns, alive, killer, stats and milestones. The card data comes from
data/cards.json and data/decks.json (python3 -m commander_sim.tools.export_cards), so re-export after changing a
card or a list.
"""
import json
from collections import Counter
from types import SimpleNamespace
from commander_sim import DATA

PROFILE = ['conservative']        # set by poolmode._setup


def _module():
    try:
        from commander_sim import rust_sim
    except ImportError as e:
        raise SystemExit('--engine rust needs the Rust module: run rust/build-py.sh') from e
    return rust_sim


def play_pool_game(seed, seats, ai, temp, max_rounds=20, trace=False):
    """ais.play_pool_game in Rust. seats: [(key, cards, commander name), ...] in turn order"""
    spec = json.dumps([[k, list(cards), cmd] for k, cards, cmd in seats])
    out = json.loads(_module().play_game(DATA, seed, spec, PROFILE[0], ai, temp, max_rounds, trace))
    players = [SimpleNamespace(key=p['key'], turns=p['turns'], alive=p['alive'], life=p['life'],
                               stats=Counter(p['stats']), milestone=p['milestone'], first_bomb=None, killer=None)
               for p in out['players']]
    by_key = {p.key: p for p in players}
    for p, raw in zip(players, out['players']):
        p.killer = by_key.get(raw['killer'])
    return SimpleNamespace(players=players, winner=by_key.get(out['winner']), wintype=out['wintype'],
                           round=out['round'], stopped=out['stopped'], search_n=out['search_n'],
                           search_work=out['search_work'], log=out['log'])
