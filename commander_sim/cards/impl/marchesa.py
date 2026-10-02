"""Card rules for Marchesa, the Black Rose (decklists/mine/marchesa-grixis-recursion.md): the commander's recursion
engine, and the cards of that deck that need more than their tags. Hooks here are live in every game (pool rules).
"""
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim.cards import cardimpl as CI
from commander_sim.cards.cardimpl import on, _eot
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
        if p.key == 'marchesa': p.milestone.setdefault('recur', p.turns)
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
def _opp_x1(g, p, n=1):
    return [m for q in g.opps(p) for m in q.perms if m.creature and not m.phased and etgh(g, m) <= n
            and not untargetable(g, m)]


def card_etb_value(g, p, c):
    """what creature card c does for p when it enters the battlefield, on the board as it is now"""
    if c is None or not c.creature: return 0.0
    opps = g.opps(p)
    n = c.name
    if n == 'Accursed Marauder':
        return 1.2 * sum(1 for q in opps if any(m.creature and not m.token and not m.phased for m in q.perms))
    if n == 'Burglar Rat': return 0.8 * sum(1 for q in opps if q.hand)
    if n in ('Phyrexian Rager', 'Crackling Drake'): return 1.4
    if n == 'Forge Devil':
        tg = _opp_x1(g, p)
        return max((0.8 * pval(g, m) for m in tg), default=0.0) - 0.3
    if n == 'Phyrexian Delver':
        tg = [x for x in p.gy if x.creature and x is not c and p.life - x.cmc >= 10]
        return max((0.5 * (x.bomb or x.pow) + card_etb_value(g, p, x) for x in tg), default=0.0)
    if n == "Sephiroth, Planet's Heir":
        return 0.7 * sum(pval(g, m) for q in opps for m in q.perms if m.creature and not m.phased and etgh(g, m) <= 2)
    if n == 'Deepglow Skate': return 0.5 * sum(m.plus for m in p.perms if m.plus > 0)
    if n == 'Triskelion': return 3.0
    if n == 'Skullport Merchant': return 1.0
    if n == 'Balustrade Spy': return 0.6
    v = sum(x for k, x in IM.ETB_VALUE.items() if k in c.tags)
    if 'draw' in c.tags: v += 1.5 * int(c.tags['draw'])
    if CI.live(n) and 'etb' in CI.HOOKS[n]: v = max(v, 1.5)
    if c.dsl and any(a.get('type') == 'triggered' and a.get('event') == 'etb' for a in c.dsl): v = max(v, 1.5)
    return v


CI.card_etb_value = card_etb_value


# ------------------------------------------------------------------ sacrifice outlets
def outlets(g, p, exclude=None):
    """ways p can sacrifice a creature right now, cheapest first: (label, fn(m) -> sacrifices m)"""
    out = []
    for src in IC.outlets(p):
        out.append((src.cd.name, lambda m, src=src: IC.sac_through(g, p, src, m)))
    for src in p.perms:
        if src.cd is not None and src.cd.name == 'Skullport Merchant' and not src.phased and can_pay(g, p, 1, 'B') \
                and not stopped(g, src.cd.name):
            def merchant(m, src=src):
                if m is src or not can_pay(g, p, 1, 'B'): return False
                pay(g, p, 1, 'B'); log(f'  {NAME(p)} sacrifices {m.name} to Skullport Merchant', g)
                die(g, m, 'sac'); draw(g, p, 1); return True
            out.append(('Skullport Merchant', merchant))
    return out


def sac_with(g, p, m, how):
    if m not in p.perms: return False
    r = how(m)
    return True if r is None else r


# ------------------------------------------------------------------ the AI's Marchesa plays
def marchesa_options(g, p, s, post):
    """re-buy countered creatures, the Triskelion loop, and sacrificing creatures stolen for the turn"""
    o = []
    if post is None: return o
    outs = outlets(g, p)
    if not outs: return o
    label, how = outs[0]
    cheap = [x for x in outs if x[0] != 'Skullport Merchant']
    # creatures stolen until end of turn: sacrifice them (they come back to you with a counter and Marchesa)
    for m in [m for m in getattr(p, 'borrowed', None) or [] if m in p.perms]:
        if post is False and not m.tapped and not m.sick: continue           # attack with it first
        u = 2.0 + 0.8 * pval(g, m) + (3.0 if returns(g, m) else 0.0)
        o.append((u, f'sacrifice stolen {m.name} ({label})', lambda m=m: sac_with(g, p, m, how)))
    if not marchesa_out(p) or post is not True: return o
    # a countered creature that did its job this turn: sacrifice it, it returns at the end step and enters again
    for m in p.perms:
        if not m.creature or m.phased or m.token or m.plus <= 0 or 'marchesa' in m.cd.tags or not returns(g, m): continue
        if m.cd.name == 'Triskelion' and m.plus >= 2: continue               # its counters are damage first
        ev = card_etb_value(g, p, m.cd)
        if ev < 1.0: continue
        use = cheap or outs
        lbl, fn = use[0]
        extra = 0.4 if lbl == 'Carrion Feeder' else (0.6 if lbl == 'Skullport Merchant' else 0.0)
        o.append((1.0 + ev - 0.3 * m.plus + extra, f'sacrifice {m.name} for Marchesa ({lbl})',
                  lambda m=m, fn=fn: sac_with(g, p, m, fn)))
    # Triskelion: shoot down to one counter, then sacrifice it; it returns with three
    for t in [x for x in p.perms if x.cd is not None and x.cd.name == 'Triskelion' and not x.phased]:
        if t.plus < 2 or not returns(g, t): continue
        lbl, fn = (cheap or outs)[0]

        def loop(t=t, fn=fn):
            if t not in p.perms or t.plus < 1: return False
            while t.plus > 1 and not g.over:
                trisk_shot(g, p, t)
            if t in p.perms and not g.over: sac_with(g, p, t, fn)
            return True
        o.append((3.0 + 0.8 * (t.plus - 1), f'Triskelion loop ({lbl})', loop))
    return o


