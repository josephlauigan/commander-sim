"""Your decks: the lists in decklists/mine/, read from each file's '## Import list' block.

Deck files are looked for in $SIM_DECKS, then decklists/mine/ in the repository, then /mnt/project (Claude's
project copy). The outside decks live in decklists/pool/ (pools.py). Any listed card without hand-written tags is
looked up on Scryfall and modeled automatically (cached in data/scryfall_cache.json).

    python3 -m commander_sim.decks        # card counts, and any card with no data
"""
import os
from commander_sim import ROOT
from commander_sim.engine import DB


def load(path):
    txt = open(path).read(); block = txt.split('## Import list')[1].split('```')[1]
    out = []
    for line in block.strip().splitlines():
        n, name = line.split(' ', 1); out += [name.strip()] * int(n)
    return out


_cands = [os.environ.get('SIM_DECKS', ''), os.path.join(ROOT, 'decklists', 'mine'), '/mnt/project']
P = next(d for d in _cands if d and os.path.exists(os.path.join(d, 'sephiroth-phyrexian-reanimator.md'))) + os.sep
DECKS = {'seph': load(P + 'sephiroth-phyrexian-reanimator.md'), 'veyran': load(P + 'veyran-izzet-spellslinger.md'),
         'sauron': load(P + 'sauron-grixis-amass.md')}


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
