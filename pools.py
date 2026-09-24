"""Opponent pools: 25 outside decks in five power tiers (opponents/<tier folder>/<deck>.md).

Deck files use the same layout as the four main decks, so they go through decks.load().
The commander comes from the '**Commander:**' line.

    python3 pools.py                 # list the pools
    python3 pools.py --validate      # structure, card names, colour identity, bans, Game Changers per tier
"""
import collections, glob, os, random, re, sys
from decks import load, P

POOL_DIR = os.path.join(P, 'opponents')
BASICS = {'Plains', 'Island', 'Swamp', 'Mountain', 'Forest', 'Wastes',
          'Snow-Covered Plains', 'Snow-Covered Island', 'Snow-Covered Swamp',
          'Snow-Covered Mountain', 'Snow-Covered Forest'}
# Game Changer rules per tier (opponents/README.md): (min, max); None = unrestricted
GC_RULES = {'t1': (0, 0), 't2': (1, 2), 't3': (3, 3), 't4': None, 't5': None}
TIERS = ('t1', 't2', 't3', 't4', 't5')


class PoolDeck:
    def __init__(s, path):
        s.path = path
        s.folder = os.path.basename(os.path.dirname(path))
        s.tier = s.folder.split('-')[0]
        s.key = os.path.splitext(os.path.basename(path))[0]
        txt = open(path, encoding='utf-8').read()
        s.title = txt.splitlines()[0].lstrip('# ').strip()
        m = re.search(r'^- \*\*Commander:\*\*\s*(.+?)\s*$', txt, re.M)
        s.commander = m.group(1) if m else None
        m = re.search(r'\*\*Game Changers \((\d+)\)', txt)
        s.gc_claimed = int(m.group(1)) if m else None
        m = re.search(r'^## Sim modeling notes\s*\n(.*?)(?=^## )', txt, re.M | re.S)
        s.modeling_notes = m.group(1).strip() if m else ''
        s.cards = load(path)

    def __repr__(s):
        return f'PoolDeck({s.tier}/{s.key})'


_CACHE = {}


def load_pool(tier=None):
    """All pool decks, or one tier's ('t1'..'t5'), sorted by tier then file name."""
    if 'all' not in _CACHE:
        _CACHE['all'] = [PoolDeck(f) for f in sorted(glob.glob(os.path.join(POOL_DIR, 't*', '*.md')))]
    return [d for d in _CACHE['all'] if tier is None or d.tier == tier]


def by_key():
    return {d.key: d for d in load_pool()}


def short_name(d):
    return d.commander.split(' // ')[0].split(',')[0]


_REGISTERED = set()


def register(decks=None, verbose=False):
    """Make pool decks playable: card data in engine.DB, a seat entry (colour identity, display name) and the
    deck's AI configuration. Idempotent, and cheap after the first call (worker processes call it too)."""
    import engine, cards, scryfall, pool_ai, pool_decks, cardimpl, pool_cards
    cardimpl.load()
    decks = load_pool() if decks is None else decks
    todo = [d for d in decks if d.key not in _REGISTERED]
    if not todo: return
    added, missing = cards.ensure_cards(sorted({n for d in todo for n in d.cards}), verbose=verbose)
    if missing: raise SystemExit('Pool cards not found on Scryfall: ' + ', '.join(missing))
    pool_cards.apply()
    recs = scryfall.fetch([d.commander for d in todo], verbose=False)
    for d in todo:
        ident = ''.join(c for c in 'WUBRG' if c in (recs[d.commander].get('color_identity') or []))
        engine.register_seat(d.key, ident, short_name(d))
        pool_ai.CONFIG[d.key] = pool_decks.CONFIG.get(d.key, {})
        _REGISTERED.add(d.key)


# ------------------------------------------------------------------ seating
def draw_seats(seed, pool_keys, me=None, k=3):
    """Seat order for one game: k opponents drawn without replacement from pool_keys, plus `me` (if given),
    in a random order. Depends only on the seed, the pool and me's key -- never on anyone's card list --
    so two versions of a deck see exactly the same opponents in the same seats."""
    r = random.Random(f'pool:{seed}')
    seats = r.sample(sorted(pool_keys), k) + ([me] if me is not None else [])
    r.shuffle(seats)
    return seats