def trisk_shot(g, p, t):
    """remove a +1/+1 counter from Triskelion: 1 damage to an X/1 worth killing, else the opponent nearest death"""
    t.plus -= 1
    opps = g.opps(p)
    lethal = [q for q in opps if q.life <= 1]
    tg = sorted(_opp_x1(g, p), key=lambda m: -pval(g, m))
    if lethal: lose_life(g, lethal[0], 1, p, kind='triggers', damage=True)
    elif tg and pval(g, tg[0]) >= 2.0: apply_removal(g, p, tg[0], 'dmg1')
    elif opps: lose_life(g, min(opps, key=lambda q: q.life), 1, p, kind='triggers', damage=True)
    check_state(g)


def protect(g, owner, m, kind, actor, spell):
    """a removal spell aimed at a creature of Marchesa's deck: sacrifice it first when Marchesa returns it and the
    removal would not (exile, bounce, shuffle, theft)"""
    if kind not in ('exile', 'bounce', 'tuck', 'steal', 'elk', 'mutate', 'forest') or not m.creature: return False
    if not returns(g, m): return False
    outs = outlets(g, owner)
    if not outs: return False
    owner.stats['sac_saves'] += 1
    return sac_with(g, owner, m, outs[0][1])


def wipe_response(g, q, kind):
    """an exile or bounce wipe: countered creatures are sacrificed first so Marchesa returns them"""
    if kind not in ('exile', 'evac', 'rift', 'rebuke') or E.human_choice(g, q) is not None: return
    outs = [x for x in outlets(g, q) if x[0] != 'Skullport Merchant']
    if not outs: return
    prev, g.batch = getattr(g, 'batch', None), None      # these sacrifices come before the wipe, one at a time
    try:
        for m in sorted([m for m in q.perms if m.creature and not m.phased and returns(g, m)],
                        key=lambda m: 'marchesa' in m.cd.tags):           # Marchesa last: she must see the others die
            if returns(g, m): sac_with(g, q, m, outs[0][1])
    finally:
        g.batch = prev


def wipe_loss(g, p, m, v):
    """the value p loses when a wipe destroys m: little when Marchesa returns it"""
    return 0.15 * v if returns(g, m) else v


def rean_targets(g, p, kind):
    """reanimation targets for Marchesa's deck: a creature's body plus what it does when it enters"""
    res = []
    for c in p.gy:
        if not c.creature: continue
        v = 0.6 * (c.bomb or c.pow) + card_etb_value(g, p, c) + (1.5 if c.name == 'Hellkite Tyrant' else 0)
        if v >= 3.0: res.append((v, c, p))
    if kind in ('reanimate', 'animate', 'necro'):
        for q in g.opps(p):
            for c in q.gy:
                if c.creature and c.pow >= 5: res.append((c.pow - 0.5, c, q))
    res.sort(key=lambda x: -x[0])
    return res


def marchesa_dredge(g, p):
    """Stinkweed Imp: dredge 5 instead of the draw-step draw when the graveyard is worth filling"""
    imps = [c for c in p.gy if 'dredge' in c.tags]
    if not imps or len(p.library) < 25: return False
    rean = any(c.tags.get('rean') or 'unearth' in c.tags or c.name == 'Phyrexian Delver' for c in p.hand)
    if not rean or any(c.creature and card_etb_value(g, p, c) >= 2.0 for c in p.gy): return False
    p.gy.remove(imps[0]); p.hand.append(imps[0])
    mill(g, p, 5); p.stats['dredged'] += 1
    log(f'  {NAME(p)} dredges Stinkweed Imp (mills five)', g)
    return True


CI.marchesa_dredge = marchesa_dredge


# ================================================================== the deck's cards
# ------------------------------------------------------------------ Act of Treason
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


@on('Act of Treason', 'resolve')
def _treason(g, p, c, ctx):
    t = ctx.get('target') or (None if _you(g, p) else best_steal(g, p, c))
    if t is None or t not in t.owner.perms or t.owner is p or untargetable(g, t): return
    from commander_sim import ais
    if ais.protect_response(g, t.owner, t, 'steal', p, c) or t not in t.owner.perms: return
    steal(g, p, t, until_eot=True)


def treason_value(g, p):
    """what Act of Treason is worth now: removing the creature for good with a sacrifice outlet, else one attack"""
    c = next((x for x in p.hand if x.name == 'Act of Treason'), None)
    t = best_steal(g, p, c) if c is not None else None
    if t is None: return 0
    v = pval(g, t)
    if outlets(g, p) or any(x.name in ('Deadly Dispute', 'Lethal Throwdown') for x in p.hand): return v
    return 0.3 * v if epow(g, t) >= 4 else 0
