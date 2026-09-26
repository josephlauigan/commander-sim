"""Adaptive AI: probabilistic, board-state-driven decisions.

How a decision is made
  1. read the board (Situation): turn, mana, hand, how much damage is pointed at me,
     who is leading, whether an opponent is close to a combo, what my opponents
     probably hold (estimated from public information only).
  2. list every legal play and give it a utility (a score on a ~0-10 scale).
  3. turn utilities into a probability distribution (softmax with a temperature) and
     sample a play.  Higher scores are more likely, not certain.
  4. do the play, re-read the board, rebuild the distribution, repeat until the AI
     samples "stop here" (hold up mana) or nothing legal is left.

Temperature controls how human the AI is: low = near-optimal and predictable,
high = looser, more varied play.  Each deck also has a play style (aggression,
caution) that shifts the utilities.
"""
import importlib
import math
from collections import defaultdict
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim import ais as A

# ------------------------------------------------------------------ play styles
STYLE = {
    # temp: randomness of choices; aggression: attacking/pressure; caution: holding up
    # interaction and keeping blockers home
    'seph':    {'temp': 1.0, 'aggression': 0.55, 'caution': 0.60},
    'veyran':  {'temp': 1.0, 'aggression': 0.40, 'caution': 0.80},
    'sauron':  {'temp': 1.0, 'aggression': 0.55, 'caution': 0.75},
    'najeela': {'temp': 1.0, 'aggression': 0.90, 'caution': 0.30},
}
TEMP_SCALE = 1.0     # global multiplier, set from --temp


def style(p):
    return STYLE[p.key] if p.key in STYLE else importlib.import_module('commander_sim.ai.pool_ai').style(p)


def T(p):
    return max(0.05, style(p)['temp'] * TEMP_SCALE)


def sig(x):
    return 1.0 / (1.0 + math.exp(-max(-30, min(30, x))))


def gumbel_order(rng, items, temp):
    """Sample an ordering from the softmax distribution (Plackett-Luce, via Gumbel keys)."""
    keyed = []
    for u, o in items:
        r = rng.random() or 1e-12
        keyed.append((u / temp - math.log(-math.log(r)), o))
    keyed.sort(key=lambda x: -x[0])
    return [o for _, o in keyed]


def sample(rng, items, temp):
    return gumbel_order(rng, items, temp)[0] if items else None


def explain(g, p, items, chosen, what):
    """Trace line: the distribution the AI sampled from."""
    if g.log is None or len(items) < 2: return
    t = T(p)
    mx = max(u for u, _ in items)
    ws = [(math.exp((u - mx) / t), lbl) for u, lbl in items]
    tot = sum(w for w, _ in ws)
    top = sorted(ws, key=lambda x: -x[0])[:4]
    dist = ', '.join(f'{lbl} {100 * w / tot:.0f}%' for w, lbl in top)
    log(f'      [{NAME(p)} {what}: {dist} -> {chosen}]', g)


# ------------------------------------------------------------------ opponent model
def full_deck(p):
    if not hasattr(p, 'full'):
        p.full = list(p.library) + list(p.hand) + list(p.gy) + [p.cmd]
    return p.full


def prob_holding(g, q, pred):
    """P(opponent q holds >=1 card matching pred), from public info only:
    their decklist minus copies already seen (graveyard, exile, battlefield)."""
    total = sum(1 for c in full_deck(q) if pred(c))
    seen = sum(1 for c in q.gy + q.exile if pred(c)) + sum(1 for m in q.perms if m.cd is not None and pred(m.cd))
    left = max(0, total - seen)
    unknown = len(q.library) + len(q.hand)
    if left == 0 or unknown == 0 or not q.hand: return 0.0
    d = min(1.0, left / unknown)
    return 1.0 - (1.0 - d) ** len(q.hand)


def open_mana(g, q, colour=None):
    U = mana_units(g, q)
    tot = sum(u[2] for u in U)
    if colour is None: return tot
    return tot if any(colour in u[1] for u in U) else 0


def is_counter(c): return 'ctr' in c.tags
def is_instant_removal(c): return c.instant and 'rem' in c.tags


def counter_risk(g, p):
    """P(my next important spell gets countered)."""
    miss = 1.0
    for q in g.opps(p):
        ph = prob_holding(g, q, is_counter)
        if not ph: continue
        up = open_mana(g, q, 'U')
        can = 1.0 if up >= 2 else (0.4 if up >= 1 else 0.0)
        if any('free' in c.tags for c in full_deck(q) if is_counter(c)): can = max(can, 0.25)
        miss *= 1.0 - ph * can * 0.85
    return 1.0 - miss


def removal_risk(g, p):
    """P(an opponent answers my best threat at instant speed this cycle)."""
    miss = 1.0
    for q in g.opps(p):
        ph = prob_holding(g, q, is_instant_removal)
        if ph and open_mana(g, q) >= 2: miss *= 1.0 - ph * 0.8
    return 1.0 - miss


