"""Deck-specific play plans for the opponent pools (priorities, wish lists, holds), for decks whose plan the generic
pool AI can't see: what to cast first, what tutors fetch, what to keep mana up for. Registered in pool_decks.CONFIG.
A prio function returns a 0-90 cast priority, or None to fall back to the generic priority.
"""
import engine as E
from engine import pval, total_mana, epow


def on_bf(p, name):
    return any(m.cd is not None and m.cd.name == name and not m.phased for m in p.perms)


def board(p):
    return [m for m in p.perms if m.creature and not m.phased]


# ------------------------------------------------------------------ Chulane, Teller of Tales
BOUNCERS = ('Whitemane Lion', 'Shrieking Drake', 'Kor Skyfisher', "Man-o'-War")
FINISHERS = ('Craterhoof Behemoth', 'Avenger of Zendikar')


def chulane_prio(g, p, c):
    """with Chulane out every creature spell draws and ramps: creatures first, cheap self-bouncers recast; the
    finishers once the board or the land count can end the game"""
    chulane = any(m.is_cmd for m in p.perms)
    if c.name == 'Craterhoof Behemoth':
        return 80 if len(board(p)) >= 6 else 20
    if c.name == 'Avenger of Zendikar':
        return 78 if len(p.lands) >= 7 else 45
    if chulane and c.creature:
        if c.name in BOUNCERS: return 76
        return 70 + min(8, c.cmc)
    if c.name == 'Aluren':
        return 75 if chulane or p.cmd_in_zone else 40
    return None


def chulane_wish(g, p, *a):
    import impl_combos
    miss = impl_combos.missing_pieces(g, p)
    if len(board(p)) >= 6: return ['Craterhoof Behemoth'] + miss
    if miss: return miss
    return ['Avenger of Zendikar', 'Consecrated Sphinx', 'Craterhoof Behemoth']


# ------------------------------------------------------------------ Grand Arbiter Augustin IV (stax)
STAX_EARLY = ('Sphere of Resistance', 'Thorn of Amethyst', 'Thalia, Guardian of Thraben', 'Rule of Law', 'Drannith Magistrate',
              'Deafening Silence', 'Ethersworn Canonist', 'Archon of Emeria', 'Glowrider', 'Vryn Wingmare', 'Lavinia, Azorius Renegade',
              'Spirit of the Labyrinth', 'Thalia, Heretic Cathar', 'Grand Abolisher')
PILLOW = ('Ghostly Prison', 'Propaganda')


def opp_power(g, p):
    return sum(epow(g, m) for q in g.opps(p) for m in q.perms if m.creature and not m.phased)


def gaa_prio(g, p, c):
    """rocks, then the taxes and spell limits while opponents are still developing; pillowfort as soon as creatures
    show up; the commander (a tax piece that also discounts your spells) early"""
    if c is p.cmd: return 84 if p.turns >= 3 else 0
    if 'rock' in c.tags: return 86 if p.turns <= 5 else 45
    if c.name in PILLOW:
        return 60 + min(25, int(opp_power(g, p) * 1.5))
    if c.name in STAX_EARLY:
        return 76 if p.turns <= 5 else 55
    return None


def gaa_wish(g, p, *a):
    import impl_combos
    if opp_power(g, p) >= 10 and not any(on_bf(p, n) for n in PILLOW): return list(PILLOW)
    return impl_combos.missing_pieces(g, p) or list(PILLOW)