card('Act of Treason', '', types='S', dsl=[])
full('Act of Treason', 'gains control of the best opposing creature until end of turn, untapped and hasty; the AI '
     'casts it before combat when a sacrifice outlet can remove the creature for good (Marchesa returns it to you '
     'if it picked up a counter)')


# ------------------------------------------------------------------ Enslave
@on('Enslave', 'etb')
def _enslave(g, src, p, m):
    if m is not src: return
    t = _you(g, p).pick_creature(g, p, 'Enslave: enchant (and gain control of) which creature?') if _you(g, p) \
        else best_steal(g, p, src.cd)
    if t is None or t.owner is p:
        leave(g, src); to_zone_card(g, src, 'gy'); return        # no legal target: the Aura goes to the graveyard
    from commander_sim import ais
    if ais.protect_response(g, t.owner, t, 'steal', p, src.cd) or t not in t.owner.perms:
        leave(g, src); to_zone_card(g, src, 'gy'); return
    steal(g, p, t, until_eot=False)
    src.attached = t


@on('Enslave', 'upkeep')
def _enslave_upkeep(g, src, p):
    """at the beginning of your upkeep, the enchanted creature deals 1 damage to its owner"""
    h = src.attached
    if p is not src.owner or h is None or h not in src.owner.perms or not h.orig.alive or h.orig is src.owner: return
    if trigger_window(g, p, src, f'1 damage to {NAME(h.orig)}'): lose_life(g, h.orig, 1, src.owner, kind='triggers', damage=True)


@on('Enslave', 'leaves')
def _enslave_leaves(g, src):
    h = src.attached
    if h is not None and h in src.owner.perms and h.orig is not src.owner and h.orig.alive:
        src.owner.perms.remove(h); h.owner = h.orig; h.orig.perms.append(h); g.bf_ver = getattr(g, 'bf_ver', 0) + 1


@on('Enslave', 'sba')
def _enslave_sba(g, src):
    if src in src.owner.perms and (src.attached is None or src.attached not in src.attached.owner.perms):
        src.attached = None; leave(g, src); to_zone_card(g, src, 'gy')   # the creature left: the Aura goes too
card('Enslave', 'aura', types='E', dsl=[])
full('Enslave', 'steals the best opposing creature for as long as it stays attached; your upkeep: it deals 1 damage '
     'to its owner; control returns if Enslave leaves, and Enslave goes to the graveyard if the creature does')


# ------------------------------------------------------------------ Soul Enervation
def _gy_creatures(p):
    return {id(c) for c in p.gy if c.creature}


@on('Soul Enervation', 'etb')
def _enervation(g, src, p, m):
    if m is not src: return
    o = src.owner
    if not trigger_window(g, o, src, 'a creature gets -4/-4'): return
    src.data = dict(src.data or {}, gy=_gy_creatures(o))
    if _you(g, o):
        t = _you(g, o).pick_creature(g, o, 'Soul Enervation: which creature gets -4/-4?')
        if t is not None: _eot(g, t, -4, -4); check_state(g)
        return
    tg = [x for x in legal_targets(g, o, 'shrink4', 'c', spell=src.cd)]
    if tg: apply_removal(g, o, max(tg, key=lambda x: pval(g, x)), 'shrink4', src.cd)
    else:                                          # a mandatory target: shrink an opponent's biggest creature anyway
        any_opp = [x for q in g.opps(o) for x in q.perms if x.creature and not x.phased and not untargetable(g, x)]
        if any_opp: t = max(any_opp, key=lambda x: pval(g, x)); _eot(g, t, -4, -4)


@on('Soul Enervation', 'sba')
def _enervation_drain(g, src):
    """whenever one or more creature cards leave your graveyard: each opponent loses 1 life, you gain 1"""
    p = src.owner
    if src not in p.perms or src.phased: return
    now = _gy_creatures(p)
    before = (src.data or {}).get('gy', set())
    src.data = dict(src.data or {}, gy=now)
    if before - now:
        for q in g.opps(p): lose_life(g, q, 1, p, kind='drain')
        gain(p, 1)
        log(f'    Soul Enervation drains each opponent for 1', g)


@on('Soul Enervation', 'hand_options')
def _enervation_flash(g, c, p, s, post):
    """flash: cast at the end of an opponent's turn to kill a creature"""
    if post is not None or g.active is p or c not in p.hand or not can_pay(g, p, 3, 'B'): return []
    tg = legal_targets(g, p, 'shrink4', 'c', spell=c)
    if not tg: return []
    v = max(pval(g, m) for m in tg)
    if v < 3.0: return []
    return [(v - 2.0, 'Soul Enervation (flash)', lambda: IC_cast_perm(g, p, c, 3, 'B'))]


def IC_cast_perm(g, p, c, gen, pips):
    if c not in p.hand or not can_pay(g, p, gen, pips): return False
    pay(g, p, gen, pips); cast_card(g, p, c, 'hand', {}); return True
card('Soul Enervation', 'flash', types='E', dsl=[])
full('Soul Enervation', 'flash (cast at an opponent\'s end of turn for a kill); enters: target creature gets -4/-4; '
     'whenever creature cards leave your graveyard (Marchesa, reanimation, exile), each opponent loses 1, you gain 1')


# ------------------------------------------------------------------ Al Bhed Salvagers
def _salvage(g, src):
    p = src.owner
    opps = g.opps(p)
    if not opps: return
    if E.human_choice(g, p) is not None:
        q = E.human_choice(g, p).target_opponent(g, p, 'Al Bhed Salvagers')
        lose_life(g, q, 1, p, kind='drain'); gain(p, 1); return
    lethal = [q for q in opps if q.life <= 1]
    lose_life(g, lethal[0] if lethal else min(opps, key=lambda q: q.life), 1, p, kind='drain'); gain(p, 1)


