"""Rules the pool cards were approximating, made exact (pool games only): mana sources (Signets, Talismans, pain,
Moxen, Lotus Petal, conditional dorks), tax payments (Smothering Tithe, Rhystic Study, Mystic Remora), counter
interactions (Mana Drain, Flusterstorm, Veil of Summer, Siren Stormtamer), removal restrictions and taxes,
"becomes a 3/3" effects, Council's Judgment's vote, Fact or Fiction's split, ability locks (Collector Ouphe, Cursed
Totem, Karn, Grand Abolisher), planeswalker ultimates and emblems, and the remaining single-card clauses.
"""
from engine import *
import engine as E
import cardimpl as CI
from cardimpl import on, _eot, count_type
from pool_cards import card, note
import impl_common as IC
from impl_common import best_opp_creature, best_opp_nonland, make_artifact_tokens, WALKERS

MAIN = CI.main_cards()


def set_tags(name, add=(), remove=()):
    """change a card's tags for pool games; main-deck cards get only tags old mode never reads"""
    cd = E.DB.get(name)
    if cd is None: return
    for k in remove:
        if name not in MAIN: cd.tags.pop(k, None)
    for k in add:
        k, _, v = k.partition('=')
        if name in MAIN and k not in ('pain', 'nonblack', 'labyrinth'): continue
        cd.tags[k] = v if v else True


def full(name, text):
    note(name, 'Full', text)


def turn_now(g):
    return turn_stamp(g)


def spare_after(g, p, n):
    """can p pay n now and still cast the most expensive spell it holds"""
    if not can_pay(g, p, n, ''): return False
    need = max((cost_of(p, c)[0] + len(cost_of(p, c)[1]) for c in p.hand if not c.land), default=0)
    return total_mana(g, p) - n >= need


# ================================================================== mana sources
def fix_mana():
    for n, col in (('Azorius Signet', 'WU'), ('Boros Signet', 'WR'), ('Dimir Signet', 'UB'), ('Golgari Signet', 'BG'),
                   ('Rakdos Signet', 'BR'), ('Selesnya Signet', 'WG'), ('Simic Signet', 'UG'), ('Izzet Signet', 'UR'),
                   ('Gruul Signet', 'RG'), ('Orzhov Signet', 'WB')):
        if n in E.DB:
            if n not in MAIN: E.DB[n].tags['rock'] = f'1:{col}'
            full(n, '{1},{T}: two mana of its colours (net one coloured mana)')
    for n, col in (('Talisman of Conviction', 'RW'), ('Talisman of Curiosity', 'GU'), ('Talisman of Dominance', 'UB'),
                   ('Talisman of Hierarchy', 'WB'), ('Talisman of Impulse', 'RG'), ('Talisman of Indulgence', 'BR'),
                   ('Talisman of Progress', 'WU'), ('Talisman of Resilience', 'BG')):
        if n in E.DB:
            if n not in MAIN: E.DB[n].tags['rock'] = f'1:{col}'
            E.DB[n].tags['pain'] = True
            full(n, '{T}: {C}, or one of its colours for 1 damage')
    for n in ('Adarkar Wastes', 'Battlefield Forge', 'Brushland', 'Caves of Koilos', 'Llanowar Wastes', 'Sulfurous Springs',
              'Underground River', 'Yavimaya Coast', 'Shivan Reef', 'Karplusan Forest', 'Sulfur Falls'):
        if n in E.DB:
            E.DB[n].tags['pain'] = True
            full(n, '{T}: {C}, or a coloured mana for 1 damage')
    set_tags('Elves of Deep Shadow', add=('pain',)); full('Elves of Deep Shadow', '{T}: {B}, 1 damage to you')
    for n in ('Boreal Druid', 'Elvish Mystic', 'Fyndhorn Elves', 'Sylvan Caryatid'):
        full(n, 'mana creature')
    full('Arcane Signet', 'one mana of any colour in your commander\'s identity')
    full('Fellwar Stone', 'one mana of a colour an opponent\'s land could produce')


fix_mana()


def _moxamber(g, p, m):
    return 1 if any(x.cd is not None and 'leg' in x.cd.tags and (x.creature or 'P' in x.cd.types) and not x.phased
                    for x in p.perms) else 0


def _moxopal(g, p, m):
    return 1 if sum(1 for x in p.perms if (x.cd is not None and 'A' in x.cd.types) or (x.token and 'artifact' in x.ttypes)) >= 3 else 0


def _drum(g, p, m):
    return 1 if any(x.creature and not x.tapped and not x.phased and x is not m for x in p.perms) else 0


def _drum_tap(g, p, m, used):
    cs = [x for x in p.perms if x.creature and not x.tapped and not x.phased]
    if cs: min(cs, key=lambda x: (not x.sick, pval(g, x))).tapped = True


def _petal_sac(g, p, m, used):
    if m in p.perms: leave(g, m); to_zone_card(g, m, 'gy')


