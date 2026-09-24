"""Tier 1 pool decks: Isshin, Lathril, Light-Paws, Tatyova, Teysa."""
from engine import *
import engine as E
from cardimpl import on, _eot
from pool_cards import card, note


def first_attack(g, src):
    return once_per_turn(g, src.owner, f'first_attack_{id(src)}')


def untap_all(p, only=None):
    for m in p.perms:
        if m.creature and (only is None or m in only): m.tapped = False


def attacking_tokens(g, p, n, pw, tg=None, **kw):
    return make_tokens(g, p, n, pw, tg, attacking=True, sick=False, **kw)


# ======================================================== Isshin, Two Heavens as One
@on('Isshin, Two Heavens as One', 'trigger_copies')
def _isshin(g, src, p, kind, x):
    return 1 if kind == 'attack' and src.owner is p else 0
note('Isshin, Two Heavens as One', 'Full', 'attack-caused triggers of your permanents fire twice (hand tags, '
     'compiled triggers and hooks); ETB of tokens entering attacking is not doubled, matching the card')


@on('Hellrider', 'attack')
def _hellrider(g, src, p, atk, d):
    if src.owner is p:
        for _ in atk: lose_life(g, d, 1, p, kind='triggers')
note('Hellrider', 'Full', 'haste; 1 damage to the defending player per attacking creature')
card('Hellrider', 'pow=3 haste')


@on('Brutal Hordechief', 'attack')
def _hordechief(g, src, p, atk, d):
    if src.owner is p:
        for _ in atk: lose_life(g, d, 1, p, kind='drain'); gain(p, 1)
card('Brutal Hordechief', 'pow=3 warrior')
note('Brutal Hordechief', 'Partial', 'drain per attacker modeled; the forced-block activation is not')


@on('Aurelia, the Warleader', 'attack')
def _aurelia(g, src, p, atk, d):
    if src.owner is p and src in atk and first_attack(g, src):
        untap_all(p); p.extra_combats += 1
        log(f'    Aurelia: untap, additional combat', g)
card('Aurelia, the Warleader', 'leg pow=3 tgh=4 fly vig haste')
note('Aurelia, the Warleader', 'Full', 'first attack each turn: untap all creatures, one additional combat')


@on('Karlach, Fury of Avernus', 'attack')
def _karlach(g, src, p, atk, d):
    if src.owner is p and getattr(p, 'combat_no', 1) == 1:
        untap_all(p, atk)
        for m in atk: g.eot_kw.setdefault(id(m), set()).add('first strike')
        p.extra_combats += 1
card('Karlach, Fury of Avernus', 'leg pow=5 tgh=4')
note('Karlach, Fury of Avernus', 'Full', 'first combat: attackers untap and gain first strike; additional combat')


@on('Scourge of the Throne', 'attack')
def _scourge(g, src, p, atk, d):
    if src.owner is p and src in atk and first_attack(g, src) and d.life >= max(q.life for q in g.opps(p)):
        untap_all(p, atk); p.extra_combats += 1
card('Scourge of the Throne', 'pow=5 fly bomb=5')
note('Scourge of the Throne', 'Full', 'dethrone; first attack at the highest life total: untap attackers, extra combat')


@on('Brimaz, King of Oreskos', 'attack')
def _brimaz(g, src, p, atk, d):
    if src.owner is p and src in atk: return attacking_tokens(g, p, 1, 1, color='W')
card('Brimaz, King of Oreskos', 'leg pow=3 tgh=4 vig')
note('Brimaz, King of Oreskos', 'Approximate', 'attacking Cat token modeled; the blocking token is not')


@on('Hanweir Garrison', 'attack')
def _garrison(g, src, p, atk, d):
    if src.owner is p and src in atk: return attacking_tokens(g, p, 2, 1, color='R')
card('Hanweir Garrison', 'pow=2 tgh=3')
note('Hanweir Garrison', 'Full', 'two attacking Human tokens (meld ignored: its partner is not in the deck)')


@on('Anim Pakal, Thousandth Moon', 'attack')
def _anim(g, src, p, atk, d):
    if src.owner is not p or not any(not has_type(m, 'gnome') for m in atk): return
    src.plus += 1
    made = attacking_tokens(g, p, max(0, src.plus), 1, color='')
    for m in made: m.ttypes = frozenset(('gnome',))
    return made
card('Anim Pakal, Thousandth Moon', 'leg human pow=1 tgh=2')
note('Anim Pakal, Thousandth Moon', 'Full', 'counter, then X attacking Gnome tokens (X = its +1/+1 counters)')


@on('Caesar, Legion\'s Emperor', 'attack')
def _caesar(g, src, p, atk, d):
    if src.owner is not p: return
    fod = [m for m in p.perms if m.creature and m is not src and m not in atk and (m.token or pval(g, m) < 2)]
    if not fod: return
    die(g, min(fod, key=lambda m: pval(g, m)), 'sac')
    made = attacking_tokens(g, p, 2, 1, color='RW')
    for m in made: m.sick = False
    toks = sum(1 for m in p.perms if m.token and m.creature)
    tgt = max(g.opps(p), key=lambda q: threat(g, p, q)) if g.opps(p) else None
    if tgt is not None and toks >= 3: lose_life(g, tgt, toks, p, kind='triggers')
    else: draw(g, p, 1); lose_life(g, p, 1, p)
    return made
card('Caesar, Legion\'s Emperor', 'leg human pow=4')
note('Caesar, Legion\'s Emperor', 'Approximate', 'sacrifices spare fodder: two attacking tokens, then damage '
     'equal to tokens (3+) or draw; mode choice is a fixed rule')


@on('Goblin Rabblemaster', 'upkeep')
def _rabble_combat(g, src, p):
    if src.owner is p:
        for m in make_tokens(g, p, 1, 1, sick=False, color='R'): m.ttypes = frozenset(('goblin',))


@on('Goblin Rabblemaster', 'attack')
def _rabble_attack(g, src, p, atk, d):
    if src.owner is p and src in atk:
        _eot(g, src, sum(1 for m in atk if m is not src and has_type(m, 'goblin')), 0)
card('Goblin Rabblemaster', 'pow=2 warrior', dsl=[])
note('Goblin Rabblemaster', 'Approximate', 'hasty Goblin each turn, +1/+0 per other attacking Goblin; the '
     'must-attack clause is ignored (the AI attacks when it wants)')