@on('Al Bhed Salvagers', 'dies')
def _salvagers_dies(g, src, m, cause):
    if m is src or m.owner is not src.owner or src not in src.owner.perms or src.phased: return
    if (m.creature or (m.cd is not None and 'A' in m.cd.types)) and trigger_window(g, src.owner, src, 'an opponent loses 1 life'):
        _salvage(g, src)


@on('Al Bhed Salvagers', 'self_dies')
def _salvagers_self(g, m, cause):
    if trigger_window(g, m.owner, m, 'an opponent loses 1 life'): _salvage(g, m)


@on('Al Bhed Salvagers', 'sacrifice')
def _salvagers_treasure(g, src, p, what):
    if p is src.owner and what in ('Treasure', 'Clue', 'Food') and src in p.perms and not src.phased \
            and trigger_window(g, p, src, 'an opponent loses 1 life'): _salvage(g, src)
card('Al Bhed Salvagers', 'human warrior pow=2 tgh=3', types='C', dsl=[])
full('Al Bhed Salvagers', 'whenever it or another creature or artifact you control dies (Treasures too): the opponent '
     'nearest death loses 1 life and you gain 1')
IC.DEATH_DRAIN['Al Bhed Salvagers'] = 1


# ------------------------------------------------------------------ Balustrade Spy
@on('Balustrade Spy', 'etb')
def _spy(g, src, p, m):
    """target player reveals until a land and mills the rest: you, when the graveyard feeds reanimation"""
    if m is not src: return
    o = src.owner
    if not trigger_window(g, o, src, 'a player reveals until a land'): return
    rean = any(c.tags.get('rean') or 'unearth' in c.tags or c.name in ('Phyrexian Delver', 'Zombify') for c in o.hand) \
        or any(x.cd is not None and x.cd.name == 'Grave Researcher // Reanimate' for x in o.perms)
    q = o if (rean and len(o.library) >= 25) else max(g.opps(o), key=lambda x: threat(g, o, x), default=o)
    if _you(g, o):
        ps = [x for x in g.players if x.alive]
        q = ps[E.human_choice(g, o).choose(g, o, 'target', 'Balustrade Spy: which player reveals until a land and mills?',
                                           [NAME(x) + (' (you)' if x is o else '') for x in ps], cancel=None)]
    n = 0
    while q.library:
        c = q.library.pop(); q.gy.append(c); n += 1
        if c.land: break
    log(f'    Balustrade Spy mills {n} from {NAME(q)}', g)
card('Balustrade Spy', 'pow=2 tgh=3 fly', types='C', dsl=[])
full('Balustrade Spy', 'flying; enters: a player mills until a land (you, when you hold reanimation, else an opponent)')


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


# ------------------------------------------------------------------ Relic of Legends
def _legends_to_tap(g, p):
    """untapped legendary creatures whose tapping costs nothing: summoning sick, or after combat"""
    return [m for m in p.perms if m.creature and not m.tapped and not m.phased and m.cd is not None and IM.is_legendary(g, m)
            and (m.sick or g.active is not p or g.step in ('main2', 'end') or m.noatk)]


def _legend_to_tap(g, p):
    ls = _legends_to_tap(g, p)
    return ls[0] if ls else None


@on('Relic of Legends', 'etb')
def _relic_legends_live(g, src, p, m): pass            # registers the card so its mana hook is live


CI.DYN_MANA['Relic of Legends'] = lambda g, p, m: 1 + (len(_legends_to_tap(g, p)) if human_choice(g, p) is None else 0)


def _relic_legends_tap(g, p, m, used):
    """the AI taps one spare legend per extra mana (the person uses Relic's second ability themselves)"""
    if human_choice(g, p) is not None: return
    for x in _legends_to_tap(g, p)[:max(0, used - 1)]: x.tapped = True
CI.ON_TAP['Relic of Legends'] = _relic_legends_tap
full('Relic of Legends', '{T}: one mana of any colour; tap an untapped legendary creature you control: one more (the '
     'AI taps those that would not attack anyway: summoning sick, or after combat)')


# ------------------------------------------------------------------ Gemstone Mine, Vivid lands
def _counters(L, n0):
    return (L.data or {}).get('ctr', n0)


def _set_counters(L, n):
    L.data = dict(L.data or {}, ctr=n)


CI.LAND_COLS['Gemstone Mine'] = lambda g, p, L: p.ident


def _mine_tap(g, p, L, used):
    k = _counters(L, 3) - 1
    _set_counters(L, k)
    if k <= 0 and L in p.lands:
        p.lands.remove(L); p.gy.append(L.cd)
        log(f'    Gemstone Mine runs out and is sacrificed', g)
CI.ON_TAP['Gemstone Mine'] = _mine_tap
full('Gemstone Mine', 'enters with three mining counters; each use removes one for a mana of any colour, and it is '
     'sacrificed when the last is gone')

