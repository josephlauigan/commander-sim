"""Your decks: the lists in decklists/JD/ and decklists/Avery/ (one folder per owner, see deck_files.py), read from
each file's '## Import list' block. The outside decks live in decklists/pool/ (pools.py).
Any listed card without hand-written tags is looked up on Scryfall and modeled automatically (cached in
data/scryfall_cache.json).

    python3 -m commander_sim.decks        # card counts, and any card with no data
"""
from commander_sim.engine import DB
from commander_sim.deck_files import FILES, OWNERS, deck_path, is_deck_file  # noqa: F401


def load(path):
    with open(path, encoding='utf-8') as fh: txt = fh.read()
    block = txt.split('## Import list')[1].split('```')[1]
    out = []
    for line in block.strip().splitlines():
        n, name = line.split(' ', 1); out += [name.strip()] * int(n)
    return out


DECKS = {k: load(deck_path(k)) for k in FILES}


def _ensure_all():
    from commander_sim.cards import sources as cards
    names = sorted({n for v in DECKS.values() for n in v})
    added, missing = cards.ensure_cards(names)
    if missing:
        raise SystemExit('Cards not found on Scryfall (check spelling in the deck file): ' + ', '.join(missing))


_ensure_all()

if __name__ == '__main__':
    for k, v in DECKS.items():
        print(k, len(v), 'cards, missing:', [n for n in v if n not in DB])