CI.DYN_MANA['Mox Amber'] = _moxamber
CI.DYN_MANA['Mox Opal'] = _moxopal
CI.DYN_MANA['Springleaf Drum'] = _drum
CI.ON_TAP['Springleaf Drum'] = _drum_tap
CI.ON_TAP['Lotus Petal'] = _petal_sac
CI.DYN_MANA['Jaspera Sentinel'] = _drum
CI.ON_TAP['Jaspera Sentinel'] = _drum_tap
CI.DYN_MANA['Sanctum Weaver'] = lambda g, p, m: max(0, sum(1 for x in p.perms if x.cd is not None and 'E' in x.cd.types and not x.phased))
set_tags('Sanctum Weaver', add=('dork=A',))
full('Mox Amber', 'mana only while you control a legendary creature or planeswalker')
full('Mox Opal', 'metalcraft: mana only with three or more artifacts')
full('Springleaf Drum', '{T}, tap an untapped creature you control: any colour')
full('Lotus Petal', '{T}, sacrifice: one mana of any colour')
full('Jaspera Sentinel', 'reach; {T}, tap another untapped creature: any colour')
full('Sanctum Weaver', '{T}: X mana of one colour, X = enchantments you control')
set_tags('Delighted Halfling', add=('dork=C',))
full('Delighted Halfling', '{C}, or any colour for a legendary spell')


def halfling_colors(p):
    """Delighted Halfling's coloured mana only pays for legendary spells"""
    c = E.PAY_FOR
    return p.ident if c is not None and 'leg' in c.tags else ''


# Goldspan Dragon: a flying haste attacker; Treasures on attack / when targeted; Treasures make two mana
card('Goldspan Dragon', 'pow=4 tgh=4 fly haste bomb=6', dsl=[])


@on('Goldspan Dragon', 'attack')
def _goldspan_atk(g, src, p, atk, d):
    if p is src.owner and src in atk: add_treasure(g, p, 1)


@on('Goldspan Dragon', 'treasure_bonus')
def _goldspan_bonus(g, src, p):
    return 1 if p is src.owner else 0
full('Goldspan Dragon', 'flying, haste; a Treasure when it attacks; your Treasures tap for two mana')

# Ashnod's Altar makes colourless mana
note("Ashnod's Altar", 'Full', 'sacrifice a creature: {C}{C} (spent this turn)')
note('Phyrexian Altar', 'Full', 'sacrifice a creature: one mana of any colour')


# ================================================================== tax payments
def tithe_unpaid(g, p):
    """Smothering Tithe: p pays {2} when it can spare the mana, otherwise the Tithe's owner gets a Treasure"""
    if spare_after(g, p, 2):
        pay(g, p, 2, ''); return False
    return True
full('Smothering Tithe', 'opponents pay {2} per draw only when they can spare it; otherwise you get a Treasure')


def rhystic_unpaid(g, p):
    if spare_after(g, p, 1):
        pay(g, p, 1, ''); return False
    return True


note('Rhystic Study', 'Full', 'opponents pay {1} per spell when they can spare it; otherwise you draw')


@on('Mystic Remora', 'cast')
def _remora(g, src, caster, c):
    o = src.owner
    if caster is o or c.creature or c.land: return
    if spare_after(g, caster, 4): pay(g, caster, 4, ''); return
    draw(g, o, 1)


@on('Mystic Remora', 'upkeep')
def _remora_age(g, src, p):
    if p is not src.owner: return
    if src.data is None: src.data = {}
    age = src.data.get('age', 0) + 1; src.data['age'] = age
    if age <= 3 and can_pay(g, p, age, ''): pay(g, p, age, '')
    else:
        log(f'    {NAME(p)} sacrifices Mystic Remora (cumulative upkeep {age})', g); leave(g, src); to_zone_card(g, src, 'gy')
set_tags('Mystic Remora', remove=('eng', 'remora'))
full('Mystic Remora', 'cumulative upkeep {1} (kept three turns); draws when an opponent casts a noncreature spell '
     'unless they can spare {4}')

REPLACED_TAG_ENGINES = {'Mystic Remora', 'Sylvan Library'}     # hand-implemented: skip the generic 'eng' upkeep draw


# ================================================================== counters
def veil_response(g, p, q, ctr):
    """q is about to counter p's spell with a blue or black counterspell: p answers with Veil of Summer"""
    if p.key in E.CTHRESH or not ('U' in ctr.pips or 'B' in ctr.pips or ctr.name in ('Force of Will', 'Force of Negation')): return False
    v = next((c for c in p.hand if c.name == 'Veil of Summer'), None)
    if v is None or not can_pay(g, p, 0, 'G') or not castable(g, p, v): return False
    pay(g, p, 0, 'G'); p.hand.remove(v); p.gy.append(v); on_cast(g, p, v)
    log(f'    {NAME(p)} responds with Veil of Summer: the spell can\'t be countered', g)
    draw(g, p, 1)
    return True
card('Veil of Summer', '', types='I', dsl=[])
full('Veil of Summer', 'in response to a blue or black counterspell: your spells can\'t be countered this turn, draw a card')
full('Mana Drain', 'counters a spell; its mana value in mana at your next main phase')
full('Flusterstorm', 'counter unless its controller pays {1} per copy (storm: a copy per spell cast before it this turn)')
full('Arcane Denial', 'counter; the caster draws two and you draw one at the next upkeep')

# Siren Stormtamer: counters a spell targeting you or your creature (as protection), not any spell
set_tags('Siren Stormtamer', remove=('ctr',))
import pool_ai
pool_ai.PROTECTORS['Siren Stormtamer'] = ('bf_sac', (0, 'U'), 'all_targeted', 'one', False)
full('Siren Stormtamer', 'flying; {U}, sacrifice: counters a spell targeting you or one of your creatures')