@on('Legion Warboss', 'upkeep')
def _warboss(g, src, p):
    if src.owner is p:
        for m in make_tokens(g, p, 1, 1, sick=False, color='R'): m.ttypes = frozenset(('goblin',))
card('Legion Warboss', 'pow=2', dsl=[], kws={'mentor'})
note('Legion Warboss', 'Approximate', 'hasty Goblin each turn and mentor; the token is not forced to attack')


@on('Tilonalli\'s Summoner', 'attack')
def _tilonalli(g, src, p, atk, d):
    if src.owner is not p or src not in atk: return
    x = total_mana(g, p) - 1
    if x < 1 or not can_pay(g, p, x, 'R'): return
    pay(g, p, x, 'R')
    made = attacking_tokens(g, p, x, 1, color='R')
    city = len(p.perms) + len(p.lands) >= 10
    if not city:
        for m in made: m.temp = True                         # exiled at the end step
    return made
card('Tilonalli\'s Summoner', 'human shaman pow=1')
note('Tilonalli\'s Summoner', 'Approximate', 'spends all spare mana on X attacking tokens; kept with 10+ permanents')


@on('Shared Animosity', 'attack')
def _animosity(g, src, p, atk, d):
    if src.owner is not p: return
    for m in atk:
        n = sum(1 for x in atk if x is not m and _shares_type(m, x))
        if n: _eot(g, m, n, 0)


def _shares_type(a, b):
    return bool(subtypes(a) & subtypes(b))


card('Shared Animosity', '', types='E')
note('Shared Animosity', 'Approximate', '+1/+0 per other attacker sharing a creature type (tokens count as sharing)')


@on('Moonshaker Cavalry', 'etb')
def _moonshaker(g, src, p, m):
    if m is src:
        n = sum(1 for x in p.perms if x.creature)
        for x in p.perms:
            if x.creature:
                _eot(g, x, n, n); g.eot_kw.setdefault(id(x), set()).add('flying')
card('Moonshaker Cavalry', 'pow=6 fly bomb=6')
note('Moonshaker Cavalry', 'Full', 'ETB: creatures gain flying and +X/+X')


@on('Ogre Battledriver', 'etb')
def _battledriver(g, src, p, m):
    if m is not src and src.owner is p and m.creature and m.owner is p:
        _eot(g, m, 2, 0); m.sick = False
card('Ogre Battledriver', 'pow=3 warrior')
note('Ogre Battledriver', 'Full', 'other creatures entering get +2/+0 and haste')


@on('Mentor of the Meek', 'etb')
def _mentor_meek(g, src, p, m):
    if m is not src and src.owner is p and m.creature and m.owner is p and epow(g, m) <= 2 and can_pay(g, p, 1, '') \
            and len(p.hand) <= 6:
        pay(g, p, 1, ''); draw(g, p, 1)
card('Mentor of the Meek', 'human pow=2')
note('Mentor of the Meek', 'Full', 'pays {1} to draw when a small creature enters (if it can, hand not full)')


@on('Welcoming Vampire', 'etb')
def _welcoming(g, src, p, m):
    if m is not src and src.owner is p and m.creature and m.owner is p and epow(g, m) <= 2 \
            and once_per_turn(g, p, f'welcoming{id(src)}'):
        draw(g, p, 1)
card('Welcoming Vampire', 'pow=2 tgh=3 fly')
note('Welcoming Vampire', 'Full', 'draw once each turn when small creatures enter')


@on('Laelia, the Blade Reforged', 'attack')
def _laelia(g, src, p, atk, d):
    if src.owner is p and src in atk and p.library:
        c = p.library.pop(); p.hand.append(c); p.impulse.append(c); p.seen_names.add(c.name)
        src.plus += 1
card('Laelia, the Blade Reforged', 'leg warrior pow=2 haste')
note('Laelia, the Blade Reforged', 'Approximate', 'attack: exile top card playable this turn (as an impulse draw), +1/+1 counter')


@on('Outlaws\' Merriment', 'upkeep')
def _merriment(g, src, p):
    if src.owner is not p: return
    k = g.rng.randrange(3)
    if k == 0: make_tokens(g, p, 1, 3, 1, sick=False, color='RW')
    elif k == 1: make_tokens(g, p, 1, 2, 1, sick=False, lifelink=True, color='RW')
    else:
        make_tokens(g, p, 1, 1, 2, sick=False, color='RW'); bolt_something(g, p, 1)
card('Outlaws\' Merriment', '', types='E')
note('Outlaws\' Merriment', 'Approximate', 'random hasty token each upkeep (trample on the 3/1 ignored)')


@on('Black Market Connections', 'upkeep')
def _bmc(g, src, p):
    if src.owner is not p: return
    p.treasures += 1; lose_life(g, p, 1, p)
    if p.life > 15: draw(g, p, 1); lose_life(g, p, 2, p)
    if p.life > 20: make_tokens(g, p, 1, 3, 2, color=''); lose_life(g, p, 3, p)
card('Black Market Connections', '', types='E')
note('Black Market Connections', 'Approximate', 'each turn: Treasure; plus a card above 15 life and a 3/2 above 20')


@on('Loyal Apprentice', 'upkeep')
def _apprentice(g, src, p):
    if src.owner is p and commander_out(p): make_tokens(g, p, 1, 1, fly=True, sick=False, color='')
card('Loyal Apprentice', 'human pow=2 haste')
note('Loyal Apprentice', 'Full', 'Thopter at the beginning of combat while you control your commander')


@on('Legion\'s Landing // Adanto, the First Fort', 'attack')
def _landing(g, src, p, atk, d):
    if src.owner is p and len(atk) >= 3 and src in p.perms:
        leave(g, src)
        from engine import Land
        p.lands.append(Land(ADANTO, False))
        log('    Legion\'s Landing transforms into Adanto', g)


ADANTO = E.CD('Adanto, the First Fort', 'L', '-', 'c=W')
note('Legion\'s Landing // Adanto, the First Fort', 'Approximate', 'lifelink token, flips into a land with 3+ '
     'attackers; the land\'s token ability is not used')