# ------------------------------------------------------------------ board reading
class Situation:
    def __init__(s, g, p):
        s.turn = p.turns
        s.opps = g.opps(p)
        s.mana = total_mana(g, p)
        s.hand = len(p.hand)
        s.lands_in_hand = sum(1 for c in p.hand if c.land)
        s.threat = {q: threat(g, p, q) for q in s.opps}
        s.leader = max(s.opps, key=s.threat.get) if s.opps else None
        s.max_threat = max(s.threat.values()) if s.opps else 0
        inc = 0.0
        for q in s.opps:
            share = 1.0 / max(1, len(g.opps(q)))
            for m in q.perms:
                if m.creature and not m.noatk and not m.phased:
                    inc += epow(g, m) * (1.0 if m.fly else 0.6) * share
        s.incoming = inc
        s.danger = inc / max(1, p.life)
        s.combo_near = any(
            (q.key == 'veyran' and has(q, 'vkitten') and has(q, 'vfire')) or
            (q.key == 'sauron' and has(q, 'sword') and has(q, 'assault')) or
            (q.key == 'veyran' and has(q, 'aether') and q.life >= 40)
            for q in s.opps)
        s.ctr_risk = counter_risk(g, p)
        s.phase = 'early' if s.turn <= 4 else ('late' if s.turn >= 9 else 'mid')


HELD = lambda c: (('ctr' in c.tags and 'free' not in c.tags) or c.tags.get('prot') in ('hi', 'phase', 'indes', 'blink')
                  or 'tide' in c.tags)


def hold_value(g, p, s):
    """How much the AI wants to keep mana open right now."""
    held = [c for c in p.hand if HELD(c) and can_pay(g, p, c.generic, c.pips)]
    if not held: return None, 0.0
    c = min(held, key=lambda c: c.cmc)
    caution = style(p)['caution']
    v = 1.0 + 3.0 * caution * min(1.0, s.max_threat / 20.0) + 2.5 * s.combo_near
    if p.key not in STYLE:          # outside decks: three opponents will cast something worth answering this cycle
        n = sum(1 for x in held if 'ctr' in x.tags)
        v = max(v, 1.5 + 2.5 * caution * min(2, n))
    if p.key == 'seph' and A.bomb_on_bf(p): v += 2.0 * removal_risk(g, p) + 1.0
    if s.turn <= 2: v -= 2.0
    return c, v


def reserve_penalty(g, p, s, c, hold_card, hold_v):
    if hold_card is None or c is hold_card: return 0.0
    cg, cp = cost_of(p, c)
    if can_pay(g, p, cg + hold_card.generic, cp + hold_card.pips): return 0.0
    return 0.7 * hold_v


# ------------------------------------------------------------------ card utilities
PRIO = {'seph': A.seph_prio, 'veyran': A.veyran_prio, 'sauron': A.sauron_prio, 'najeela': A.najeela_prio}


def draws_cards(c):
    if 'draw' in c.tags: return True
    return any(e.get('do') == 'draw' and e.get('who', 'you') == 'you'
               for a in (c.dsl or ()) if a.get('type') in ('spell', 'triggered') for e in a.get('effects', ()))


def card_utility(g, p, s, c):
    base = PRIO[p.key](g, p, c) if p.key in PRIO else A.deck_prio(g, p, c)
    if p.key not in PRIO and len(p.library) < 8 and draws_cards(c): return None    # don't draw yourself out
    if p.key not in PRIO and 'ctr' in c.tags and not c.creature: return None       # counters wait for a spell to counter
    if base <= 0 and c.dsl and E.DSLMOD is not None: base = E.DSLMOD.card_value(g, p, c) * 10
    if base <= 0: return None
    u = base / 10.0                                   # deck knowledge as a prior (0-9)
    t = c.tags
    if 'rock' in t or 'dork' in t or 'lr' in t:
        if s.lands_in_hand == 0 and s.turn <= 6: u += 1.5
        u -= 0.35 * max(0, s.turn - 5)
    if 'draw' in t or 'eng' in t:
        if s.hand <= 2: u += 1.2
        if s.danger > 0.8: u -= 0.8                  # no time to durdle
    if c.creature and c.pow >= 2 and s.danger > 0.5: u += 0.8   # need bodies to block
    if c is p.cmd: u -= 0.35 * p.tax                   # recasting gets pricier each time
    if (c.instant or c.sorcery) and p.key == 'veyran':
        # each spell fires every magecraft payoff (doubled by Veyran): casting is value in itself
        n_pay = sum(1 for m in p.perms if m.cd is not None and not m.phased and
                    any(k in m.cd.tags for k in ('ping', 'dragoncaller', 'mystic', 'spelldraw', 'aether', 'spelltok', 'kiln')))
        u += 0.9 * n_pay * (2 if has(p, 'veyran') else 1)
    if (c.bomb >= 5 or c is p.cmd or base >= 70) and not c.land:
        u *= 1.0 - 0.35 * s.ctr_risk                  # walking into open counter mana
    return u


# ------------------------------------------------------------------ generic executors
# cards whose casting needs deck-specific choices (targets, modes, X); never cast them generically
SPECIAL = ('rean', 'fill', 'yawg', 'avarice', 'mastery', 'crackle', 'tokx_special')


