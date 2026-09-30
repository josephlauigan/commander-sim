"""The rules check for the human seat: is this move legal right now? Each check returns None (legal) or a one-line
reason to show the player. Nothing here changes the game."""
from commander_sim import engine as E
from commander_sim.play import mana

MAIN_STEPS = ('main1', 'main2')


def land_drops(g, p):
    """land drops p has this turn (Exploration, Azusa and similar add more)"""
    return 1 + (E.CI.total(g, 'extra_lands', p) if g.hooks else 0) + getattr(p, 'extra_land_now', 0)


def sorcery_timing(g, p):
    """None if p may do sorcery-speed things now (its own main phase), else why not"""
    if g.active is not p: return "It isn't your turn."
    if getattr(g, 'step', None) not in MAIN_STEPS: return 'You can only do that in one of your main phases.'
    return None


def check_land(g, p, c):
    if c not in p.hand: return f"{c.name} isn't in your hand."
    if not c.land: return f"{c.name} isn't a land."
    why = sorcery_timing(g, p)
    if why: return why.replace('do that', 'play a land')
    if getattr(p, 'lands_played', 0) >= land_drops(g, p):
        return "You've already played a land this turn." if land_drops(g, p) == 1 else "You've used all your land drops this turn."
    return None


def instant_speed(c):
    return c.instant or 'flash' in c.tags or 'flash' in (c.kws or ())


def check_cast(g, p, c, zone='hand'):
    """can p cast c from zone ('hand' or 'cmd') now? Checks where it is, timing, locks and the mana in the pool"""
    if zone == 'hand' and c not in p.hand: return f"{c.name} isn't in your hand."
    if zone == 'cmd' and not (c is p.cmd and p.cmd_in_zone): return 'Your commander is not in the command zone.'
    if c.land: return f'{c.name} is a land: play it as your land drop instead.'
    if not instant_speed(c):
        why = sorcery_timing(g, p)
        if why: return why.replace('do that', f'cast {c.name}')
    if not E.castable(g, p, c, zone): return f"Something on the battlefield stops you casting {c.name} right now."
    gen, pips = E.cost_of(p, c)
    why = mana.cost_problem(g, p, gen, pips)
    if why: return f"Can't cast {c.name}. {why}"
    return None
