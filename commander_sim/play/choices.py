"""The smaller choices, made by the person in the human seat: mulligans, discards, sacrifices, tutors, basic lands
searched for, and scry / surveil. The engine calls these (through `human_choice`) where it would otherwise let the
AI decide; they do nothing in a game without a human seat."""
from commander_sim import engine as E
from commander_sim.play import legal
from commander_sim.play.controller import controller_of, Request
from commander_sim.play.human import choose

BASICS = ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest', 'Wastes')


def human_seat(g, p):
    """is p played by a person in this game?"""
    ctl = getattr(g, 'controllers', None)
    return bool(ctl) and p.key in ctl and getattr(ctl[p.key], 'human', False)


def card_label(c):
    cost = ('{' + str(c.generic) + '}' if c.generic else '') + ''.join('{' + x + '}' for x in c.pips)
    return f'{c.name} {cost}'.strip()


def pick_cards(g, p, cards, n, prompt, kind='choose'):
    """the person picks n of cards (fewer if there are fewer); no cancelling. Returns the picked cards"""
    left, picked = list(cards), []
    for i in range(min(n, len(left))):
        k = choose(g, p, kind, f'{prompt} ({i + 1} of {min(n, len(cards))})', [card_label(c) for c in left], cancel=None)
        picked.append(left.pop(k))
    return picked


# ------------------------------------------------------------------ mulligan
def mulligan(g, p, rng):
    """London mulligan with a free first mulligan (multiplayer Commander): draw seven, keep or shuffle back and draw
    seven again; on keeping after k mulligans, put k - 1 cards on the bottom"""
    ctl = controller_of(g, p)
    k = 0
    while True:
        p.library.extend(p.hand); p.hand = []; rng.shuffle(p.library)
        for _ in range(7): p.hand.append(p.library.pop())
        bottom = max(0, k - 1)
        note = '' if bottom == 0 else f' If you keep, you put {bottom} card(s) on the bottom.'
        if k >= 6: break
        ans = ctl.ask(Request('mulligan', f'Your opening hand ({7 - bottom} to keep).{note}',
                              choices=['keep', 'mulligan'], data={'hand': [card_label(c) for c in p.hand]}))
        if ans in (0, 'keep'): break
        k += 1
    bottom = max(0, k - 1)
    if bottom:
        for c in pick_cards(g, p, p.hand, bottom, 'Put a card on the bottom of your library'):
            p.hand.remove(c); p.library.insert(0, c)
    p.stats['mulls'] = k
    E.log(f'{E.NAME(p)} keeps {len(p.hand)} cards' + (f' after {k} mulligan(s)' if k else ''), g)


# ------------------------------------------------------------------ discards and sacrifices
def discard(g, p, n, why='Discard a card'):
    """p discards n cards of their choice"""
    cards = pick_cards(g, p, p.hand, n, why)
    if cards: E.discard_cards(g, p, cards)


def discard_to_hand_size(g, p, size=7):
    over = len(p.hand) - size
    if over > 0: discard(g, p, over, f'End of turn: discard down to {size}')


def sacrifice_creature(g, q, cands, why='Sacrifice a creature'):
    """q sacrifices one of cands (their creatures)"""
    if not cands: return
    k = choose(g, q, 'choose', why, [legal.describe_target(g, q, m) for m in cands], cancel=None)
    E.die(g, cands[k], 'sac')


# ------------------------------------------------------------------ searching the library
def search(g, p, pred, k=1, prompt='Search your library for a card', allow_none=True):
    """p searches their library for up to k cards matching pred; returns them (taken out of the library, not yet
    put anywhere). Then the library is shuffled"""
    pool = sorted({c.name: c for c in E.searchable(g, p) if pred(c)}.values(), key=lambda c: c.name)
    got = []
    for i in range(k):
        if not pool: break
        j = choose(g, p, 'search', f'{prompt}' + (f' ({i + 1} of {k})' if k > 1 else ''),
                   [card_label(c) for c in pool], cancel='find nothing' if allow_none else None)
        if j is None: break
        c = pool.pop(j); p.library.remove(c); got.append(c)
    g.rng.shuffle(p.library)
    return got


def pick_basic(g, p, prompt='Search your library for a basic land'):
    """one basic land card of p's choice (by name), taken out of the library, or None"""
    names = sorted({c.name for c in E.searchable(g, p) if c.land and c.name in BASICS})
    if not names: return None
    j = choose(g, p, 'search', prompt, names, cancel='find nothing')
    if j is None: return None
    c = next(c for c in E.searchable(g, p) if c.name == names[j])
    p.library.remove(c)
    return c


# ------------------------------------------------------------------ the top of the library
def scry(g, p, n, to='bottom'):
    """scry n (to='bottom') or surveil n (to='gy'): each card on top or away, in the order kept"""
    n = min(n, len(p.library))
    if n <= 0: return
    top = [p.library.pop() for _ in range(n)]              # top card first
    away_word = 'the bottom' if to == 'bottom' else 'your graveyard'
    keep, away = [], []
    for c in top:
        k = choose(g, p, 'choose', f"{'Scry' if to == 'bottom' else 'Surveil'}: {card_label(c)}",
                   ['keep on top', f'put on {away_word}'], cancel=None)
        (keep if k == 0 else away).append(c)
    if to == 'gy': p.gy.extend(away)
    else: p.library[:0] = away
    for c in reversed(keep): p.library.append(c)             # kept cards stay in the order they were
    E.log(f'  {E.NAME(p)} {"scries" if to == "bottom" else "surveils"} {n}: {len(keep)} on top', g)


# ------------------------------------------------------------------ modes and "you may"
EFFECT_WORDS = {'regrow': 'return a card from your graveyard to your hand', 'discard': 'a player discards',
                'destroy': 'destroy', 'exile': 'exile', 'damage': 'deal damage', 'draw': 'draw cards',
                'bounce': "return a permanent to its owner's hand", 'tap': 'tap', 'untap': 'untap',
                'counter_spell': 'counter a spell', 'token': 'create tokens', 'grant': 'give an ability',
                'sacrifice': 'a player sacrifices', 'gain_life': 'gain life', 'lose_life': 'lose life',
                'counters': 'put counters', 'pump': 'pump', 'mill': 'mill', 'search': 'search your library',
                'reanimate': 'return a creature to the battlefield'}


def describe_mode(ms):
    """'deal damage 2' / 'destroy (artifact)': a short label for one mode (a list of effects)"""
    out = []
    for e in ms:
        w = EFFECT_WORDS.get(e.get('do'), e.get('do', '?').replace('_', ' '))
        n = e.get('n')
        if isinstance(n, int) and n > 1 or (isinstance(n, int) and e.get('do') == 'damage'): w += f' {n}'
        what = (e.get('what') or e.get('to') or {}).get('filter', {}) or {}
        if what.get('type'): w += f" ({what['type'].replace('_', ' ')})"
        out.append(w)
    return ', then '.join(out)


def pick_modes(g, p, modes, k, name):
    """the person chooses k different modes"""
    left = list(range(len(modes))); got = []
    for i in range(k):
        j = choose(g, p, 'choose', f'{name}: choose a mode' + (f' ({i + 1} of {k})' if k > 1 else ''),
                   [describe_mode(modes[x]) for x in left], cancel=None)
        got.append(modes[left.pop(j)])
    return got


def yes_no(g, p, prompt):
    return choose(g, p, 'choose', prompt, ['yes', 'no'], cancel=None) == 0