def do_cast(g, p, c, zone=None):
    if any(k in c.tags for k in SPECIAL) and not c.dsl and not (c.creature and p.key not in STYLE): return False
    if c.dsl and not additional_cost(g, p, c, dry=True): return False
    if zone in (None, 'hand') and c not in p.hand and not (c is p.cmd and p.cmd_in_zone): return False
    if zone == 'yawg' and c not in p.gy: return False
    if g.hooks and not castable(g, p, c, 'gy' if zone == 'yawg' else zone or ('cmd' if (c is p.cmd and c not in p.hand) else 'hand')): return False
    cv = 'convoke' in c.tags
    if zone == 'gy':
        cg, cp = parse_cost(c.tags['fb'])
    else:
        cg, cp = cost_of(p, c)
    E.PAY_FOR = c
    try:
        if not can_pay(g, p, cg, cp, cv): return False
        pay(g, p, cg, cp, cv)
    finally:
        E.PAY_FOR = None
    if zone is None: zone = 'cmd' if (c is p.cmd and c not in p.hand) else 'hand'
    ctx = {}
    if 'tokx' in c.tags:
        x = total_mana(g, p, cv); pay(g, p, x, '', cv); ctx['x'] = x
    elif 'xtutor' in c.tags:                         # X spells of outside decks: X = all spare mana
        x = total_mana(g, p, cv); pay(g, p, x, '', cv); ctx['x'] = x; g.last_x = x
    if c.dsl: additional_cost(g, p, c)
    ok = cast_card(g, p, c, 'gy' if zone == 'yawg' else zone, ctx)     # from the graveyard: exiled after
    if p.key == 'seph' and ok and (c is p.cmd or c.bomb >= 4): A.note_bomb(p, c)
    return True


def removal_options(g, p, s):
    out = []
    for c in p.hand:
        t = c.tags
        if 'rem' not in t or c.creature: continue
        extra = 4 if ('sacor4' in t and not [m for m in p.perms if m.creature and (m.token or not m.cd.bomb)]) else 0
        free = ('freecmd' in t and commander_out(p)) or ('snuff' in t and p.life > 12 and any(
            'swamp' in L.cd.subtypes or L.cd.name == 'Swamp' for L in p.lands))
        if not free and not can_pay(g, p, c.generic + extra, c.pips, 'convoke' in t): continue
        if 'needart3' in t and sum(1 for m in p.perms if m.cd is not None and 'A' in m.cd.types) < 3: continue
        tg = legal_targets(g, p, t['rem'], t.get('tgt', 'c'), 'mv4' in t, spell=c)
        if not tg: continue
        best = max(tg, key=lambda m: pval(g, m))
        v = pval(g, best)
        if best.owner is s.leader: v *= 1.25
        if s.combo_near and pval(g, best) >= 8: v += 2.0
        if p.key not in STYLE and v < 2.5: continue                          # outside decks: no removal on trivial targets
        u = v - 3.5
        if c.instant: u -= 1.2 * style(p)['caution'] * E.INSTANT_EXTRA / 2.0   # instants are worth holding (profile-dependent)
        if 'needsac' in t and not [m for m in p.perms if m.creature and (m.token or not m.cd.bomb)]: continue
        if 'pactpay' in t and not E.pact_affordable(g, p, t['pactpay']): continue   # Slaughter Pact: or lose the game
        out.append((u, f'{c.name} -> {best.name}', lambda c=c, tg=tg: cast_removal(g, p, c, tg)))
        if 'kick' in t and t['rem'].startswith('dmg') and can_pay(g, p, c.generic + int(t['kick']), c.pips):
            kd = f"dmg{int(t['rem'][3:]) + 2}"                       # Burst Lightning kicked: 4 damage
            tg4 = legal_targets(g, p, kd, t.get('tgt', 'c'), 'mv4' in t, spell=c)
            b4 = max(tg4, key=lambda m: pval(g, m)) if tg4 else None
            if b4 is not None and pval(g, b4) > v + 1.0:
                out.append((pval(g, b4) - 4.5, f'{c.name} (kicked) -> {b4.name}',
                            lambda c=c, tg4=tg4, kd=kd: cast_removal(g, p, c, tg4, int(c.tags['kick']), kd)))
    return out


def cast_removal(g, p, c, tg, kick=0, kind=None):
    cv = 'convoke' in c.tags
    fods = [m for m in p.perms if m.creature and (m.token or not m.cd.bomb)]
    extra = 4 if ('sacor4' in c.tags and not fods) else 0
    free = ('freecmd' in c.tags and commander_out(p)) or ('snuff' in c.tags and p.life > 12 and any(
        'swamp' in L.cd.subtypes or L.cd.name == 'Swamp' for L in p.lands))
    extra += kick
    if c not in p.hand or (not free and not can_pay(g, p, c.generic + extra, c.pips, cv)): return False
    if g.hooks and not castable(g, p, c): return False
    live = [m for m in tg if m in m.owner.perms and not untargetable(g, m)]
    if not live: return False
    target = sample(g.rng, [(pval(g, m) / 1.5, m) for m in live], T(p))
    fod = None
    if 'needsac' in c.tags or ('sacor4' in c.tags and fods):
        if not fods: return False
        fod = min(fods, key=lambda x: pval(g, x))
    if free and 'snuff' in c.tags and not ('freecmd' in c.tags and commander_out(p)): lose_life(g, p, 4, p)
    elif not free: pay(g, p, c.generic + extra, c.pips, cv)
    if fod: die(g, fod, 'sac')
    cast_card(g, p, c, 'hand', dict({'target': target}, **({'rem_kind': kind} if kind else {})))
    p.stats['removal_cast'] += 1
    return True