# ================================================================== removal restrictions and taxes
for _n in ('Snuff Out', 'Nekrataal', 'Shriekmaw', 'Doom Blade', 'Go for the Throat'):
    set_tags(_n, add=('nonblack',))
full('Snuff Out', 'free for 4 life with a Swamp; destroys a nonblack creature')
full('Nekrataal', 'first strike; destroys a nonartifact, nonblack creature on entry')


def removal_taxes(g, actor, m, kind):
    """targeting costs: Terror of the Peaks (3 life); damage to Phyrexian Obliterator costs permanents"""
    if m.cd.name == 'Terror of the Peaks':
        lose_life(g, actor, 3, actor)
    if m.cd.name == 'Phyrexian Obliterator' and kind.startswith('dmg'):
        n = int(kind[3:] or 0)
        for _ in range(n):
            ps = [x for x in actor.perms if not x.phased]
            if ps: die(g, min(ps, key=lambda x: pval(g, x)), 'sac')
    return True
full('Terror of the Peaks', 'flying; opponents\' spells targeting it cost 3 life; damage equal to power when your '
     'creatures enter')
full('Phyrexian Obliterator', 'trample; each source dealing it damage costs its controller that many permanents')


def transform_away(g, m, kind):
    """'becomes a 3/3 Elk' (Kenrith's Transformation, Oko), 'becomes a 0/1 indestructible Insect' (Darksteel Mutation),
    'becomes a Forest' (Song of the Dryads)"""
    if m not in m.owner.perms: return
    if kind == 'forest':
        leave(g, m)
        m.owner.lands.append(Land(E.DB['Forest'], m.tapped))
        log(f'    {m.name} becomes a Forest', g)
        return
    m.neutered = True
    if m.data is None: m.data = {}
    if kind == 'elk':
        m.pow, m.tgh = 3, 3; m.fly = False; m.dt = False; m.life = False
        m.data['elk'] = True
    else:
        m.pow, m.tgh = 0, 1; m.fly = False; m.dt = False; m.life = False
        m.data['indestr'] = True
    m.plus = min(m.plus, 0) if kind == 'mutate' else m.plus
    g.bf_ver = getattr(g, 'bf_ver', 0) + 1; g.hook_cache = None
    log(f'    {m.name} becomes a {"3/3 Elk" if kind == "elk" else "0/1 Insect"} with no abilities', g)


card("Kenrith's Transformation", 'rem=elk tgt=c etb draw=1', types='E', dsl=[])
full("Kenrith's Transformation", 'draw a card; the creature becomes a 3/3 Elk with no abilities')
card('Darksteel Mutation', 'rem=mutate tgt=c etb', types='E', dsl=[])
full('Darksteel Mutation', 'the creature becomes a 0/1 indestructible Insect artifact with no abilities')
card('Song of the Dryads', 'rem=forest tgt=nl etb', types='E', dsl=[])
full('Song of the Dryads', 'the permanent becomes a Forest land')


def _oko_elk_exact(g, p, src):
    t = best_opp_creature(g, p)
    if t is not None: transform_away(g, t, 'elk')


# ================================================================== Council's Judgment: the vote
@IC.spell("Council's Judgment", prio=lambda g, p, c: 58 if best_opp_nonland(g, p) is not None and pval(g, best_opp_nonland(g, p)) >= 4 else 0,
          tags='', types='S', status=('Full', 'will of the council: each player votes for a nonland permanent they don\'t '
                                              'control (their biggest threat); every permanent with the most votes is exiled'))
def _judgment(g, p, c, ctx):
    votes = {}
    for v in [q for q in g.players if q.alive]:
        cands = [m for q in g.players if q is not v and q.alive for m in q.perms if not m.phased]
        if v is p: cands = [m for m in cands if m.owner is not p]
        if not cands: continue
        pick = max(cands, key=lambda m: pval(g, m) * (1.0 + 0.3 * (threat(g, v, m.owner) if m.owner is not v else 0)))
        votes[id(pick)] = (votes.get(id(pick), (0, pick))[0] + 1, pick)
    if not votes: return
    top = max(n for n, _ in votes.values())
    for n, m in votes.values():
        if n == top and m in m.owner.perms:
            log(f'    Council\'s Judgment exiles {m.name} ({n} votes)', g); exile_perm(g, m)


# ================================================================== Fact or Fiction: the split
@IC.spell('Fact or Fiction', prio=45, tags='draw=2', types='I',
          status=('Full', 'the most threatened opponent splits the five into the piles that leave you least; you take '
                          'the better pile'))
def _fof(g, p, c, ctx):
    top = [p.library.pop() for _ in range(min(5, len(p.library)))]
    if not top: return
    w = [max(0.0, card_worth(g, p, x)) for x in top]
    best = None
    for mask in range(1 << len(top)):
        a = sum(w[i] for i in range(len(top)) if mask >> i & 1); b = sum(w) - a
        if best is None or max(a, b) < best[0]: best = (max(a, b), mask, a >= b)
    _, mask, take_a = best
    pile = [top[i] for i in range(len(top)) if bool(mask >> i & 1) == take_a]
    for x in top:
        if x in pile: p.hand.append(x); p.seen_names.add(x.name)
        else: p.gy.append(x)


