"""Card rules first written for the Marchesa deck (removed 2026-10-07), kept for the cards other lists still run:
Marchesa, the Black Rose (Jodah's 99), Coalition Relic, Deep Analysis, Notion Thief's flash, Hellkite Tyrant's
alternate win, Accursed Marauder, and the shared helpers other modules import (best_steal and steal, card_etb_value).
Hooks here are live in every game, keyed by card name.
"""
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim.cards import cardimpl as CI
from commander_sim.cards.cardimpl import on
from commander_sim.cards.pool_cards import card, note
from commander_sim.cards.impl import common as IC
from commander_sim.cards.impl import mine as IM


def full(name, text): note(name, 'Full', text)


# ================================================================== Marchesa, the Black Rose
MARCHESA = 'Marchesa, the Black Rose'


def _you(g, p):
    """practice mode: the human seat's versions of these cards' choices (play/cards.py), or None for the AI"""
    return importlib.import_module('commander_sim.play.cards') if E.human_choice(g, p) is not None else None


def marchesa_out(p):
    return has(p, 'marchesa')


@on(MARCHESA, 'etb')
def _marchesa_etb(g, src, p, m):
    if m is src: g.marchesa_on = True               # the engine starts checking deaths for her trigger


def marchesa_dies(g, p, m):
    """creature m that p controlled died: Marchesa's trigger when p controls her, when she is the creature, or when
    she died in the same event (a wipe: her ability looks back in time)"""
    is_m = 'marchesa' in m.cd.tags
    batch = getattr(g, 'batch', None)
    if is_m and batch is not None: p.marchesa_batch = batch
    if m.plus <= 0: return
    if not (is_m or marchesa_out(p) or (batch is not None and getattr(p, 'marchesa_batch', None) is batch)): return
    cd = m.phys or m.cd
    if cd is m.orig.cmd: m.orig.cmd_pending = True     # back from the graveyard at end step: don't recast her meanwhile
    g.marchesa_due = (getattr(g, 'marchesa_due', None) or []) + [(p, cd, m.orig)]
    log(f'    Marchesa: {cd.name} returns at the beginning of the next end step', g)


def marchesa_return(g):
    """at the beginning of the end step: the cards Marchesa's triggers are waiting on come back (if still there)"""
    due, g.marchesa_due = g.marchesa_due, []
    for p, cd, owner in due:
        if g.over or not p.alive: continue
        if cd is owner.cmd:
            owner.cmd_pending = False
            if not owner.cmd_in_zone: continue           # she went to the graveyard, not the command zone
            owner.cmd_in_zone = False
        elif cd in owner.gy: owner.gy.remove(cd)
        else: continue                                   # exiled, reanimated or returned by something else meanwhile
        m = enter(g, p, cd, orig=owner)
        if cd is owner.cmd and owner is p: m.is_cmd = True
        p.stats['marchesa_returns'] += 1
        log(f'  Marchesa returns {cd.name} to the battlefield ({NAME(p)})', g)
    check_state(g)


def returns(g, m):
    """m (a creature with a +1/+1 counter) comes back if it dies now"""
    p = m.owner
    if m.token or m.plus <= 0 or not (marchesa_out(p) or 'marchesa' in m.cd.tags): return False
    return not (g.hooks and CI.total(g, 'no_graveyard', p))


def marchesa_sac_worth(g, m, v):
    """a creature Marchesa will return costs little to sacrifice, and re-buys what it does when it enters"""
    if not returns(g, m): return v
    return 0.15 * v + 0.3 * m.plus + (0.5 * len(IC.auras_on(g, m)) if getattr(g, 'auras', None) else 0) \
        - card_etb_value(g, m.owner, m.phys or m.cd)


CI.marchesa_dies = marchesa_dies
CI.marchesa_return = marchesa_return
CI.marchesa_sac_worth = marchesa_sac_worth
card(MARCHESA, 'leg human wizard pow=3 tgh=3 marchesa', types='C', dsl=[], kws={'dethrone'})
full(MARCHESA, 'dethrone, and other creatures you control have dethrone; a creature you control with a +1/+1 counter '
     'that dies (a stolen one too, and those dying in the same wipe as Marchesa) returns under your control at the '
     'beginning of the next end step; she returns herself from the command zone the same way')


# ------------------------------------------------------------------ what re-entering is worth (sacrifice decisions)
def card_etb_value(g, p, c):
    """what creature card c does for p when it enters the battlefield, on the board as it is now"""
    if c is None or not c.creature: return 0.0
    opps = g.opps(p)
    n = c.name
    if n == 'Accursed Marauder':
        return 1.2 * sum(1 for q in opps if any(m.creature and not m.token and not m.phased for m in q.perms))
    if n == 'Burglar Rat': return 0.8 * sum(1 for q in opps if q.hand)
    if n == 'Deepglow Skate': return 0.5 * sum(m.plus for m in p.perms if m.plus > 0)
    if n == 'Triskelion': return 3.0
    v = sum(x for k, x in IM.ETB_VALUE.items() if k in c.tags)
    if 'draw' in c.tags: v += 1.5 * int(c.tags['draw'])
    if CI.live(n) and 'etb' in CI.HOOKS[n]: v = max(v, 1.5)
    if c.dsl and any(a.get('type') == 'triggered' and a.get('event') == 'etb' for a in c.dsl): v = max(v, 1.5)
    return v


