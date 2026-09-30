"""The human seat's turn: priority in the main phases, and the actions a player takes (tap for mana, play a land,
cast a spell, pass). Each action is checked by legal.py first; an illegal one is refused with the reason and the
player keeps priority.

Actions are dicts, the same from the text client and (later) the browser:
  {'do': 'tap', 'source': id, 'colour': 'B'}      a mana source from mana.sources()
  {'do': 'land', 'card': i}                       the i-th card in hand as your land drop
  {'do': 'cast', 'card': i}                       the i-th card in hand
  {'do': 'cast', 'zone': 'cmd'}                   your commander from the command zone
  {'do': 'pass'}
"""
from commander_sim import engine as E, ais
from commander_sim.play import mana, legal
from commander_sim.play.controller import controller_of, Request
from commander_sim.play.view import build_view

STEP_NAMES = {'main1': 'Main phase 1', 'main2': 'Main phase 2'}


def is_human(g, p):
    ctl = getattr(g, 'controllers', None)
    return bool(ctl) and p.key in ctl and getattr(ctl[p.key], 'human', False)


def human_main(g, p, post):
    """p's main phase, played by the person: act until they pass"""
    ctl = controller_of(g, p)
    while not g.over and p.alive:
        step = STEP_NAMES.get(getattr(g, 'step', None), 'Your turn')
        act = ctl.ask(Request('priority', f'{step}: you have priority', data={'view': build_view(g, p.key)}))
        if not isinstance(act, dict): act = {}
        if act.get('do') == 'pass': return
        why = apply(g, p, act)
        if why: ctl.tell('invalid', why)


def _hand_card(p, act):
    i = act.get('card')
    if not isinstance(i, int) or not 0 <= i < len(p.hand): return None
    return p.hand[i]


def apply(g, p, act):
    """carry out one action; returns None, or the reason it isn't legal (nothing changes then)"""
    do = act.get('do')
    if do == 'tap':
        return mana.tap(g, p, act.get('source', -1), act.get('colour'))
    if do == 'land':
        c = _hand_card(p, act)
        if c is None: return 'There is no such card in your hand.'
        why = legal.check_land(g, p, c)
        if why: return why
        p.hand.remove(c)
        ais.play_land_card(g, p, c)
        E.check_state(g)
        return None
    if do == 'cast':
        zone = 'cmd' if act.get('zone') == 'cmd' else 'hand'
        c = p.cmd if zone == 'cmd' else _hand_card(p, act)
        if c is None: return 'There is no such card in your hand.'
        why = legal.check_cast(g, p, c, zone)
        if why: return why
        return cast(g, p, c, zone)
    return 'Unknown action.'


def cast(g, p, c, zone):
    """cast c (already checked legal): extra costs, pay from the pool, then the engine casts and resolves it. Choices
    not yet made by the player (X, a creature to sacrifice, targets) are made by the AI and reported as automatic"""
    from commander_sim.ai import brain
    ctl = controller_of(g, p)
    gen, pips = E.cost_of(p, c)
    fodder = None
    if 'needsac' in c.tags:                                  # Diabolic Intent: sacrifice a creature
        fodder = brain.spare_creature(g, p)
        if fodder is None: return f'{c.name} needs a creature to sacrifice as it is cast, and you have none you can spare.'
    if c.dsl and not E.additional_cost(g, p, c, dry=True): return f"You can't pay {c.name}'s additional cost."
    why = mana.pay_from_pool(g, p, gen, pips)
    if why: return f"Can't cast {c.name}. {why}"
    ctx = {}
    if 'tokx' in c.tags or 'xtutor' in c.tags:              # X: everything left in the pool (automatic for now)
        x = mana.pool_of(p).total(); mana.pool_of(p).empty(); ctx['x'] = x
        if 'xtutor' in c.tags: g.last_x = x
        ctl.tell('auto', f'X = {x} (the rest of your mana pool)')
    if c.dsl: E.additional_cost(g, p, c)
    if fodder is not None:
        E.die(g, fodder, 'sac'); ctl.tell('auto', f'Sacrificed {fodder.name} for {c.name}')
    if 'phyU' in c.tags and 'U' in c.pips and pips.count('U') < c.pips.count('U'):
        E.lose_life(g, p, 2, p)                              # {U/P} paid with 2 life
    E.cast_card(g, p, c, zone, ctx)
    return None