# ================================================================== Boros Charm
@on('Boros Charm', 'hand_options')
def _boros_charm(g, c, p, s, post):
    if not can_pay(g, p, 0, 'RW') or not castable(g, p, c): return []
    lethal = [q for q in g.opps(p) if q.life <= 4 and not shielded(q)]
    if lethal:
        q = lethal[0]

        def go():
            if c not in p.hand or not can_pay(g, p, 0, 'RW'): return False
            pay(g, p, 0, 'RW'); p.hand.remove(c); on_cast(g, p, c)
            if counter_window(g, p, c, 6, {}): lose_life(g, q, 4, p, kind='burn', damage=True)
            p.gy.append(c); return True
        return [(9.0, 'Boros Charm (4 damage, lethal)', go)]
    return []
card('Boros Charm', '', types='I', dsl=[])
full('Boros Charm', 'permanents indestructible in response to removal and wipes, or 4 damage to a player at 4 or less')


# ================================================================== ability locks
def ability_locked(g, src, p):
    """can src's activated abilities be used right now? (Collector Ouphe / Karn: artifacts; Cursed Totem: creatures;
    Grand Abolisher: nothing of yours during its controller's turn)"""
    if not g.hooks or src.cd is None and not src.creature: return False
    art = (src.cd is not None and 'A' in src.cd.types) or (src.token and 'artifact' in src.ttypes)
    for q in g.players:
        if not q.alive: continue
        for m in q.perms:
            if m.cd is None or m.phased or m.neutered: continue
            n = m.cd.name
            if n == 'Collector Ouphe' and art: return True
            if n == 'Karn, the Great Creator' and art and p is not q: return True
            if n == 'Cursed Totem' and src.creature: return True
            if n == 'Grand Abolisher' and p is not q and g.active is q: return True
    return False


full('Collector Ouphe', 'activated abilities of artifacts can\'t be activated (mana rocks, Treasures and the rest)')
full('Cursed Totem', 'activated abilities of creatures can\'t be activated (mana creatures and the rest)')
full('Grand Abolisher', 'during your turn opponents can\'t cast spells or activate abilities of their permanents')


# ================================================================== emblems
def give_emblem(p, kind):
    if not hasattr(p, 'emblems') or p.emblems is None: p.emblems = set()
    p.emblems.add(kind)


def emblem_cast(g, p, c):
    if 'chandra' in p.emblems:
        t = best_opp_creature(g, p, lambda m: etgh(g, m) <= 5)
        lethal = [q for q in g.opps(p) if q.life <= 5]
        if lethal: lose_life(g, lethal[0], 5, p, kind='burn', damage=True)
        elif t is not None and pval(g, t) >= 4: apply_removal(g, p, t, 'dmg5')
        elif g.opps(p): lose_life(g, min(g.opps(p), key=lambda q: q.life), 5, p, kind='burn', damage=True)


def emblem_draw(g, p):
    if 'teferi' in p.emblems:
        t = best_opp_nonland(g, p)
        if t is not None: apply_removal(g, p, t, 'exile')


def emblem_combat(g, p, a, d, dmg):
    if dmg <= 0 or not a.creature: return
    if 'vraska' in p.emblems and d.alive:
        log(f'    Vraska\'s emblem: {NAME(d)} loses the game', g); d.life = 0; d.last_src = p; check_state(g)
    if 'kaito' in p.emblems:
        cs = [c for c in searchable(g, p) if c.creature and ('U' in c.pips or 'B' in c.pips)]
        if cs:
            c = max(cs, key=lambda c: (c.bomb, card_worth(g, p, c))); p.library.remove(c); g.rng.shuffle(p.library)
            enter(g, p, c)


def ult(name, delta, label, value, effect, text):
    WALKERS[name].append((delta, label, value, effect))
    note(name, 'Full', text)


def _always(x): return lambda g, p, src: x


ult('Chandra, Torch of Defiance', -7, 'emblem', _always(9.0), lambda g, p, src: give_emblem(p, 'chandra'),
    'both +1s, -3 for 4 damage, -7 emblem (5 damage per spell you cast)')
ult('Vraska, Golgari Queen', -9, 'emblem', _always(12.0), lambda g, p, src: give_emblem(p, 'vraska'),
    '+2 sacrifice for a card, -3 destroy MV 3 or less, -9 emblem (combat damage makes a player lose)')
ult('Teferi, Hero of Dominaria', -8, 'emblem', _always(10.0), lambda g, p, src: give_emblem(p, 'teferi'),
    '+1 draw and untap two lands, -3 tuck, -8 emblem (exile an opposing permanent whenever you draw)')
ult('Kaito Shizuki', -7, 'emblem', _always(8.0), lambda g, p, src: give_emblem(p, 'kaito'),
    'phases out the turn it enters; +1 draw, -2 unblockable Ninja, -7 emblem (combat damage puts a creature from your library onto the battlefield)')


def _tamiyo_ult(g, p, src):
    draw(g, p, 3); give_emblem(p, 'tamiyo')


ult('Tamiyo, Field Researcher', -7, 'draw three and emblem', _always(10.0), _tamiyo_ult,
    '+1 draws on combat damage, -2 freezes two permanents, -7 draw three and cast spells from hand for free')


def _lotv_ult(g, p, src):
    q = max(g.opps(p), key=lambda q: sum(pval(g, m) for m in q.perms))
    ps = sorted([m for m in q.perms if not m.phased], key=lambda m: -pval(g, m))
    a, b = ps[0::2], ps[1::2]
    keep = a if sum(pval(g, m) for m in a) >= sum(pval(g, m) for m in b) else b
    for m in ps:
        if m not in keep and m in q.perms: die(g, m, 'sac')


