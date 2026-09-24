"""Combos for the opponent pools. A combo is assembled from pieces; when a player has everything it needs, the AI
goes for it: any spell still to be cast goes through the normal counter window, then each opponent gets one chance to
answer a key piece with instant-speed removal (or with a counter / ability counter where noted). If nothing stops it,
the loop is executed abstractly: the player wins (or, for locks, the lock applies).

Opponents value the pieces of a combo that is one step from completion much higher (removal, counterspells), and
combo decks get tutor wish lists for their missing pieces.
"""
from engine import *
import engine as E
import cardimpl as CI
from pool_cards import note

COMBOS = []
PIECES = set()


class Combo:
    def __init__(s, name, groups, ready, mana=(0, ''), finish=None, status='Approximate', text=''):
        """groups: list of tuples of card names (one of each group is needed); ready(g, p) -> (ok, key permanents,
        cards to cast from hand); finish(g, p): what happens (default: p wins)"""
        s.name, s.groups, s.ready, s.mana, s.finish = name, groups, ready, mana, finish
        s.status, s.text = status, text
        for grp in groups: PIECES.update(grp)


def combo(name, groups, mana=(0, ''), finish=None, text='', status='Approximate'):
    def deco(fn):
        COMBOS.append(Combo(name, groups, fn, mana, finish, status, text))
        return fn
    return deco


def on_bf(p, name, untapped=False, unsick=False):
    for m in p.perms:
        if m.cd is not None and m.cd.name == name and not m.phased and (not untapped or not m.tapped) and \
                (not unsick or not m.sick or E.DSLMOD.has_kw(E.CUR_G, m, 'haste')):
            return m
    return None


def in_hand(p, name):
    return next((c for c in p.hand if c.name == name), None)


def any_bf(p, names, **kw):
    for n in names:
        m = on_bf(p, n, **kw)
        if m is not None: return m
    return None


def any_hand(p, names):
    for n in names:
        c = in_hand(p, n)
        if c is not None: return c
    return None


def free_outlet(p):
    import impl_common as IC
    outs = IC.outlets(p)
    return outs[0] if outs else None


def drain_payoff(p):
    names = ('Blood Artist', 'Zulaport Cutthroat', 'Cruel Celebrant', 'Bastion of Remembrance', 'Falkenrath Noble',
             'Mayhem Devil', 'Syr Konrad, the Grim', 'Vindictive Vampire', 'Poison-Tip Archer', 'Goblin Bombardment',
             'Elas il-Kor, Sadistic Pilgrim')
    return any_bf(p, names) or next((m for m in p.perms if m.cd is not None and ('bartist' in m.cd.tags or 'drain' in m.cd.tags)), None)


# ------------------------------------------------------------------ execution
def attempt(g, p, cmb):
    ok, keys, casts = cmb.ready(g, p)
    if not ok: return False
    gen, pips = cmb.mana
    if not can_pay(g, p, gen + sum(c.generic for c in casts), pips + ''.join(c.pips for c in casts)): return False
    if getattr(p, 'combo_turn', None) == (p.turns, cmb.name): return False
    p.combo_turn = (p.turns, cmb.name)
    p.stats['combo_attempt'] += 1
    p.milestone.setdefault('combo', p.turns)
    log(f'  {NAME(p)} goes for {cmb.name}', g)
    for i, c in enumerate(casts):                       # the last pieces are cast normally: they can be countered
        if c not in p.hand or not castable(g, p, c) or not can_pay(g, p, c.generic, c.pips):
            if i: log(f'    ...{cmb.name} fizzles ({c.name} is gone)', g); p.stats['combo_stopped'] += 1
            return i > 0                                 # something already happened: the turn state changed
        pay(g, p, c.generic, c.pips)
        g.combo_spell = True
        try:
            ok = cast_card(g, p, c, 'hand', {})
        finally:
            g.combo_spell = False
        if not ok:
            log(f'    ...{c.name} is countered: {cmb.name} stopped', g); p.stats['combo_stopped'] += 1; return True
    if gen or pips: pay(g, p, gen, pips)
    keys = [k for k in keys if k is not None and k in k.owner.perms]
    if interrupted(g, p, keys):
        log(f'    ...{cmb.name} is stopped', g); p.stats['combo_stopped'] += 1; return True
    if cmb.finish is not None: cmb.finish(g, p)
    else:
        import ais; ais.win(g, p, 'combo')
    return True


