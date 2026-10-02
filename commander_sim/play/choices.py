"""The smaller choices, made by the person in the human seat: mulligans, discards, sacrifices, tutors, basic lands
searched for, and scry / surveil. The engine calls these (through `human_choice`) where it would otherwise let the
AI decide; they do nothing in a game without a human seat."""
from commander_sim import engine as E
from commander_sim.play import legal
from commander_sim.play.controller import controller_of, Request
from commander_sim.play.human import choose

BASICS = ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest', 'Wastes')


def human_seat(g, p):
    """is p played by a person in this game?"""
    ctl = getattr(g, 'controllers', None)
    return bool(ctl) and p.key in ctl and getattr(ctl[p.key], 'human', False)


def card_label(c):
    cost = ('{' + str(c.generic) + '}' if c.generic else '') + ''.join('{' + x + '}' for x in c.pips)
    return f'{c.name} {cost}'.strip()


def pick_cards(g, p, cards, n, prompt, kind='choose'):
    """the person picks n of cards (fewer if there are fewer); no cancelling. Returns the picked cards"""
    left, picked = list(cards), []
    for i in range(min(n, len(left))):
        k = choose(g, p, kind, f'{prompt} ({i + 1} of {min(n, len(cards))})', [card_label(c) for c in left], cancel=None)
        picked.append(left.pop(k))
    return picked


# ------------------------------------------------------------------ mulligan
def mulligan(g, p, rng):
    """London mulligan with a free first mulligan (multiplayer Commander): draw seven, keep or shuffle back and draw
    seven again; on keeping after k mulligans, put k - 1 cards on the bottom"""
    ctl = controller_of(g, p)
    k = 0
    while True:
        p.library.extend(p.hand); p.hand = []; rng.shuffle(p.library)
        for _ in range(7): p.hand.append(p.library.pop())
        bottom = max(0, k - 1)
        note = '' if bottom == 0 else f' If you keep, you put {bottom} card(s) on the bottom.'
        if k >= 6: break
        ans = ctl.ask(Request('mulligan', f'Your opening hand ({7 - bottom} to keep).{note}',
                              choices=['keep', 'mulligan'],
                              data={'hand': [card_label(c) for c in p.hand], 'names': [c.name for c in p.hand]}))
        if ans in (0, 'keep'): break
        k += 1
    bottom = max(0, k - 1)
    if bottom:
        for c in pick_cards(g, p, p.hand, bottom, 'Put a card on the bottom of your library'):
            p.hand.remove(c); p.library.insert(0, c)
    p.stats['mulls'] = k
    E.log(f'{E.NAME(p)} keeps {len(p.hand)} cards' + (f' after {k} mulligan(s)' if k else ''), g)


# ------------------------------------------------------------------ discards and sacrifices
def discard(g, p, n, why='Discard a card'):
    """p discards n cards of their choice"""
    cards = pick_cards(g, p, p.hand, n, why)
    if cards: E.discard_cards(g, p, cards)


def discard_to_hand_size(g, p, size=7):
    over = len(p.hand) - size
    if over > 0: discard(g, p, over, f'End of turn: discard down to {size}')


def sacrifice_creature(g, q, cands, why='Sacrifice a creature'):
    """q sacrifices one of cands (their creatures)"""
    if not cands: return
    k = choose(g, q, 'choose', why, [legal.describe_target(g, q, m) for m in cands], cancel=None)
    E.die(g, cands[k], 'sac')


# ------------------------------------------------------------------ searching the library
def search(g, p, pred, k=1, prompt='Search your library for a card', allow_none=True):
    """p searches their library for up to k cards matching pred; returns them (taken out of the library, not yet
    put anywhere). Then the library is shuffled"""
    pool = sorted({c.name: c for c in E.searchable(g, p) if pred(c)}.values(), key=lambda c: c.name)
    got = []
    for i in range(k):
        if not pool: break
        j = choose(g, p, 'search', f'{prompt}' + (f' ({i + 1} of {k})' if k > 1 else ''),
                   [card_label(c) for c in pool], cancel='find nothing' if allow_none else None)
        if j is None: break
        c = pool.pop(j); p.library.remove(c); got.append(c)
    g.rng.shuffle(p.library)
    return got