def wipe_options(g, p, s):
    out = []
    for c in p.hand:
        kind = c.tags.get('wipe')
        if not kind: continue
        cg, cp = A.wipe_cost(p, c)
        if not can_pay(g, p, cg, cp): continue
        ol, ml, victim = A.wipe_eval(g, p, kind)
        swing = ol - 1.2 * ml
        u = swing / 2.2 - 2.5 + 2.0 * min(1.5, s.danger)

        def go(c=c, cg=cg, cp=cp, victim=victim):
            if c not in p.hand or not can_pay(g, p, cg, cp): return False
            if g.hooks and not castable(g, p, c): return False
            pay(g, p, cg, cp); cast_card(g, p, c, 'hand', {'victim': victim}); p.stats['wipes_cast'] += 1
            return True
        out.append((u, c.name, go))
    return out


# ------------------------------------------------------------------ deck-specific plays
def _payable_in_hand(g, p, pred):
    return any(pred(c) and can_pay(g, p, c.generic, c.pips) for c in p.hand)


def special_options(g, p, s, post):
    """Only legal plays are listed, so the distribution is over real choices."""
    k = p.key; o = []
    if k == 'seph':
        gy_bomb = A.own_bomb_in_gy(g, p)
        rean = A.has_rean_access(g, p)
        # reanimation: best payable (spell, target) pair
        best_t = 0
        for c, zone, cg, cp in A.rean_options(g, p):
            tg = A.rean_targets(g, p, c.tags['rean'])
            if c.tags['rean'] == 'reanimate': tg = [x for x in tg if p.life - x[1].cmc >= 12]
            if tg and can_pay(g, p, cg, cp): best_t = max(best_t, tg[0][0])
        if best_t:
            o.append((best_t * (1 - 0.45 * s.ctr_risk) + (1.0 if s.danger > 0.5 else 0), 'reanimate',
                      lambda: A.seph_reanimate(g, p)))
        ys = [c for c in p.hand if 'yawg' in c.tags]
        reans_gy = [c for c in p.gy if 'rean' in c.tags]
        if ys and reans_gy and gy_bomb and not any('rean' in c.tags for c in p.hand):
            ch = min(reans_gy, key=lambda c: c.cmc)
            if can_pay(g, p, 2 + ch.generic, 'B' + ch.pips):
                o.append((7.5, "Yawgmoth's Will line", lambda: A.seph_yawg(g, p)))
        te = any(m.cd is not None and m.cd.tags.get('fill') == 'tortured' for m in p.perms)
        if (te and not gy_bomb and rean and getattr(p, 'te_used', -1) != p.turns
                and any(c.creature and c.bomb >= 6 for c in p.hand) and can_pay(g, p, 0, 'B')):
            o.append((6.0, 'Tortured Existence discard', lambda: A.seph_tortured(g, p)))
        tut_in_hand = any(c.tags.get('tut') for c in p.hand)
        if (not gy_bomb and (rean or tut_in_hand) and any(c.creature and c.bomb >= 5 for c in p.library)
                and _payable_in_hand(g, p, lambda c: c.name in ('Entomb', 'Buried Alive', 'Unmarked Grave', 'Grisly Salvage'))):
            o.append((6.3 if rean else 3.0, 'fill graveyard', lambda: A.seph_fill(g, p)))
        if (_payable_in_hand(g, p, lambda c: c.tags.get('tut') == 'any') and not (A.bomb_on_bf(p) and p.turns < 6)
                and A.seph_tutor_target(g, p)):
            missing = not A.bomb_on_bf(p) and (rean != gy_bomb)
            o.append((6.2 if missing else 3.0, 'tutor', lambda: A.seph_tutor(g, p)))
        if any('avarice' in c.tags for c in p.hand) and can_pay(g, p, 2, 'B'):
            o.append((5.0, 'Insatiable Avarice', lambda: A.seph_avarice(g, p)))
        hc = 0
        if p.cmd_in_zone and can_pay(g, p, *cost_of(p, p.cmd)): hc = 8.0
        for c in p.hand:
            if c.creature and c.bomb >= 4 and can_pay(g, p, c.generic, c.pips):
                hc = max(hc, A.seph_bval(g, p, c) - 1.0)
        if hc: o.append((hc * (1 - 0.35 * s.ctr_risk), 'hardcast bomb', lambda: A.seph_hardcast(g, p)))
        if (any(any(L.cd.tags.get('desert') for L in q.lands) for q in s.opps) and (rean or gy_bomb)
                and _payable_in_hand(g, p, lambda c: c.tags.get('tgt') == 'p' and c.tags.get('rem') == 'destroy')):
            o.append((7.5, 'Trophy the Scavenger Grounds', lambda: A.seph_trophy_grounds(g, p)))
        if (has(p, 'clamp') and can_pay(g, p, 1, '') and getattr(p, 'clamp_n', 0) < 2 and
                any(m.creature and etgh(g, m) == 1 and (m.token or m.cd.tags.get('fill') in ('stitcher', 'wayfinder'))
                    for m in p.perms)):
            o.append((4.0, 'Skullclamp', lambda: A.seph_clamp(g, p)))
        if (_payable_in_hand(g, p, lambda c: c.tags.get('fill') == 'dispute') and
                (p.treasures or any(m.creature and (m.token or m.cd.tags.get('fill') == 'stitcher') for m in p.perms))):
            o.append((3.5, 'Deadly Dispute', lambda: A.seph_dispute(g, p)))
        if (can_pay(g, p, 1, '') and any(e.cd.tags.get('prot') == 'boots' and e.attached is None for e in find(p, 'prot'))
                and any(m.creature and m.cd is not None and m.cd.bomb >= 6 and not untargetable(g, m) for m in p.perms)):
            o.append((3.0 + 4.0 * removal_risk(g, p), 'equip Boots', lambda: A.seph_boots(g, p)))
    elif k == 'veyran':
        if (has(p, 'vkitten') and has(p, 'vfire') and A.payoff(p) and not p.combo_tried
                and can_pay(g, p, 2, 'R')):
            risk = 1.0 - (1.0 - counter_risk(g, p)) * (1.0 - removal_risk(g, p))
            u = 12.0 - 8.0 * risk + (3.0 if s.danger > 0.6 else 0.0)
            o.append((u, 'go for the combo', lambda: A.veyran_try_combo(g, p)))
        if has(p, 'aether') and p.life >= 51:
            o.append((11.0, 'Aetherflux shot', lambda: A.aether_check(g, p)))
        if (any(L.cd.tags.get('fair') and not L.tapped for L in p.lands) and not has(p, 'aether')
                and not any('aether' in c.tags for c in p.hand) and any('aether' in c.tags for c in p.library)
                and sum(1 for m in p.perms if m.cd is not None and 'A' in m.cd.types) >= 3 and total_mana(g, p) >= 5):
            o.append((6.5, "Inventors' Fair for Aetherflux", lambda: A.veyran_fair(g, p)))
        if (can_pay(g, p, 1, '') and any(e.cd.tags.get('prot') == 'boots' and (e.attached is None or e.attached not in p.perms)
                                          for e in find(p, 'prot'))
                and any(m.creature and m.cd is not None and ('veyran' in m.cd.tags or 'vkitten' in m.cd.tags) for m in p.perms)):
            o.append((3.0 + 5.0 * removal_risk(g, p), 'equip Boots', lambda: A.veyran_boots(g, p)))
        ms = [c for c in p.hand if 'mastery' in c.tags]
        k = sum(1 for c in p.gy if (c.instant or c.sorcery) and 'ctr' not in c.tags)
        if ms and k >= 4 and can_pay(g, p, 5, 'RRR'):
            def mastery(c=ms[0]):
                if c not in p.hand or not can_pay(g, p, 5, 'RRR'): return False
                pay(g, p, 5, 'RRR'); cast_card(g, p, c, 'hand', {'overload': True}); return True
            payoff_n = sum(1 for m in p.perms if m.cd is not None and ('ping' in m.cd.tags or 'dragoncaller' in m.cd.tags
                                                                      or 'aether' in m.cd.tags or 'mystic' in m.cd.tags))
            o.append((3.0 + 0.5 * k + 1.5 * payoff_n, f"Mizzix's Mastery overload ({k} spells)", mastery))
        if ms and k >= 1 and can_pay(g, p, 3, 'R'):                  # one target: the best instant/sorcery copied free
            best = max((E.card_worth(g, p, x, in_gy=True) for x in p.gy if (x.instant or x.sorcery) and 'ctr' not in x.tags), default=0)
            def mastery1(c=ms[0]):
                if c not in p.hand or not can_pay(g, p, 3, 'R'): return False
                pay(g, p, 3, 'R'); cast_card(g, p, c, 'hand', {}); return True
            if best >= 4: o.append((1.0 + 0.4 * best, "Mizzix's Mastery (one spell)", mastery1))
    elif k == 'sauron':
        a = army_of(p)
        if a and not equipped(a, 'cloak') and can_pay(g, p, 2, ''):
            for tag in ('sword', 'cloak'):
                if not equipped(a, tag) and any(e.attached is None or e.attached not in p.perms for e in find(p, tag)):
                    o.append((6.5, 'equip the Army', lambda: A.sauron_equip(g, p))); break
        ht = A.helm_target(g, p) if find(p, 'helm') and can_pay(g, p, 1, '') else None
        if ht is not None:
            leg = importlib.import_module('commander_sim.cards.impl.mine').is_legendary(g, ht)
            u = 3.0 + (4.0 * removal_risk(g, p) + 0.2 * pval(g, ht) if leg else 0.0)
            o.append((u, f"equip Champion's Helm to {ht.name}", lambda ht=ht: A.helm_equip(g, p, ht)))
        if (any(not m.tapped and not m.sick for m in find(p, 'archivist')) and importlib.import_module('commander_sim.cards.impl.mine').archivist_worth(g, p)
                and can_pay(g, p, 0, 'U')):
            o.append((5.0, "Jace's Archivist wheel", lambda: A.sauron_archivist(g, p)))
    elif k == 'najeela' and post:
        for c in p.hand:
            if 'tokx' in c.tags and total_mana(g, p) >= len(c.pips) + 3:
                o.append((3.5 + (total_mana(g, p) - len(c.pips)) / 2.0, f'{c.name} (X)',
                          lambda c=c: do_cast(g, p, c)))
    return o