ult('Liliana of the Veil', -6, 'split permanents', lambda g, p, src: 8.0 if g.opps(p) else None, _lotv_ult,
    '+1 each player discards, -2 edict, -6 two piles (the opponent keeps the better one)')


def _ldg_ult(g, p, src):
    for q in g.opps(p):
        keep = {}
        for m in sorted([m for m in q.perms if not m.phased], key=lambda m: -pval(g, m)):
            k = 'C' if m.creature else (m.cd.types[0] if m.cd is not None and m.cd.types else 'T')
            keep.setdefault(k, m)
        for m in list(q.perms):
            if m not in keep.values(): die(g, m, 'sac')
        if len(q.lands) > 1:
            L = max(q.lands, key=lambda L: int(L.cd.tags.get('amt', 1)))
            for x in [x for x in q.lands if x is not L]: q.lands.remove(x); q.gy.append(x.cd)


ult('Liliana, Dreadhorde General', -9, 'each opponent keeps one of each type', _always(12.0), _ldg_ult,
    'creatures dying draw; +1 Zombie, -4 each player sacrifices two creatures, -9 opponents keep one permanent of each type')


def _jtms_ult(g, p, src):
    q = max(g.opps(p), key=lambda q: threat(g, p, q))
    q.exile.extend(q.library); q.library = list(q.hand); q.hand = []; g.rng.shuffle(q.library)
    log(f'    Jace exiles {NAME(q)}\'s library', g)


ult('Jace, the Mind Sculptor', -12, 'exile a library', _always(10.0), _jtms_ult,
    '+2 fateseal, 0 Brainstorm, -1 bounce, -12 exile a library (the hand becomes the library)')


def _karn_ult(g, p, src):
    log(f'    Karn Liberated restarts the game', g)
    for q in g.opps(p): q.life = 0; q.last_src = p
    check_state(g)


ult('Karn Liberated', -14, 'restart the game', _always(15.0), _karn_ult,
    '+4 exile a card from a hand, -3 exile a permanent, -14 restart the game with Karn\'s exiled cards (read as a win)')


def _ugin_ult(g, p, src):
    gain(p, 7); draw(g, p, 7)
    for c in sorted([c for c in p.hand if c.perm and not c.land], key=lambda c: -card_worth(g, p, c))[:7]:
        p.hand.remove(c); enter(g, p, c)


ult('Ugin, the Spirit Dragon', -10, 'gain 7, draw 7, put 7 permanents', _always(12.0), _ugin_ult,
    '+2 3 damage, -X exiles coloured permanents (X chosen to hit the most opposing value), -10 gain 7, draw 7, seven permanents')


def _nissa_ult(g, p, src):
    p.nissa_emblem = True
    forests = [c for c in searchable(g, p) if c.land and (c.name == 'Forest' or 'forest' in c.subtypes)]
    for c in forests: p.library.remove(c); p.lands.append(Land(c, True))
    g.rng.shuffle(p.library)


ult('Nissa, Who Shakes the World', -8, 'emblem and Forests', _always(8.0), _nissa_ult,
    'Forests tap for an extra G; +1 animates a land 0/0 with three counters, haste; -8 every Forest from the library, '
    'lands indestructible')


def _tezz_ult(g, p, src):
    for m in p.perms:
        if (m.cd is not None and 'A' in m.cd.types) or (m.token and 'artifact' in m.ttypes):
            if m.data is None: m.data = {}
            m.data['anim'] = True; m.pow = m.tgh = 5; m.sick = False
    p.tezz_turn = p.turns


ult('Tezzeret the Seeker', -5, 'artifacts become 5/5', lambda g, p, src: 1.5 * sum(
    1 for m in p.perms if (m.cd is not None and 'A' in m.cd.types and not m.creature)) if src.loyalty >= 5 else None, _tezz_ult,
    '+1 untap two artifacts, -X artifact tutor onto the battlefield, -5 artifacts become 5/5 creatures this turn')


def _jwom_ult(g, p, src):
    draw(g, p, 7)
    if not p.library: import ais; ais.win(g, p, 'Jace')


ult('Jace, Wielder of Mysteries', -8, 'draw seven', lambda g, p, src: 12.0 if len(p.library) <= 7 else 4.0, _jwom_ult,
    'drawing from an empty library wins; +1 mill two and draw; -8 draw seven and win with an empty library')


def _oko_exchange(g, p, src):
    mine = [m for m in p.perms if (m.token and 'food' in m.ttypes) or (m.creature and pval(g, m) < 2)]
    t = best_opp_creature(g, p, lambda m: epow(g, m) <= 3)
    if t is None: return
    q = t.owner
    q.perms.remove(t); t.owner = p; p.perms.append(t); t.sick = True
    if mine:
        x = mine[0]; p.perms.remove(x); x.owner = q; q.perms.append(x)
    g.bf_ver = getattr(g, 'bf_ver', 0) + 1; g.hook_cache = None
    log(f'    Oko exchanges control: {NAME(p)} takes {t.name}', g)


WALKERS['Oko, Thief of Crowns'][1] = (1, 'Elk an opposing creature', lambda g, p, src: (
    pval(g, t) - 2.0 if (t := best_opp_creature(g, p)) is not None and pval(g, t) >= 4 else None), _oko_elk_exact)