def pick_basic(g, p, prompt='Search your library for a basic land'):
    """one basic land card of p's choice (by name), taken out of the library, or None"""
    names = sorted({c.name for c in E.searchable(g, p) if c.land and c.name in BASICS})
    if not names: return None
    j = choose(g, p, 'search', prompt, names, cancel='find nothing')
    if j is None: return None
    c = next(c for c in E.searchable(g, p) if c.name == names[j])
    p.library.remove(c)
    return c


# ------------------------------------------------------------------ the top of the library
def scry(g, p, n, to='bottom'):
    """scry n (to='bottom') or surveil n (to='gy'): each card on top or away, in the order kept"""
    n = min(n, len(p.library))
    if n <= 0: return
    top = [p.library.pop() for _ in range(n)]              # top card first
    away_word = 'the bottom' if to == 'bottom' else 'your graveyard'
    keep, away = [], []
    for c in top:
        k = choose(g, p, 'choose', f"{'Scry' if to == 'bottom' else 'Surveil'}: {card_label(c)}",
                   ['keep on top', f'put on {away_word}'], cancel=None)
        (keep if k == 0 else away).append(c)
    if to == 'gy': p.gy.extend(away)
    else: p.library[:0] = away
    for c in reversed(keep): p.library.append(c)             # kept cards stay in the order they were
    E.log(f'  {E.NAME(p)} {"scries" if to == "bottom" else "surveils"} {n}: {len(keep)} on top', g)


# ------------------------------------------------------------------ modes and "you may"
EFFECT_WORDS = {'regrow': 'return a card from your graveyard to your hand', 'discard': 'a player discards',
                'destroy': 'destroy', 'exile': 'exile', 'damage': 'deal damage', 'draw': 'draw cards',
                'bounce': "return a permanent to its owner's hand", 'tap': 'tap', 'untap': 'untap',
                'counter_spell': 'counter a spell', 'token': 'create tokens', 'grant': 'give an ability',
                'sacrifice': 'a player sacrifices', 'gain_life': 'gain life', 'lose_life': 'lose life',
                'counters': 'put counters', 'pump': 'pump', 'mill': 'mill', 'search': 'search your library',
                'reanimate': 'return a creature to the battlefield'}


def describe_mode(ms):
    """'deal damage 2' / 'destroy (artifact)': a short label for one mode (a list of effects)"""
    out = []
    for e in ms:
        w = EFFECT_WORDS.get(e.get('do'), e.get('do', '?').replace('_', ' '))
        n = e.get('n')
        if isinstance(n, int) and n > 1 or (isinstance(n, int) and e.get('do') == 'damage'): w += f' {n}'
        what = (e.get('what') or e.get('to') or {}).get('filter', {}) or {}
        if what.get('type'): w += f" ({what['type'].replace('_', ' ')})"
        out.append(w)
    return ', then '.join(out)


def pick_modes(g, p, modes, k, name):
    """the person chooses k different modes"""
    left = list(range(len(modes))); got = []
    for i in range(k):
        j = choose(g, p, 'choose', f'{name}: choose a mode' + (f' ({i + 1} of {k})' if k > 1 else ''),
                   [describe_mode(modes[x]) for x in left], cancel=None)
        got.append(modes[left.pop(j)])
    return got


def yes_no(g, p, prompt):
    return choose(g, p, 'choose', prompt, ['yes', 'no'], cancel=None) == 0


# ------------------------------------------------------------------ "deals N damage to any target"
def damage_targets(g, p, colours=''):
    """everything a damage ability may hit: creatures and planeswalkers (hexproof, shroud, protection respected) and
    players (not under protection from everything)"""
    out = []
    for q in g.players:
        if not q.alive: continue
        for m in q.perms:
            if m.phased or not (m.creature or (m.cd is not None and 'P' in m.cd.types)): continue
            if (m.owner is not p and E.untargetable(g, m)) or (colours and E.protected_from(g, m, colours)): continue
            out.append(m)
    out += [q for q in g.players if legal.player_targetable(q)]
    return out


