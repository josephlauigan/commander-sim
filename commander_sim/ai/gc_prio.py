"""Cast priorities (0-90 scale, like every deck's prio function) for Game Changers and other high-impact cards,
computed from the game instead of flat numbers. Every deck's AI uses these: your decks' prio functions in ais.py and
cards/impl/*.py, and the outside decks' pool_ai.generic_prio.

The shared measure is the life margin: life above what the table could hit you for (ais.necro_floor: the biggest
board one opponent could attack with, plus 6, at least 10). Life-for-cards engines spend only that margin."""
import importlib

from commander_sim import engine as E
from commander_sim.engine import pval, total_mana, commander_out


def _A():
    return importlib.import_module('commander_sim.ais')


def life_margin(g, p):
    return p.life - _A().necro_floor(g, p)


def _mv(p, c):
    """the printed cost: cost_of's reductions (delve) value graveyard cards with card_worth, which can call these
    priorities back (Jeska's Will in the graveyard): a loop"""
    return c.generic + len(c.pips)


def unlocks(g, p, c, gain):
    """would casting c (a mana source that adds `gain` this turn) let you cast a spell from hand you can't now?"""
    avail = total_mana(g, p)
    after = avail - _mv(p, c) + gain
    return any(avail < _mv(p, x) <= after for x in p.hand if x is not c and not x.land)


# ------------------------------------------------------------------ life-for-cards engines
def citadel_prio(g, p, c):
    """Bolas's Citadel: worth it with life to spend above the threat floor; more margin, more cards off the top"""
    m = life_margin(g, p)
    return 0 if m < 8 else int(min(80, 40 + 2 * m))


def citadel_floor(g, p):
    """the life Citadel keeps when casting off the top"""
    return max(10, _A().necro_floor(g, p))


def adnaus_prio(g, p, c):
    """Ad Nauseam: about 2.5 life per card in the decks that run it; worth it for four or more cards"""
    cards = life_margin(g, p) / 2.5
    return 0 if cards < 4 else int(min(80, 40 + 4 * cards))


def one_ring_prio(g, p, c):
    """The One Ring: a turn of protection from everything matters most when the table could kill you; the burden
    costs life every upkeep, so less at low life"""
    v = 55
    if life_margin(g, p) < 6: v += 15
    if p.life <= 10: v -= 20
    return int(max(20, min(80, v)))


# ------------------------------------------------------------------ draw and mana
def rhystic_prio(g, p, c):
    """Rhystic Study: a tax on every opponent's spell, worth more with more opponents and the earlier it lands"""
    n = sum(1 for q in g.opps(p) if q.alive)
    early = max(0, 10 - g.round)
    return int(min(80, 30 + 6 * n + 2 * early))


def jeska_prio(g, p, c):
    """Jeska's Will: {R} per card in the biggest opposing hand (net of its own 3) and, with your commander out, three
    exiled cards to play this turn; the mana counts only if a spell in hand can use it"""
    most = max((len(q.hand) for q in g.opps(p) if q.alive), default=0)
    v = 25 + (12 if commander_out(p) else 0)
    gain = most - 3
    if gain >= 2 and unlocks(g, p, c, most): v += 6 * min(gain, 5)
    return int(min(80, v))


def ramp_prio(g, p, c, gain):
    """a fast-mana source (Chrome Mox, Mox Diamond, Grim Monolith): first when its mana casts something now, still
    good early, little late"""
    if unlocks(g, p, c, gain): return 82
    return 70 if p.turns <= 5 else 30


def seedborn_prio(g, p, c):
    """Seedborn Muse: untapping on every other turn pays for instants and abilities, and rocks that don't untap"""
    k = sum(1 for x in p.hand if x.instant or 'flash' in x.tags)
    k += sum(1 for m in p.perms if m.cd is not None and 'nountap' in m.cd.tags)
    return int(min(70, 40 + 6 * k))


# ------------------------------------------------------------------ boards and locks
def braids_prio(g, p, c):
    """Braids, Cabal Minion: everyone sacrifices each upkeep; good when you have spare permanents (tokens, undying
    creatures) and the opponents don't"""
    def spare(q):
        return sum(1 for m in q.perms if not m.phased and (m.token or getattr(m, 'undying', False)))
    mine = spare(p)
    theirs = min((sum(1 for m in q.perms if not m.phased and (m.creature or (m.cd is not None and 'A' in m.cd.types)))
                  for q in g.opps(p) if q.alive), default=0)
    return int(max(15, min(75, 30 + 8 * min(mine, 4) - 3 * min(theirs, 6))))


def shards_prio(g, p, c):
    """Aura Shards: each creature you play destroys an artifact or enchantment, so it's worth the opponents' targets
    and the creatures you're about to play"""
    tg = sum(pval(g, m) for q in g.opps(p) for m in q.perms
             if m.cd is not None and not m.phased and ('A' in m.cd.types or 'E' in m.cd.types))
    soon = sum(1 for x in p.hand if x.creature)
    return int(max(20, min(75, 30 + 3 * min(tg, 12) + 3 * min(soon, 3))))


def drannith_prio(g, p, c):
    """Drannith Magistrate: shuts off casting from anywhere but hand; worth the opponents' commanders still in the
    command zone and their graveyard-casting engines"""
    k = sum(1 for q in g.opps(p) if q.alive and q.cmd_in_zone)
    k += sum(1 for q in g.opps(p) if q.alive and any(m.cd is not None and m.cd.tags.get('breach') for m in q.perms))
    return int(min(76, 40 + 10 * k))


def agent_prio(g, p, c):
    """Opposition Agent: you control their searches; worth more against decks that tutor"""
    tutor_decks = sum(1 for q in g.opps(p) if q.alive and any(x.tags.get('tut') or 'seal' in x.tags
                                                             for x in q.library + q.hand))
    return int(min(70, 40 + 8 * tutor_decks))