ult('Oko, Thief of Crowns', -5, 'exchange control', lambda g, p, src: (
    pval(g, t) - 1.0 if (t := best_opp_creature(g, p, lambda m: epow(g, m) <= 3)) is not None and pval(g, t) >= 5 else None),
    _oko_exchange, '+2 Food, +1 the best opposing creature becomes a vanilla 3/3 Elk, -5 exchange a spare creature for '
    'their best creature with power 3 or less')

# Dovin, Hand of Control: -1 neutralises a permanent until your next turn
WALKERS.setdefault('Dovin, Hand of Control', [])


def _dovin_minus(g, p, src):
    t = best_opp_creature(g, p)
    if t is not None:
        if t.data is None: t.data = {}
        t.data['dovin'] = (p, p.turns)
        log(f'    Dovin: damage to and from {t.name} is prevented until {NAME(p)}\'s next turn', g)


if 'Dovin, Hand of Control' in E.DB:
    IC.walker('Dovin, Hand of Control', [
        (-1, 'prevent damage to and from a creature', lambda g, p, src: (pval(g, t) - 2.5 if (t := best_opp_creature(g, p)) is not None
                                                                       and pval(g, t) >= 4 else None), _dovin_minus)],
        ('Full', 'opponents\' artifact, instant and sorcery spells cost {1} more; -1 neutralises their best attacker'))
    IC._tax_spell('Dovin, Hand of Control', 1, lambda c: 'A' in c.types or c.instant or c.sorcery)
    note('Dovin, Hand of Control', 'Full', 'opponents\' artifact, instant and sorcery spells cost {1} more; -1 prevents '
         'damage to and from their best creature until your next turn')


def dovin_blocked(m):
    d = (m.data or {}).get('dovin')
    return bool(d and d[0].alive and d[0].turns == d[1])


# ================================================================== single-card clauses
# Silver-Fur Master: Ninjas and Rogues
card('Silver-Fur Master', 'pow=2', dsl=[
    {'type': 'static', 'static': 'anthem', 'filter': {'type': 'creature', 'controller': 'you', 'other': True, 'subtype': 'ninja'}, 'pow': 1, 'tgh': 1},
    {'type': 'static', 'static': 'anthem', 'filter': {'type': 'creature', 'controller': 'you', 'other': True, 'subtype': 'rogue'}, 'pow': 1, 'tgh': 1}])
full('Silver-Fur Master', 'ninjutsu costs {1} less; other Ninjas and Rogues you control +1/+1')


# Spirit of the Labyrinth: each player draws at most one card a turn
set_tags('Spirit of the Labyrinth', add=('labyrinth',), remove=('narset',))
full('Spirit of the Labyrinth', 'each player (you too) can\'t draw more than one card each turn')


# Frost Titan: the tapped creature stays tapped
@on('Frost Titan', 'etb')
def _frost(g, src, p, m):
    if m is src: _freeze(g, src)


@on('Frost Titan', 'attack')
def _frost_atk(g, src, p, atk, d):
    if src in atk: _freeze(g, src)


def _freeze(g, src):
    t = best_opp_nonland(g, src.owner, lambda m: m.creature or (m.cd is not None and 'A' in m.cd.types))
    if t is None: return
    t.tapped = True
    if t.data is None: t.data = {}
    t.data['frozen'] = 1
    log(f'    Frost Titan taps {t.name} (it doesn\'t untap next turn)', g)
full('Frost Titan', 'ward {2}; taps a permanent on entry and attack; it doesn\'t untap during its controller\'s next untap step')


# Tithe Taker: afterlife
@on('Tithe Taker', 'self_dies')
def _tithe_afterlife(g, m, cause):
    make_tokens(g, m.owner, 1, 1, fly=True, color='W', types=('spirit',))
full('Tithe Taker', 'spells cost {1} more during your turn; afterlife 1')


# Bloodghast: haste while an opponent has 10 or less life
@on('Bloodghast', 'etb')
def _bloodghast(g, src, p, m):
    if m is src and any(q.life <= 10 for q in g.opps(src.owner)): src.sick = False
full('Bloodghast', 'can\'t block; haste while an opponent is at 10 or less; returns on landfall')


# Mogg War Marshal: echo is never paid (the sacrifice makes a second Goblin)
@on('Mogg War Marshal', 'upkeep')
def _mwm_echo(g, src, p):
    if p is src.owner and (src.data or {}).get('echo') != 'done':
        if src.data is None: src.data = {}
        src.data['echo'] = 'done'
        log(f'    {NAME(p)} doesn\'t pay echo for Mogg War Marshal', g); die(g, src, 'sac')
full('Mogg War Marshal', 'a Goblin on entry and on death; echo is left unpaid (sacrificed for the second Goblin)')


# Nature's Claim: its controller gains 4 life
@on("Nature's Claim", 'resolve')
def _claim(g, p, c, ctx):
    t = ctx.get('target')
    if t is None: t = best_opp_nonland(g, p, lambda m: m.cd is not None and ('A' in m.cd.types or 'E' in m.cd.types))
    if t is not None and t in t.owner.perms:
        q = t.owner; apply_removal(g, p, t, 'destroy', c); gain(q, 4)
full("Nature's Claim", 'destroys an artifact or enchantment; its controller gains 4 life')