def extra_options(g, p, s, post, sorcery_ok):
    o = []
    # Disruptor Flute (flash): worth casting when there's a name that really hurts an opponent
    for c in p.hand:
        if 'flute' in c.tags and can_pay(g, p, *cost_of(p, c)):
            name, score = A.flute_pick(g, p)
            if name and score >= 3:
                o.append((score * 0.8 - 1.0, f'Disruptor Flute (naming {name})', lambda c=c: do_cast(g, p, c)))
            break
    # flashback from the graveyard
    for c in list(p.gy):
        if 'fb' not in c.tags or (c.sorcery and not sorcery_ok): continue
        cg, cp = parse_cost(c.tags['fb'])
        if not can_pay(g, p, cg, cp): continue
        u = card_utility(g, p, s, c)
        if 'fbnib' in c.tags:
            ol, ml, _ = A.wipe_eval(g, p, 'nib'); u = (ol - 1.2 * ml) / 2.2 - 1.0 + 1.5
        if u is None: continue
        o.append((u - 0.5, f'{c.name} (flashback)', lambda c=c: do_cast(g, p, c, 'gy')))
    # Crackle with Power: X = (mana - 2) / 3
    for c in p.hand:
        if 'crackle' in c.tags and (sorcery_ok or has(p, 'gandalf')):
            x = (total_mana(g, p) - 2) // 3
            if x < 1: continue
            lethal = sum(1 for q in s.opps if q.life <= 5 * x)
            u = 2.0 + 3.0 * lethal + x
            def crackle(c=c, x=x):
                if c not in p.hand or not can_pay(g, p, 3 * x, 'RR'): return False
                pay(g, p, 3 * x, 'RR'); cast_card(g, p, c, 'hand', {'x': x}); return True
            o.append((u, f'Crackle with Power X={x}', crackle))
    # burn to the face when it kills
    thor = 1 if has(p, 'thor') else 0
    for c in p.hand:
        r = c.tags.get('rem', '')
        if 'face' not in c.tags or not r.startswith('dmg') or not can_pay(g, p, c.generic, c.pips): continue
        dmg = int(r[3:]) + thor
        kick = int(c.tags['kick']) if 'kick' in c.tags and can_pay(g, p, c.generic + int(c.tags['kick']), c.pips) else 0
        for q in s.opps:
            if q.life <= dmg:
                o.append((9.0, f'{c.name} to the face ({NAME(q)})',
                          lambda c=c, q=q: (pay(g, p, c.generic, c.pips), cast_card(g, p, c, 'hand', {'face': q}))[1] is not None))
            elif kick and q.life <= dmg + 2:                       # kicked burn is lethal
                o.append((9.0, f'{c.name} kicked to the face ({NAME(q)})',
                          lambda c=c, q=q, kick=kick: (pay(g, p, c.generic + kick, c.pips),
                                                       cast_card(g, p, c, 'hand', {'face': q, 'rem_kind': f"dmg{int(c.tags['rem'][3:]) + 2}"}))[1] is not None))
    # clues
    if p.clues and can_pay(g, p, 2, ''):
        def crack():
            if not p.clues or not can_pay(g, p, 2, ''): return False
            pay(g, p, 2, ''); p.clues -= 1; draw(g, p, 1)
            if g.hooks: E.CI.fire(g, 'sacrifice', p, 'Clue')
            return True
        o.append((1.5 if not post else 2.5, 'crack a Clue', crack))
    if p.key == 'najeela' and not post:
        # Rhys the Redeemed: copy every creature token
        rh = [m for m in find(p, 'rhys') if not m.tapped and not m.sick and not A.blocked(g, p, m.cd.name)]
        toks = [m for m in p.perms if m.token and m.creature]
        if rh and len(toks) >= 4 and can_pay(g, p, 4, 'GW'):
            def double(m=rh[0]):
                if not can_pay(g, p, 4, 'GW'): return False
                pay(g, p, 4, 'GW'); m.tapped = True
                for x in [x for x in p.perms if x.token and x.creature]:
                    make_tokens(g, p, 1, x.pow, x.tgh, fly=x.fly, warrior=x.warrior)
                return True
            o.append((4.0 + len(toks) / 3.0, 'Rhys doubles the tokens', double))
        # Sauron, the Lidless Eye: {1}{B}{R}: +2/+0 and each opponent loses 2
        if has(p, 'lidless') and can_pay(g, p, 1, 'BR') and not stopped(g, 'Sauron, the Lidless Eye'):
            atk = [m for m in p.perms if m.creature and not m.sick and not m.noatk]
            def lidless():
                if not can_pay(g, p, 1, 'BR'): return False
                pay(g, p, 1, 'BR'); p.pumpadd += 2
                for q in g.opps(p): lose_life(g, q, 2, p, kind='drain')
                check_state(g); return True
            o.append((2.5 + 0.4 * len(atk), 'Lidless Eye pump', lidless))
    return o