def deal_damage(g, p, n, source, kind='triggers'):
    """p's `source` deals n damage to a target p picks: a creature dies if n reaches its toughness, a planeswalker loses
    that much loyalty, a player loses that much life"""
    if n <= 0: return
    tg = damage_targets(g, p)
    if not tg: return
    k = choose(g, p, 'target', f'{source}: deal {n} damage to which target?', [legal.describe_target(g, p, x) for x in tg],
               cancel=None)
    x = tg[k]
    if isinstance(x, E.Player):
        E.lose_life(g, x, n, p, kind=kind); return
    if x.creature and E.etgh(g, x) <= n:
        E.apply_removal(g, p, x, f'dmg{n}')
    elif x.creature:
        E.log(f'    {source} deals {n} damage to {x.name} (it survives)', g)
    if x in x.owner.perms and x.cd is not None and 'P' in x.cd.types and x.loyalty is not None:
        x.loyalty -= n
        E.log(f'    {source} deals {n} damage to {x.name} (loyalty {max(0, x.loyalty)})', g)
        if x.loyalty <= 0: E.leave(g, x); E.to_zone_card(g, x, 'gy')
    E.check_state(g)


# ------------------------------------------------------------------ the Ring
def ring_bearer(g, p, cands):
    k = choose(g, p, 'choose', 'The Ring tempts you: choose your Ring-bearer', [legal.describe_target(g, p, m) for m in cands],
               cancel=None)
    return cands[k]


# ------------------------------------------------------------------ creatures that remove something when they enter
def etb_removal(g, p, m):
    """Noxious Gearhulk and similar: "when this enters, you may destroy another target creature" (pick, or none)"""
    tg = [x for x in (legal.spell_targets(g, p, m.cd) or []) if x is not m and isinstance(x, E.Perm)]
    if not tg: return
    k = choose(g, p, 'target', f'{m.name} enters: choose a target', [legal.describe_target(g, p, x) for x in tg],
               cancel='no target')
    if k is None: return
    E.apply_removal(g, p, tg[k], m.cd.tags['rem'], m.cd)


# ------------------------------------------------------------------ taxes: "unless that player pays {N}"
def pay_tax(g, p, n, what):
    """an opponent's tax (Rhystic Study, Smothering Tithe, Mystic Remora): ask p whether to pay {n}. Paid from p's
    floating mana first, then by tapping p's sources (as you would at the table). Returns True if paid"""
    from commander_sim.play import mana
    pool = mana.pool_of(p)
    can = pool.total() + sum(s['amount'] for s in mana.sources(g, p))
    if can < n:
        E.log(f"    {E.NAME(p)} can't pay {{{n}}} for {what}", g)
        return False
    if not yes_no(g, p, f'{what}: pay {{{n}}}?'): return False
    while pool.total() < n and mana.sources(g, p):
        mana.tap(g, p, mana.sources(g, p)[0]['id'])
    if not pool.pay(n, ''): return False
    E.log(f'    {E.NAME(p)} pays {{{n}}} for {what}', g)
    return True


# ------------------------------------------------------------------ reanimation and filling the graveyard
ANY_GRAVEYARD = ('animate', 'necro', 'reanimate')      # Animate Dead, Necromancy, Reanimate: any graveyard


def rean_candidates(g, p, kind):
    """[(creature card, graveyard owner)] a reanimation spell of this kind may return"""
    owners = [q for q in g.players if q.alive] if kind in ANY_GRAVEYARD else [p]
    return [(c, q) for q in owners for c in q.gy if c.creature]


def choose_rean(g, p, c):
    """the target of reanimation spell c: (card, owner), or None if cancelled / nothing to return"""
    cands = rean_candidates(g, p, c.tags.get('rean', 'animate'))
    if not cands: return None
    labels = [f"{card_label(x)} ({'your graveyard' if q is p else E.NAME(q) + chr(39) + 's graveyard'})" for x, q in cands]
    k = choose(g, p, 'target', f'{c.name}: return which creature card?', labels)
    return None if k is None else cands[k]