# ======================================================== Lathril, Blade of the Elves (and Elf support shared with Marwyn)
from cardimpl import DYN_MANA, ON_TAP, count_type

ELF_WARRIOR = ('elf', 'warrior')


def elf_tokens(g, p, n):
    return make_tokens(g, p, n, 1, color='G', types=ELF_WARRIOR)


@on('Lathril, Blade of the Elves', 'combat_damage')
def _lathril_dmg(g, src, p, a, d, dmg):
    if a is src: elf_tokens(g, p, dmg)


@on('Lathril, Blade of the Elves', 'options')
def _lathril_drain(g, src, p, s, post):
    if src.tapped or src.sick or post is None: return []
    elves = [m for m in p.perms if m is not src and m.creature and not m.tapped and not m.phased and has_type(m, 'elf')]
    if len(elves) < 10: return []

    def go():
        es = [m for m in p.perms if m is not src and m.creature and not m.tapped and has_type(m, 'elf')]
        if len(es) < 10 or src.tapped: return False
        src.tapped = True
        for m in sorted(es, key=lambda m: (not m.sick, pval(g, m)))[:10]: m.tapped = True
        for q in g.opps(p): lose_life(g, q, 10, p, kind='drain')
        gain(p, 10); log(f'  Lathril taps ten Elves: each opponent loses 10', g); check_state(g)
        return True
    lethal = any(q.life <= 10 for q in g.opps(p))
    return [(9.0 if lethal else 6.0, 'Lathril drain 10', go)]
card('Lathril, Blade of the Elves', 'leg pow=2 tgh=3', dsl=[])
note('Lathril, Blade of the Elves', 'Full', 'menace; Elf Warrior tokens equal to combat damage; tap ten Elves: drain 10')


DYN_MANA['Priest of Titania'] = lambda g, p, m: count_type(g, p, 'elf', everyone=True)
DYN_MANA['Elvish Archdruid'] = lambda g, p, m: count_type(g, p, 'elf')
DYN_MANA['Wirewood Channeler'] = lambda g, p, m: count_type(g, p, 'elf', everyone=True)
DYN_MANA['Llanowar Tribe'] = lambda g, p, m: 3
DYN_MANA['Canopy Tactician'] = lambda g, p, m: 3
card('Priest of Titania', 'pow=1 dork=G noatk')
card('Wirewood Channeler', 'pow=2 dork=A noatk')
card('Llanowar Tribe', 'pow=3 dork=G')
for _n in ('Priest of Titania', 'Wirewood Channeler', 'Llanowar Tribe'):
    note(_n, 'Full', 'taps for its full scaling amount')
note('Elvish Archdruid', 'Full', 'Elf lord; taps for G per Elf you control')
note('Canopy Tactician', 'Full', 'Elf lord; taps for GGG')
card('Canopy Tactician', 'pow=3 warrior dork=G',
     dsl=[{'type': 'static', 'static': 'anthem', 'filter': {'type': 'creature', 'controller': 'you', 'other': True,
                                                            'subtype': 'elf'}, 'pow': 1, 'tgh': 1}])
card('Elvish Archdruid', 'pow=2 dork=G noatk',
     dsl=[{'type': 'static', 'static': 'anthem', 'filter': {'type': 'creature', 'controller': 'you', 'other': True,
                                                            'subtype': 'elf'}, 'pow': 1, 'tgh': 1}])


