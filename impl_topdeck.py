"""Top-of-library manipulation for the opponent pools: scry, surveil, cantrips, Sensei's Divining Top, Scroll Rack,
Sylvan Library, and the stacking Yuriko decks do (the highest mana value on top for its reveal).

desire(g, p, c) is how much p wants to draw c next:
  - a missing combo piece or wish-list card beats everything,
  - a land when p is short on lands, a spell when it is flooded,
  - otherwise the card's worth to p's AI.
Yuriko decks (config top_pref 'mv') want the highest mana value on top while Yuriko is available.
p.library[-1] is the top card.
"""
from engine import *
import engine as E
import cardimpl as CI
from cardimpl import on
from pool_cards import card, note
import impl_common as IC


def mv_mode(p):
    """Yuriko decks stack high mana values on top once Yuriko can connect"""
    import pool_ai
    if pool_ai.config(p).get('top_pref') != 'mv': return False
    return p.cmd_in_zone or any(m.is_cmd for m in p.perms)


def land_need(p):
    lands = len(p.lands) + sum(1 for x in p.hand if x.land)
    return 2 if lands < 4 else 1 if lands < 6 else 0


def desire(g, p, c):
    if mv_mode(p): return c.cmc * 10 + (5 if c.land and land_need(p) == 2 else 0)
    import pool_ai
    wish = pool_ai.wish_list(g, p)
    if c.name in wish: return 200 - wish.index(c.name)
    if c.land: return (70 if land_need(p) == 2 else 30 if land_need(p) == 1 else 5)
    return card_worth(g, p, c)


KEEP = 35            # scry: cards at least this desirable stay on top


def arrange(g, p, cards):
    """put cards on top of the library, the most desirable on top"""
    p.library.extend(sorted(cards, key=lambda c: desire(g, p, c)))


def look(p, n):
    return [p.library.pop() for _ in range(min(n, len(p.library)))]


def scry(g, p, n, to='bottom'):
    """scry n (to='bottom') or surveil n (to='gy'): keep the good cards on top in the best order"""
    if not E.POOL_RULES or n <= 0: return
    top = look(p, n)
    keep = [c for c in top if desire(g, p, c) >= KEEP]
    rest = [c for c in top if c not in keep]
    if to == 'gy': p.gy.extend(rest)
    else: p.library[:0] = rest
    arrange(g, p, keep)


def put_back(g, p, n, exclude=()):
    """put n cards from hand on top (Brainstorm, Scroll Rack): the least wanted, or in Yuriko mode the biggest"""
    for _ in range(n):
        rest = [x for x in p.hand if x not in exclude]
        if not rest: return
        if mv_mode(p): x = max(rest, key=lambda c: (c.cmc >= 6, c.cmc))
        else: x = min(rest, key=lambda c: desire(g, p, c))
        p.hand.remove(x); p.library.append(x)
    if mv_mode(p) and n > 1:                  # the biggest last so it is revealed first
        k = min(n, len(p.library))
        top = p.library[-k:]; del p.library[-k:]
        p.library.extend(sorted(top, key=lambda c: c.cmc))


# ------------------------------------------------------------------ cantrips
def _spell(name, prio, status, types='S', tags=''):
    return IC.spell(name, prio=prio, tags=tags, types=types, status=status)


@_spell('Brainstorm', lambda g, p, c: 40 if len(p.library) > 10 else 0,
        ('Full', 'draw three, then put back the two least wanted cards (Yuriko: the two biggest, for the reveal)'),
        types='I')
def _brainstorm(g, p, c, ctx):
    draw(g, p, 3)
    put_back(g, p, 2, exclude=(c,))


@_spell('Ponder', lambda g, p, c: 42 if len(p.library) > 10 else 0,
        ('Full', 'look at the top three: keep them in the best order, or shuffle if none is wanted; then draw'))
def _ponder(g, p, c, ctx):
    top = look(p, 3)
    if top and max(desire(g, p, x) for x in top) < KEEP:
        p.library.extend(top); g.rng.shuffle(p.library)
    else:
        arrange(g, p, top)
    draw(g, p, 1)


@_spell('Preordain', lambda g, p, c: 42 if len(p.library) > 10 else 0, ('Full', 'scry 2, then draw'))
def _preordain(g, p, c, ctx):
    scry(g, p, 2); draw(g, p, 1)


@_spell('Opt', lambda g, p, c: 38 if len(p.library) > 10 else 0, ('Full', 'scry 1, then draw'), types='I')
def _opt(g, p, c, ctx):
    scry(g, p, 1); draw(g, p, 1)


