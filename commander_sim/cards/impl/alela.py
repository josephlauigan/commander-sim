"""Alela, Artful Provocateur (Avery's Esper Faerie Flyers deck, key `alela`, decklists/Avery/alela-esper-faeries.md).

- Alela: flying, deathtouch, lifelink; other creatures you control with flying get +1/+0 (an anthem with a flying
  filter); whenever you cast an artifact or enchantment spell, create a 1/1 blue Faerie creature token with flying.
- The AI: the outside decks' generic priorities (it shares their generic plays), plus the Faerie each artifact or
  enchantment makes while Alela is out (more with Wispdrinker Vampire draining and Tetsuko Umezawa unblocking them).
- The 99's cards that need code: the Jace token (empower Jace: Keeper of the Quiet Hour, Plan for All Outcomes,
  Fatehold Charm), Opposition, Karmic Justice, Static Prison and Aether Hub (energy), Multiply by Zero, Prophesied
  End, Spite // Malice, Muddle the Mixture's transmute, Ray of Command, the flier payoffs (Plumecreed Mentor,
  Pileated Provisioner, Jackdaw Savior, Stirring Hopesinger, Desperate Futurescribe), Malcator, Emry, Airlift
  Chaplain, Reconstructed Thopter, Strixhaven Skycoach, Helping Hand, Daydream, Feather of Flight, Ajani Fells the
  Godsire and Venser, the Sojourner.
"""
import importlib
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim.cards import cardimpl as CI
from commander_sim.cards.cardimpl import on
from commander_sim.cards.pool_cards import card, note

ALELA = 'Alela, Artful Provocateur'