def _heritage_amt(g, p, m):
    elves = [x for x in p.perms if x.creature and not x.tapped and not x.phased and has_type(x, 'elf')
             and not (x.cd is not None and 'dork' in x.cd.tags)]
    return 3 * (len(elves) // 3) if len(elves) >= 3 else 0


def _heritage_tap(g, p, m, used):
    m.tapped = False                                 # Heritage Druid itself isn't tapped; three other Elves are
    elves = sorted([x for x in p.perms if x is not m and x.creature and not x.tapped and has_type(x, 'elf')
                    and not (x.cd is not None and 'dork' in x.cd.tags)], key=lambda x: (not x.sick, pval(g, x)))
    for x in elves[:3 * ((used + 2) // 3)]: x.tapped = True


DYN_MANA['Heritage Druid'] = _heritage_amt
ON_TAP['Heritage Druid'] = _heritage_tap
card('Heritage Druid', 'pow=1 dork=G noatk')
note('Heritage Druid', 'Approximate', 'GGG per three untapped non-mana Elves (summoning-sick ones included, as on the card)')


@on("Dwynen's Elite", 'etb')
def _dwynen(g, src, p, m):
    if m is src and any(x is not src and has_type(x, 'elf') for x in p.perms): elf_tokens(g, p, 1)
card("Dwynen's Elite", 'pow=2 warrior', dsl=[])
note("Dwynen's Elite", 'Full', '')


@on('Elvish Warmaster', 'etb')
def _warmaster(g, src, p, m):
    if m is not src and m.owner is src.owner and has_type(m, 'elf') and once_per_turn(g, src.owner, f'warm{id(src)}'):
        elf_tokens(g, src.owner, 1)


@on('Elvish Warmaster', 'options')
def _warmaster_pump(g, src, p, s, post):
    if post is not False or not can_pay(g, p, 5, 'GG'): return []
    elves = [m for m in p.perms if m.creature and has_type(m, 'elf') and not m.sick and not m.tapped]
    if len(elves) < 5: return []

    def go():
        if not can_pay(g, p, 5, 'GG'): return False
        pay(g, p, 5, 'GG')
        for m in p.perms:
            if m.creature and has_type(m, 'elf'): _eot(g, m, 2, 2); g.eot_kw.setdefault(id(m), set()).add('deathtouch')
        return True
    return [(1.0 + 0.5 * len(elves), 'Elvish Warmaster pump', go)]
card('Elvish Warmaster', 'pow=2 warrior', dsl=[])
note('Elvish Warmaster', 'Full', 'Elf Warrior once a turn when Elves enter; pump before combat with a wide board')


@on('Lys Alana Huntmaster', 'cast')
def _lys(g, src, caster, c):
    if caster is src.owner and c.creature and 'elf' in c.subtypes: elf_tokens(g, caster, 1)
card('Lys Alana Huntmaster', 'pow=3 warrior', dsl=[])
note('Lys Alana Huntmaster', 'Full', '')


@on('Leaf-Crowned Visionary', 'cast')
def _leafcrowned(g, src, caster, c):
    if caster is src.owner and c.creature and 'elf' in c.subtypes and can_pay(g, caster, 0, 'G') and len(caster.hand) < 7:
        pay(g, caster, 0, 'G'); draw(g, caster, 1)
card('Leaf-Crowned Visionary', 'pow=1',
     dsl=[{'type': 'static', 'static': 'anthem', 'filter': {'type': 'creature', 'controller': 'you', 'other': True,
                                                            'subtype': 'elf'}, 'pow': 1, 'tgh': 1}])
note('Leaf-Crowned Visionary', 'Full', 'Elf lord; pays G to draw on Elf casts when it can')


card('Elvish Promenade', 'mktok=1:1:0', types='S',
     dsl=[{'type': 'spell', 'effects': [{'do': 'token', 'n': 'type_you:elf', 'pow': 1, 'tgh': 1, 'keywords': [],
                                         'types': ['elf', 'warrior']}]}])
note('Elvish Promenade', 'Full', 'an Elf Warrior token for each Elf you control')


@on('Prowess of the Fair', 'dies')
def _prowess(g, src, m, cause):
    if m.owner is src.owner and not m.token and m.creature and has_type(m, 'elf') and m is not src:
        elf_tokens(g, src.owner, 1)
card('Prowess of the Fair', '', types='E', dsl=[])
note('Prowess of the Fair', 'Full', '')


@on('Eyeblight Cullers', 'self_dies')
def _cullers(g, m, cause):
    p = m.owner
    if p.alive:
        elf_tokens(g, p, 3); mill(g, p, 3)
        for q in g.opps(p): lose_life(g, q, 2, p, kind='drain')
        gain(p, 2)
card('Eyeblight Cullers', 'pow=3 warrior', dsl=[])
note('Eyeblight Cullers', 'Full', '')


def look_take(g, p, n, pred, to='hand', rest='bottom', k=1):
    """look at the top n cards, take up to k matching (best first), the rest to the bottom"""
    top = [p.library.pop() for _ in range(min(n, len(p.library)))]
    hits = sorted([c for c in top if pred(c)], key=lambda c: -card_worth(g, p, c))[:k]
    for c in hits:
        top.remove(c)
        if to == 'hand': p.hand.append(c); p.seen_names.add(c.name)
        elif to == 'land_bf': p.lands.append(Land(c, True)); landfall(g, p)
        elif to == 'bf': enter(g, p, c)
    g.rng.shuffle(top); p.library[:0] = top
    return hits


@on('Harald, King of Skemfar', 'etb')
def _harald(g, src, p, m):
    if m is src: look_take(g, p, 5, lambda c: 'elf' in c.subtypes or 'warrior' in c.subtypes or c.name.startswith('Tyvar'))
card('Harald, King of Skemfar', 'leg pow=3 tgh=2 warrior', dsl=[])
note('Harald, King of Skemfar', 'Full', 'menace; top five: an Elf/Warrior/Tyvar to hand')


@on('Sylvan Messenger', 'etb')
def _messenger(g, src, p, m):
    if m is src: look_take(g, p, 4, lambda c: 'elf' in c.subtypes, k=4)
card('Sylvan Messenger', 'pow=2 trample', dsl=[])
note('Sylvan Messenger', 'Full', '')


@on('Elvish Rejuvenator', 'etb')
def _rejuvenator(g, src, p, m):
    if m is src: look_take(g, p, 5, lambda c: c.land, to='land_bf')
card('Elvish Rejuvenator', 'pow=1', dsl=[])
note('Elvish Rejuvenator', 'Full', '')


@on('Shaman of the Pack', 'etb')
def _shaman_pack(g, src, p, m):
    if m is src and g.opps(p):
        q = max(g.opps(p), key=lambda o: threat(g, p, o)); lose_life(g, q, count_type(g, p, 'elf'), p, kind='drain')
card('Shaman of the Pack', 'pow=3 tgh=2 shaman', dsl=[])
note('Shaman of the Pack', 'Full', '')


@on('Skemfar Avenger', 'dies')
def _skemfar(g, src, m, cause):
    if m.owner is src.owner and m is not src and not m.token and (has_type(m, 'elf') or has_type(m, 'berserker')):
        draw(g, src.owner, 1); lose_life(g, src.owner, 1, src.owner)
card('Skemfar Avenger', 'pow=3 tgh=1', dsl=[])
note('Skemfar Avenger', 'Full', '')


@on('Timberwatch Elf', 'options')
def _timberwatch(g, src, p, s, post):
    if post is not False or src.tapped or src.sick: return []
    atk = [m for m in p.perms if m.creature and not m.tapped and not m.sick and m is not src and not m.noatk]
    if not atk: return []
    x = count_type(g, p, 'elf', everyone=True)

    def go():
        if src.tapped: return False
        src.tapped = True
        t = max(atk, key=lambda m: (m.fly or m.is_cmd, epow(g, m))); _eot(g, t, x, x); return True
    return [(1.0 + 0.3 * x, 'Timberwatch Elf pump', go)]
card('Timberwatch Elf', 'pow=1 tgh=2', dsl=[])
note('Timberwatch Elf', 'Approximate', 'pumps its best attacker before combat')


@on('Tyvar the Bellicose', 'attack')
def _tyvar(g, src, p, atk, d):
    if src.owner is p:
        for m in atk:
            if has_type(m, 'elf'): g.eot_kw.setdefault(id(m), set()).add('deathtouch')
card('Tyvar the Bellicose', 'leg pow=5 tgh=4 warrior', dsl=[])
note('Tyvar the Bellicose', 'Partial', 'attacking Elves gain deathtouch; the +1/+1 counters on mana creatures are not modeled')


@on('Ezuri, Renegade Leader', 'options')
def _ezuri(g, src, p, s, post):
    if post is not False or not can_pay(g, p, 2, 'GGG'): return []
    elves = [m for m in p.perms if m.creature and has_type(m, 'elf') and not m.sick and not m.tapped]
    if len(elves) < 4: return []

    def go():
        if not can_pay(g, p, 2, 'GGG'): return False
        pay(g, p, 2, 'GGG')
        for m in p.perms:
            if m.creature and has_type(m, 'elf'): _eot(g, m, 3, 3)
        p.trample = True; return True
    return [(1.0 + 0.8 * len(elves), 'Ezuri overrun', go)]
card('Ezuri, Renegade Leader', 'leg pow=2 warrior', dsl=[])
note('Ezuri, Renegade Leader', 'Approximate', 'overrun before combat with 4+ Elves; regeneration not modeled')


@on('Allosaurus Shepherd', 'options')
def _allosaurus(g, src, p, s, post):
    if post is not False or not can_pay(g, p, 4, 'GG'): return []
    elves = [m for m in p.perms if m.creature and has_type(m, 'elf') and not m.sick and not m.tapped and epow(g, m) < 5]
    if len(elves) < 4: return []

    def go():
        if not can_pay(g, p, 4, 'GG'): return False
        pay(g, p, 4, 'GG')
        for m in p.perms:
            if m.creature and has_type(m, 'elf'): _eot(g, m, max(0, 5 - epow(g, m)), max(0, 5 - etgh(g, m)))
        return True
    return [(1.0 + 0.6 * len(elves), 'Allosaurus Shepherd 5/5s', go)]
card('Allosaurus Shepherd', 'pow=1 unc', dsl=[])
note('Allosaurus Shepherd', 'Approximate', 'uncounterable; Elves become 5/5 before combat; other green spells still counterable')


@on('Beastmaster Ascension', 'attack')
def _bma(g, src, p, atk, d):
    if src.owner is p: src.plus += len(atk)


card('Beastmaster Ascension', '', types='E',
     dsl=[{'type': 'static', 'static': 'bma_anthem'}])
note('Beastmaster Ascension', 'Full', 'quest counter per attacker; +5/+5 at seven')


@on('Tendershoot Dryad', 'upkeep')
def _tendershoot(g, src, p):
    o = src.owner
    city = len(o.perms) + len(o.lands) >= 10
    for m in make_tokens(g, o, 1, 3 if city else 1, types=('saproling',), color='G'): pass
card('Tendershoot Dryad', 'pow=2', dsl=[])
note('Tendershoot Dryad', 'Approximate', 'a Saproling every upkeep; the city\'s blessing bonus is applied when the token is made')


@on('Guardian Project', 'etb')
def _guardian(g, src, p, m):
    o = src.owner
    if m.owner is o and m.creature and not m.token and m.cd is not None and \
            not any(x is not m and x.cd is not None and x.cd.name == m.cd.name for x in o.perms) and \
            not any(c.name == m.cd.name for c in o.gy):
        draw(g, o, 1)
card('Guardian Project', '', types='E', dsl=[])
note('Guardian Project', 'Full', '')


@on('Nath of the Gilt-Leaf', 'upkeep')
def _nath_up(g, src, p):
    if src.owner is p and g.opps(p):
        q = max(g.opps(p), key=lambda o: (len(o.hand) > 0, threat(g, p, o)))
        if q.hand: discard_index(g, q, g.rng.randrange(len(q.hand)))


@on('Nath of the Gilt-Leaf', 'discard')
def _nath_disc(g, src, q, c):
    if q is not src.owner:
        for m in make_tokens(g, src.owner, 1, 1, dt=True, color='G', types=ELF_WARRIOR): pass
card('Nath of the Gilt-Leaf', 'leg pow=4 warrior', dsl=[])
note('Nath of the Gilt-Leaf', 'Full', 'random discard each upkeep; Elf Warrior (deathtouch) whenever an opponent discards')


@on('Elderfang Ritualist', 'self_dies')
def _ritualist(g, m, cause):
    p = m.owner
    cs = [c for c in p.gy if 'elf' in c.subtypes and c is not m.cd]
    if cs: c = max(cs, key=lambda c: card_worth(g, p, c)); p.gy.remove(c); p.hand.append(c)
card('Elderfang Ritualist', 'pow=3 tgh=1', dsl=[])
note('Elderfang Ritualist', 'Full', '')


@on('Elvish Harbinger', 'etb')
def _harbinger(g, src, p, m):
    if m is not src: return
    cs = [c for c in p.library if 'elf' in c.subtypes]
    if cs:
        c = max(cs, key=lambda c: card_worth(g, p, c)); p.library.remove(c); g.rng.shuffle(p.library); p.library.append(c)
card('Elvish Harbinger', 'pow=1 tgh=2 dork=A noatk', dsl=[])
note('Elvish Harbinger', 'Full', 'Elf to the top; taps for any colour')


card('Return of the Wildspeaker', 'rotw', types='I')
note('Return of the Wildspeaker', 'Approximate', 'draw mode after combat, +3/+3 mode before combat, whichever is bigger')
card('Jagged-Scar Archers', 'pow=0 tgh=0 reach')
note('Jagged-Scar Archers', 'Partial', 'P/T = Elves you control; the flier-shooting ability is not modeled')


# ======================================================== Light-Paws, Emperor's Voice (Aura voltron)
import impl_common as IC


def my_auras(p):
    return [m for m in p.perms if m.cd is not None and 'aura' in m.cd.subtypes]


@on('Light-Paws, Emperor\'s Voice', 'etb')
def _lightpaws(g, src, p, m):
    o = src.owner
    if m is src or m.owner is not o or m.cd is None or 'aura' not in m.cd.subtypes or not getattr(g, 'last_cast_etb', False):
        return
    have = {a.cd.name for a in my_auras(o)}
    cands = [c for c in o.library if 'aura' in c.subtypes and c.cmc <= m.cd.cmc and c.name not in have
             and c.name in IC.AURA and IC.AURA[c.name]['target'] == 'own'
             and (IC.AURA[c.name]['host_ok'] is None or IC.AURA[c.name]['host_ok'](g, o, src))]
    if not cands: return
    c = max(cands, key=lambda c: (c.cmc, card_worth(g, o, c)))
    o.library.remove(c); g.rng.shuffle(o.library)
    log(f'    Light-Paws fetches {c.name}', g)
    g.attach_to = src
    try:
        enter(g, o, c)
    finally:
        g.attach_to = None
card('Light-Paws, Emperor\'s Voice', 'leg pow=2')
note('Light-Paws, Emperor\'s Voice', 'Full', 'cast Aura -> fetch an Aura with lower or equal MV and a new name, '
     'onto Light-Paws')


@on('Flickering Ward', 'options')
def _ward_loop(g, src, p, s, post):
    """{W}: return to hand, recast for W: each cast re-triggers Light-Paws for a one-mana Aura"""
    lp = [m for m in p.perms if m.cd is not None and m.cd.name == 'Light-Paws, Emperor\'s Voice']
    if post is None or not lp or not can_pay(g, p, 0, 'WW'): return []
    have = {a.cd.name for a in my_auras(p)}
    if not any('aura' in c.subtypes and c.cmc <= 1 and c.name not in have and c.name in IC.AURA for c in p.library): return []

    def go():
        if src not in p.perms or not can_pay(g, p, 0, 'WW'): return False
        pay(g, p, 0, 'W'); leave(g, src); c = src.cd
        pay(g, p, 0, 'W'); cast_card(g, p, c, 'hand', {}) if c in p.hand else None
        return True
    return [(5.0, 'Flickering Ward recast (Light-Paws)', go)]


@on('Sram, Senior Edificer', 'cast')
def _sram(g, src, caster, c):
    if caster is src.owner and ('aura' in c.subtypes or 'equipment' in c.subtypes or 'vehicle' in c.subtypes):
        draw(g, caster, 1)
card('Sram, Senior Edificer', 'leg pow=2')
note('Sram, Senior Edificer', 'Full', '')


@on('Kor Spiritdancer', 'cast')
def _spiritdancer(g, src, caster, c):
    if caster is src.owner and 'aura' in c.subtypes: draw(g, caster, 1)


@on('Kor Spiritdancer', 'etb')
def _selfpt_on(g, src, p, m):
    if m is src: g.selfpt = True


IC.SELF_PT['Kor Spiritdancer'] = lambda g, p, m: (2 * len(IC.auras_on(g, m)) if getattr(g, 'auras', None) else 0,) * 2
card('Kor Spiritdancer', 'pow=0 tgh=2 wizard')
note('Kor Spiritdancer', 'Full', '+2/+2 per Aura on it; draw on Aura casts')


@on('Eidolon of Countless Battles', 'etb')
def _eidolon_on(g, src, p, m):
    if m is src: g.selfpt = True


IC.SELF_PT['Eidolon of Countless Battles'] = lambda g, p, m: (
    (k := sum(1 for x in p.perms if x.creature) + sum(1 for x in p.perms if x.cd is not None and 'aura' in x.cd.subtypes)), k)
card('Eidolon of Countless Battles', 'pow=0 tgh=0')
note('Eidolon of Countless Battles', 'Partial', 'cast as a creature with its +1/+1 per creature and Aura; bestow not modeled')


@on('Archon of Sun\'s Grace', 'etb')
def _archon_sg(g, src, p, m):
    o = src.owner
    if m.owner is o and m.cd is not None and 'E' in m.cd.types:
        make_tokens(g, o, 1, 2, fly=True, lifelink=True, color='W', types=('pegasus',))
card('Archon of Sun\'s Grace', 'pow=3 tgh=4 fly lifelink')
note('Archon of Sun\'s Grace', 'Full', 'constellation: 2/2 flying lifelink Pegasus')


@on('Heliod\'s Pilgrim', 'etb')
def _pilgrim(g, src, p, m):
    if m is src: tutor_named(g, p, lambda c: 'aura' in c.subtypes)
card('Heliod\'s Pilgrim', 'human pow=1 tgh=2')
note('Heliod\'s Pilgrim', 'Full', '')


def tutor_named(g, p, pred, k=1, to='hand'):
    have = {c.name for c in p.hand} | {m.cd.name for m in p.perms if m.cd is not None}
    import pool_ai
    wish = pool_ai.wish_list(g, p)
    import impl_common
    lib = [c for c in p.library if c.name in wish and c.name not in have]

    def chain(c):                          # a tutor that can find a wished card (Recruiter -> Spellseeker)
        f = impl_common.TUTOR_PRED.get(c.name)
        return f is not None and c.name not in have and any(f(x) for x in lib)
    rank = lambda c: (c.name in wish and c.name not in have, chain(c), -wish.index(c.name) if c.name in wish else 0,
                      c.name not in have, card_worth(g, p, c))
    cands = sorted({c.name: c for c in searchable(g, p) if pred(c)}.values(), key=rank, reverse=True)[:k]
    for c in cands:
        p.library.remove(c)
        a = agent_for(g, p)
        if a is not None: agent_take(g, a, p, c); continue
        if to == 'hand': p.hand.append(c); p.seen_names.add(c.name)
        p.stats['tutored'] += 1
    g.rng.shuffle(p.library)
    return cands


card('Three Dreams', 'threedreams', types='S')
note('Three Dreams', 'Full', 'three Auras with different names to hand')


@on('Land Tax', 'upkeep')
def _landtax(g, src, p):
    if src.owner is p and any(len(q.lands) > len(p.lands) for q in g.opps(p)):
        for _ in range(3): land_to_hand(g, p)
card('Land Tax', '', types='E')
note('Land Tax', 'Full', '')


@on('Weathered Wayfarer', 'options')
def _wayfarer(g, src, p, s, post):
    if post is None or src.tapped or src.sick or not can_pay(g, p, 0, 'W'): return []
    if not any(len(q.lands) > len(p.lands) for q in g.opps(p)) or any(c.land for c in p.hand): return []

    def go():
        if src.tapped or not can_pay(g, p, 0, 'W'): return False
        pay(g, p, 0, 'W'); src.tapped = True
        tutor_named(g, p, lambda c: c.land); return True
    return [(2.5, 'Weathered Wayfarer', go)]
card('Weathered Wayfarer', 'human pow=1')
note('Weathered Wayfarer', 'Full', '')


# ======================================================== Tatyova, Benthic Druid (lands matter)
def _extra(n):
    return lambda g, src, p: n if src.owner is p else 0


for _n, _k in (('Exploration', 1), ('Azusa, Lost but Seeking', 2), ('Dryad of the Ilysian Grove', 1),
               ('Oracle of Mul Daya', 1), ('Aesi, Tyrant of Gyre Strait', 1)):
    on(_n, 'extra_lands')(_extra(_k))
    note(_n, 'Full', f'{_k} additional land drop(s) each turn')
card('Exploration', '', types='E', dsl=[])
card('Azusa, Lost but Seeking', 'leg human pow=1 tgh=2', dsl=[])
card('Dryad of the Ilysian Grove', 'pow=2 tgh=4', dsl=[])
note('Dryad of the Ilysian Grove', 'Approximate', 'extra land drop; lands of every basic type ignored')
on('Oracle of Mul Daya', 'lands_from_top')(lambda g, src, p: 1 if src.owner is p else 0)
card('Oracle of Mul Daya', 'pow=2 shaman', dsl=[])
note('Oracle of Mul Daya', 'Full', 'extra land drop; plays lands from the top of the library')
for _n in ('Crucible of Worlds', 'Ramunap Excavator', 'Ancient Greenwarden'):
    on(_n, 'lands_from_gy')(lambda g, src, p: 1 if src.owner is p else 0)
card('Crucible of Worlds', '', types='A', dsl=[])
note('Crucible of Worlds', 'Full', 'land drops from the graveyard')
card('Ramunap Excavator', 'pow=2 tgh=3', dsl=[])
note('Ramunap Excavator', 'Full', 'land drops from the graveyard')


@on('Ancient Greenwarden', 'trigger_copies')
def _greenwarden(g, src, p, kind, x):
    return 1 if kind == 'landfall' and src.owner is p else 0
card('Ancient Greenwarden', 'pow=5 tgh=7 reach bomb=5', dsl=[])
note('Ancient Greenwarden', 'Full', 'lands from the graveyard; landfall triggers twice')


@on('Aesi, Tyrant of Gyre Strait', 'landfall')
def _aesi(g, src, p):
    if src.owner is p and len(p.library) > 15: draw(g, p, 1)       # "you may draw"
card('Aesi, Tyrant of Gyre Strait', 'leg pow=5 bomb=5', dsl=[])


@on('Burgeoning', 'land_play')
def _burgeoning(g, src, p, c):
    o = src.owner
    if p is not o:
        ls = [x for x in o.hand if x.land]
        if ls:
            import ais
            x = ls[0]; o.hand.remove(x); o.lands.append(Land(x, ais.land_enters_tapped(o, x))); landfall(g, o)
card('Burgeoning', '', types='E', dsl=[])
note('Burgeoning', 'Full', '')


card('Explore', 'draw=1 explore', types='S')
note('Explore', 'Full', 'draw; one extra land drop this turn')


@on('Druid Class', 'landfall')
def _druidclass(g, src, p):
    if src.owner is p: gain(p, 1)


@on('Druid Class', 'extra_lands')
def _druidclass_extra(g, src, p):
    return 1 if src.owner is p and (src.data or {}).get('level', 1) >= 2 else 0


@on('Druid Class', 'options')
def _druidclass_level(g, src, p, s, post):
    if post is not False or (src.data or {}).get('level', 1) >= 2 or not can_pay(g, p, 2, 'G'): return []

    def go():
        if not can_pay(g, p, 2, 'G'): return False
        pay(g, p, 2, 'G'); src.data = {'level': 2}; return True
    return [(2.0, 'Druid Class level 2', go)]
card('Druid Class', '', types='E', dsl=[])
note('Druid Class', 'Partial', 'landfall life and the level-2 extra land drop; level 3 (land creature) not modeled')


@on('Avenger of Zendikar', 'etb')
def _avenger(g, src, p, m):
    if m is src:
        for t in make_tokens(g, src.owner, len(src.owner.lands), 0, 1, color='G', types=('plant',)): pass


@on('Avenger of Zendikar', 'landfall')
def _avenger_lf(g, src, p):
    if src.owner is p:
        for m in p.perms:
            if m.token and has_type(m, 'plant'): m.plus += 1
card('Avenger of Zendikar', 'pow=5 bomb=6', dsl=[])
note('Avenger of Zendikar', 'Full', '0/1 Plant per land; landfall: +1/+1 counter on each Plant')


@on('Scute Swarm', 'landfall')
def _scute(g, src, p):
    if src.owner is not p: return
    if len(p.lands) >= 6: enter_token_copy(g, p, src.cd)
    else: make_tokens(g, p, 1, 1, color='G', types=('insect',))
card('Scute Swarm', 'pow=1', dsl=[])
note('Scute Swarm', 'Full', 'landfall: Insect, or a copy of itself with six or more lands (token cap 250)')


@on('Roil Elemental', 'landfall')
def _roil(g, src, p):
    if src.owner is not p: return
    cands = [m for q in g.opps(p) for m in q.perms if m.creature and not untargetable(g, m) and not m.is_cmd]
    if not cands: return
    m = max(cands, key=lambda m: pval(g, m))
    if pval(g, m) < 2: return
    q = m.owner; q.perms.remove(m); m.owner = p; m.attached = None; p.perms.append(m); g.bf_ver = getattr(g, 'bf_ver', 0) + 1
    if src.data is None: src.data = {}
    src.data.setdefault('stolen', []).append(m)
    log(f'    Roil Elemental steals {m.name}', g)


@on('Roil Elemental', 'leaves')
def _roil_leaves(g, src):
    for m in (src.data or {}).get('stolen', []):
        if m in src.owner.perms:
            src.owner.perms.remove(m); m.owner = m.orig; m.orig.perms.append(m); g.bf_ver = getattr(g, 'bf_ver', 0) + 1
card('Roil Elemental', 'pow=3 tgh=2 fly', dsl=[])
note('Roil Elemental', 'Full', 'landfall: steal the best opposing creature while it stays')


@on('Nissa, Resurgent Animist', 'landfall')
def _nissa_ra(g, src, p):
    if src.owner is not p: return
    p.floatA += 1
    k = f'nissa{id(src)}'
    st = turn_stamp(g)
    n = p.flag_turn.get(k)
    n = (st, (n[1] + 1) if n and n[0] == st else 1); p.flag_turn[k] = n
    if n[1] == 2:
        for c in reversed(p.library):
            if 'elf' in c.subtypes or 'elemental' in c.subtypes:
                p.library.remove(c); p.hand.append(c); break
card('Nissa, Resurgent Animist', 'leg pow=3', dsl=[])
note('Nissa, Resurgent Animist', 'Approximate', 'landfall mana (spendable this turn); second landfall finds an Elf or '
     'Elemental (the other revealed cards go to the bottom in the original, here they stay)')


@on('Tireless Provisioner', 'landfall')
def _provisioner(g, src, p):
    if src.owner is p: p.treasures += 1
card('Tireless Provisioner', 'pow=3 tgh=2', dsl=[])
note('Tireless Provisioner', 'Approximate', 'always takes the Treasure')


@on('Tireless Tracker', 'landfall')
def _tracker(g, src, p):
    if src.owner is p: p.clues += 1


@on('Tireless Tracker', 'sacrifice')
def _tracker_clue(g, src, p, what):
    if p is src.owner and what == 'Clue': src.plus += 1
card('Tireless Tracker', 'human pow=3 tgh=2', dsl=[])
note('Tireless Tracker', 'Full', '')


@on('Titania, Protector of Argoth', 'etb')
def _titania(g, src, p, m):
    if m is src:
        ls = [c for c in src.owner.gy if c.land]
        if ls:
            c = max(ls, key=lambda c: len(c.tags.get('c', ''))); src.owner.gy.remove(c)
            src.owner.lands.append(Land(c, True)); landfall(g, src.owner)


@on('Titania, Protector of Argoth', 'land_gy')
def _titania_gy(g, src, p, cd):
    if p is src.owner: make_tokens(g, p, 1, 5, 3, color='G', types=('elemental',))
card('Titania, Protector of Argoth', 'leg pow=5 tgh=3 bomb=5', dsl=[])
note('Titania, Protector of Argoth', 'Full', 'returns a land; 5/3 Elemental when a land goes to the graveyard '
     '(fetch lands, land sacrifices)')


@on('Coiling Oracle', 'etb')
def _coiling(g, src, p, m):
    if m is src and src.owner.library:
        c = src.owner.library.pop()
        if c.land: src.owner.lands.append(Land(c, False)); landfall(g, src.owner)
        else: src.owner.hand.append(c)
card('Coiling Oracle', 'pow=1', dsl=[])
note('Coiling Oracle', 'Full', '')


@on('Uro, Titan of Nature\'s Wrath', 'etb')
def _uro(g, src, p, m):
    if m is not src: return
    _uro_value(g, src.owner)
    if getattr(g, 'uro_escaping', False): src.data = {'escaped': True}
    else: die(g, src, 'sac')


@on('Uro, Titan of Nature\'s Wrath', 'attack')
def _uro_atk(g, src, p, atk, d):
    if src in atk: _uro_value(g, p)


def _uro_value(g, p):
    gain(p, 3); draw(g, p, 1)
    import dsl
    dsl.run(g, p, {'do': 'put_land'}, None, {}, None, 0)


@on('Uro, Titan of Nature\'s Wrath', 'gy_options')
def _uro_escape(g, c, p, s, post):
    if post is None or not can_pay(g, p, 0, 'GGUU') or len(p.gy) < 6: return []

    def go():
        if c not in p.gy or not can_pay(g, p, 0, 'GGUU') or len(p.gy) < 6: return False
        pay(g, p, 0, 'GGUU'); p.gy.remove(c)
        others = sorted(p.gy, key=lambda x: card_worth(g, p, x, True))[:5]
        for x in others: p.gy.remove(x); p.exile.append(x)
        p.cast_names.add(c.name); p.spells_this_turn += 1; on_cast(g, p, c)
        g.uro_escaping = True
        try: enter(g, p, c, was_cast=True)
        finally: g.uro_escaping = False
        return True
    return [(6.0, 'escape Uro', go)]


card('Uro, Titan of Nature\'s Wrath', 'leg pow=6 bomb=6', dsl=[])
note('Uro, Titan of Nature\'s Wrath', 'Approximate', 'enters: 3 life, draw, land (then sacrificed); escape from '
     'the graveyard as a 6/6 that repeats it on attack (escaped Uro is not countered here)')


card('Life from the Loam', 'loam', types='S', dsl=[])
note('Life from the Loam', 'Approximate', 'returns up to three lands; dredges itself back instead of a draw when '
     'the hand is short on lands')


@on('Zendikar Resurgent', 'cast')
def _zr(g, src, caster, c):
    if caster is src.owner and c.creature: draw(g, caster, 1)


@on('Zendikar Resurgent', 'land_mana')
def _zr_mana(g, src, p, L):
    return 1 if src.owner is p else 0
card('Zendikar Resurgent', '', types='E', dsl=[])
note('Zendikar Resurgent', 'Full', 'lands tap for double; creature spells draw')


card('Hydroid Krasis', 'pow=0 tgh=0 krasis fly trample', dsl=[])
note('Hydroid Krasis', 'Full', 'cast with all spare mana as X: draw and gain X/2, X counters, flying, trample')
card('Rishkar\'s Expertise', 'rishkar', types='S', dsl=[])
note('Rishkar\'s Expertise', 'Full', 'draw equal to the greatest power, then cast a spell with MV 5 or less free')
card('Mulldrifter', 'pow=2 fly draw=2', dsl=[])
note('Mulldrifter', 'Approximate', 'hard-cast only (evoke not used)')
card('Krosan Grip', 'rem=destroy tgt=ae unc', types='I', dsl=[])
note('Krosan Grip', 'Full', 'split second read as uncounterable')
card('Nature\'s Claim', 'rem=destroy tgt=ae', types='I', dsl=[])
note('Nature\'s Claim', 'Approximate', 'the 4 life for the controller is ignored')
card('Retreat to Coralhelm', '', types='E', dsl=[])
note('Retreat to Coralhelm', 'Unmodeled', 'tap/untap and scry on landfall not modeled')


# ======================================================== Teysa Karlov (aristocrats)
@on('Teysa Karlov', 'trigger_copies')
def _teysa(g, src, p, kind, m):
    return 1 if kind == 'dies' and src.owner is p and (m is None or m.creature) else 0


@on('Teysa Karlov', 'grant_kw')
def _teysa_kw(g, src, m, kw):
    return kw in ('vigilance', 'lifelink') and m.token and m.creature and m.owner is src.owner
card('Teysa Karlov', 'leg human pow=2 tgh=4')
note('Teysa Karlov', 'Full', 'death-caused triggers of your permanents fire twice; tokens have vigilance and lifelink')