def interrupted(g, p, keys):
    """each opponent may answer one key piece with instant-speed removal (or a stifle effect) before the loop runs"""
    if not keys: return False
    for q in g.after(p):
        if not q.alive or q is p or g.over: continue
        if not castable_any(g, q): continue
        if g.rng.random() > 0.95: continue
        for c in list(q.hand):
            t = c.tags
            if 'rem' not in t or not (c.instant or 'flash' in t) or not castable(g, q, c): continue
            tg = [m for m in legal_targets(g, q, t['rem'], t.get('tgt', 'c'), 'mv4' in t, spell=c) if m in keys]
            if not tg or not can_pay(g, q, *cost_of(q, c)): continue
            pay(g, q, *cost_of(q, c))
            cast_card(g, q, c, 'hand', {'target': tg[0]})
            if tg[0] not in tg[0].owner.perms or tg[0].phased:
                q.stats['combo_stops_removal'] += 1
                return True
            break
        # Tishana's Tidebinder / Siren Stormtamer style: counter the activated ability
        for c in list(q.hand):
            if c.name == "Tishana's Tidebinder" and can_pay(g, q, 2, 'U') and castable(g, q, c):
                pay(g, q, 2, 'U'); q.hand.remove(c); enter(g, q, c); q.stats['combo_stops_ability'] += 1
                return True
    return False


def castable_any(g, q):
    return not g.hooks or CI.allowed(g, q, type('X', (), {'creature': False, 'land': False, 'types': 'I', 'pips': '',
                                                          'name': '', 'cmc': 0, 'tags': {}, 'instant': True})(), 'hand')


def combo_options(g, p, s, post):
    if post is None or not getattr(g, 'combo_decks', None) or p.key not in g.combo_decks: return []
    out = []
    for cmb in COMBOS:
        if not any(any(on_bf(p, n) or in_hand(p, n) for n in grp) for grp in cmb.groups): continue
        ok, keys, casts = cmb.ready(g, p)
        if not ok: continue
        gen, pips = cmb.mana
        if not can_pay(g, p, gen + sum(c.generic for c in casts), pips + ''.join(c.pips for c in casts)): continue
        risk = s.ctr_risk if casts else 0.0
        out.append((14.0 - 6.0 * risk, f'combo: {cmb.name}', lambda cmb=cmb: attempt(g, p, cmb)))
    return out


def missing_pieces(g, p):
    """for tutors: pieces of this player's closest combo that it doesn't have"""
    best = None
    for cmb in COMBOS:
        have = [grp for grp in cmb.groups if any(on_bf(p, n) or in_hand(p, n) for n in grp)]
        miss = [grp for grp in cmb.groups if grp not in have]
        if not have or not miss: continue
        lib = {c.name for c in p.library}
        want = [n for grp in miss for n in grp if n in lib]
        if not want: continue
        if best is None or len(miss) < best[0]: best = (len(miss), want)
    return best[1] if best else []


def piece_threat(g, m):
    """extra value of m if it's a piece of a combo whose other pieces its controller has on the battlefield"""
    if m.cd is None or m.cd.name not in PIECES: return 0
    p = m.owner
    for cmb in COMBOS:
        if not any(m.cd.name in grp for grp in cmb.groups): continue
        others = [grp for grp in cmb.groups if m.cd.name not in grp]
        if all(any(on_bf(p, n) for n in grp) for grp in others): return 7.0
        if sum(1 for grp in others if any(on_bf(p, n) or in_hand(p, n) for n in grp)) >= len(others) - 1: return 2.5
    return 0


def combo_imp(g, p, c):
    """importance of a spell for counterspell decisions: it completes (or nearly completes) a combo"""
    if c.name not in PIECES: return 0
    for cmb in COMBOS:
        if not any(c.name in grp for grp in cmb.groups): continue
        others = [grp for grp in cmb.groups if c.name not in grp]
        if all(any(on_bf(p, n) for n in grp) for grp in others): return 9
        if all(any(on_bf(p, n) or in_hand(p, n) for n in grp) for grp in others): return 7
    return 0


CI.combo_options = combo_options
CI.piece_threat = piece_threat
CI.combo_imp = combo_imp


# ================================================================== the combos
KIKI = ('Kiki-Jiki, Mirror Breaker',)
KIKI_TARGETS = ('Zealous Conscripts', 'Felidar Guardian', 'Restoration Angel', 'Pestermite', 'Deceiver Exarch')


@combo('Kiki-Jiki + Zealous Conscripts / Felidar / Resto', [KIKI + ('Fable of the Mirror-Breaker // Reflection of Kiki-Jiki',), KIKI_TARGETS],
       text='infinite hasty copies')