def structural_problems(cards, commander, size=100):
    """Checks that need no card data: size, singleton, commander present. Returns a list of strings."""
    probs = []
    if len(cards) != size: probs.append(f'{len(cards)} cards, expected {size}')
    if not commander: probs.append('no **Commander:** line')
    elif commander not in cards: probs.append(f'commander {commander!r} is not in the import list')
    dups = sorted(n for n, k in collections.Counter(cards).items() if k > 1 and n not in BASICS)
    if dups: probs.append('not singleton: ' + ', '.join(dups))
    return probs


def resolved_name_ok(requested, rec):
    """Did Scryfall return the card we asked for (not a fuzzy near-miss)?"""
    key = requested.strip().lower()
    names = [rec.get('name', '')] + [f.get('name', '') for f in rec.get('card_faces') or []]
    return key in (n.lower() for n in names)


def card_problems(deck, recs):
    """Checks that need Scryfall data. recs: {name: slim record}. Returns (problems, info)."""
    probs = []
    names = sorted(set(deck.cards))
    missing = [n for n in names if n not in recs]
    if missing: probs.append('not found on Scryfall: ' + ', '.join(missing))
    fuzzy = [f'{n} -> {recs[n]["name"]}' for n in names if n in recs and not resolved_name_ok(n, recs[n])]
    if fuzzy: probs.append('resolved only by fuzzy match (check spelling): ' + ', '.join(fuzzy))
    cmd = recs.get(deck.commander)
    ident = set(cmd.get('color_identity') or []) if cmd else None
    if cmd is not None:
        tl = cmd.get('type_line') or ''
        text = (cmd.get('oracle_text') or '') + ' '.join(f.get('oracle_text') or '' for f in cmd.get('card_faces') or [])
        if not ('Legendary' in tl and 'Creature' in tl) and 'can be your commander' not in text:
            probs.append(f'commander {deck.commander} is not a legendary creature')
    if ident is not None:
        off = [n for n in names if n in recs and not set(recs[n].get('color_identity') or []) <= ident]
        if off: probs.append(f'outside colour identity {"".join(sorted(ident)) or "C"}: ' + ', '.join(off))
    banned = [n for n in names if n in recs and (recs[n].get('legalities') or {}).get('commander') == 'banned']
    if banned: probs.append('banned in Commander: ' + ', '.join(banned))
    gcs = sorted(n for n in names if n in recs and recs[n].get('game_changer'))
    rule = GC_RULES.get(deck.tier)
    if rule and not rule[0] <= len(gcs) <= rule[1]:
        probs.append(f'{len(gcs)} Game Changers, tier {deck.tier} allows {rule[0]}-{rule[1]}')
    if deck.gc_claimed is not None and deck.gc_claimed != len(gcs):
        probs.append(f'file says {deck.gc_claimed} Game Changers, Scryfall says {len(gcs)}')
    return probs, {'gcs': gcs, 'identity': ''.join(sorted(ident)) if ident is not None else '?'}


def validate_all(verbose=True):
    import scryfall
    decks = load_pool()
    recs = scryfall.fetch(sorted({n for d in decks for n in d.cards}), verbose=verbose)
    report = []
    for d in decks:
        probs = structural_problems(d.cards, d.commander)
        p2, info = card_problems(d, recs)
        report.append((d, probs + p2, info))
    return report


def main():
    if '--validate' in sys.argv:
        bad = 0
        for d, probs, info in validate_all():
            status = 'OK ' if not probs else 'ERR'
            bad += bool(probs)
            print(f'{status} {d.tier} {d.key:40s} {d.commander} [{info["identity"]}] GCs={len(info["gcs"])}')
            for p in probs: print('      - ' + p)
        n = len(load_pool())
        print(f'\n{n - bad} of {n} decks passed' if bad else '\nall decks passed')
        sys.exit(1 if bad else 0)
    for d in load_pool():
        print(f'{d.tier} {d.key:40s} {d.commander}')


if __name__ == '__main__':
    main()