# ------------------------------------------------------------------ main phase
def main_options(g, p, post):
    """every legal main-phase play for p right now, as (utility, label, fn); fn None is 'stop'"""
    s = Situation(g, p)
    hold_card, hold_v = hold_value(g, p, s)
    opts = []
    # Najeela keeps WUBRG up before combat when the attack is on
    naj_hold = (p.key == 'najeela' and not post and has(p, 'najeela')
                and len([m for m in p.perms if m.creature]) >= 3)
    for c in list(p.hand) + ([p.cmd] if p.cmd_in_zone else []):
        if c.land: continue
        u = card_utility(g, p, s, c)
        if u is None: continue
        cg, cp = cost_of(p, c)
        E.PAY_FOR = c; ok = can_pay(g, p, cg, cp, 'convoke' in c.tags); E.PAY_FOR = None
        if not ok: continue
        u -= reserve_penalty(g, p, s, c, hold_card, hold_v)
        if p.key not in STYLE and not post:                     # outside decks: mana kept for combat (ninjutsu)
            rsv = importlib.import_module('commander_sim.ai.pool_ai').combat_reserve(g, p)
            if rsv and not can_pay(g, p, cg + rsv[0], cp + rsv[1]): u -= 6.0
        if naj_hold and not can_pay(g, p, cg, cp + 'WUBRG'): u -= 4.0
        opts.append((u, c.name, lambda c=c: do_cast(g, p, c)))
    if getattr(p, 'yawg', False):                # Yawgmoth's Will: spells from the graveyard (reanimation has its own path)
        for c in list(p.gy):
            if c.land or id(c) not in p.yawg_gy or any(k in c.tags for k in SPECIAL): continue
            u = card_utility(g, p, s, c)
            if u is None: continue
            cg, cp = cost_of(p, c)
            if not can_pay(g, p, cg, cp): continue
            opts.append((u, c.name + ' (Yawgmoth\'s Will)', lambda c=c: do_cast(g, p, c, 'yawg')))
    opts += removal_options(g, p, s)
    opts += wipe_options(g, p, s)
    opts += special_options(g, p, s, post)
    opts += extra_options(g, p, s, post, sorcery_ok=True)
    opts += A.breach_gc_options(g, p)
    if E.DSLMOD is not None and g.dsl_on: opts += E.DSLMOD.ability_options(g, p, True)
    if E.CI is not None: opts += hook_options(g, p, s, post)
    stop_u = (hold_v if hold_card is not None else -3.0) + (1.5 if naj_hold else 0.0)
    if p.key not in STYLE and not post and importlib.import_module('commander_sim.ai.pool_ai').combat_reserve(g, p):
        stop_u = max(stop_u, 4.5)                                  # ninjutsu window: go to combat, cast after
    opts.append((stop_u, 'stop (hold mana)' if hold_card is not None else 'stop', None))
    return opts