for _n, _c in (('Vivid Creek', 'U'), ('Vivid Marsh', 'B')):
    CI.LAND_COLS[_n] = lambda g, p, L, c=_c: p.ident if _counters(L, 2) > 0 else c

    def _vivid_tap(g, p, L, used, c=_c):
        if any(x != c for x in E.TAP_COLS): _set_counters(L, max(0, _counters(L, 2) - 1))
    CI.ON_TAP[_n] = _vivid_tap
    full(_n, f'enters tapped with two charge counters; {{T}}: {{{_c}}}, or remove a counter for any colour (counted '
         'only when another colour was needed)')


# ------------------------------------------------------------------ Crackling Drake
IC.SELF_PT['Crackling Drake'] = lambda g, p, m: (sum(1 for c in m.orig.gy + m.orig.exile if c.instant or c.sorcery),
                                                 0)


@on('Crackling Drake', 'etb')
def _drake(g, src, p, m):
    if m is src:
        g.selfpt = True
        if trigger_window(g, src.owner, src, 'draw a card'): draw(g, src.owner, 1)
card('Crackling Drake', 'pow=0 tgh=4 fly', types='C', dsl=[])
full('Crackling Drake', 'flying; power = instants and sorceries you own in graveyard and exile; enters: draw a card')


# ------------------------------------------------------------------ Deep Analysis (flashback pays 3 life)
card('Deep Analysis', 'draw=2 fb=1U fblife=3', types='S', dsl=[])
full('Deep Analysis', 'draw two; flashback {1}{U} and 3 life')


# ------------------------------------------------------------------ Disintegrate: X damage, exiled if it would die
@on('Disintegrate', 'hand_options')
def _disintegrate(g, c, p, s, post):
    if post is None or c not in p.hand: return []
    x_max = total_mana(g, p) - 1
    if x_max < 1 or not can_pay(g, p, x_max, 'R'): return []
    o = []
    lethal = [q for q in g.opps(p) if q.life <= x_max and not shielded(q)]
    if lethal:
        q = min(lethal, key=lambda q: q.life)
        o.append((12.0, f'Disintegrate X={q.life} -> {NAME(q)}', lambda q=q: _dis_face(g, p, c, q)))
    tg = [m for m in legal_targets(g, p, f'dmg{x_max}', 'c', spell=c)]
    if tg:
        t = max(tg, key=lambda m: pval(g, m) - 0.25 * etgh(g, m))
        u = pval(g, t) - 3.0 - 0.25 * etgh(g, t)
        if u > 0: o.append((u, f'Disintegrate X={etgh(g, t)} -> {t.name}', lambda t=t: _dis_creature(g, p, c, t)))
    return o


def _dis_face(g, p, c, q):
    x = q.life
    if c not in p.hand or not can_pay(g, p, x, 'R'): return False
    pay(g, p, x, 'R'); p.stats['removal_cast'] += 1
    cast_card(g, p, c, 'hand', {'face': q, 'rem_kind': f'dmg{x}'}); return True


def _dis_creature(g, p, c, t):
    x = max(0, etgh(g, t))
    if c not in p.hand or t not in t.owner.perms or not can_pay(g, p, x, 'R'): return False
    pay(g, p, x, 'R'); p.stats['removal_cast'] += 1
    cast_card(g, p, c, 'hand', {'target': t, 'rem_kind': f'dmg{x}'}); return True
@on('Disintegrate', 'resolve')
def _disintegrate_resolve(g, p, c, ctx):
    x = int(ctx.get('rem_kind', 'dmg0')[3:])
    if ctx.get('face') is not None: lose_life(g, ctx['face'], x, p, kind='burn', damage=True)
    elif ctx.get('target') is not None and ctx['target'] in ctx['target'].owner.perms:
        apply_removal(g, p, ctx['target'], f'dmg{x}', c)
card('Disintegrate', 'exiledie noregen', types='S', dsl=[])
full('Disintegrate', 'X damage to any target (X = the toughness of the creature it kills, or a lethal player\'s life); '
     'a creature it would kill is exiled and can\'t be regenerated')


# ------------------------------------------------------------------ Disembowel: destroy a creature with MV X
def _disembowel_opts(g, c, p, s, post):
    if c not in p.hand: return []
    tg = [m for m in legal_targets(g, p, 'destroy', 'c', spell=c) if m.cd is not None and can_pay(g, p, m.cd.cmc, 'B')]
    if not tg: return []
    t = max(tg, key=lambda m: pval(g, m))
    u = pval(g, t) - 3.5 - 0.15 * t.cd.cmc
    if post is None: u += 1.2 * brain_caution(p)
    if u <= -1.0: return []

    def go(t=t):
        if c not in p.hand or t not in t.owner.perms or not can_pay(g, p, t.cd.cmc, 'B'): return False
        pay(g, p, t.cd.cmc, 'B'); p.stats['removal_cast'] += 1
        cast_card(g, p, c, 'hand', {'target': t}); return True
    return [(u, f'Disembowel X={t.cd.cmc} -> {t.name}', go)]


def brain_caution(p):
    from commander_sim.ai import brain
    return brain.style(p)['caution']
on('Disembowel', 'hand_options')(_disembowel_opts)
@on('Disembowel', 'resolve')
def _disembowel_resolve(g, p, c, ctx):
    t = ctx.get('target')
    if t is not None and t in t.owner.perms: apply_removal(g, p, t, 'destroy', c)
card('Disembowel', '', types='I', dsl=[])
full('Disembowel', 'instant: destroy target creature with mana value X (paid as X = its mana value)')


# ------------------------------------------------------------------ Lethal Throwdown
def _modified(g, m):
    return m.plus > 0 or any(e.attached is m for e in m.owner.perms) or bool(getattr(g, 'auras', None) and IC.auras_on(g, m))


