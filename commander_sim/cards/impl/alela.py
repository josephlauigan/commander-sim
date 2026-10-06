"""Alela, Artful Provocateur (Avery's Esper Faerie Flyers deck, key `alela`, decklists/Avery/alela-esper-faeries.md).

- Alela: flying, deathtouch, lifelink; other creatures you control with flying get +1/+0 (an anthem with a flying
  filter); whenever you cast an artifact or enchantment spell, create a 1/1 blue Faerie creature token with flying.
- The AI: the outside decks' generic priorities (it shares their generic plays), plus the Faerie each artifact or
  enchantment makes while Alela is out (more with Wispdrinker Vampire draining and Tetsuko Umezawa unblocking them).
"""
import importlib
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim.cards import cardimpl as CI
from commander_sim.cards.cardimpl import on
from commander_sim.cards.pool_cards import card, note

ALELA = 'Alela, Artful Provocateur'

card(ALELA, 'leg pow=2 tgh=3 fly dt lifelink', dsl=[
    {'static': 'anthem', 'pow': 1, 'tgh': 0,
     'filter': {'type': 'creature', 'controller': 'you', 'other': True, 'keyword': 'flying'}}])


def alela_perm(p):
    return next((m for m in p.perms if m.cd is not None and m.cd.name == ALELA and not m.phased), None)


@on(ALELA, 'cast')
def _alela_cast(g, src, caster, c):
    """whenever you cast an artifact or enchantment spell: a 1/1 blue Faerie with flying"""
    o = src.owner
    if caster is not o or c.land or not ('A' in c.types or 'E' in c.types): return
    if not trigger_window(g, o, src, 'a 1/1 blue Faerie with flying', imp=3): return
    o.milestone.setdefault('faerie', o.turns)
    o.stats['alela_faeries'] += 1
    make_tokens(g, o, 1, 1, 1, fly=True, color='U', types=('faerie',))
    log(f'    Alela: {NAME(o)} creates a 1/1 Faerie with flying', g)


note(ALELA, 'Full', 'flying, deathtouch, lifelink; your other fliers get +1/+0; each artifact or enchantment spell you '
     'cast makes a 1/1 blue Faerie with flying')


# ================================================================== the AI
def faerie_bonus(g, p):
    """what one more Faerie is worth to p now (on the 0-90 priority scale): a 2/1 flier with Alela out, more with
    Wispdrinker Vampire (each one drains the table) and Tetsuko Umezawa (each one is unblockable)"""
    if alela_perm(p) is None: return 0
    names = {m.cd.name for m in p.perms if m.cd is not None and not m.phased}
    return 10 + (6 if 'Wispdrinker Vampire' in names else 0) + (4 if 'Tetsuko Umezawa, Fugitive' in names else 0)


def alela_prio(g, p, c):
    """the outside decks' generic priority, plus a Faerie for each artifact or enchantment spell while Alela is out"""
    from commander_sim.ai import pool_ai
    v = pool_ai.generic_prio(g, p, c) or 0
    if c is p.cmd: return max(v, 85)
    if not v and c.dsl and E.DSLMOD is not None: v = int(E.DSLMOD.card_value(g, p, c) * 10)   # untagged: its value
    if v and not c.land and ('A' in c.types or 'E' in c.types): v += faerie_bonus(g, p)
    return min(90, v)


CI.alela_prio = alela_prio