card(ALELA, 'leg pow=2 tgh=3 fly dt lifelink', dsl=[
    {'type': 'static', 'static': 'anthem', 'pow': 1, 'tgh': 0,
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
    if not v and 'landfall2' in c.tags:                       # Felidar Retreat: a 2/2 Cat for each land still to come
        v = 50 if p.turns <= 6 else 30
    if v and not c.land and ('A' in c.types or 'E' in c.types): v += faerie_bonus(g, p)
    return min(90, v)


CI.alela_prio = alela_prio


# ================================================================== the 99: cards that need code
from commander_sim.cards.impl import common as IC


def full(name, text): note(name, 'Full', text)


def human(g, p):
    return E.human_choice(g, p)


def _named(p, name):
    return [m for m in p.perms if m.cd is not None and m.cd.name == name and not m.phased]


def _flier(g, m):
    return m.fly or 'flying' in g.eot_kw.get(id(m), ())


def _counter_nonflier(g, p, src):
    """+1/+1 counter on target creature you control without flying (Plumecreed Mentor, Pileated Provisioner)"""
    cs = [m for m in p.perms if m.creature and not m.phased and not _flier(g, m)]
    if not cs: return
    t = max(cs, key=lambda m: (m.is_cmd, not m.noatk, pval(g, m)))
    t.plus += 1
    log(f'    {src.cd.name}: +1/+1 counter on {t.name}', g)


# ------------------------------------------------------------------ the Jace token (empower Jace)
JACE = 'Jace Token'
E.DB[JACE] = E.CD(JACE, 'P', '0', 'tokpw')       # a planeswalker token: no card, it ceases to exist when it leaves


def jace_token(p):
    return next(iter(_named(p, JACE)), None)


def empower(g, p, n):
    """empower Jace n: put n loyalty counters on your Jace token, creating it first if you don't control one (a blue
    Jace planeswalker token with "-1: Surveil 1" and "-3: Draw a card")"""
    j = jace_token(p)
    if j is None:
        j = Perm(p, E.DB[JACE]); j.loyalty = 0; j.colors = 'U'
        g.enter_no = getattr(g, 'enter_no', 0) + 1; j.born = g.enter_no
        p.perms.append(j); g.hooks.append(j); g.hook_cache = None
        g.bf_ver = getattr(g, 'bf_ver', 0) + 1
        log(f'    {NAME(p)} creates a Jace planeswalker token', g)
    j.loyalty += n
    log(f'    empower Jace {n}: loyalty {j.loyalty}', g)


def _jace_surveil(g, p, src):
    """-1 surveil 1: only while the token can't reach the -3 soon (no Plan for All Outcomes growing it)"""
    if src.loyalty >= 3 or (src.loyalty == 2 and _named(p, 'Plan for All Outcomes')): return None
    return 0.7


IC.walker(JACE, [
    (-1, 'surveil 1', _jace_surveil, lambda g, p, src: importlib.import_module('commander_sim.cards.impl.topdeck').scry(g, p, 1, 'gy')),
    (-3, 'draw a card', lambda g, p, src: 3.0, lambda g, p, src: draw(g, p, 1)),
], status=('Full', 'empower Jace: a Jace planeswalker token (-1 surveil 1, -3 draw a card) that Fatehold Charm, Keeper of '
           'the Quiet Hour and Plan for All Outcomes grow'), tags='tokpw')


@on('Keeper of the Quiet Hour', 'etb')
def _keeper(g, src, p, m):
    if m is src and trigger_window(g, src.owner, src, 'empower Jace 2'): empower(g, src.owner, 2)
card('Keeper of the Quiet Hour', 'pow=3 tgh=2', dsl=[])
full('Keeper of the Quiet Hour', '3/2; empower Jace 2 on entry')


@on('Plan for All Outcomes', 'etb')
def _plan_etb(g, src, p, m):
    if m is not src: return
    o = src.owner
    t = IC.best_opp_nonland(g, o)
    if t is None or pval(g, t) < 2: return
    if not trigger_window(g, o, src, f'{t.name} to the top or bottom of its library', imp=5): return
    if t in t.owner.perms and not untargetable(g, t): apply_removal(g, o, t, 'top')   # its owner keeps it on top


@on('Plan for All Outcomes', 'cast')
def _plan_cast(g, src, caster, c):
    o = src.owner
    if caster is not o or c.creature or c.land: return
    if casts_this_turn(g, o, lambda x: not x.creature and not x.land) != 1: return     # first noncreature spell each turn
    if trigger_window(g, o, src, 'empower Jace 1'): empower(g, o, 1)


def _plan_prio(g, p, c):
    t = IC.best_opp_nonland(g, p)
    v = pval(g, t) if t is not None else 0
    return 40 + min(25, int(5 * v)) if v >= 2 else 28
CI.SPELL_PRIO['Plan for All Outcomes'] = _plan_prio
card('Plan for All Outcomes', '', types='E', dsl=[])
full('Plan for All Outcomes', "the best opposing nonland permanent goes on top of its owner's library (tokens are gone); "
     'empower Jace 1 on your first noncreature spell each turn')


def _fatehold_draw(g, c, p, s, post):
    """the draw mode, at the end of the turn before yours (its bounce mode is held as a counter or for an attacker)"""
    if post is not None or g.active is p or c not in p.hand or not can_pay(g, p, 0, 'WU'): return []

    def go():
        if c not in p.hand or not can_pay(g, p, 0, 'WU'): return False
        pay(g, p, 0, 'WU'); p.hand.remove(c); p.spells_this_turn += 1; on_cast(g, p, c)
        ok = counter_window(g, p, c, 2, {})
        p.gy.append(c)
        if ok:
            draw(g, p, 1); empower(g, p, 2)
        log(f'  {NAME(p)} casts Fatehold Charm: draw a card, empower Jace 2', g)
        return True
    return [(1.4, 'Fatehold Charm (draw, empower Jace 2)', go)]
on('Fatehold Charm', 'hand_options')(_fatehold_draw)
card('Fatehold Charm', 'ctr=any remand rem=bounce tgt=c', types='I', dsl=[])
note('Fatehold Charm', 'Approximate', 'draw a card and empower Jace 2 at the end of the turn before yours, or held to '
     'return a spell (to its owner\'s hand) or an attacking creature; the +1/+2 team mode is not used')


# ------------------------------------------------------------------ removal and control
card('Multiply by Zero', 'rem=zero tgt=c', types='I', dsl=[])
full('Multiply by Zero', 'base power and toughness 0/0 until end of turn: kills a creature with no counters or pumps')
card('Prophesied End', 'rem=destroy tgt=c enddraw', types='I', dsl=[])
full('Prophesied End', "destroy target creature; its controller draws unless it was attacking (so it's best on an attacker)")
card('Spite // Malice', 'ctr=nc noregen', types='I', cost='3U', dsl=[])


@on('Spite // Malice', 'hand_options')
def _malice(g, c, p, s, post):
    """Malice ({3}{B}): destroy target nonblack creature, when it beats holding Spite for a noncreature spell"""
    if c not in p.hand or not can_pay(g, p, 3, 'B'): return []
    tg = [m for m in legal_targets(g, p, 'destroy', 'c', spell=c) if 'B' not in colors_of(m)]
    if not tg: return []
    t = max(tg, key=lambda m: pval(g, m))
    if pval(g, t) < 4: return []

    def go():
        if c not in p.hand or not can_pay(g, p, 3, 'B') or t not in t.owner.perms: return False
        pay(g, p, 3, 'B'); p.hand.remove(c); p.spells_this_turn += 1; on_cast(g, p, c)
        ok = counter_window(g, p, c, 4, {})
        p.gy.append(c)
        log(f'  {NAME(p)} casts Malice on {t.name}', g)
        if ok and t in t.owner.perms and not untargetable(g, t): apply_removal(g, p, t, 'destroy', c)   # noregen tag
        return True
    return [(pval(g, t) * 0.5 - 1.0, f'Malice on {t.name}', go)]
full('Spite // Malice', 'Spite: held to counter a noncreature spell; Malice: destroys a nonblack creature (no '
     'regeneration) when the creature is worth more than holding the counter')


@on('Muddle the Mixture', 'hand_options')
def _transmute(g, c, p, s, post):
    """transmute {1}{U}{U} (sorcery speed): the best two-mana card in the library, when it beats holding the counter"""
    if post is None or g.active is not p or c not in p.hand or not can_pay(g, p, 1, 'UU'): return []
    cs = [x for x in searchable(g, p) if x.cmc == 2]
    if not cs: return []
    best = max(cs, key=lambda x: card_worth(g, p, x))
    gain_ = card_worth(g, p, best) - card_worth(g, p, c)
    if gain_ < 8: return []

    def go():
        if c not in p.hand or not can_pay(g, p, 1, 'UU') or best not in p.library: return False
        pay(g, p, 1, 'UU'); p.hand.remove(c); p.gy.append(c)
        if not ability_window(g, p, c, 'transmute'): return True
        p.library.remove(best); g.rng.shuffle(p.library); p.hand.append(best)
        log(f'  {NAME(p)} transmutes Muddle the Mixture for {best.name}', g)
        return True
    return [(gain_ / 12.0, f'transmute Muddle the Mixture ({best.name})', go)]
card('Muddle the Mixture', 'ctr=is', types='I', dsl=[])
full('Muddle the Mixture', 'counter target instant or sorcery; transmute {1}{U}{U} for a two-mana card when that card '
     'is worth clearly more than the counter')


@on('Ray of Command', 'hand_options')
def _ray(g, c, p, s, post):
    """before combat on your turn: take the best opposing creature, attack with it (it returns tapped)"""
    if post is not False or g.active is not p or c not in p.hand or not can_pay(g, p, 3, 'U'): return []
    import importlib as _il
    t = _il.import_module('commander_sim.cards.impl.marchesa').best_steal(g, p, c)
    if t is None or epow(g, t) < 3: return []
    u = 0.3 * epow(g, t) + 0.2 * pval(g, t) - 1.2

    def go():
        if c not in p.hand or not can_pay(g, p, 3, 'U') or t not in t.owner.perms: return False
        pay(g, p, 3, 'U'); p.hand.remove(c); p.spells_this_turn += 1; on_cast(g, p, c)
        ok = counter_window(g, p, c, 4, {})
        p.gy.append(c)
        if not ok or t not in t.owner.perms or t.owner is p or untargetable(g, t): return True
        from commander_sim import ais
        if ais.protect_response(g, t.owner, t, 'steal', p, c) or t not in t.owner.perms: return True
        _il.import_module('commander_sim.cards.impl.marchesa').steal(g, p, t, until_eot=True)
        t.data = dict(t.data or {}, tap_on_return=True)
        return True
    return [(u, f'Ray of Command on {t.name}', go)]
card('Ray of Command', '', types='I', dsl=[])
full('Ray of Command', 'before combat: untap and take the best opposing creature until end of turn (haste) and attack '
     'with it; it is tapped when it goes back')


@on('Karmic Justice', 'dies')
def _karmic(g, src, m, cause):
    _karmic_fire(g, src, m, cause)


@on('Karmic Justice', 'self_dies')
def _karmic_self(g, m, cause):
    _karmic_fire(g, m, m, cause)


def _karmic_fire(g, src, m, cause):
    """an opponent's spell or ability destroyed a noncreature permanent you control: destroy their best permanent"""
    o = src.owner
    d = getattr(g, 'destroyer', None)
    if cause != 'destroy' or m.creature or m.owner is not o or d is None or d is o or d not in g.opps(o): return
    cands = [x for x in d.perms if not x.phased and not untargetable(g, x)]
    if not cands: return
    t = max(cands, key=lambda x: pval(g, x))
    if not trigger_window(g, o, src, f'destroy {t.name}', imp=4): return
    if t in d.perms: apply_removal(g, o, t, 'destroy')
CI.SPELL_PRIO['Karmic Justice'] = 30
card('Karmic Justice', '', types='E', dsl=[])
full('Karmic Justice', "when an opponent's spell or ability destroys a noncreature permanent you control (Karmic "
     'Justice itself too), destroy their best permanent')


# ------------------------------------------------------------------ Opposition
def _tappers(g, q):
    """q's untapped creatures to tap for Opposition, cheapest first (no {T} in the cost: summoning sickness is fine)"""
    return sorted([m for m in q.perms if m.creature and not m.tapped and not m.phased and not m.is_cmd],
                  key=lambda m: (not m.token, pval(g, m)))


def _turns_left(g, q, a):
    """opponents' turns from a's (this one) until q's next turn"""
    n, x = 1, a
    for x in g.after(a):
        if x is q: break
        if x.alive: n += 1
    return n


def _share(g, q, a):
    t = _tappers(g, q)
    k = -(-len(t) // _turns_left(g, q, a))
    return t[:k]


def _opp_tap(g, q, src, tapper, what, name):
    tapper.tapped = True; what.tapped = True
    log(f'    Opposition: {NAME(q)} taps {tapper.name} to tap {name}', g)


@on('Opposition', 'upkeep')
def _opposition_upkeep(g, src, p):
    """an opponent's upkeep: tap their mana (best rocks and lands), keeping a share of tappers for their attackers"""
    q = src.owner
    if p is q or p not in g.opps(q) or human(g, q) is not None or src is not _named(q, 'Opposition')[0]: return
    use = _share(g, q, p)
    atk = sum(1 for m in p.perms if m.creature and not m.phased and epow(g, m) >= 3)
    use = use[:max(0, len(use) - atk)]
    srcs = [(int(m.cd.tags['rock'].split(':')[0]) + 0.5, m, m.name) for m in p.perms
            if m.cd is not None and 'rock' in m.cd.tags and not m.tapped and not m.phased and not untargetable(g, m)]
    srcs += [(int(L.cd.tags.get('amt', 1) or 1) + 0.1 * len(land_cols(p, L, False)), L, L.cd.name)
             for L in p.lands if not L.tapped]
    srcs.sort(key=lambda x: -x[0])
    for tapper, (_, what, name) in zip(use, srcs):
        if not trigger_window(g, q, src, f'tap {name}', imp=1): break
        _opp_tap(g, q, src, tapper, what, name)


def opposition_precombat(g, q):
    """q at the beginning of another player's combat: tap their attackers with Opposition"""
    if not _named(q, 'Opposition'): return
    a = g.active
    src = _named(q, 'Opposition')[0]
    cands = sorted([m for m in a.perms if m.creature and not m.tapped and not m.phased and not untargetable(g, m)
                    and epow(g, m) >= 2 and (not m.sick or importlib.import_module('commander_sim.ais').has_haste(g, m))],
                   key=lambda m: -epow(g, m))
    for tapper, t in zip(_share(g, q, a), cands):
        if epow(g, t) <= epow(g, tapper) and not t.is_cmd: break       # the tapper blocks it as well
        _opp_tap(g, q, src, tapper, t, t.name)
CI.opposition_precombat = opposition_precombat


@on('Opposition', 'crew')
def _opposition_mine(g, src, p):
    """your beginning of combat: creatures that can't attack tap the opponents' fliers that could block"""
    if p is not src.owner or human(g, p) is not None or src is not _named(p, 'Opposition')[0]: return
    idle = [m for m in _tappers(g, p) if (m.sick and not importlib.import_module('commander_sim.ais').has_haste(g, m))
            or m.noatk or epow(g, m) == 0]
    blockers = sorted([m for q in g.opps(p) for m in q.perms if m.creature and not m.tapped and not m.phased
                       and not untargetable(g, m) and (_flier(g, m) or (m.cd is not None and 'reach' in m.cd.tags))],
                      key=lambda m: -pval(g, m))
    for tapper, t in zip(idle, blockers): _opp_tap(g, p, src, tapper, t, t.name)


def _opposition_prio(g, p, c):
    n = sum(1 for m in p.perms if m.creature and not m.phased)
    return min(65, 35 + 4 * n)
CI.SPELL_PRIO['Opposition'] = _opposition_prio
card('Opposition', '', types='E', dsl=[])
full('Opposition', "untapped creatures tap opponents' rocks and lands in their upkeep and their attackers at their "
     "combat (split across the turns until yours), and on your turn the fliers that could block")


# ------------------------------------------------------------------ Static Prison, Aether Hub (energy)
def _energy(p): return getattr(p, 'energy', 0)


@on('Static Prison', 'etb')
def _prison(g, src, p, m):
    if m is not src: return
    o = src.owner
    if trigger_window(g, o, src, 'exile a permanent; you get {E}{E}', imp=5) and src in o.perms:
        IC.oring_exile(g, src, o, lambda x: True)
    o.energy = _energy(o) + 2


@on('Static Prison', 'leaves')
def _prison_leaves(g, m):
    IC.oring_return(g, m)


@on('Static Prison', 'upkeep')
def _prison_upkeep(g, src, p):
    if p is not src.owner: return
    if _energy(p) >= 1 and (src.data or {}).get('oring'):
        p.energy -= 1; log(f'    Static Prison: {NAME(p)} pays {{E}} ({p.energy} left)', g)
    elif trigger_window(g, p, src, 'sacrifice it (no energy)', imp=2) and src in p.perms: die(g, src, 'sac')


def _prison_prio(g, p, c):
    t = IC.best_opp_nonland(g, p)
    v = pval(g, t) if t is not None else 0
    return 45 + min(30, int(6 * v)) if v >= 2 else 0
CI.SPELL_PRIO['Static Prison'] = _prison_prio
card('Static Prison', '', types='E', dsl=[])
full('Static Prison', 'exiles the best opposing nonland permanent until it leaves; {E}{E}; each turn pay {E} or '
     'sacrifice it (Aether Hub energy keeps it longer)')


def _hub_etb(g, p, L): p.energy = _energy(p) + 1


def _hub_spare(g, p):
    return _energy(p) - 2 * len(_named(p, 'Static Prison'))       # the Prison's upkeep comes first


def _hub_tap(g, p, L, used):
    if any(x in 'WUBRG' for x in E.TAP_COLS): p.energy = max(0, _energy(p) - 1)
CI.LAND_ETB['Aether Hub'] = _hub_etb
CI.LAND_COLS['Aether Hub'] = lambda g, p, L: p.ident if _hub_spare(g, p) > 0 else ''
CI.ON_TAP['Aether Hub'] = _hub_tap
full('Aether Hub', '{E} on entry; {T}: {C}, or pay {E} for any colour (energy that Static Prison needs is kept)')


# ------------------------------------------------------------------ creatures
@on('Airlift Chaplain', 'etb')
def _chaplain(g, src, p, m):
    if m is not src or not trigger_window(g, src.owner, src, 'mill three'): return
    o = src.owner
    top = o.library[-3:]
    mill(g, o, 3)
    cs = [c for c in top if c in o.gy and (c.name == 'Plains' or (c.creature and c.cmc <= 3))]
    if cs:
        c = max(cs, key=lambda c: card_worth(g, o, c))
        o.gy.remove(c); o.hand.append(c)
        log(f'    Airlift Chaplain: {NAME(o)} puts {c.name} into hand', g)
    elif src in o.perms: src.plus += 1
card('Airlift Chaplain', 'pow=1 tgh=1 fly', dsl=[])
full('Airlift Chaplain', 'flying; mill three on entry, keep a Plains or a creature card with MV 3 or less (else a '
     '+1/+1 counter)')


@on('Jackdaw Savior', 'dies')
def _jackdaw(g, src, m, cause):
    if m.owner is src.owner and m is not src and m.creature and _flier(g, m): _jackdaw_return(g, src.owner, src, m)


@on('Jackdaw Savior', 'self_dies')
def _jackdaw_self(g, m, cause):
    _jackdaw_return(g, m.owner, m, m)


def _jackdaw_return(g, o, src, dead):
    """return another creature card with lesser mana value from your graveyard to the battlefield"""
    if dead.token or dead.cd is None: return
    mv = dead.cd.cmc
    cs = [c for c in o.gy if c.creature and c.cmc < mv and c is not dead.cd]
    if not cs: return
    c = max(cs, key=lambda c: (CI.card_etb_value(g, o, c) if hasattr(CI, 'card_etb_value') else 0) + c.cmc + c.pow)
    if not trigger_window(g, o, src, f'return {c.name}', imp=3) or c not in o.gy: return
    o.gy.remove(c); enter(g, o, c)
    log(f'    Jackdaw Savior returns {c.name}', g)
card('Jackdaw Savior', 'pow=3 tgh=1 fly', dsl=[])
full('Jackdaw Savior', 'flying; when it or another flier of yours dies, return a creature card with lesser mana value '
     'from your graveyard to the battlefield')


@on('Plumecreed Mentor', 'etb')
def _plume(g, src, p, m):
    if m.owner is src.owner and m.creature and _flier(g, m) and trigger_window(g, src.owner, src, '+1/+1 counter'):
        _counter_nonflier(g, src.owner, src)


@on('Plumecreed Mentor', 'tokens_enter')
def _plume_tokens(g, src, p, toks):
    if p is not src.owner: return
    for t in toks:
        if t.fly and t in p.perms and trigger_window(g, p, src, '+1/+1 counter'): _counter_nonflier(g, p, src)
card('Plumecreed Mentor', 'pow=2 tgh=3 fly', dsl=[])
full('Plumecreed Mentor', 'flying; whenever it or another flier of yours enters (Faerie tokens too), a +1/+1 counter on '
     'a creature of yours without flying')


@on('Pileated Provisioner', 'etb')
def _provisioner(g, src, p, m):
    if m is src and trigger_window(g, src.owner, src, '+1/+1 counter'): _counter_nonflier(g, src.owner, src)
card('Pileated Provisioner', 'pow=3 tgh=4 fly', dsl=[])
full('Pileated Provisioner', 'flying; a +1/+1 counter on a creature of yours without flying on entry')


@on('Stirring Hopesinger', 'cast')
def _hopesinger(g, src, caster, c):
    o = src.owner
    if caster is not o or not (c.instant or c.sorcery): return
    t = c.tags
    targets_creature = ('rem' in t and t.get('tgt', 'c') in ('c', 'cp', 'cap', 'ce', 'ac', 'cna')) or \
        c.name in ('Daydream', 'Ray of Command', 'Spite // Malice', 'Multiply by Zero', 'Fatehold Charm')
    if not targets_creature or not trigger_window(g, o, src, '+1/+1 counter on each creature you control'): return
    for m in o.perms:
        if m.creature and not m.phased: m.plus += 1
card('Stirring Hopesinger', 'pow=1 tgh=3 fly lifelink', dsl=[])
full('Stirring Hopesinger', 'flying, lifelink; your instants and sorceries that target a creature (removal, blink, '
     'steal) put a +1/+1 counter on each creature you control')


@on('Desperate Futurescribe', 'crew')
def _futurescribe(g, src, p):
    """beginning of combat on your turn: +1/+1 to another creature (a +1/+1 counter if you scried or surveilled)"""
    if p is not src.owner: return
    cs = [m for m in p.perms if m.creature and m is not src and not m.phased]
    if not cs: return
    t = max(cs, key=lambda m: (not m.tapped and not m.sick, _flier(g, m), epow(g, m)))
    if not trigger_window(g, p, src, f'+1/+1 to {t.name}') or t not in p.perms: return
    if getattr(p, 'scry_turn', None) == turn_stamp(g): t.plus += 1
    else:
        a, b = g.eot_pt.get(id(t), (0, 0)); g.eot_pt[id(t)] = (a + 1, b + 1); g.dsl_on = True
card('Desperate Futurescribe', 'pow=3 tgh=4 fly', dsl=[])
full('Desperate Futurescribe', 'flying; each combat on your turn another creature gets +1/+1 (a +1/+1 counter if you '
     'scried or surveilled this turn)')


@on('Malcator, Purity Overseer', 'etb')
def _malcator(g, src, p, m):
    if m is src and trigger_window(g, src.owner, src, 'a 3/3 Golem'): _golem(g, src.owner)


def _golem(g, p):
    _art_entered(g, p, 1)
    make_tokens(g, p, 1, 3, 3, color='', types=('golem', 'phyrexian'))


def _art_entered(g, p, n):
    st = turn_stamp(g)
    if getattr(p, 'art_tok', None) is None or p.art_tok[0] != st: p.art_tok = (st, 0)
    p.art_tok = (st, p.art_tok[1] + n)


@on('Malcator, Purity Overseer', 'token_created')
def _malcator_tok(g, src, p, kinds, n):
    if p is src.owner: _art_entered(g, p, n)


@on('Malcator, Purity Overseer', 'end_step')
def _malcator_end(g, src, p):
    if p is not src.owner: return
    st = turn_stamp(g)
    n = sum(1 for t, m in getattr(g, 'entered', []) if t == st and m.owner is p and m.cd is not None and 'A' in m.cd.types)
    tok = getattr(p, 'art_tok', None)
    n += tok[1] if tok is not None and tok[0] == st else 0
    if n >= 3 and trigger_window(g, p, src, 'a 3/3 Golem'): _golem(g, p)
card('Malcator, Purity Overseer', 'leg pow=1 tgh=1 wizard', dsl=[])
full('Malcator, Purity Overseer', 'a 3/3 Golem on entry; another at your end step if three or more artifacts entered '
     'under your control this turn (tokens count)')


E.SELF_COST['Emry, Lurker of the Loch'] = lambda g, p, c: -(sum(
    1 for m in p.perms if m.cd is not None and 'A' in m.cd.types and not m.phased) + p.treasures)


@on('Emry, Lurker of the Loch', 'etb')
def _emry(g, src, p, m):
    if m is src and trigger_window(g, src.owner, src, 'mill four'): mill(g, src.owner, 4)


@on('Emry, Lurker of the Loch', 'options')
def _emry_cast(g, src, p, s, post):
    """{T}: cast an artifact card from your graveyard this turn (the AI casts it at once)"""
    if src.owner is not p or post is None or g.active is not p or src.tapped or src.sick: return []
    out = []
    for x in [c for c in p.gy if 'A' in c.types and not c.land and castable(g, p, c, 'gy')]:
        gen, pips = cost_of(p, x)
        if not can_pay(g, p, gen, pips): continue
        from commander_sim import ais
        v = ais.deck_prio(g, p, x)
        if v <= 0: continue

        def go(x=x):
            if src.tapped or x not in p.gy: return False
            gen, pips = cost_of(p, x)
            E.PAY_FOR = x
            try:
                if not can_pay(g, p, gen, pips): return False
                src.tapped = True
                if not ability_window(g, p, src, f'cast {x.name} from the graveyard'): return True
                pay(g, p, gen, pips)
            finally:
                E.PAY_FOR = None
            log(f'  {NAME(p)} casts {x.name} from the graveyard (Emry)', g)
            cast_card(g, p, x, 'mgy')
            return True
        out.append((v / 10.0 - 0.3, f'Emry: cast {x.name} from the graveyard', go))
    return out
card('Emry, Lurker of the Loch', 'leg pow=1 tgh=2 wizard', dsl=[])
full('Emry, Lurker of the Loch', 'affinity for artifacts; mill four on entry; {T}: cast an artifact card from your '
     'graveyard')


@on('Reconstructed Thopter', 'gy_options')
def _thopter(g, c, p, s, post):
    """unearth {2} (sorcery speed, once: it is exiled after)"""
    if post is not False or g.active is not p or c not in p.gy or not can_pay(g, p, 2, ''): return []
    if id(c) in getattr(p, 'unearthed', ()): return []
    u = 0.8 + (1.0 if alela_perm(p) is not None else 0)

    def go():
        if c not in p.gy or not can_pay(g, p, 2, ''): return False
        pay(g, p, 2, '')
        p.unearthed = getattr(p, 'unearthed', set()) | {id(c)}
        if not ability_window(g, p, c, 'unearth') or c not in p.gy: return True
        p.gy.remove(c)
        m = enter(g, p, c); m.sick = False
        m.data = dict(m.data or {}, unearth=True)
        log(f'  {NAME(p)} unearths Reconstructed Thopter', g)
        return True
    return [(u, 'unearth Reconstructed Thopter', go)]


@on('Reconstructed Thopter', 'end_step')
def _thopter_exile(g, src, p):
    if src.data and src.data.get('unearth') and src in src.owner.perms and p is src.owner:
        if not trigger_window(g, p, src, 'exile it (unearth)', imp=2) or src not in p.perms: return
        leave(g, src); p.exile.append(src.cd)
card('Reconstructed Thopter', 'pow=2 tgh=1 fly', types='AC', dsl=[])
full('Reconstructed Thopter', 'flying; unearth {2} once (hasty, exiled at end of turn)')


@on('Strixhaven Skycoach', 'etb')
def _skycoach(g, src, p, m):
    if m is src and trigger_window(g, src.owner, src, 'search for a basic land'): land_to_hand(g, src.owner)


@on('Strixhaven Skycoach', 'crew')
def _skycoach_crew(g, src, p):
    """crew 2: tap creatures that can't attack (or are worse attackers than the 3/2 flier)"""
    if p is not src.owner or src.tapped or src.sick: return
    cands = sorted([m for m in p.perms if m.creature and not m.tapped and not m.phased and m is not src and not m.is_cmd],
                   key=lambda m: (not (m.sick or m.noatk), pval(g, m)))
    power, pick = 0, []
    for m in cands:
        if power >= 2: break
        if m.sick or m.noatk or epow(g, m) < 3: pick.append(m); power += epow(g, m)
    if power < 2 or sum(epow(g, m) for m in pick if not (m.sick or m.noatk)) >= 3: return
    for m in pick: m.tapped = True
    src.data = dict(src.data or {}, anim=True, crewed=turn_stamp(g)); src.pow, src.tgh = 3, 2
    g.bf_ver = getattr(g, 'bf_ver', 0) + 1
    log(f'  {NAME(p)} crews Strixhaven Skycoach', g)
card('Strixhaven Skycoach', 'fly', types='A', dsl=[])
full('Strixhaven Skycoach', 'a basic land to hand on entry; crew 2 with creatures that would not attack anyway: a 3/2 '
     'flier')


# ------------------------------------------------------------------ sorceries, Auras, sagas, Venser
def _pick_reanimate(g, p, pred):
    cs = [c for c in p.gy if c.creature and pred(c)]
    return max(cs, key=lambda c: (CI.card_etb_value(g, p, c) if hasattr(CI, 'card_etb_value') else 0) + c.pow + c.cmc) \
        if cs else None


@IC.spell('Helping Hand', prio=lambda g, p, c: 35 + 3 * min(10, _pick_reanimate(g, p, lambda x: x.cmc <= 3).cmc + 2)
          if _pick_reanimate(g, p, lambda x: x.cmc <= 3) is not None else 0, tags='', types='S',
          status=('Full', 'returns the best creature card with MV 3 or less from your graveyard, tapped'))
def _helping_hand(g, p, c, ctx):
    x = _pick_reanimate(g, p, lambda x: x.cmc <= 3)
    if x is None: return
    p.gy.remove(x); m = enter(g, p, x); m.tapped = True


def _daydream_target(g, p):
    from commander_sim.cards.impl import t2
    cs = [m for m in p.perms if m.creature and not m.token and not m.phased and m.cd is not None]
    return max(cs, key=lambda m: t2.blink_value(g, p, m) + 0.1 * pval(g, m), default=None)


def _daydream_prio(g, p, c):
    from commander_sim.cards.impl import t2
    t = _daydream_target(g, p)
    v = t2.blink_value(g, p, t) if t is not None else 0
    return 30 + min(30, int(5 * v)) if v >= 2 else 0


@IC.spell('Daydream', prio=_daydream_prio, tags='fb=2W', types='S',
          status=('Full', 'blinks your creature with the best entry ability (+1/+1 counter); flashback {2}{W}'))
def _daydream(g, p, c, ctx):
    from commander_sim.cards.impl import t2
    t = _daydream_target(g, p)
    if t is None: return
    n = t2.blink(g, p, t)
    if n is not None: n.plus += 1


def _feather_host(g, p, spec, a):
    cs = [m for m in p.perms if m.creature and not m.phased and m is not a]
    if not cs: return None
    return max(cs, key=lambda m: (not _flier(g, m)) * 3 + epow(g, m) + m.is_cmd)
IC.aura('Feather of Flight', 1, 0, ('flying',), on_etb=lambda g, p, a, h: draw(g, p, 1), tags='aura flash',
        host_pick=_feather_host, status=('Full', 'flash; draw a card; +1/+0 and flying (on a creature without flying)'))


@on('Ajani Fells the Godsire', 'etb')
def _ajani(g, src, p, m):
    if m is not src: return
    src.data = {'lore': 1}
    o = src.owner
    t = IC.best_opp_creature(g, o, lambda x: epow(g, x) >= 3)
    if t is not None and trigger_window(g, o, src, f'chapter I: exile {t.name}', imp=5) and t in t.owner.perms:
        apply_removal(g, o, t, 'exile')


@on('Ajani Fells the Godsire', 'upkeep')
def _ajani_lore(g, src, p):
    if p is not src.owner or not src.data or 'lore' not in src.data: return
    n = src.data['lore'] = src.data['lore'] + 1
    ok = trigger_window(g, p, src, 'chapter II: a 2/1 Cat Warrior, a vigilance counter' if n == 2 else
                        'chapter III: double strike', imp=3)
    if ok and n == 2:
        make_tokens(g, p, 1, 2, 1, color='W', types=('cat', 'warrior'))
        cs = [m for m in p.perms if m.creature and not m.phased and not m.vig]
        if cs: max(cs, key=lambda m: (m.is_cmd, epow(g, m))).vig = True
    elif ok and n >= 3:
        cs = [m for m in p.perms if m.creature and not m.phased and not m.tapped]
        if cs:
            t = max(cs, key=lambda m: (_flier(g, m), epow(g, m) + (3 if m.dt else 0)))
            g.eot_kw.setdefault(id(t), set()).add('double strike'); g.dsl_on = True
    if n >= 3 and src in p.perms: die(g, src, 'sac')


def _ajani_prio(g, p, c):
    t = IC.best_opp_creature(g, p, lambda x: epow(g, x) >= 3)
    return 40 + min(30, int(5 * pval(g, t))) if t is not None else 32
CI.SPELL_PRIO['Ajani Fells the Godsire'] = _ajani_prio
card('Ajani Fells the Godsire', '', types='E', dsl=[])
full('Ajani Fells the Godsire', 'I: exile an opposing creature with power 3+; II: a 2/1 Cat Warrior and a vigilance '
     'counter; III: double strike to your best attacker')


def _venser_blink_target(g, p):
    from commander_sim.cards.impl import t2
    cs = [m for m in p.perms if not m.token and m.cd is not None and not m.phased and not m.is_cmd
          and 'tokpw' not in m.cd.tags and m.cd.name != 'Venser, the Sojourner']
    return max(cs, key=lambda m: t2.blink_value(g, p, m), default=None)


def _venser_unbl_value(g, p):
    """Venser's -1 (creatures can't be blocked this turn): what the attack it frees is worth"""
    ais = importlib.import_module('commander_sim.ais')
    atk = [m for m in p.perms if m.creature and not m.tapped and not m.phased and not m.noatk
           and (not m.sick or ais.has_haste(g, m))]
    dmg = sum(epow(g, m) for m in atk)
    if dmg < 5: return 0
    low = min((q.life for q in g.opps(p)), default=99)
    return dmg * 0.5 + (8 if dmg >= low else 0)


def _venser_plus(g, p, src):
    if _venser_unbl_value(g, p) > 4 and src.loyalty >= 1: return None      # the -1 at combat is worth more
    from commander_sim.cards.impl import t2
    t = _venser_blink_target(g, p)
    return 1.2 + (0.5 * t2.blink_value(g, p, t) if t is not None else 0)


def _venser_blink(g, p, src):
    from commander_sim.cards.impl import t2
    t = _venser_blink_target(g, p)
    if t is not None and t2.blink_value(g, p, t) > 0: t2.blink(g, p, t)


IC.walker('Venser, the Sojourner', [
    (2, 'blink a permanent', _venser_plus, _venser_blink),
    (-8, 'emblem', lambda g, p, src: 10.0,
     lambda g, p, src: importlib.import_module('commander_sim.cards.impl.rules').give_emblem(p, 'venser')),
], status=('Full', '+2 blinks your permanent with the best entry ability (back at once, not at end step); -1 makes '
           'your creatures unblockable before a big attack; -8 emblem (exile a permanent per spell you cast)'))


@on('Venser, the Sojourner', 'crew')
def _venser_minus(g, src, p):
    """-1 just before combat (it's a main-phase ability: used as the attack is about to start)"""
    if p is not src.owner or human(g, p) is not None or src.loyalty is None or src.loyalty < 1: return
    if IC._uses(g, p, src) >= IC._allowed(p) or _venser_unbl_value(g, p) <= 4: return
    src.loyalty_used = (g.round, p.key, IC._uses(g, p, src) + 1)
    src.loyalty -= 1
    log(f'  {NAME(p)} uses Venser, the Sojourner (-1): creatures can\'t be blocked this turn', g)
    if ability_window(g, p, src, '-1'): p.unbl_all = turn_stamp(g)
    if src in p.perms and src.loyalty <= 0: leave(g, src); to_zone_card(g, src, 'gy')


# ------------------------------------------------------------------ the rest
card("Sphinx's Approach", 'draw=2', types='I', dsl=[])
full("Sphinx's Approach", 'draw two (the five-copy Sphinx clause needs four more copies: not in a singleton deck)')
note('Page, Loose Leaf', 'Full', '{T}: {C}; grandeur needs a second copy (never in a singleton deck)')
note('Initiates of the Ebon Hand', 'Approximate', '1/1; its {1}: {B} filter is not modeled (it adds no mana, only fixes)')