# Plaguecrafter: a player with nothing to sacrifice discards
@on('Plaguecrafter', 'etb')
def _plague(g, src, p, m):
    if m is not src: return
    for q in [q for q in g.players if q.alive]:
        cs = [x for x in q.perms if (x.creature or (x.cd is not None and 'P' in x.cd.types)) and not x.phased]
        if q is src.owner: cs = [x for x in cs if x is not src] or cs
        if cs: die(g, min(cs, key=lambda x: pval(g, x)), 'sac')
        elif q.hand: discard_worst(g, q, 1)
card('Plaguecrafter', 'pow=3 tgh=2', dsl=[])
full('Plaguecrafter', 'each player sacrifices a creature or planeswalker, or discards a card if they can\'t')


# Liliana's Triumph: discard rider
@on("Liliana's Triumph", 'resolve')
def _triumph(g, p, c, ctx):
    for q in g.opps(p):
        cs = [x for x in q.perms if x.creature and not x.phased]
        if cs: die(g, min(cs, key=lambda x: pval(g, x)), 'sac')
    if any(m.cd is not None and 'Liliana' in m.cd.name for m in p.perms):
        for q in g.opps(p):
            if q.hand: discard_worst(g, q, 1)
full("Liliana's Triumph", 'each opponent sacrifices a creature; with a Liliana out, each discards too')


# Painful Quandary: opponents choose (discard a junk card, else 5 life)
@on('Painful Quandary', 'cast')
def _quandary(g, src, caster, c):
    if caster is src.owner: return
    junk = [x for x in caster.hand if card_worth(g, caster, x) < 30]
    if junk and caster.life > 10: discard_cards(g, caster, [min(junk, key=lambda x: card_worth(g, caster, x))])
    elif caster.hand and caster.life <= 10: discard_worst(g, caster, 1)
    else: lose_life(g, caster, 5, src.owner, kind='triggers')
card('Painful Quandary', '', types='E', dsl=[])
full('Painful Quandary', 'each opponent\'s spell: they discard a junk card (or any card when low on life), else lose 5')


# Sticky Fingers: draw when the creature dies
@on('Sticky Fingers', 'dies')
def _sticky_dies(g, src, m, cause):
    if src.attached is m: draw(g, src.owner, 1)
full('Sticky Fingers', 'menace; a Treasure on combat damage; draw when the enchanted creature dies')


# Reckless Fireweaver: every artifact entering
@on('Reckless Fireweaver', 'etb')
def _fireweaver_etb(g, src, p, m):
    if m.owner is src.owner and m is not src and m.cd is not None and 'A' in m.cd.types:
        for q in g.opps(src.owner): lose_life(g, q, 1, src.owner, kind='triggers')
full('Reckless Fireweaver', 'each artifact (cast or token) entering under your control deals 1 to each opponent')


# Kappa Cannoneer: unblockable only the turn an artifact entered
full('Kappa Cannoneer', 'ward {4}; +1/+1 counter and unblockable this turn when an artifact enters; improvise')


# Signal Pest: only fliers and reach can block it
full('Signal Pest', 'battle cry; can\'t be blocked except by creatures with flying or reach')


# Shriekmaw fear
full('Shriekmaw', 'fear; destroys a nonartifact, nonblack creature on entry; evoke {1}{B} when the full cost is out of reach')


def evasion_blocked(g, b, a):
    """extra blocking restrictions (pool games): fear, Signal Pest, Legion Loyalist, protection from creatures"""
    if a.cd is None: return False
    n = a.cd.name
    if n == 'Shriekmaw' and not ((b.cd is not None and 'A' in b.cd.types) or 'B' in colors_of(b)): return True
    if n == 'Signal Pest' and not (b.fly or (b.cd is not None and 'reach' in b.cd.tags)
                                   or (E.DSLMOD is not None and (E.DSLMOD.has_kw(g, b, 'flying') or E.DSLMOD.has_kw(g, b, 'reach')))): return True
    if b.token and getattr(a.owner, 'loyalist_turn', None) == turn_stamp(g): return True
    if dovin_blocked(a) or dovin_blocked(b): return False
    return False


# Legion Loyalist: battalion also stops tokens from blocking
@on('Legion Loyalist', 'attack')
def _loyalist2(g, src, p, atk, d):
    if src in atk and len(atk) >= 3:
        for m in atk: g.eot_kw.setdefault(id(m), set()).update(('first strike', 'trample'))
        p.loyalist_turn = turn_stamp(g)
full('Legion Loyalist', 'battalion: attackers gain first strike and trample and can\'t be blocked by tokens')


# Master of Cruelties attacks alone
full('Master of Cruelties', 'first strike, deathtouch; attacks alone; unblocked, the defender goes to 1 life')


# Moraug: +1/+0 per attack this turn
@on('Moraug, Fury of Akoum', 'attack')
def _moraug_pump(g, src, p, atk, d):
    if p is not src.owner: return
    for m in atk:
        if m.data is None: m.data = {}
        k = m.data.get('attacks')
        n = k[1] + 1 if k and k[0] == turn_stamp(g) else 1
        m.data['attacks'] = (turn_stamp(g), n)
        _eot(g, m, 1, 0)
full('Moraug, Fury of Akoum', 'creatures +1/+0 for each time they attacked this turn; landfall in your main phase: '
     'an additional combat')


# Brimaz: a blocking Cat
@on('Brimaz, King of Oreskos', 'blocks')
def _brimaz_block(g, src, p, atk, d, assign):
    if src.owner is d and src in assign.values(): make_tokens(g, d, 1, 1, color='W', types=('cat',))
