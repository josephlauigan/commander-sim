"""Card images for the browser table: found on Scryfall and downloaded once into data/images/ (not in git), only for
the cards of the decks in this game and the tokens those cards make.

data/images/index.json maps a card name (lower case) to {'id': scryfall id, 'files': [image file per face],
'urls': [image url per face], 'tokens': [token scryfall ids]}, and 'token:<id>' to a token's entry with its 'name'.
The simulator's card cache (data/scryfall_cache.json) is left alone, so nothing here can change a simulation.

Requests follow Scryfall's guidelines through the same pacing and User-Agent as cards/scryfall.py: a lookup of 75
cards per request, then one request per image not yet on disk. A card whose image can't be had is drawn by the page
from its name instead; the game starts either way.
"""
import json
import os
import urllib.error
import urllib.request

from commander_sim import DATA
from commander_sim.cards import scryfall

DIR = os.path.join(DATA, 'images')
INDEX = os.path.join(DIR, 'index.json')
SIZE = 'normal'                                     # 488x680


def _key(name):
    return name.strip().lower()


def load_index():
    if os.path.exists(INDEX):
        try:
            with open(INDEX, encoding='utf-8') as f: return json.load(f)
        except ValueError:
            pass
    return {}


def save_index(idx):
    os.makedirs(DIR, exist_ok=True)
    tmp = INDEX + '.tmp'
    with open(tmp, 'w', encoding='utf-8') as f: json.dump(idx, f)
    os.replace(tmp, INDEX)


def _entry(card):
    """the index entry for a Scryfall card object"""
    if card.get('image_uris'): urls = [card['image_uris'].get(SIZE)]
    else: urls = [f.get('image_uris', {}).get(SIZE) for f in card.get('card_faces') or []]
    urls = [u for u in urls if u]
    files = [f"{card['id']}{'' if i == 0 else f'-{i}'}.jpg" for i in range(len(urls))]
    return {'id': card['id'], 'name': card.get('name'), 'urls': urls, 'files': files,
            'tokens': [p['id'] for p in card.get('all_parts') or [] if p.get('component') == 'token']}


def _collection(identifiers):
    """Scryfall /cards/collection for up to 75 identifiers; [] if offline"""
    try:
        return scryfall._request(scryfall.API + '/cards/collection', {'identifiers': identifiers}).get('data', [])
    except (urllib.error.URLError, OSError, ValueError):
        return []


def _download(url, path):
    scryfall._wait()
    req = urllib.request.Request(url, headers={'User-Agent': scryfall.HEADERS['User-Agent']})
    with urllib.request.urlopen(req, timeout=30) as r: data = r.read()
    tmp = path + '.tmp'
    with open(tmp, 'wb') as f: f.write(data)
    os.replace(tmp, path)


def game_names(specs):
    """every card name in the decks of a game: [(key, cards, commander)] seat specs"""
    names = []
    for _, cards, cmd in specs:
        names += list(cards) + ([cmd] if cmd else [])
    return list(dict.fromkeys(names))


def prepare(names, progress=None, lookup=_collection, download=_download):
    """make sure every card in names (and the tokens they make) has its images on disk. progress(done, total) is
    called as it goes. Returns {card name: [image files]} for the cards it has images for, tokens under their name"""
    progress = progress or (lambda done, total: None)
    idx = load_index()
    want = [n for n in names if _key(n) not in idx]
    for i in range(0, len(want), 75):                       # look the cards up, 75 to a request
        batch = want[i:i + 75]
        found = {}
        for card in lookup([{'name': n.split(' // ')[0]} for n in batch]):
            found[_key(card['name'])] = card
            if card.get('card_faces'): found[_key(card['card_faces'][0]['name'])] = card
        for n in batch:
            card = found.get(_key(n)) or found.get(_key(n.split(' // ')[0]))
            if card is not None: idx[_key(n)] = _entry(card)
    tokens = list(dict.fromkeys(t for n in names for t in idx.get(_key(n), {}).get('tokens', [])))
    missing = [t for t in tokens if f'token:{t}' not in idx]
    for i in range(0, len(missing), 75):
        for card in lookup([{'id': t} for t in missing[i:i + 75]]):
            idx[f"token:{card['id']}"] = _entry(card)
    save_index(idx)
    entries = [(n, idx[_key(n)]) for n in names if _key(n) in idx] + \
              [(idx[f'token:{t}']['name'], idx[f'token:{t}']) for t in tokens if f'token:{t}' in idx]
    jobs = [(u, f) for _, e in entries for u, f in zip(e['urls'], e['files']) if not os.path.exists(os.path.join(DIR, f))]
    total, done = len(jobs), 0
    progress(0, total)
    os.makedirs(DIR, exist_ok=True)
    for url, f in jobs:
        try:
            download(url, os.path.join(DIR, f))
        except (urllib.error.URLError, OSError, ValueError):
            pass                                            # drawn from its name in the page
        done += 1
        progress(done, total)
    out = {}
    for name, e in entries:
        have = [f for f in e['files'] if os.path.exists(os.path.join(DIR, f))]
        if have: out.setdefault(name, have)
    return out


def cached(names):
    """{card name: image files} for the cards already on disk (no network)"""
    idx = load_index()
    out = {}
    for n in names:
        e = idx.get(_key(n))
        have = [f for f in (e or {}).get('files', []) if os.path.exists(os.path.join(DIR, f))]
        if have: out[n] = have
    return out