@on('Lethal Throwdown', 'hand_options')
def _throwdown(g, c, p, s, post):
    """as an additional cost, sacrifice a creature (a modified one draws a card): destroy a creature or planeswalker"""
    if post is None or c not in p.hand or not can_pay(g, p, 0, 'B'): return []
    fod = [m for m in p.perms if m.creature and not m.phased and not m.is_cmd]
    tg = legal_targets(g, p, 'destroy', 'cp', spell=c)
    if not fod or not tg: return []
    f = min(fod, key=lambda m: sac_worth(g, m) - (1.5 if _modified(g, m) else 0))
    t = max(tg, key=lambda m: pval(g, m))
    u = pval(g, t) - 3.0 - sac_worth(g, f) + (1.5 if _modified(g, f) else 0)
    if u <= 0: return []

    def go(f=f, t=t):
        if c not in p.hand or f not in p.perms or t not in t.owner.perms or not can_pay(g, p, 0, 'B'): return False
        mod = _modified(g, f)
        pay(g, p, 0, 'B'); die(g, f, 'sac'); p.stats['removal_cast'] += 1
        cast_card(g, p, c, 'hand', {'target': t, 'modified': mod}); return True
    return [(u, f'Lethal Throwdown (sacrifice {f.name}) -> {t.name}', go)]


@on('Lethal Throwdown', 'resolve')
def _throwdown_resolve(g, p, c, ctx):
    t = ctx.get('target')
    if t is not None and t in t.owner.perms: apply_removal(g, p, t, 'destroy', c)
    if ctx.get('modified'): draw(g, p, 1)
card('Lethal Throwdown', '', types='S', dsl=[])
full('Lethal Throwdown', 'sacrifice a creature (the cheapest to lose: one Marchesa returns first): destroy target '
     'creature or planeswalker; draw a card if the sacrificed creature was modified (counters, Equipment, Auras)')


# ------------------------------------------------------------------ Festering Goblin
@on('Festering Goblin', 'self_dies')
def _festering(g, m, cause):
    """when it dies, target creature gets -1/-1 until end of turn"""
    p = m.owner
    if not trigger_window(g, p, m, 'a creature gets -1/-1'): return
    if _you(g, p):
        t = _you(g, p).pick_creature(g, p, 'Festering Goblin: which creature gets -1/-1 until end of turn?')
        if t is not None: _eot(g, t, -1, -1); check_state(g)
        return
    x1 = _opp_x1(g, p)
    if x1:
        t = max(x1, key=lambda x: pval(g, x))
        if pval(g, t) >= 1.0: apply_removal(g, p, t, 'shrink1'); return
    cr = [x for q in g.opps(p) for x in q.perms if x.creature and not x.phased and not untargetable(g, x)]
    if cr: _eot(g, max(cr, key=lambda x: pval(g, x)), -1, -1)
card('Festering Goblin', 'pow=1 tgh=1', types='C', dsl=[])
full('Festering Goblin', 'dies: -1/-1 until end of turn to an opposing creature (an X/1 worth killing first)')


# ------------------------------------------------------------------ Forge Devil
@on('Forge Devil', 'etb')
def _forge_devil(g, src, p, m):
    """1 damage to target creature and 1 damage to you (the target is mandatory)"""
    if m is not src: return
    o = src.owner
    if not trigger_window(g, o, src, '1 damage to a creature and to you'): return
    if _you(g, o):
        t = _you(g, o).pick_creature(g, o, 'Forge Devil: 1 damage to which creature?')
        if t is not None: _you(g, o)._damage(g, o, t, 1, 'Forge Devil', 'triggers')
        lose_life(g, o, 1, o, damage=True); return
    x1 = _opp_x1(g, o)
    if x1: apply_removal(g, o, max(x1, key=lambda x: pval(g, x)), 'dmg1')
    elif not any(x.creature for q in g.opps(o) for x in q.perms if not untargetable(g, x)):
        own = [x for x in o.perms if x.creature and not x.phased and etgh(g, x) > 1]
        if not own: die(g, src, 'destroy')            # it has to hit itself
    lose_life(g, o, 1, o, damage=True)
card('Forge Devil', 'pow=1 tgh=1', types='C', dsl=[])
full('Forge Devil', 'enters: 1 damage to a creature (an opposing X/1 when there is one; itself when it is the only '
     'target) and 1 damage to you')


# ------------------------------------------------------------------ Grave Researcher // Reanimate
@on('Grave Researcher // Reanimate', 'upkeep')
def _researcher(g, src, p):
    """your upkeep: surveil 1, then with three or more creature cards in your graveyard it becomes prepared"""
    if p is not src.owner or src.phased: return
    if not trigger_window(g, p, src, 'surveil 1'): return
    from commander_sim.cards.impl import topdeck
    topdeck.scry(g, p, 1, to='gy')
    if sum(1 for c in p.gy if c.creature) >= 3: src.data = dict(src.data or {}, prepared=True)