full('Brimaz, King of Oreskos', 'vigilance; a 1/1 Cat attacking with it, and one blocking with it')


# Shared Animosity counts token types
full('Shared Animosity', 'each attacker +1/+0 per other attacker sharing a creature type')


# Allosaurus Shepherd: all your green spells can't be countered
@on('Allosaurus Shepherd', 'uncounterable')
def _shepherd(g, src, p, c):
    return 1 if p is src.owner and 'G' in c.pips else 0
full('Allosaurus Shepherd', 'can\'t be countered; your green spells can\'t be countered; Elves become 5/5 Dinosaurs')


# Ezuri: regenerate Elves
@on('Ezuri, Renegade Leader', 'options')
def _ezuri_none(g, src, p, s, post): return []


def ezuri_regen(g, m):
    p = m.owner
    if not E.has_type(m, 'elf') or m.cd is not None and m.cd.name == 'Ezuri, Renegade Leader': return False
    if not any(x.cd is not None and x.cd.name == 'Ezuri, Renegade Leader' and not x.phased for x in p.perms): return False
    if pval(g, m) < 3 or not can_pay(g, p, 0, 'G'): return False
    pay(g, p, 0, 'G'); m.tapped = True; log(f'    Ezuri regenerates {m.name}', g); return True
full('Ezuri, Renegade Leader', '{G}: regenerate another Elf (saves valuable Elves); overrun before combat with 4+ Elves')


# Dryad of the Ilysian Grove: lands are every basic type (any colour)
full('Dryad of the Ilysian Grove', 'an extra land each turn; your lands tap for any colour')


def dryad_colors(p):
    return any(m.cd is not None and m.cd.name == 'Dryad of the Ilysian Grove' and not m.phased for m in p.perms)


# Scions and Spawns sacrifice for mana
full('Carrier Thrall', 'a 1/1 Scion when it dies (sacrifice: {C})')
full('Sifter of Skulls', 'a 1/1 Scion when a nontoken creature of yours dies (sacrifice: {C})')
full('Pawn of Ulamog', 'a 0/1 Spawn when a nontoken creature of yours dies (sacrifice: {C})')
full('Awakening Zone', 'a 0/1 Spawn each upkeep (sacrifice: {C})')


# Mirkwood Bats: creating tokens drains too
@on('Mirkwood Bats', 'token_created')
def _bats_make(g, src, p, kinds, n):
    if p is src.owner:
        for q in g.opps(p): lose_life(g, q, n, p, kind='drain')
full('Mirkwood Bats', 'flying; each token you create or sacrifice drains each opponent 1')


# Everflowing Chalice: kicked as many times as the mana allows (up to three), taps for that many
@on('Everflowing Chalice', 'hand_options')
def _chalice(g, c, p, s, post):
    if post is not False or not castable(g, p, c): return []
    k = min(3, total_mana(g, p) // 2)
    if k < 1: return []

    def go():
        if c not in p.hand or not can_pay(g, p, 2 * k, ''): return False
        pay(g, p, 2 * k, ''); p.hand.remove(c)
        log(f'  {NAME(p)} casts Everflowing Chalice kicked {k} times', g)
        on_cast(g, p, c)
        if not counter_window(g, p, c, 2, {}): p.gy.append(c); return True
        m = enter(g, p, c, was_cast=True)
        if m.data is None: m.data = {}
        m.data['kicks'] = k
        return True
    return [(3.0 + k if p.turns <= 6 else 1.0 + k, f'Everflowing Chalice x{k}', go)]
CI.DYN_MANA['Everflowing Chalice'] = lambda g, p, m: (m.data or {}).get('kicks', 0)
CI.SPELL_PRIO['Everflowing Chalice'] = 0
full('Everflowing Chalice', 'multikicker {2} (as many kicks as the mana allows, up to three); taps for one per kick')


# Bloodchief's Thirst: kicked only when needed
@on("Bloodchief's Thirst", 'resolve')
def _thirst(g, p, c, ctx):
    t = ctx.get('target')
    if t is None or t not in t.owner.perms: return
    mv = t.cd.cmc if t.cd is not None else 0
    if mv > 2 and not getattr(p, 'thirst_kicked', False): return
    apply_removal(g, p, t, 'destroy', c)
full("Bloodchief's Thirst", 'destroys a creature or planeswalker with MV 2 or less, or anything when kicked (kicked '
     'when the target needs it and the mana is there)')


# Fatal Push: revolt
def revolt(g, p):
    return getattr(p, 'left_turn', None) == turn_stamp(g)
full('Fatal Push', 'destroys MV 2 or less, or MV 4 or less with revolt (a permanent of yours left the battlefield this turn)')


# Arbor Elf, Heritage Druid and the rest of the mana elves are exact already
for _n in ('Arbor Elf', 'Heritage Druid', 'Cabal Coffers', 'Nykthos, Shrine to Nyx', 'Carpet of Flowers', 'Urborg, Tomb of Yawgmoth'):
    note(_n, 'Full', {'Arbor Elf': 'untaps a Forest (mana)', 'Heritage Druid': 'tap three untapped Elves: GGG',
                      'Cabal Coffers': '{2},{T}: B per Swamp', 'Nykthos, Shrine to Nyx': '{2},{T}: mana equal to devotion',
                      'Carpet of Flowers': 'mana equal to an opponent\'s Islands in your main phase',
                      'Urborg, Tomb of Yawgmoth': 'every land is a Swamp'}[_n])