@_spell('Consider', lambda g, p, c: 38 if len(p.library) > 10 else 0, ('Full', 'surveil 1, then draw'), types='I')
def _consider(g, p, c, ctx):
    scry(g, p, 1, 'gy'); draw(g, p, 1)


@_spell('Serum Visions', lambda g, p, c: 40 if len(p.library) > 10 else 0, ('Full', 'draw, then scry 2'))
def _visions(g, p, c, ctx):
    draw(g, p, 1); scry(g, p, 2)


# ------------------------------------------------------------------ permanents
@on("Sensei's Divining Top", 'upkeep')
def _top(g, src, p):
    """{1}: look at the top three and rearrange them (each upkeep, before the draw)"""
    if p is not src.owner or src.tapped or not can_pay(g, p, 1, ''): return
    pay(g, p, 1, '')
    arrange(g, p, look(p, 3))


@on("Sensei's Divining Top", 'options')
def _top_draw(g, src, p, s, post):
    """{T}: draw a card and put Top on top of the library: used when the top card is wanted now (a combo piece,
    or with Bolas's Citadel, where Top can be cast again for 1 life)"""
    if p is not src.owner or src.tapped or src.sick is None or not p.library: return []
    c = p.library[-1]
    citadel = any(m.cd is not None and 'citadel' in m.cd.tags for m in p.perms)
    if not citadel and desire(g, p, c) < 150: return []

    def go():
        if src not in p.perms or src.tapped: return False
        draw(g, p, 1)
        leave(g, src); p.library.append(src.cd)
        log(f'  {NAME(p)} draws with Sensei\'s Divining Top (Top goes on top)', g)
        return True
    return [(4.0 if citadel else 3.0, "Sensei's Divining Top draw", go)]


card("Sensei's Divining Top", '', types='A', dsl=[])
note("Sensei's Divining Top", 'Full', 'rearranges the top three each upkeep; taps to draw when the top card is a key '
     'piece or Bolas\'s Citadel can recast it (then Top goes on top)')


@on('Scroll Rack', 'upkeep')
def _rack(g, src, p):
    """{1}, {T}: exile any number of cards from hand face down, put that many from the top into hand, then put
    the exiled cards on top in any order: swaps the least wanted cards (Yuriko: puts the biggest on top)"""
    if p is not src.owner or src.tapped or not can_pay(g, p, 1, '') or not p.hand or not p.library: return
    if mv_mode(p):
        big = sorted([c for c in p.hand if c.cmc >= 5], key=lambda c: c.cmc)
        if not big: return
        pay(g, p, 1, '')
        top = look(p, len(big))
        for c in big: p.hand.remove(c)
        p.hand.extend(top); p.library.extend(big)
        log(f'  {NAME(p)} uses Scroll Rack: {big[-1].name} on top', g)
        return
    worst = sorted(p.hand, key=lambda c: desire(g, p, c))[:2]
    top = p.library[-2:]
    if not top or min(desire(g, p, c) for c in top) <= max(desire(g, p, c) for c in worst): return
    pay(g, p, 1, '')
    top = look(p, len(worst))
    for c in worst: p.hand.remove(c)
    p.hand.extend(top); p.library.extend(worst)


card('Scroll Rack', '', types='A', dsl=[])
note('Scroll Rack', 'Full', 'each upkeep: swaps the least wanted cards in hand for the top cards (Yuriko: puts the '
     'biggest cards on top for the reveal)')


@on('Sylvan Library', 'draw')
def _library(g, src, p):
    """draw step: draw two more, then for each keep it for 4 life or put it back (the most wanted on top)"""
    if p is not src.owner or g.active is not p or not once_per_turn(g, p, f'sylvan{id(src)}'): return
    before = list(p.hand)
    draw(g, p, 2)
    new = [c for c in p.hand if c not in before][:2]
    for c in sorted(new, key=lambda c: -desire(g, p, c)):
        if p.life >= 24 and desire(g, p, c) >= 55:
            lose_life(g, p, 4, p); log(f'    {NAME(p)} pays 4 life for a Sylvan Library card', g)
        else:
            p.hand.remove(c); p.library.append(c)


card('Sylvan Library', 'eng=1', types='E', dsl=[])
note('Sylvan Library', 'Full', 'draws two extra each draw step; keeps a strong card for 4 life when above 24, puts '
     'the rest back in the best order')