@on('Grave Researcher // Reanimate', 'options')
def _researcher_cast(g, src, p, s, post):
    """prepared: cast a copy of Reanimate ({B}): a creature card from a graveyard onto the battlefield under your
    control, losing life equal to its mana value"""
    if p is not src.owner or post is None or not (src.data and src.data.get('prepared')) or not can_pay(g, p, 0, 'B'):
        return []
    tg = [x for x in reanimate_targets(g, p, 'reanimate') if p.life - x[1].cmc >= 12]
    if not tg: return []
    v, cd, q = tg[0]

    def go():
        if not (src.data and src.data.get('prepared')) or cd not in q.gy or not can_pay(g, p, 0, 'B'): return False
        pay(g, p, 0, 'B'); src.data['prepared'] = False
        log(f'  {NAME(p)} casts a copy of Reanimate (Grave Researcher) -> {cd.name}', g)
        def eff():
            if cd not in q.gy: return
            if g.hooks and CI.gy_response(g, p, v, q): return
            q.gy.remove(cd); enter(g, p, cd, orig=q); lose_life(g, p, cd.cmc, p)
        cast_copy(g, p, eff, name='Reanimate', instant=False, imp=max(5, v))
        return True
    return [(v * 0.9 * (1 - 0.35 * s.ctr_risk), f'Reanimate (prepared copy) -> {cd.name}', go)]


def reanimate_targets(g, p, kind):
    from commander_sim import ais
    return ais.rean_targets(g, p, kind)
card('Grave Researcher // Reanimate', 'pow=3 tgh=3', types='C', dsl=[])
full('Grave Researcher // Reanimate', 'your upkeep: surveil 1, then prepared with three or more creature cards in '
     'your graveyard; prepared: a copy of Reanimate for {B} (a creature from any graveyard, lose life equal to its '
     'mana value)')


# ------------------------------------------------------------------ Phyrexian Delver
@on('Phyrexian Delver', 'etb')
def _delver(g, src, p, m):
    """return target creature card from your graveyard to the battlefield; lose life equal to its mana value"""
    if m is not src: return
    o = src.owner
    if not any(c.creature for c in o.gy) or not trigger_window(g, o, src, 'return a creature card'): return
    if _you(g, o):
        cs = [c for c in o.gy if c.creature]
        if not cs: return
        cd = E.human_choice(g, o).pick_cards(g, o, cs, 1, 'Phyrexian Delver: return which creature card (you lose life equal to its mana value)?')[0]
        if g.hooks and CI.gy_response(g, o, 5, o): return
        o.gy.remove(cd); enter(g, o, cd, orig=o); lose_life(g, o, cd.cmc, o)
        log(f'    Phyrexian Delver returns {cd.name}', g); return
    tg = [x for x in reanimate_targets(g, o, 'evil') if x[2] is o and x[1] is not src.cd and o.life - x[1].cmc >= 8]
    if not tg:                                       # a mandatory target: the cheapest creature card, if any
        cs = [c for c in o.gy if c.creature and o.life - c.cmc > 5]
        if not cs: return
        cd = min(cs, key=lambda c: c.cmc)
    else: cd = tg[0][1]
    if g.hooks and CI.gy_response(g, o, 5, o): return
    o.gy.remove(cd); enter(g, o, cd, orig=o); lose_life(g, o, cd.cmc, o)
    log(f'    Phyrexian Delver returns {cd.name}', g)
card('Phyrexian Delver', 'pow=3 tgh=2', types='C', dsl=[])
full('Phyrexian Delver', 'enters: a creature card from your graveyard to the battlefield; you lose life equal to its '
     'mana value')


# ------------------------------------------------------------------ Pteramander: adapt 4
@on('Pteramander', 'options')
def _pteramander(g, src, p, s, post):
    if p is not src.owner or post is None or src.phased or src.plus > 0: return []
    n = max(0, 7 - sum(1 for c in p.gy if c.instant or c.sorcery))
    if not can_pay(g, p, n, 'U'): return []

    def go():
        if src not in p.perms or src.plus > 0 or not can_pay(g, p, n, 'U'): return False
        pay(g, p, n, 'U')
        log(f'  {NAME(p)} adapts Pteramander (4 counters)', g)
        if ability_window(g, p, src, 'adapt 4') and src in p.perms and src.plus <= 0: src.plus += 4
        return True
    return [(2.0 + (1.5 if marchesa_out(p) else 0) - 0.25 * n, 'Pteramander: adapt 4', go)]
card('Pteramander', 'pow=1 tgh=1 fly', types='C', dsl=[])
full('Pteramander', 'flying; {7}{U}, {1} less per instant and sorcery in your graveyard: adapt 4')


# ------------------------------------------------------------------ Skullport Merchant
@on('Skullport Merchant', 'etb')
def _merchant(g, src, p, m):
    if m is src and trigger_window(g, src.owner, src, 'a Treasure'): add_treasure(g, src.owner, 1)


@on('Skullport Merchant', 'options')
def _merchant_draw(g, src, p, s, post):
    """{1}{B}, sacrifice another creature or a Treasure: draw a card (a spare Treasure with mana left over; creatures
    through marchesa_options)"""
    if p is not src.owner or src.phased or not p.treasures or stopped(g, src.cd.name): return []
    if post is not None and post is not True: return []
    if total_mana(g, p) < 4 or not can_pay(g, p, 1, 'B'): return []

    def go():
        if not p.treasures or not can_pay(g, p, 1, 'B'): return False
        pay(g, p, 1, 'B')
        if not p.treasures: return True
        p.treasures -= 1
        if g.hooks: CI.fire(g, 'sacrifice', p, 'Treasure')
        if ability_window(g, p, src, 'draw a card'): draw(g, p, 1)
        return True
    return [(1.2 if post is None else 0.6, 'Skullport Merchant (sacrifice a Treasure): draw', go)]
