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


# ------------------------------------------------------------------ Heliod, Sun-Crowned (mono-white stax)
def heliod_prio(g, p, c):
    """rocks and taxes early like GAA; Heliod on turn three; Walking Ballista is kept for the combo (the combo
    casts it with Heliod out); the protection creatures once Heliod or a stax piece needs guarding"""
    if c is p.cmd: return 84 if p.turns >= 3 else 0
    if c.name == 'Walking Ballista': return 0
    if 'rock' in c.tags: return 86 if p.turns <= 5 else 45
    if c.name in STAX_EARLY: return 76 if p.turns <= 6 else 55
    if c.name in PILLOW: return 60 + min(25, int(opp_power(g, p) * 1.5))
    if c.name in ('Mother of Runes', 'Giver of Runes'): return 70
    return None


def heliod_wish(g, p, *a):
    import impl_combos
    if not on_bf(p, 'Walking Ballista') and not any(c.name == 'Walking Ballista' for c in p.hand):
        return ['Walking Ballista', 'Recruiter of the Guard', 'Ranger-Captain of Eos']
    if opp_power(g, p) >= 10 and not any(on_bf(p, n) for n in PILLOW): return list(PILLOW)
    return impl_combos.missing_pieces(g, p) or ['Rule of Law', 'Smothering Tithe', 'Drannith Magistrate', 'Thalia, Guardian of Thraben']


# ------------------------------------------------------------------ Yawgmoth, Thran Physician
UNDYING = ("Geralf's Messenger", 'Butcher Ghoul', 'Young Wolf', 'Nether Traitor')


def yawg_wish(g, p, *a):
    """the loop needs Yawgmoth plus two undying creatures (or Mikaeus and fodder), then a drain payoff"""
    import impl_combos
    und = sum(1 for m in p.perms if m.creature and m.cd is not None and 'undying' in m.cd.kws)
    mik = on_bf(p, 'Mikaeus, the Unhallowed') or any(c.name == 'Mikaeus, the Unhallowed' for c in p.hand)
    out = []
    if not on_bf(p, 'Yawgmoth, Thran Physician') and not p.cmd_in_zone: out.append('Yawgmoth, Thran Physician')
    if not mik and und < 2: out += ['Mikaeus, the Unhallowed'] + list(UNDYING)
    if impl_combos.drain_payoff(p) is None: out += list(impl_combos.DRAIN_PAYOFFS)
    return out + impl_combos.missing_pieces(g, p)