def _kiki(g, p):
    kiki = on_bf(p, 'Kiki-Jiki, Mirror Breaker', untapped=True, unsick=True)
    if kiki is None:
        f = on_bf(p, 'Fable of the Mirror-Breaker // Reflection of Kiki-Jiki', untapped=True)
        if f is not None and (f.data or {}).get('reflection') and (f.data or {}).get('flipped', p.turns) < p.turns: kiki = f
    if kiki is None: return False, [], []
    tgt = any_bf(p, KIKI_TARGETS)
    if tgt is not None: return True, [kiki, tgt], []
    c = any_hand(p, KIKI_TARGETS)
    if c is not None and can_pay(g, p, *cost_of(p, c)): return True, [kiki], [c]
    return False, [], []


@combo('Isochron Scepter + Dramatic Reversal', [('Isochron Scepter',), ('Dramatic Reversal',)],
       text='infinite mana with 3+ mana from nonland permanents; wins with a mana sink (Walking Ballista, Urza, Kinnan, '
            'Thassa\'s Oracle in hand ...)')
def _scepter(g, p):
    sc = on_bf(p, 'Isochron Scepter')
    rv = in_hand(p, 'Dramatic Reversal')
    if sc is None or rv is None: return False, [], []
    rocks = sum(u[2] for u in mana_units(g, p) if isinstance(u[0], Perm))
    if rocks < 3: return False, [], []
    sink = any_bf(p, ('Walking Ballista', 'Urza, Lord High Artificer', 'Kinnan, Bonder Prodigy', 'Grand Arbiter Augustin IV')) \
        or in_hand(p, 'Walking Ballista') or in_hand(p, "Thassa's Oracle")
    if sink is None: return False, [], []
    return True, [sc], []


@combo('Power Artifact / Rings of Brighthearth + Monolith', [('Power Artifact', 'Rings of Brighthearth'), ('Basalt Monolith', 'Grim Monolith')],
       text='infinite colourless mana; wins with Urza / Kinnan / Walking Ballista')
def _power_artifact(g, p):
    pa = any_bf(p, ('Power Artifact', 'Rings of Brighthearth'))
    mono = any_bf(p, ('Basalt Monolith', 'Grim Monolith'))
    if pa is None or mono is None: return False, [], []
    if pa.cd.name == 'Rings of Brighthearth' and mono.cd.name != 'Basalt Monolith': return False, [], []
    sink = any_bf(p, ('Urza, Lord High Artificer', 'Kinnan, Bonder Prodigy', 'Walking Ballista')) or in_hand(p, 'Walking Ballista')
    if sink is None: return False, [], []
    return True, [pa, mono], []


@combo('Kinnan + Basalt Monolith', [('Kinnan, Bonder Prodigy',), ('Basalt Monolith',)],
       text='infinite colourless mana; Kinnan\'s activation finds a winner')
def _kinnan_basalt(g, p):
    k = on_bf(p, 'Kinnan, Bonder Prodigy'); b = on_bf(p, 'Basalt Monolith')
    if k is None or b is None: return False, [], []
    return True, [k, b], []


@combo('Kinnan + Freed from the Real / Pemmin\'s Aura', [('Kinnan, Bonder Prodigy',), ("Freed from the Real", "Pemmin's Aura")],
       text='a mana creature that nets extra mana untaps for U: infinite mana; Kinnan\'s activation finds a winner')
def _kinnan_aura(g, p):
    k = on_bf(p, 'Kinnan, Bonder Prodigy')
    aura = any_bf(p, ("Freed from the Real", "Pemmin's Aura"))
    if k is None or aura is None: return False, [], []
    return True, [k, aura], []


@combo('Aluren + Chulane + a bouncing creature', [('Aluren',), ('Chulane, Teller of Tales',),
                                                  ('Shrieking Drake', 'Whitemane Lion', 'Kor Skyfisher', "Man-o'-War")],
       text='free creatures bounce themselves: infinite draws, land drops and ETBs')
def _aluren(g, p):
    a = on_bf(p, 'Aluren'); ch = on_bf(p, 'Chulane, Teller of Tales')
    if a is None or ch is None: return False, [], []
    b = any_bf(p, ('Shrieking Drake', 'Whitemane Lion', 'Kor Skyfisher', "Man-o'-War"))
    if b is not None: return True, [a, ch], []
    c = any_hand(p, ('Shrieking Drake', 'Whitemane Lion', 'Kor Skyfisher', "Man-o'-War"))
    if c is not None: return True, [a, ch], [c]
    return False, [], []


@combo('Sanguine Bond + Exquisite Blood', [('Sanguine Bond',), ('Exquisite Blood',)], text='infinite drain on any life change')
def _bond(g, p):
    a = on_bf(p, 'Sanguine Bond'); b = on_bf(p, 'Exquisite Blood')
    if a is not None and b is not None: return True, [a, b], []
    if a is not None and in_hand(p, 'Exquisite Blood'): return True, [a], [in_hand(p, 'Exquisite Blood')]
    if b is not None and in_hand(p, 'Sanguine Bond'): return True, [b], [in_hand(p, 'Sanguine Bond')]
    return False, [], []