card('Skullport Merchant', 'pow=1 tgh=4', types='C', dsl=[])
full('Skullport Merchant', 'enters: a Treasure; {1}{B}, sacrifice another creature or a Treasure: draw a card')


# ------------------------------------------------------------------ Dualcaster Mage
def _dualcaster_copy(g, c, p, spell, own):
    """flash it in when an instant or sorcery worth copying is cast; the copy gets new targets"""
    if getattr(g, 'in_dualcaster', False) or c not in p.hand or spell is c: return
    if not (spell.instant or spell.sorcery) or 'wipe' in spell.tags or 'ctr' in spell.tags: return
    v = IM.spell_copy_value(g, p, spell) + (1.5 if own else 0)
    if v < 4.0 or not can_pay(g, p, 1, 'RR'): return
    g.in_dualcaster = True
    try:
        pay(g, p, 1, 'RR')
        cc = getattr(g, 'cur_cast', None)
        ctx = cc[1] if own and cc is not None and cc[0] is spell else None
        log(f'  {NAME(p)} flashes in Dualcaster Mage to copy {spell.name}', g)
        if cast_card(g, p, c, 'hand', {}) and not g.over:
            copy_spell(g, p, spell, dict(ctx) if ctx else None)
    finally:
        g.in_dualcaster = False


on('Dualcaster Mage', 'hand_cast')(lambda g, c, p, spell: _dualcaster_copy(g, c, p, spell, True))
on('Dualcaster Mage', 'hand_opp_cast')(lambda g, c, p, spell: _dualcaster_copy(g, c, p, spell, False))
card('Dualcaster Mage', 'pow=2 tgh=2 flash', types='C', dsl=[])
full('Dualcaster Mage', 'flash; flashed in as an instant or sorcery worth copying is cast (yours or an opponent\'s) '
     'and copies it with new targets')


# ------------------------------------------------------------------ Notion Thief: flash at end of turn
@on('Notion Thief', 'hand_options')
def _thief_flash(g, c, p, s, post):
    if post is not None or g.active is p or c not in p.hand or not can_pay(g, p, 2, 'UB'): return []
    return [(4.0, 'Notion Thief (flash)', lambda: IC_cast_perm(g, p, c, 2, 'UB'))]


# ------------------------------------------------------------------ Crux of Fate
def _dragon(m):
    return has_type(m, 'dragon')


def crux_mode(g, p):
    """'dragons' or 'others': the mode with the better swing (your creatures Marchesa returns cost little)"""
    def swing(pred):
        v = 0.0
        for q in g.players:
            if not q.alive: continue
            for m in q.perms:
                if m.creature and not m.phased and pred(m):
                    v += -1.2 * wipe_loss(g, p, m, pval(g, m)) if q is p else pval(g, m)
        return v
    return 'dragons' if swing(_dragon) > swing(lambda m: not _dragon(m)) else 'others'


@on('Crux of Fate', 'resolve')
def _crux(g, p, c, ctx):
    mode = _you(g, p).crux_mode(g, p) if _you(g, p) else crux_mode(g, p)
    log(f'    Crux of Fate: destroy all {"Dragons" if mode == "dragons" else "non-Dragon creatures"}', g)
    from commander_sim import ais
    prev, g.batch = getattr(g, 'batch', None), object()
    try:
        for q in [q for q in g.players if q.alive]:
            prot = ais.wipe_response(g, q, 'destroy', p)
            if prot == 'all': continue
            for m in list(q.perms):
                if not m.creature or m.phased or _dragon(m) != (mode == 'dragons'): continue
                if prot == 'indes': continue
                die(g, m, 'destroy')
    finally:
        g.batch = prev
    check_state(g)
card('Crux of Fate', 'wipe=destroy', types='S', dsl=[])
full('Crux of Fate', 'destroy all Dragons or all non-Dragon creatures, whichever mode is better (Hellkite Tyrant '
     'survives the usual mode)')


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
full('Terror', 'destroy target nonartifact, nonblack creature; it can\'t be regenerated')
full('Last Gasp', 'target creature gets -3/-3 (kills an indestructible X/3)')
full('Orcish Cannonade', '2 damage to any target and 3 damage to you; draw a card')
full('Scorching Dragonfire', '3 damage to a creature, exiled if it would die (planeswalker damage is not modeled, as '
     'for every burn spell)')
full('Premature Burial', 'destroy a nonblack creature that entered since your last turn ended')
full('Zombify', 'return the best creature card from your graveyard to the battlefield')
full('Sephiroth, Planet\'s Heir', 'vigilance; enters: opponents\' creatures get -2/-2 until end of turn; a +1/+1 '
     'counter whenever a creature an opponent controls dies')
full("Tezzeret's Gambit", 'draw two, then proliferate (your creatures\' +1/+1 counters, loyalty, opponents\' -1/-1 '
     'and poison); {U/P} is paid with 2 life when blue is short')


CI.marchesa_rean_targets = rean_targets
CI.marchesa_options = marchesa_options
CI.marchesa_protect = protect
CI.marchesa_wipe_response = wipe_response
CI.marchesa_wipe_loss = wipe_loss
CI.treason_value = treason_value
CI.marchesa_steal_value = lambda g, p: max((pval(g, m) for m in legal_targets(g, p, 'steal', 'c')
                                            if not m.phased and not m.is_cmd), default=0)
