"""AI for outside decks (opponent pools): a generic cast priority built from card tags, plus per-deck
configuration (play style, tutor wish lists, key cards) registered by pools.register().

The four main decks never reach this module; they keep their hand-written priorities in ais.py.
"""
from engine import has, total_mana
import engine as E

DEFAULT_STYLE = {'temp': 1.0, 'aggression': 0.6, 'caution': 0.5}
CONFIG = {}          # deck key -> {'style': {...}, 'wish': [card names], 'key_cards': {name: priority}, ...}


def config(p):
    return CONFIG.get(p.key, {})


def style(p):
    return config(p).get('style', DEFAULT_STYLE)


def generic_prio(g, p, c):
    """0-90 cast priority from what the card is tagged to do. 0 = not cast proactively (held interaction,
    or no tags: the adaptive AI then falls back to the ability interpreter's value estimate)."""
    t = c.tags
    cfg = config(p)
    kc = cfg.get('key_cards', {})
    if c.name in kc: return kc[c.name]
    if c is p.cmd: return cfg.get('cmd_prio', 75) if p.turns >= cfg.get('cmd_turn', 2) else 0
    if 'rock' in t or 'dork' in t or 'lr' in t or 'fastmana' in t: return 85 if p.turns <= 5 else 38
    if 'chromemox' in t:
        spare = [x for x in p.hand if not x.land and 'A' not in x.types and set(x.pips) & set(p.ident)]
        return (82 if p.turns <= 5 else 30) if len(spare) >= 2 else 0
    if 'moxd' in t:
        n = sum(1 for x in p.hand if x.land)
        return (82 if p.turns <= 5 else 30) if n >= 2 or (n >= 1 and p.land_turn == p.turns) else 0
    if 'ctr' in t or 'rem' in t or 'wipe' in t or t.get('prot') or 'tide' in t: return 0   # held / cast by the response logic
    if any(k in t for k in ('rean', 'fill', 'yawg', 'avarice', 'mastery', 'crackle')) and not c.dsl: return 0
    if 'tokx' in t: return 50 if total_mana(E.CUR_G, p) >= 5 else 0
    if 'rhystic' in t or 'tithe' in t or 'eng' in t or 'necro' in t: return 64
    if 'crusade' in t or 'anth' in t or 'warleader' in t: return 60
    if any(k in t for k in ('tokup', 'tokatk', 'tok', 'spelltok', 'ping', 'kiln', 'spelldraw', 'drain', 'bartist')): return 62
    if t.get('tut'): return 58
    if 'draw' in t and (c.instant or c.sorcery): return 46
    if 'draw' in t: return 52
    if 'treas' in t or 'mktok' in t or 'drainetb' in t or 'edictetb' in t: return 50
    if c.creature: return 42 + min(16, 2 * c.pow) + (4 if 'fly' in t else 0)
    if t.get('prot') == 'boots' or 'sac' in t: return 40
    if c.dsl: return 0                       # interpreter value decides
    if 'pumpall' in t: return 0              # combat trick: no proactive value
    return 0


def tutor_pick(g, p, kind, ok):
    """a deck's wish list first (combo pieces it is missing), else None (caller falls back to priority)"""
    wish = config(p).get('wish')
    if not wish: return None
    names = {c.name for c in p.library if ok(c)}
    have = {m.cd.name for m in p.perms if m.cd is not None} | {c.name for c in p.hand}
    for w in (wish(g, p) if callable(wish) else wish):
        if w in names and w not in have: return w
    return None