CI.card_etb_value = card_etb_value


# ================================================================== cards
# ------------------------------------------------------------------ gaining control (Ray of Command, Abduction)
def best_steal(g, p, spell):
    tg = [m for m in legal_targets(g, p, 'steal', 'c', spell=spell) if not m.phased]
    return max(tg, key=lambda m: pval(g, m)) if tg else None


def steal(g, p, m, until_eot):
    q = m.owner
    q.perms.remove(m); m.owner = p; m.attached = None; p.perms.append(m)
    g.bf_ver = getattr(g, 'bf_ver', 0) + 1
    if until_eot:
        m.tapped = False; m.sick = False
        p.borrowed = getattr(p, 'borrowed', []) + [m]
    else: m.sick = True
    log(f'    {NAME(p)} gains control of {m.name} ({NAME(q)})', g)


# ------------------------------------------------------------------ Coalition Relic
@on('Coalition Relic', 'end_step')
def _relic_charge(g, src, p):
    """at your end step, an untapped Relic taps for a charge counter unless held-up instants need the mana"""
    if p is not src.owner or src.tapped or src.phased or _you(g, p): return
    held = [c for c in p.hand if c.instant and ('ctr' in c.tags or 'rem' in c.tags) and can_pay(g, p, c.generic, c.pips)]
    src.tapped = True
    if held and not any(can_pay(g, p, c.generic, c.pips) for c in held):
        src.tapped = False; return                  # it would cost the held-up instant: keep it untapped
    if ability_window(g, p, src, 'a charge counter'): src.data = dict(src.data or {}, charge=(src.data or {}).get('charge', 0) + 1)


@on('Coalition Relic', 'upkeep')
def _relic_release(g, src, p):
    """at the beginning of your precombat main phase: one mana of any colour per charge counter"""
    k = (src.data or {}).get('charge', 0)
    if p is src.owner and k and trigger_window(g, p, src, f'{k} mana of any colour'):
        k = (src.data or {}).get('charge', 0)
        src.data['charge'] = 0; p.floatA += k
card('Coalition Relic', 'rock=1:A', types='A', dsl=[])
full('Coalition Relic', '{T}: one mana of any colour; untapped at your end step it takes a charge counter instead '
     '(unless held-up instants need it), released as extra mana on your next turn')


# ------------------------------------------------------------------ Deep Analysis (flashback pays 3 life)
card('Deep Analysis', 'draw=2 fb=1U fblife=3', types='S', dsl=[])
full('Deep Analysis', 'draw two; flashback {1}{U} and 3 life')


# ------------------------------------------------------------------ Notion Thief: flash at end of turn
@on('Notion Thief', 'hand_options')
def _thief_flash(g, c, p, s, post):
    if post is not None or g.active is p or c not in p.hand or not can_pay(g, p, 2, 'UB'): return []
    return [(4.0, 'Notion Thief (flash)', lambda: IC_cast_perm(g, p, c, 2, 'UB'))]


def IC_cast_perm(g, p, c, gen, pips):
    if c not in p.hand or not can_pay(g, p, gen, pips): return False
    pay(g, p, gen, pips); cast_card(g, p, c, 'hand', {}); return True


# ------------------------------------------------------------------ Hellkite Tyrant: twenty artifacts
@on('Hellkite Tyrant', 'upkeep')
def _tyrant(g, src, p):
    if p is not src.owner or src.phased: return
    n = sum(1 for m in p.perms if m.cd is not None and 'A' in m.cd.types) + p.treasures + p.clues + getattr(p, 'food', 0)
    if n >= 20 and trigger_window(g, p, src, 'win the game', imp=10):
        log(f'  {NAME(p)} controls {n} artifacts: Hellkite Tyrant wins the game', g)
        from commander_sim import ais
        ais.win(g, p, 'alt')
note('Hellkite Tyrant', 'Full', 'flying, trample; combat damage to a player takes all their artifacts; your upkeep '
     'with twenty or more artifacts (Treasures and Clues count): you win')


# ------------------------------------------------------------------ Accursed Marauder: each player's choice
@on('Accursed Marauder', 'etb')
def _marauder(g, src, p, m):
    """each player sacrifices a nontoken creature of their choice (the one cheapest to lose: Marchesa's returns)"""
    if m is not src: return
    if not trigger_window(g, src.owner, src, 'each player sacrifices a creature', imp=5): return
    for q in g.players:
        if not q.alive: continue
        cr = [x for x in q.perms if x.creature and not x.phased and not x.token]
        if cr and E.human_choice(g, q) is not None:
            E.human_choice(g, q).sacrifice_creature(g, q, cr, 'Accursed Marauder: sacrifice a nontoken creature')
        elif cr: die(g, min(cr, key=lambda x: sac_worth(g, x)), 'sac')
card('Accursed Marauder', 'warrior pow=2 tgh=1', types='C', dsl=[])
full('Accursed Marauder', 'enters: each player sacrifices a nontoken creature of their choice (the one cheapest to '
     'lose; a Marchesa player gives up one she returns)')


# ------------------------------------------------------------------ cards whose rules are tags (carddb.py)
full('Last Gasp', 'target creature gets -3/-3 (kills an indestructible X/3)')
full("Tezzeret's Gambit", 'draw two, then proliferate (your creatures\' +1/+1 counters, loyalty, opponents\' -1/-1 '
     'and poison); {U/P} is paid with 2 life when blue is short')