def main(g, p, post):
    for _ in range(18):
        if g.over or not p.alive: return
        E.tick(g)
        opts = main_options(g, p, post)
        if len(opts) >= 2:
            from commander_sim.ai import search
            if search.enabled(g, p):
                pick = search.choose(g, p, post, opts)
                if pick is not None:
                    u, lbl, fn = pick
                    explain(g, p, [(x, l) for x, l, _ in opts], lbl + ' [search]', 'decides')
                    if fn is None: return
                    if fn(): continue
        order = gumbel_order(g.rng, [(u, (u, lbl, fn)) for u, lbl, fn in opts], T(p))
        acted = False
        for u, lbl, fn in order:
            if fn is None:
                explain(g, p, [(x, l) for x, l, _ in opts], lbl, 'decides')
                return
            if fn():
                explain(g, p, [(x, l) for x, l, _ in opts], lbl, 'decides')
                acted = True
                break
        if not acted: return


def hook_options(g, p, s, post):
    """activated abilities of hand-implemented cards (battlefield and graveyard); post=None: end-of-turn window"""
    o = []
    if g.hooks:
        lock = importlib.import_module('commander_sim.cards.impl.rules').ability_locked
        for src, fn in E.CI.hooked(g, 'options'):
            if src.owner is p and not (lock and lock(g, src, p)): o += fn(g, src, p, s, post) or []
    for c, fn in E.CI.gy_cards(p, 'gy_options'): o += fn(g, c, p, s, post) or []
    for c, fn in E.CI.hand_cards(p, 'hand_options'): o += fn(g, c, p, s, post) or []
    if p.key not in STYLE:
        o += importlib.import_module('commander_sim.ai.pool_ai').special_options(g, p, s, post)
    o += importlib.import_module('commander_sim.cards.impl.lands').land_options(g, p, s, post)
    if E.CI.combo_options is not None: o += E.CI.combo_options(g, p, s, post)
    return o


# ------------------------------------------------------------------ reactive decisions
def wants_counter(g, q, val, thr, ncounters):
    """Probability q spends a counter on a spell of importance val (to q)."""
    scarcity = 0.65 + 0.35 * min(1.0, ncounters / 2.0)
    return sig((val - thr + 0.5) / 0.8) * scarcity


