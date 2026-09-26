"""Test proposed list changes for an opponent-pool deck without editing the list.

    python3 -m commander_sim.tools.swaptest <deck key> <tier the deck plays against> <games> "Out Card>In Card; Out>In; ..."

Runs the deck alone against three decks of that tier, paired (same seeds, seats and opponents' draws), once with
the list as written and once with the swaps, and reports both win rates. The variant list is checked with the
same validator as the real lists (100 cards, singleton, colour identity, bans, Game Changer limit for its tier).
"""
import sys, dataclasses
from commander_sim import engine as E, ais, compare, pools, poolmode
from multiprocessing import Pool


def variant(deck, swaps):
    cards = list(deck.cards)
    for out, inn in swaps:
        if out not in cards: raise SystemExit(f'{out!r} is not in {deck.key}')
        if inn in cards: raise SystemExit(f'{inn!r} is already in {deck.key}')
        cards[cards.index(out)] = inn
    return cards


def check(deck, cards):
    from commander_sim.cards import scryfall
    v = dataclasses.replace(deck, cards=cards) if dataclasses.is_dataclass(deck) else deck
    if not dataclasses.is_dataclass(deck):
        v = type(deck).__new__(type(deck)); v.__dict__.update(deck.__dict__); v.cards = cards
    probs = pools.structural_problems(cards, deck.commander)
    recs = scryfall.fetch(sorted(set(cards)), verbose=False)
    p2, _ = pools.card_problems(v, recs)
    return [x for x in probs + p2 if not x.startswith('file says')]    # the header count is the file's, not the variant's


def _run(args):
    me, tier, seeds, cards = args
    pools.register(); compare.set_ai('adaptive', 1.0); E.set_profile('conservative')
    from commander_sim.cards import sources as C; C.ensure_cards(sorted(set(cards)), verbose=False)
    out = []
    for s in seeds:
        seats = pools.draw_seats(s, poolmode.pool_keys(tier), me, 3)
        r = []
        for lst in (None, cards):
            g = ais.play_pool_game(s, [poolmode.seat_spec(k, lst if k == me else None) for k in seats])
            r.append(g.winner is not None and g.winner.key == me)
        out.append(r)
    return out


def run(me, tier, n, swaps, jobs=24, seed0=500000):
    pools.register()
    deck = pools.by_key()[me]
    cards = variant(deck, swaps)
    probs = check(deck, cards)
    if probs: raise SystemExit('variant list fails validation: ' + '; '.join(probs))
    seeds = list(range(seed0, seed0 + n)); chunks = [seeds[i::jobs] for i in range(jobs)]
    with Pool(jobs) as P: res = [r for part in P.map(_run, [(me, tier, c, cards) for c in chunks]) for r in part]
    b = sum(r[0] for r in res) / n; v = sum(r[1] for r in res) / n
    return b, v


if __name__ == '__main__':
    me, tier, n = sys.argv[1], sys.argv[2], int(sys.argv[3])
    swaps = [tuple(x.strip() for x in s.split('>')) for s in sys.argv[4].split(';') if s.strip()]
    jobs = int(sys.argv[5]) if len(sys.argv) > 5 else 24
    b, v = run(me, tier, n, swaps, jobs)
    print(f'{me} alone vs {tier}: as written {100*b:.1f}%, with {len(swaps)} swaps {100*v:.1f}% ({100*(v-b):+.1f} pts, n={n})')