@combo("Thassa's Oracle + Demonic Consultation / Tainted Pact",
       [("Thassa's Oracle",), ('Demonic Consultation', 'Tainted Pact')], mana=(0, ''),
       text='exile the library, then Oracle wins; both are spells (counterable)')
def _oracle(g, p):
    o = in_hand(p, "Thassa's Oracle"); t = any_hand(p, ('Demonic Consultation', 'Tainted Pact'))
    if o is None or t is None: return False, [], []
    return True, [], [t, o]


@combo('Helm of the Host + Combat Celebrant', [('Helm of the Host',), ('Combat Celebrant',)],
       text='a fresh Celebrant each combat: infinite combats')
def _helm(g, p):
    h = on_bf(p, 'Helm of the Host'); c = on_bf(p, 'Combat Celebrant')
    if h is None or c is None or h.attached is not c or g.active is not p or getattr(p, 'combat_no', 0) > 0: return False, [], []
    if not can_pay(g, p, 0, ''): return False, [], []
    return True, [c], []


@combo('Gravecrawler + Phyrexian Altar + a Zombie + a death payoff',
       [('Gravecrawler',), ('Phyrexian Altar',)], text='infinite death triggers')
def _gravecrawler(g, p):
    if on_bf(p, 'Phyrexian Altar') is None: return False, [], []
    gc = on_bf(p, 'Gravecrawler') or next((c for c in p.gy if c.name == 'Gravecrawler'), None)
    if gc is None: return False, [], []
    zombies = [m for m in p.perms if has_type(m, 'zombie') and not (m.cd is not None and m.cd.name == 'Gravecrawler')]
    if not zombies or drain_payoff(p) is None: return False, [], []
    return True, [on_bf(p, 'Phyrexian Altar'), drain_payoff(p)], []


@combo('Yawgmoth + undying loop + a death payoff',
       [('Yawgmoth, Thran Physician',), ('Mikaeus, the Unhallowed', "Geralf's Messenger", 'Butcher Ghoul', 'Young Wolf',
                                        'Nether Traitor')],
       text='sacrifice / -1/-1 counter loop: draws and drains')
def _yawg(g, p):
    y = on_bf(p, 'Yawgmoth, Thran Physician')
    if y is None or p.life < 8: return False, [], []
    und = [m for m in p.perms if m.creature and m.cd is not None and 'undying' in m.cd.kws and m is not y]
    mik = on_bf(p, 'Mikaeus, the Unhallowed')
    fodder = [m for m in p.perms if m.creature and m is not y and m is not mik and not has_type(m, 'human')]
    loop = len(und) >= 2 or (mik is not None and len(fodder) >= 1)
    if not loop or drain_payoff(p) is None: return False, [], []
    return True, [y] + ([mik] if mik else []), []


@combo('Krenko + Thornbite Staff + a sacrifice outlet', [('Krenko, Mob Boss',), ('Thornbite Staff',)],
       text='each Goblin sacrificed untaps Krenko: infinite Goblins')
def _staff(g, p):
    k = on_bf(p, 'Krenko, Mob Boss', untapped=True, unsick=True); st = on_bf(p, 'Thornbite Staff')
    if k is None or st is None or st.attached is not k or free_outlet(p) is None: return False, [], []
    return True, [k], []


def _lock_lattice(g, p):
    g.lattice_lock = p
    log(f'    Karn + Mycosynth Lattice: opponents can\'t use mana from permanents', g)


@combo('Karn, the Great Creator + Mycosynth Lattice', [('Karn, the Great Creator',), ('Mycosynth Lattice',)],
       finish=_lock_lattice, text='lock: every permanent is an artifact and opponents can\'t activate artifacts (their lands)')
def _karn(g, p):
    if getattr(g, 'lattice_lock', None) is p: return False, [], []
    k = on_bf(p, 'Karn, the Great Creator')
    lat = on_bf(p, 'Mycosynth Lattice')
    if k is None: return False, [], []
    if lat is not None: return True, [k, lat], []
    c = in_hand(p, 'Mycosynth Lattice')
    if c is not None: return True, [k], [c]
    return False, [], []


for cmb in COMBOS:
    for grp in cmb.groups:
        for n in grp:
            if n not in __import__('pool_cards').NOTES:
                note(n, 'Approximate', f'combo piece ({cmb.name}): the loop is executed abstractly')
