"""Corrections for cards the ability compiler reads wrongly (found by reviewing every Full-auto card against its
rules text). Each entry replaces the compiled abilities with a hand-written implementation.
"""
from engine import *
import engine as E
import cardimpl as CI
from cardimpl import on
from pool_cards import card, note
import impl_common as IC
from impl_common import best_opp_nonland


def destroy_land(g, q, L):
    if L not in q.lands: return
    q.lands.remove(L); q.gy.append(L.cd)
    log(f'    {L.cd.name} ({NAME(q)}) is destroyed', g)
    if g.hooks: CI.fire(g, 'land_gy', q, L.cd)


def best_opp_land(g, p):
    """the most valuable opposing land: nonbasics that make several colors or big mana, of the leading player"""
    lands = [(q, L) for q in g.opps(p) for L in q.lands if L.cd.name not in ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest')]
    if not lands: return None
    return max(lands, key=lambda x: (int(x[1].cd.tags.get('amt', 1)), len(x[1].cd.tags.get('c', '')), threat(g, p, x[0])))


# ------------------------------------------------------------------ Acidic Slime
@on('Acidic Slime', 'etb')
def _slime(g, src, p, m):
    if m is not src: return
    o = src.owner
    t = best_opp_nonland(g, o, lambda x: x.cd is not None and not x.creature and ('A' in x.cd.types or 'E' in x.cd.types))
    if t is not None and pval(g, t) >= 2.5: apply_removal(g, o, t, 'destroy'); return
    x = best_opp_land(g, o)
    if x is not None: destroy_land(g, *x)
    elif t is not None: apply_removal(g, o, t, 'destroy')
card('Acidic Slime', 'pow=2 dt', dsl=[])
note('Acidic Slime', 'Full', 'deathtouch; destroys the best opposing artifact or enchantment, else a nonbasic land')


# ------------------------------------------------------------------ Aetherize
@on('Aetherize', 'hand_defend')
def _aetherize(g, c, d, p, atk, assign):
    """d is attacked by p: return all attacking creatures when the attack is big (at least a third of d's life, or
    it holds tokens that would simply vanish)"""
    if c not in d.hand or not castable(g, d, c) or not can_pay(g, d, *cost_of(d, c)): return
    power = sum(epow(g, a) for a in atk if a in p.perms and a not in assign)
    toks = sum(1 for a in atk if a.token)
    if power < max(8, d.life / 3) and toks < 4: return
    pay(g, d, *cost_of(d, c)); d.hand.remove(c)
    log(f'  {NAME(d)} casts Aetherize', g)
    on_cast(g, d, c)
    if not counter_window(g, d, c, 5, {}): d.gy.append(c); return
    for a in list(atk):
        if a in a.owner.perms:
            if a.token: leave(g, a)
            else: bounce(g, a)
        atk.remove(a)
    d.gy.append(c)
card('Aetherize', '', types='I', dsl=[])
note('Aetherize', 'Full', 'cast when attacked by a big or token-heavy attack: returns all attacking creatures')


# ------------------------------------------------------------------ Bastion of Remembrance (life gain is 1, not per opponent)
card('Bastion of Remembrance', '', types='E', dsl=[
    {'type': 'triggered', 'effects': [{'do': 'token', 'n': 1, 'pow': 1, 'tgh': 1, 'keywords': [], 'attacking': False,
                                       'warrior': False, 'types': ['human', 'soldier']}], 'event': 'etb', 'source': 'self'},
    {'type': 'triggered', 'effects': [{'do': 'lose_life', 'n': 1, 'who': 'each_opponent'},
                                      {'do': 'gain_life', 'n': 1, 'who': 'you', 'per_opponent': False}],
     'event': 'dies', 'source': 'you_creature', 'other': False}],
    status=('Full', 'Human Soldier on entry; each creature you control dying drains each opponent 1, you gain 1'))


# ------------------------------------------------------------------ Casualties of War: every mode with a target
@IC.spell('Casualties of War', prio=lambda g, p, c: 58 if sum(1 for q in g.opps(p) for m in q.perms if pval(g, m) >= 2) >= 2 else 0,
          types='S', status=('Full', 'destroys the best opposing artifact, creature, enchantment, planeswalker and '
                                     'nonbasic land (each mode that has a target)'))
def _casualties(g, p, c, ctx):
    for kind in ('A', 'C', 'E', 'P'):
        pred = (lambda m: m.creature) if kind == 'C' else (lambda m, k=kind: m.cd is not None and k in m.cd.types)
        t = best_opp_nonland(g, p, pred)
        if t is not None and t in t.owner.perms: apply_removal(g, p, t, 'destroy')
    x = best_opp_land(g, p)
    if x is not None: destroy_land(g, *x)


# ------------------------------------------------------------------ mana rocks that enter tapped
def _tapped_rock(name, tags, status_text):
    @on(name, 'etb')
    def _t(g, src, p, m):
        if m is src: src.tapped = True
    card(name, tags, types='A', dsl=[])
    note(name, 'Full', status_text)


_tapped_rock('Charcoal Diamond', 'rock=1:B', 'enters tapped; taps for {B}')
_tapped_rock('Fire Diamond', 'rock=1:R', 'enters tapped; taps for {R}')
_tapped_rock('Coldsteel Heart', 'rock=1:A', 'enters tapped; taps for the chosen color (a color the deck needs)')
_tapped_rock('Worn Powerstone', 'rock=2:C', 'enters tapped; taps for {C}{C}')


# ------------------------------------------------------------------ Chord of Calling: X limits the creature
import impl_t3
impl_t3._x_tutor('Chord of Calling', 'GGG', status=('Full', 'convoke; X = spare mana and creatures; a creature with '
                                                            'mana value X or less onto the battlefield'))
card('Chord of Calling', 'xtutor convoke', types='I', dsl=[])


# ------------------------------------------------------------------ Crackling Doom: the greatest power
@IC.spell('Crackling Doom', prio=lambda g, p, c: 55 if any(m.creature for q in g.opps(p) for m in q.perms) else 0, types='I',
          status=('Full', '2 damage to each opponent; each opponent sacrifices its greatest-power creature'))
def _doom(g, p, c, ctx):
    for q in g.opps(p):
        lose_life(g, q, 2, p, kind='burn')
    for q in g.opps(p):
        cs = [m for m in q.perms if m.creature and not m.phased]
        if cs:
            m = max(cs, key=lambda m: (epow(g, m), -pval(g, m)))
            log(f'    {NAME(q)} sacrifices {m.name}', g); die(g, m, 'sac')


# ------------------------------------------------------------------ Deathrite Shaman
def _drs_mana(g, p, m):
    return 1 if any(x.land for q in g.players for x in q.gy) else 0


def _drs_tap(g, p, m, used):
    for q in sorted(g.players, key=lambda q: q is p):          # an opponent's land first
        x = next((x for x in q.gy if x.land), None)
        if x is not None: q.gy.remove(x); q.exile.append(x); return


CI.DYN_MANA['Deathrite Shaman'] = _drs_mana
CI.ON_TAP['Deathrite Shaman'] = _drs_tap


@on('Deathrite Shaman', 'options')
def _drs(g, src, p, s, post):
    """{B},{T}: exile an instant or sorcery card from a graveyard, each opponent loses 2;
    {G},{T}: exile a creature card from a graveyard, you gain 2 (prefers an opponent's reanimation target)"""
    if p is not src.owner or src.tapped or src.sick or post is None: return []
    o = []
    spell = [(q, x) for q in g.players for x in q.gy if x.instant or x.sorcery]
    cre = [(q, x) for q in g.players for x in q.gy if x.creature]
    if spell and can_pay(g, p, 0, 'B'):
        q, x = max(spell, key=lambda t: (t[0] is not p, card_worth(g, t[0], t[1], True)))

        def drain(q=q, x=x):
            if src.tapped or x not in q.gy or not can_pay(g, p, 0, 'B'): return False
            pay(g, p, 0, 'B'); src.tapped = True; q.gy.remove(x); q.exile.append(x)
            for o_ in g.opps(p): lose_life(g, o_, 2, p, kind='triggers')
            log(f'  {NAME(p)} uses Deathrite Shaman: exiles {x.name}, each opponent loses 2', g); return True
        o.append((2.2, 'Deathrite Shaman drain', drain))
    if cre and can_pay(g, p, 0, 'G'):
        q, x = max(cre, key=lambda t: (t[0] is not p, t[1].bomb, t[1].cmc))
        if q is not p:
            def hate(q=q, x=x):
                if src.tapped or x not in q.gy or not can_pay(g, p, 0, 'G'): return False
                pay(g, p, 0, 'G'); src.tapped = True; q.gy.remove(x); q.exile.append(x); gain(p, 2)
                log(f'  {NAME(p)} uses Deathrite Shaman: exiles {x.name}', g); return True
            o.append((1.2 + 0.4 * x.bomb, 'Deathrite Shaman exile', hate))
    return o
card('Deathrite Shaman', 'shaman pow=1 tgh=2 dork=A', dsl=[])
note('Deathrite Shaman', 'Full', 'mana while a land card is in a graveyard (exiling it); drains 2 by exiling an '
     'instant/sorcery; exiles opposing creature cards for 2 life')


# ------------------------------------------------------------------ Decimate
@on('Decimate', 'can_cast')
def _decimate_ok(g, src, caster, c, zone):
    return True


def _decimate_targets(g, p):
    a = best_opp_nonland(g, p, lambda m: m.cd is not None and 'A' in m.cd.types)
    cr = best_opp_nonland(g, p, lambda m: m.creature)
    e = best_opp_nonland(g, p, lambda m: m.cd is not None and 'E' in m.cd.types)
    lands = [(q, L) for q in g.players if q.alive for L in q.lands]
    return a, cr, e, lands


def _decimate_prio(g, p, c):
    a, cr, e, lands = _decimate_targets(g, p)
    if a is None or cr is None or e is None or not lands: return 0
    return 60 if pval(g, a) + pval(g, cr) + pval(g, e) >= 9 else 0


@IC.spell('Decimate', prio=_decimate_prio, types='S',
          status=('Full', 'needs an artifact, a creature, an enchantment and a land to target; destroys the best of each'))
def _decimate(g, p, c, ctx):
    a, cr, e, lands = _decimate_targets(g, p)
    for t in (a, cr, e):
        if t is not None and t in t.owner.perms: apply_removal(g, p, t, 'destroy')
    x = best_opp_land(g, p)
    if x is not None: destroy_land(g, *x)
    elif lands: destroy_land(g, *min(lands, key=lambda t: (t[0] is not p, 0)))


# ------------------------------------------------------------------ Elvish Clancaller
@on('Elvish Clancaller', 'options')
def _clancaller(g, src, p, s, post):
    if p is not src.owner or src.tapped or src.sick or post is None or not can_pay(g, p, 4, 'GG'): return []
    c = next((x for x in p.library if x.name == 'Elvish Clancaller'), None)
    if c is None: return []

    def go():
        if src.tapped or c not in p.library or not can_pay(g, p, 4, 'GG'): return False
        pay(g, p, 4, 'GG'); src.tapped = True; p.library.remove(c); g.rng.shuffle(p.library); enter(g, p, c)
        log(f'  {NAME(p)} fetches another Elvish Clancaller', g); return True
    return [(2.0, 'Elvish Clancaller search', go)]


card('Elvish Clancaller', 'pow=1', dsl=[{'type': 'static', 'static': 'anthem', 'filter': {
    'type': 'creature', 'controller': 'you', 'other': True, 'subtype': 'elf'}, 'pow': 1, 'tgh': 1}])
note('Elvish Clancaller', 'Full', 'other Elves you control +1/+1; {4}{G}{G},{T}: a card named Elvish Clancaller onto the battlefield')


# ------------------------------------------------------------------ Gamble: tutor, then discard at random
@IC.spell('Gamble', prio=58, types='S', status=('Full', 'tutor any card to hand, then discard a card at random'))
def _gamble(g, p, c, ctx):
    tutor(g, p, 'any')
    rest = [x for x in p.hand if x is not c]
    if rest:
        x = g.rng.choice(rest); p.hand.remove(x); p.gy.append(x)
        log(f'    Gamble discards {x.name}', g)
        if g.hooks: CI.fire(g, 'discard', p, x)


# ------------------------------------------------------------------ Goblin Trashmaster: sacrifice a Goblin
@on('Goblin Trashmaster', 'options')
def _trashmaster(g, src, p, s, post):
    if p is not src.owner or post is None: return []
    t = best_opp_nonland(g, p, lambda m: m.cd is not None and 'A' in m.cd.types)
    fod = [m for m in p.perms if m.creature and m is not src and E.has_type(m, 'goblin') and (m.token or pval(g, m) < 2)]
    if t is None or not fod or pval(g, t) < 3: return []
    f = min(fod, key=lambda m: pval(g, m))

    def go():
        if f not in p.perms or t not in t.owner.perms: return False
        die(g, f, 'sac'); apply_removal(g, p, t, 'destroy'); return True
    return [(pval(g, t) - 1.5, 'Goblin Trashmaster', go)]


card('Goblin Trashmaster', 'warrior pow=3', dsl=[{'type': 'static', 'static': 'anthem', 'filter': {
    'type': 'creature', 'controller': 'you', 'other': True, 'subtype': 'goblin'}, 'pow': 1, 'tgh': 1}])
note('Goblin Trashmaster', 'Full', 'other Goblins +1/+1; sacrifices a spare Goblin to destroy a valuable artifact')


# ------------------------------------------------------------------ Mystic Snake: a counterspell with a body
card('Mystic Snake', 'pow=2 flash ctr=any', dsl=[])
note('Mystic Snake', 'Full', 'flash; used as a counterspell (the Snake enters the battlefield)')


# ------------------------------------------------------------------ Reshape: X limits the artifact
@IC.spell('Reshape', prio=lambda g, p, c: 55 if total_mana(g, p) >= 4 and any(
        m.cd is not None and 'A' in m.cd.types and (m.token or pval(g, m) < 3) for m in p.perms) else 0, types='S',
          status=('Full', 'sacrifices the least valuable artifact; X = spare mana; the best artifact with MV X or less '
                          'onto the battlefield'))
def _reshape(g, p, c, ctx):
    arts = [m for m in p.perms if m.cd is not None and 'A' in m.cd.types and not m.phased]
    if not arts: return
    die(g, min(arts, key=lambda m: pval(g, m)), 'sac')
    x = ctx.get('x', 0)
    cs = [y for y in searchable(g, p) if 'A' in y.types and y.cmc <= x]
    if cs:
        import impl_t5
        y = max(cs, key=lambda y: (y.name in impl_t5.IC_COMBO_ART, card_worth(g, p, y), y.cmc))
        p.library.remove(y); g.rng.shuffle(p.library); enter(g, p, y)
card('Reshape', 'xtutor', types='S', dsl=[])


# ------------------------------------------------------------------ Retrofitter Foundry
@on('Retrofitter Foundry', 'options')
def _foundry(g, src, p, s, post):
    """{2},{T}: Servo; {1},{T}, sac a Servo: Thopter; {T}, sac a Thopter: 4/4 Construct ({3}: untap)"""
    if p is not src.owner or post is None: return []
    if src.tapped:
        if not can_pay(g, p, 3, '') or total_mana(g, p) < 5: return []

        def untap():
            if not src.tapped or not can_pay(g, p, 3, ''): return False
            pay(g, p, 3, ''); src.tapped = False; return True
        return [(0.8, 'Retrofitter Foundry untap', untap)]
    thop = [m for m in p.perms if m.token and E.has_type(m, 'thopter')]
    servo = [m for m in p.perms if m.token and E.has_type(m, 'servo')]

    def make(kind, cost, fod):
        def go():
            if src.tapped or not can_pay(g, p, cost, ''): return False
            if fod is not None and fod not in p.perms: return False
            pay(g, p, cost, ''); src.tapped = True
            if fod is not None: leave(g, fod)
            if kind == 'construct': make_tokens(g, p, 1, 4, types=('construct', 'artifact'))
            elif kind == 'thopter': make_tokens(g, p, 1, 1, fly=True, types=('thopter', 'artifact'))
            else: make_tokens(g, p, 1, 1, types=('servo', 'artifact'))
            return True
        return go
    if thop: return [(3.0, 'Retrofitter Foundry: Construct', make('construct', 0, thop[0]))]
    if servo and can_pay(g, p, 1, ''): return [(2.0, 'Retrofitter Foundry: Thopter', make('thopter', 1, servo[0]))]
    if can_pay(g, p, 2, ''): return [(1.5, 'Retrofitter Foundry: Servo', make('servo', 2, None))]
    return []


card('Retrofitter Foundry', '', types='A', dsl=[])
note('Retrofitter Foundry', 'Full', 'Servo -> Thopter -> 4/4 Construct chain; untaps for {3} with spare mana')


# ------------------------------------------------------------------ Rishkar, Peema Renegade
@on('Rishkar, Peema Renegade', 'etb')
def _rishkar(g, src, p, m):
    if m is not src: return
    o = src.owner
    cs = sorted([x for x in o.perms if x.creature and not x.phased], key=lambda x: (x is src, -pval(g, x)), reverse=True)
    for x in cs[:2]: x.plus += 1


@on('Rishkar, Peema Renegade', 'extra_mana')
def _rishkar_mana(g, src, p, U):
    """each creature you control with a counter on it taps for {G}"""
    have = {id(u[0]) for u in U}
    return [[m, 'G', 1] for m in p.perms if m.creature and m.plus > 0 and not m.tapped and not m.sick and not m.phased
            and id(m) not in have and not (m.cd is not None and 'dork' in m.cd.tags)]


card('Rishkar, Peema Renegade', 'leg pow=2', dsl=[])
note('Rishkar, Peema Renegade', 'Full', '+1/+1 counters on two of your creatures; creatures with counters tap for {G}')


# ------------------------------------------------------------------ Elvish Champion: forestwalk
card('Elvish Champion', 'pow=2', dsl=[{'type': 'static', 'static': 'anthem', 'filter': {
    'type': 'creature', 'other': True, 'subtype': 'elf'}, 'pow': 1, 'tgh': 1}])
note('Elvish Champion', 'Full', 'other Elves +1/+1 and forestwalk')


@on('Elvish Champion', 'grant_kw')
def _forestwalk(g, src, m, kw):
    return kw == 'forestwalk' and m is not src and E.has_type(m, 'elf')
