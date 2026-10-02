"""What the setup screen shows: your four decks (commander, bracket, Game Changers, how they do in the simulations) and
the five tiers (their decks, and how the chosen deck does against each). Read from the repository's own files:
decklists/mine/*.md, decklists/pool/, the card cache's Game Changer flags, and the look-ahead results table in
decklists/pool/pool-results.md (section 0d)."""
import os
import re

from commander_sim import ROOT, ais, pools
from commander_sim.decks import DECKS, P as MINE_DIR

RESULTS = os.path.join(ROOT, 'decklists', 'pool', 'pool-results.md')
FILES = {'seph': 'sephiroth-phyrexian-reanimator.md', 'veyran': 'veyran-izzet-spellslinger.md',
         'sauron': 'sauron-grixis-amass.md', 'marchesa': 'marchesa-grixis-recursion.md'}
ROW_NAMES = {'sephiroth': 'seph', 'veyran': 'veyran', 'sauron': 'sauron', 'marchesa': 'marchesa'}


def display_name(key):
    """the commander as the deck file names it (Sephiroth's deck plays Atraxa, Grand Unifier as Sephiroth, the Savior)"""
    path = os.path.join(MINE_DIR, FILES[key])
    if os.path.exists(path):
        with open(path, encoding='utf-8') as f: txt = f.read()
        m = re.search(r'^\*\*Commander \(1\)\.\*\*\s*(.+?)\s*(?:\(.*\))?\s*$', txt, re.M)
        if m: return m.group(1)
    return ais.CMDS[key]


def win_rates(path=RESULTS):
    """{deck key: {tier: win %}} from the results table of your decks against the tiers (look-ahead AI)"""
    if not os.path.exists(path): return {}
    with open(path, encoding='utf-8') as f: txt = f.read()
    m = re.search(r'^### 0d\..*?\n(.*?)(?=^#)', txt, re.M | re.S)
    out = {}
    for line in (m.group(1) if m else '').splitlines():
        cells = [c.strip() for c in line.strip().strip('|').split('|')]
        key = ROW_NAMES.get(cells[0].lower()) if cells else None
        if key is None or len(cells) < 6: continue
        rates = [re.search(r'([\d.]+)%', c) for c in cells[1:6]]
        if all(rates): out[key] = {t: float(r.group(1)) for t, r in zip(pools.TIERS, rates)}
    return out


def game_changers(cards):
    from commander_sim.cards import scryfall
    cache = scryfall.load_cache()
    return sorted({c for c in cards if (cache.get(c.strip().lower()) or {}).get('game_changer')})


def bracket(key, gcs):
    """the bracket the deck file states, else 4 above three Game Changers and 3 otherwise"""
    path = os.path.join(MINE_DIR, FILES[key])
    if os.path.exists(path):
        with open(path, encoding='utf-8') as f: txt = f.read()
        m = re.search(r'^\**Bracket (\d)', txt, re.M)
        if m: return int(m.group(1))
    return 4 if len(gcs) > 3 else 3


def tier_label(tier):
    """'T3 · High B3' from the tier's folder name (t3-high-b3)"""
    d = pools.load_pool(tier)
    words = d[0].folder.split('-')[1:] if d else []
    text = ' '.join(w.upper() if re.fullmatch(r'b\d', w) else w.title() for w in words).replace(' Low', ' / Low')
    return f'{tier.upper()} · {text}'


def catalog():
    rates = win_rates()
    decks = []
    for key in FILES:
        gcs = game_changers(DECKS[key])
        r = rates.get(key)
        decks.append({'key': key, 'commander': ais.CMDS[key], 'name': display_name(key), 'bracket': bracket(key, gcs), 'game_changers': gcs,
                      'win_rates': r, 'average': round(sum(r.values()) / len(r), 1) if r else None})
    tiers = [{'key': t, 'label': tier_label(t),
              'decks': [{'key': d.key, 'commander': d.commander, 'name': pools.short_name(d)} for d in pools.load_pool(t)]}
             for t in pools.TIERS]
    return {'decks': decks, 'tiers': tiers}


def commanders():
    """every commander on the setup screen (for their images)"""
    c = catalog()
    return [d['commander'] for d in c['decks']] + [x['commander'] for t in c['tiers'] for x in t['decks']]