def fill(g, p, kind):
    """Entomb, Buried Alive, Unmarked Grave, Grisly Salvage: the person picks what goes where"""
    if kind == 'entomb':
        for c in search(g, p, lambda c: True, 1, 'Entomb: put a card from your library into your graveyard'): p.gy.append(c)
    elif kind == 'buried':
        for c in search(g, p, lambda c: c.creature, 3, 'Buried Alive: up to three creature cards into your graveyard'):
            p.gy.append(c)
    elif kind == 'unmarked':
        for c in search(g, p, lambda c: 'leg' not in c.tags, 1, 'Unmarked Grave: a nonlegendary card into your graveyard'):
            p.gy.append(c)
    elif kind == 'grisly':
        top = [p.library.pop() for _ in range(min(5, len(p.library)))]
        ok = [c for c in top if c.creature or c.land]
        if ok:
            k = choose(g, p, 'choose', 'Grisly Salvage: put a creature or land card into your hand?', [card_label(c) for c in ok],
                       cancel='none')
            if k is not None: top.remove(ok[k]); p.hand.append(ok[k])
        p.gy.extend(top)
        E.log(f'  {E.NAME(p)} reveals five with Grisly Salvage', g)
    elif kind == 'dispute':
        E.add_treasure(g, p, 1)


def sac_artifact_or_creature(g, p, what):
    """an additional cost: sacrifice an artifact or creature (a Treasure counts). False if p has none"""
    cands = [m for m in p.perms if (m.creature or (m.cd is not None and 'A' in m.cd.types)) and not m.phased]
    labels = [legal.describe_target(g, p, m) for m in cands] + (['a Treasure token'] if p.treasures else [])
    if not labels: return False
    k = choose(g, p, 'choose', f'{what}: sacrifice an artifact or creature', labels, cancel=None)
    if k == len(cands): p.treasures -= 1; E.log(f'  {E.NAME(p)} sacrifices a Treasure', g)
    else: E.die(g, cands[k], 'sac')
    return True


def target_opponent(g, p, source):
    """p picks a target opponent (Archon of Cruelty)"""
    opps = [q for q in g.opps(p) if legal.player_targetable(q)] or g.opps(p)
    k = choose(g, p, 'target', f'{source}: target opponent?', [legal.describe_target(g, p, q) for q in opps], cancel=None)
    return opps[k]


# ------------------------------------------------------------------ copies of spells
def copy_targets(g, p, c, ctx):
    """a copy of spell c (Thousand-Year Storm, Jin-Gitaxias, Ral, Return the Favor, Mizzix's Mastery): you may choose
    new targets for it. Returns the copy's ctx"""
    ctx = dict(ctx or {})
    old = ctx.get('target') if ctx.get('target') is not None else ctx.get('face')
    tg = legal.spell_targets(g, p, c)
    if not tg: return ctx
    keep = old is not None and old in tg
    k = choose(g, p, 'target', f'The copy of {c.name}: choose its target', [legal.describe_target(g, p, x) for x in tg],
               cancel=f'keep the same target ({legal.describe_target(g, p, old)})' if keep else None)
    if k is None: return ctx
    ctx.pop('target', None); ctx.pop('face', None)
    ctx['face' if isinstance(tg[k], E.Player) else 'target'] = tg[k]
    return ctx


# ------------------------------------------------------------------ Atraxa, Grand Unifier (Sephiroth, the Savior)
TYPE_NAMES = (('A', 'artifact'), ('B', 'battle'), ('C', 'creature'), ('E', 'enchantment'), ('I', 'instant'),
              ('L', 'land'), ('P', 'planeswalker'), ('S', 'sorcery'))


def atraxa_pick(g, p, top):
    """Atraxa's enter trigger: the top ten revealed; for each card type you may put a card of that type into your hand
    (a card with two types fills one of them). Returns the cards taken"""
    E.log(f"    {E.NAME(p)} reveals the top {len(top)}: {', '.join(c.name for c in top)}", g)
    taken = []
    for t, word in TYPE_NAMES:
        cands = [c for c in top if t in c.types and c not in taken]
        if not cands: continue
        k = choose(g, p, 'choose', f'Atraxa: put {"an" if word[0] in "aei" else "a"} {word} card into your hand? '
                   f'(revealed: {", ".join(c.name for c in top)})', [card_label(c) for c in cands], cancel=f'no {word}')
        if k is not None: taken.append(cands[k])
    return taken