def choose_defender(g, p):
    opps = [q for q in g.opps(p) if not shielded(q)] or g.opps(p)     # combat damage to a protected player is prevented
    my = sum(epow(g, m) for m in p.perms if m.creature and not m.tapped and not m.noatk and not m.sick)
    aggr = style(p)['aggression']
    items = []
    grudge = getattr(p, 'grudge', {})
    for q in opps:
        u = 0.25 * threat(g, p, q)
        if my >= q.life * 0.8: u += 4.0 + 2.0 * aggr               # go for the kill
        u += 0.15 * grudge.get(q.key, 0)                               # hit back whoever hit you
        if p.key not in STYLE: u += importlib.import_module('commander_sim.ai.pool_ai').ninja_defender_bonus(g, p, q)
        from commander_sim.cards.impl import common as impl_common                                              # planeswalkers about to ultimate draw attacks
        u += sum(3.0 * min(1.0, impl_common.ult_pressure(m)) for m in q.perms
                 if m.cd is not None and 'P' in m.cd.types and m.loyalty and impl_common.ult_pressure(m) >= 0.6)
        blockers = sum(1 for m in q.perms if m.creature and not m.tapped)
        u -= 0.15 * blockers * (1 - aggr)
        if g.hooks:                                                     # attack taxes and caps on q
            tax, cap = A.attack_restrictions(g, p, q)
            u -= 1.2 * tax + (1.5 if cap is not None and cap <= 2 else 0)
        items.append((u, q))
    d = sample(g.rng, items, T(p))
    explain(g, p, [(u, NAME(q)) for u, q in items], NAME(d), 'attacks')
    return d


def filter_attackers(g, p, atk):
    """Keep some ground creatures home when the table threatens a lot of damage."""
    s = Situation(g, p)
    caution = style(p)['caution']; aggr = style(p)['aggression']
    if s.danger < 0.25: return atk
    keep_p = sig((s.danger * caution - 0.35 * aggr - 0.15) / 0.12)
    out = []
    for m in atk:
        if m.fly or m.vig or m.army or m.token and p.key == 'najeela':
            out.append(m); continue
        if g.rng.random() < keep_p * (0.5 + 0.5 * min(1.0, etgh(g, m) / 4.0)):
            continue
        out.append(m)
    if len(out) < len(atk): log(f'      [{NAME(p)} keeps {len(atk) - len(out)} creature(s) home as blockers]', g)
    return out


def chump_prob(g, d, incoming):
    frac = incoming / max(1, d.life)
    return sig((frac - 0.35) / 0.08)


def note_damage(p, src, n):
    if src is None or src is p or not hasattr(p, 'key'): return
    if not hasattr(p, 'grudge'): p.grudge = defaultdict(float)
    p.grudge[src.key] = p.grudge[src.key] * 0.7 + n


E.DAMAGE_HOOK = note_damage


# ------------------------------------------------------------------ end-of-turn window
def end_of_turn_window(g, p):
    A.erebos_draw(g, p)
    """At the end of the turn before yours, spend mana you held up but didn't need:
    instant-speed draw, removal and token spells.  Mana untaps next, so holding has no value."""
    for _ in range(6):
        if g.over or not p.alive: return
        s = Situation(g, p)
        opts = []
        gand = has(p, 'gandalf')               # Gandalf: sorceries at instant speed
        for c in p.hand:
            if not (c.instant or (gand and c.sorcery)) or c.tags.get('prot') or 'rem' in c.tags or 'wipe' in c.tags: continue
            if any(k in c.tags for k in SPECIAL): continue       # cast only through their deck-specific logic
            if 'ctr' in c.tags:
                if 'eotdraw' in c.tags and can_pay(g, p, c.generic, c.pips):     # Mystic Confluence: draw three
                    def conf(c=c):
                        if c not in p.hand or not can_pay(g, p, c.generic, c.pips): return False
                        p.hand.remove(c); pay(g, p, c.generic, c.pips); p.gy.append(c)
                        p.spells_this_turn += 1; on_cast(g, p, c); draw(g, p, int(c.tags['eotdraw'])); return True
                    opts.append((2.5, f'{c.name} (draw three)', conf))
                continue
            cv = 'convoke' in c.tags
            if not can_pay(g, p, c.generic, c.pips, cv): continue
            if 'draw' in c.tags or 'tokx' in c.tags or 'treas' in c.tags or c.sorcery or 'gifts' in c.tags or 'intuition' in c.tags \
                    or 'seal' in c.tags or 'adnaus' in c.tags:
                u = card_utility(g, p, s, c)
                if u is None: u = 3.0
                if 'tokx' in c.tags: u = 3.0 + total_mana(g, p, cv) / 2.0
                opts.append((u, c.name, lambda c=c: do_cast(g, p, c)))
        opts += [x for x in extra_options(g, p, s, True, sorcery_ok=gand) if 'Rhys' not in x[1] and 'Lidless' not in x[1]]
        if E.DSLMOD is not None and g.dsl_on: opts += E.DSLMOD.ability_options(g, p, False)
        if E.CI is not None: opts += hook_options(g, p, s, None)
        opts += A.monolith_untap_options(g, p)
        for u, lbl, fn in removal_options(g, p, s):
            if lbl.split(' -> ')[0] in [c.name for c in p.hand if c.instant]:
                opts.append((u + 1.2 * style(p)['caution'], lbl, fn))
        if not opts: return
        opts.append((-1.5, 'pass', None))
        order = gumbel_order(g.rng, [(u, (u, lbl, fn)) for u, lbl, fn in opts], T(p))
        done = False
        for u, lbl, fn in order:
            if fn is None: return
            if fn():
                p.stats['eot_casts'] += 1
                explain(g, p, [(x, l) for x, l, _ in opts], lbl, 'end of turn')
                done = True; break
        if not done: return