"""Scryfall client: look up cards by name, cache them locally.

Follows Scryfall's API guidelines (https://scryfall.com/docs/api):
  - a descriptive User-Agent and an Accept header on every request
  - at most ~10 requests per second (we wait 100 ms between calls)
  - batch lookups through /cards/collection (75 names per request)
  - cache results instead of re-downloading (data/scryfall_cache.json in the repository)

Usage from the command line:
    python3 -m commander_sim.cards.scryfall "Lightning Bolt" "Talrand, Sky Summoner"     # fetch + show
    python3 -m commander_sim.cards.scryfall --deck veyran                                 # fetch a whole deck list
"""
import json, os, sys, time, urllib.request, urllib.parse, urllib.error

from commander_sim import DATA
CACHE_PATH = os.path.join(DATA, 'scryfall_cache.json')
API = 'https://api.scryfall.com'
HEADERS = {'User-Agent': 'commander-pod-sim/1.0 (personal deck testing)', 'Accept': 'application/json'}
_last = [0.0]
_cache = None


def _wait():
    dt = time.time() - _last[0]
    if dt < 0.1: time.sleep(0.1 - dt)
    _last[0] = time.time()


def _request(url, body=None):
    _wait()
    data = json.dumps(body).encode() if body is not None else None
    h = dict(HEADERS)
    if data is not None: h['Content-Type'] = 'application/json'
    req = urllib.request.Request(url, data=data, headers=h, method='POST' if data is not None else 'GET')
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.loads(r.read().decode())


def load_cache():
    global _cache
    if _cache is None:
        if os.path.exists(CACHE_PATH):
            with open(CACHE_PATH, encoding='utf-8') as fh: _cache = json.load(fh)
        else: _cache = {}
    return _cache


def save_cache():
    if _cache is not None:
        tmp = CACHE_PATH + '.tmp'
        json.dump(_cache, open(tmp, 'w'))
        os.replace(tmp, CACHE_PATH)


def slim(card):
    """keep only what the simulator uses"""
    keep = ('name', 'mana_cost', 'cmc', 'type_line', 'oracle_text', 'power', 'toughness', 'loyalty', 'colors',
            'color_identity', 'keywords', 'produced_mana', 'layout', 'game_changer', 'legalities')
    out = {k: card.get(k) for k in keep if k in card}
    if 'legalities' in out: out['legalities'] = {'commander': card['legalities'].get('commander')}
    if card.get('card_faces'):
        out['card_faces'] = [{k: f.get(k) for k in ('name', 'mana_cost', 'type_line', 'oracle_text', 'power',
                                                   'toughness', 'loyalty') if k in f} for f in card['card_faces']]
    return out


def _key(name):
    return name.strip().lower()


def fetch(names, verbose=True):
    """Return {name: slim card} for every name, using the cache and fetching the rest."""
    cache = load_cache()
    want = [n for n in dict.fromkeys(names) if _key(n) not in cache]
    if want:
        if verbose: print(f'Scryfall: fetching {len(want)} card(s)...', file=sys.stderr)
        for i in range(0, len(want), 75):
            batch = want[i:i + 75]
            ids = [{'name': n.split(' // ')[0] if ' // ' in n else n} for n in batch]
            try:
                res = _request(API + '/cards/collection', {'identifiers': ids})
            except urllib.error.URLError as e:
                print(f'Scryfall request failed ({e}). Check your internet connection; '
                      f'using {len(names) - len(want)} cached card(s) only.', file=sys.stderr)
                break
            found = {}
            for card in res.get('data', []):
                found[_key(card['name'])] = card
                if card.get('card_faces'): found[_key(card['card_faces'][0]['name'])] = card
            for n in batch:
                card = found.get(_key(n)) or found.get(_key(n.split(' // ')[0]))
                if card is None:                          # fall back to fuzzy lookup for odd spellings
                    try:
                        card = _request(API + '/cards/named?' + urllib.parse.urlencode({'fuzzy': n}))
                    except urllib.error.HTTPError:
                        card = None
                if card is not None: cache[_key(n)] = slim(card)
                elif verbose: print(f'  not found on Scryfall: {n}', file=sys.stderr)
        save_cache()
    return {n: cache[_key(n)] for n in names if _key(n) in cache}


if __name__ == '__main__':
    args = sys.argv[1:]
    if args[:1] == ['--deck']:
        from commander_sim.decks import DECKS
        args = sorted(set(DECKS[args[1]]))
    for n, c in fetch(args).items():
        print(f"{n}: {c.get('mana_cost', '')} | {c.get('type_line', '')} | {c.get('oracle_text', '')[:200]}")
