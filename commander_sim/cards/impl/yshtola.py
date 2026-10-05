"""Y'shtola, Night's Blessed (your Esper Drain deck, key `yshtola`, decklists/mine/yshtola-esper-drain.md).

- Y'shtola: whenever you cast a noncreature spell with mana value 3 or greater (X counts), she deals 2 damage to each
  opponent and you gain 2 life; at the beginning of each end step, if a player lost 4 or more life this turn, you draw.
- The drain package: Exsanguinate and Debt to the Deathless (X chosen by the AI or the person), Ill-Gotten
  Inheritance, Urborg Syphon-Mage, Marauding Blight-Priest (Sanguine Bond is in t4.py).
- Cards taken from opponents and cast with mana of any type: Gonti, Lord of Luxury, Hostage Taker, Thief of Sanity.
  The card is held in your hand (as Opposition Agent's are), counted out of your hand size, and goes back to its
  owner's graveyard; Hostage Taker's card returns to the battlefield if the Taker leaves before you cast it.
- The rest that needs code: Curiosity (on Y'shtola it draws for each opponent her trigger hits), Jester's Cap, Dark
  Petition (spell mastery adds {B}{B}{B}), Plea for Guidance, Take Up the Shield, Champion's Helm for the AI.
- Zur the Enchanter in the 99: its attack trigger (cards/impl/zur.py) fetches with this deck's values (fetch_value).
- The AI: cast priorities (a spell that triggers Y'shtola is worth more), X and drain timing, tutor targets,
  protection and wipe responses (Clever Concealment, Rootborn Defenses, Take Up the Shield, blinks).
"""
import importlib
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim.cards import cardimpl as CI
from commander_sim.cards.cardimpl import on
from commander_sim.cards.pool_cards import card, note
from commander_sim.cards.impl import common as IC

YSH = "Y'shtola, Night's Blessed"


def full(name, text): note(name, 'Full', text)


def human(g, p):
    return E.human_choice(g, p)


def _zur():
    return importlib.import_module('commander_sim.cards.impl.zur')


def _legal():
    return importlib.import_module('commander_sim.play.legal')


def named(p, name):
    return next((m for m in p.perms if m.cd is not None and m.cd.name == name and not m.phased), None)


def ysh_perm(p):
    """Y'shtola on p's battlefield (the commander, or a copy under p's control)"""
    return next((m for m in p.perms if m.cd is not None and m.cd.name == YSH and not m.phased and not m.neutered), None)


def lost_this_turn(g, q):
    lt = getattr(q, 'lost_turn', None)
    return lt[1] if lt and lt[0] == turn_stamp(g) else 0


# ================================================================== Y'shtola
card(YSH, 'leg pow=2 tgh=4 vig', dsl=[])


def cast_mv(g, c):
    """the mana value of spell c as it's cast: X counts (Exsanguinate, Debt to the Deathless, Secure the Wastes)"""
    cur = getattr(g, 'cur_cast', None)
    x = cur[1].get('x', 0) if cur is not None and cur[0] is c else 0
    return c.cmc + (3 * x if 'crackle' in c.tags else x)


@on(YSH, 'cast')
def _ysh_cast(g, src, caster, c):
    """whenever you cast a noncreature spell with mana value 3 or greater: 2 damage to each opponent, gain 2 life"""
    o = src.owner
    if caster is not o or c.creature or c.land or cast_mv(g, c) < 3 or not g.opps(o): return
    if not trigger_window(g, o, src, '2 damage to each opponent; you gain 2 life', imp=4): return
    ysh_drain(g, o, src)


def ysh_drain(g, o, src):
    o.milestone.setdefault('yshtola', o.turns)
    o.stats['ysh_triggers'] += 1
    log(f"    Y'shtola deals 2 damage to each opponent; {NAME(o)} gains 2 life", g)
    hit = []
    for q in g.opps(o):
        before = q.life
        lose_life(g, q, 2, o, kind='triggers', damage=True)
        if q.life < before: hit.append(q)
    gain(o, 2)
    if src in o.perms and hit: curiosity_draws(g, src, len(hit))
    check_state(g)


@on(YSH, 'end_step')
def _ysh_end(g, src, p):
    """at the beginning of each end step, if a player lost 4 or more life this turn, you draw a card"""
    if not any(lost_this_turn(g, q) >= 4 for q in g.players): return
    if not trigger_window(g, src.owner, src, 'draw a card (a player lost 4 or more life this turn)', imp=3): return
    src.owner.stats['ysh_draws'] += 1
    draw(g, src.owner, 1)


full(YSH, 'vigilance; your noncreature spells with mana value 3 or more (X counts) deal 2 damage to each opponent and '
     'gain you 2 life; at each end step you draw if any player lost 4 or more life this turn')


# ================================================================== Curiosity
def _curiosity_host(g, p, spec, a):
    """on Y'shtola (her trigger hits every opponent), else your best evasive creature"""
    if human(g, p) is not None: return None
    y = ysh_perm(p)
    if y is not None and not _zur().untargetable_by_you(g, y): return y
    cands = [m for m in p.perms if m.creature and not m.phased and m is not a and not m.noatk]
    return max(cands, key=lambda m: ((3 if m.fly else 0) + epow(g, m) + pval(g, m) * 0.2), default=None)


IC.aura('Curiosity', target='either', host_pick=_curiosity_host,
        status=('Full', "whenever the enchanted creature deals damage to an opponent you draw (combat damage, and "
                        "Y'shtola's trigger: one card for each opponent it hits); the AI puts it on Y'shtola"))


def may_draw(g, p):
    return len(p.library) > 3


@on('Curiosity', 'combat_damage')
def _curiosity(g, src, p, a, d, dmg):
    o = src.owner
    if a is not src.attached or d not in g.opps(o) or dmg <= 0 or not may_draw(g, o): return
    if not trigger_window(g, o, src, 'draw a card'): return
    draw(g, o, 1)


def curiosity_draws(g, creature, n):
    """creature (Y'shtola) dealt damage to n opponents outside combat: each Curiosity on it draws n"""
    for a in list(getattr(g, 'auras', None) or ()):
        if a.attached is not creature or a.cd is None or a.cd.name != 'Curiosity' or a.phased or a not in a.owner.perms:
            continue
        o = a.owner
        for _ in range(n):
            if not may_draw(g, o): break
            if trigger_window(g, o, a, 'draw a card'): draw(g, o, 1)


# ================================================================== the drain package
def drain_extra(g, p):
    """life each opponent also loses for each time you gain life (Marauding Blight-Priest)"""
    return sum(1 for m in p.perms if m.cd is not None and m.cd.name == 'Marauding Blight-Priest' and not m.phased)


def bond_on(p):
    return any(m.cd is not None and m.cd.name == 'Sanguine Bond' and not m.phased for m in p.perms)


@on('Marauding Blight-Priest', 'gain_life')
def _priest(g, src, p, n):
    o = src.owner
    if p is not o or not g.opps(o): return
    if not trigger_window(g, o, src, 'each opponent loses 1 life'): return
    for q in g.opps(o): lose_life(g, q, 1, o, kind='drain')


card('Marauding Blight-Priest', 'pow=3 tgh=2', dsl=[])
full('Marauding Blight-Priest', 'whenever you gain life, each opponent loses 1 life')


# ------------------------------------------------------------------ Exsanguinate, Debt to the Deathless
X_DRAIN = {'Exsanguinate': 1, 'Debt to the Deathless': 2}      # each opponent loses (this) x X


def x_drain(g, p, c, x):
    lost = 0
    k = X_DRAIN[c.name]
    for q in g.opps(p):
        b = q.life
        lose_life(g, q, k * x, p, kind='drain')
        lost += max(0, b - q.life)
    log(f'    {c.name} (X = {x}): each opponent loses {k * x} life; {NAME(p)} gains {lost}', g)
    if lost: gain(p, lost)
    check_state(g)


def _x_resolve(g, p, c, ctx):
    x_drain(g, p, c, ctx.get('x', 0))
    return 'gy'


for _n in X_DRAIN:
    CI.HOOKS.setdefault(_n, {})['resolve'] = _x_resolve
card('Exsanguinate', 'xdrain', types='S', dsl=[])
card('Debt to the Deathless', 'xdrain', types='S', dsl=[])
full('Exsanguinate', 'each opponent loses X life and you gain the total; the AI casts it for a kill or a big X')
full('Debt to the Deathless', 'each opponent loses twice X life and you gain the total; the AI casts it for a kill or a '
     'big X')


def drain_kills(g, p, per_opp, gained):
    """opponents dead after each loses per_opp and you gain `gained` (Sanguine Bond sends it to one more)"""
    opps = g.opps(p)
    dead = [q for q in opps if q.life <= per_opp and not q.life_locked]
    if bond_on(p):
        left = [q for q in opps if q not in dead and not q.life_locked]
        if any(q.life <= per_opp + gained for q in left): dead.append(min(left, key=lambda q: q.life))
    return len(dead)


def x_drain_option(g, p, c):
    """the best X for a drain spell in hand, as (utility, label, fn), or None"""
    gen, pips = cost_of(p, c)
    if not can_pay(g, p, gen, pips): return None
    x = total_mana(g, p) - gen - len(pips)
    if x < 1: return None
    k = X_DRAIN[c.name]
    opps = g.opps(p)
    ysh = ysh_perm(p) is not None and c.cmc + x >= 3
    pri = drain_extra(g, p)
    per = k * x + (2 if ysh else 0) + pri * (2 if ysh else 1)          # its gain, and Y'shtola's, each set off the Priest
    total = sum(min(q.life, per) for q in opps if not q.life_locked)
    kills = drain_kills(g, p, per, total)
    late = p.turns >= 8 or total_mana(g, p) >= 9
    if kills == 0 and (x < 4 or (not late and k * x * len(opps) < 15)): return None
    u = 1.5 + 0.12 * total + 7.0 * kills + (1.0 if bond_on(p) else 0)

    def go():
        if c not in p.hand: return False
        g2, p2 = cost_of(p, c)
        xx = total_mana(g, p) - g2 - len(p2)
        if xx < 1 or not can_pay(g, p, g2 + xx, p2): return False
        pay(g, p, g2 + xx, p2)
        cast_card(g, p, c, 'hand', {'x': xx})
        return True
    return u, f'{c.name} (X = {x})', go


def _x_option(g, c, p, s, post):
    if post is None or g.active is not p or c not in p.hand or p.key != 'yshtola': return []
    o = x_drain_option(g, p, c)
    return [o] if o else []


for _n in X_DRAIN: CI.HOOKS[_n]['hand_options'] = _x_option


# ------------------------------------------------------------------ Ill-Gotten Inheritance
@on('Ill-Gotten Inheritance', 'upkeep')
def _inheritance(g, src, p):
    o = src.owner
    if p is not o or not g.opps(o): return
    if not trigger_window(g, o, src, '1 damage to each opponent; you gain 1 life'): return
    for q in g.opps(o): lose_life(g, q, 1, o, kind='triggers', damage=True)
    gain(o, 1)
    check_state(g)


@on('Ill-Gotten Inheritance', 'options')
def _inheritance_sac(g, src, p, s, post):
    """{5}{B}, sacrifice it: 4 damage to target opponent and you gain 4 life (for a kill, or at an opponent's end
    step with the mana spare)"""
    if p is not src.owner or human(g, p) is not None or not can_pay(g, p, 5, 'B') or not g.opps(p): return []
    t, kill = inheritance_target(g, p)
    if not kill and (post is not None or g.active is p or p.turns < 9): return []

    def go():
        if src not in p.perms or not can_pay(g, p, 5, 'B') or not t.alive: return False
        pay(g, p, 5, 'B')
        inheritance_sac(g, p, src, t)
        return True
    return [(9.0 if kill else 1.5 + (1.0 if bond_on(p) else 0), f'Ill-Gotten Inheritance: 4 damage to {NAME(t)}', go)]


def inheritance_target(g, p):
    per = 4 + drain_extra(g, p)
    opps = [q for q in g.opps(p) if not q.life_locked and not prevents_damage(g, q, p)] or g.opps(p)
    dead = [q for q in opps if q.life <= per]
    if dead: return max(dead, key=lambda q: threat(g, p, q)), True
    return max(opps, key=lambda q: threat(g, p, q)), False


def inheritance_sac(g, p, src, t):
    log(f'  {NAME(p)} sacrifices Ill-Gotten Inheritance: 4 damage to {NAME(t)}', g)
    die(g, src, 'sac')
    if not ability_window(g, p, src.cd, f'4 damage to {NAME(t)}; gain 4 life', imp=4): return
    if t.alive: lose_life(g, t, 4, p, kind='triggers', damage=True)
    gain(p, 4)
    check_state(g)


card('Ill-Gotten Inheritance', '', types='E', dsl=[])
full('Ill-Gotten Inheritance', 'your upkeep: 1 damage to each opponent, gain 1; {5}{B}, sacrifice: 4 damage to an '
     'opponent, gain 4 (the AI uses it for a kill, or late at an opponent\'s end step)')


# ------------------------------------------------------------------ Urborg Syphon-Mage
@on('Urborg Syphon-Mage', 'options')
def _syphon(g, src, p, s, post):
    """{2}{B}, {T}, discard a card: each other player loses 2 life; you gain the total"""
    if p is not src.owner or human(g, p) is not None or src.tapped or src.sick or not can_pay(g, p, 2, 'B'): return []
    worst = syphon_discard(g, p)
    if worst is None: return []
    per = 2 + drain_extra(g, p)
    total = sum(min(q.life, per) for q in g.opps(p) if not q.life_locked)
    kills = drain_kills(g, p, per, total)
    spare = worst.land and sum(1 for c in p.hand if c.land) >= 2 and len(p.lands) >= 5
    if not kills and not spare and post is not None: return []        # otherwise at an opponent's end step only
    if not kills and post is None and g.active is p: return []
    cost = card_worth(g, p, worst) / 20.0
    u = 1.0 + 0.15 * total + 8.0 * kills + (1.0 if bond_on(p) else 0) - cost

    def go():
        if src not in p.perms or src.tapped or worst not in p.hand or not can_pay(g, p, 2, 'B'): return False
        pay(g, p, 2, 'B'); src.tapped = True
        discard_cards(g, p, [worst])
        syphon(g, p, src)
        return True
    return [(u, f'Urborg Syphon-Mage (discard {worst.name})', go)] if u > 0.5 else []


def syphon_discard(g, p):
    stolen = getattr(p, 'stolen', None) or {}
    cs = [c for c in p.hand if id(c) not in stolen]
    return min(cs, key=lambda c: card_worth(g, p, c)) if cs else None


def syphon(g, p, src):
    log(f'  {NAME(p)} activates Urborg Syphon-Mage', g)
    if not ability_window(g, p, src, 'each other player loses 2 life', imp=4): return
    lost = 0
    for q in [q for q in g.players if q is not p and q.alive]:
        b = q.life
        lose_life(g, q, 2, p, kind='drain')
        lost += max(0, b - q.life)
    if lost: gain(p, lost)
    check_state(g)


card('Urborg Syphon-Mage', 'human pow=2 tgh=2', dsl=[])
full('Urborg Syphon-Mage', "{2}{B}, {T}, discard a card: each other player loses 2 life and you gain the total (the AI: "
     "for a kill, a spare land, or at an opponent's end step)")


# ================================================================== cards taken from opponents
def take_card(g, p, c, owner, why):
    """p may cast c (owner's card, in exile) with mana of any type: held in p's hand, as Opposition Agent's cards"""
    p.hand.append(c); p.agent_ids.add(id(c))
    if getattr(p, 'stolen', None) is None: p.stolen = {}
    p.stolen[id(c)] = owner
    p.stats['cards_stolen'] += 1
    log_secret(g, p, f'    {NAME(p)} exiles a card face down ({why})', f'    {NAME(p)} exiles {c.name} with {why}')


def stolen_worth(g, p, c):
    """how much p wants to cast opponent's card c (a land can't be cast)"""
    if c.land: return -1.0
    from commander_sim.ai import pool_ai
    v = pool_ai.generic_prio(g, p, c) or (E.DSLMOD.card_value(g, p, c) * 10 if c.dsl and E.DSLMOD is not None else 0)
    t = c.tags
    if not v: v = 55 if 'ctr' in t else 50 if ('rem' in t or 'wipe' in t) else 30
    return v + 4 * c.bomb + (6 if c.cmc <= len(p.lands) + 1 else 0)


def steal_pick(g, p, top, owner, why):
    """p looks at cards `top` from owner's library and exiles one: the person's pick, else the best to cast"""
    hc = human(g, p)
    if hc is not None:
        k = hc.choose(g, p, 'choose', f'{why}: exile which card face down? (you may cast it, with mana of any type)',
                      [c.name + (' (a land: it can\'t be cast)' if c.land else '') for c in top], cancel=None)
        return top[k]
    return max(top, key=lambda c: stolen_worth(g, p, c))


def exile_for(g, p, pick, owner, why):
    if pick.land: owner.exile.append(pick); log_secret(g, p, f'    {NAME(p)} exiles a card face down ({why})',
                                                      f'    {NAME(p)} exiles {pick.name} with {why}')
    else: take_card(g, p, pick, owner, why)


@on('Gonti, Lord of Luxury', 'etb')
def _gonti(g, src, p, m):
    """look at the top four of target opponent's library, exile one face down (castable), the rest on the bottom"""
    if m is not src: return
    o = src.owner
    opps = [q for q in g.opps(o) if q.library]
    if not opps: return
    if not trigger_window(g, o, src, "look at the top four of an opponent's library, exile one", imp=4): return
    opps = [q for q in g.opps(o) if q.library]
    if not opps: return
    hc = human(g, o)
    if hc is not None and len(opps) > 1:
        k = hc.choose(g, o, 'target', "Gonti: look at which opponent's library?", [NAME(q) for q in opps], cancel=None)
        q = opps[k]
    else:
        q = max(opps, key=lambda q: threat(g, o, q))
    top = [q.library.pop() for _ in range(min(4, len(q.library)))]
    pick = steal_pick(g, o, top, q, 'Gonti')
    rest = [c for c in top if c is not pick]
    g.rng.shuffle(rest)
    for c in rest: q.library.insert(0, c)
    exile_for(g, o, pick, q, 'Gonti')


card('Gonti, Lord of Luxury', 'leg pow=2 tgh=3 dt', dsl=[])
full('Gonti, Lord of Luxury', "deathtouch; enters: the top four of an opponent's library, one exiled face down that you "
     "may cast with mana of any type, the rest on the bottom")


@on('Thief of Sanity', 'combat_damage')
def _thief(g, src, p, a, d, dmg):
    """combat damage to a player: the top three of their library, one exiled face down (castable), the rest to the
    graveyard"""
    if a is not src or p is not src.owner or not d.library or dmg <= 0: return
    if not trigger_window(g, p, src, f"look at the top three of {NAME(d)}'s library, exile one", imp=4): return
    if not d.library: return
    top = [d.library.pop() for _ in range(min(3, len(d.library)))]
    pick = steal_pick(g, p, top, d, 'Thief of Sanity')
    for c in top:
        if c is not pick: d.gy.append(c)
    exile_for(g, p, pick, d, 'Thief of Sanity')


card('Thief of Sanity', 'pow=2 tgh=2 fly', dsl=[])
full('Thief of Sanity', 'flying; combat damage to a player: their top three, one exiled face down that you may cast '
     'with mana of any type, the rest into their graveyard')


@on('Hostage Taker', 'etb')
def _taker(g, src, p, m):
    """exile another target creature or artifact until Hostage Taker leaves; you may cast it with mana of any type"""
    if m is not src: return
    o = src.owner
    hc = human(g, o)
    t = taker_target(g, o, src) if hc is None else None
    if t is None and (hc is None or not taker_cands(g, o, src)): return
    if not trigger_window(g, o, src, f'exile {t.name}' if t is not None else 'exile a creature or artifact', imp=5): return
    if hc is not None:                              # the person's target (asked once the trigger is committed)
        cands = taker_cands(g, o, src)
        if not cands: return
        k = hc.choose(g, o, 'target', 'Hostage Taker: exile which other creature or artifact?',
                      [_legal().describe_target(g, o, x) for x in cands], cancel=None)
        t = cands[k]
    if t not in t.owner.perms or t.phased: return
    owner, cd, tok = t.orig, (t.phys or t.cd), t.token
    prev, g.rem_src = getattr(g, 'rem_src', None), src
    try:
        apply_removal(g, o, t, 'exile', src.cd)
    finally:
        g.rem_src = prev
    if tok or cd is None or t in t.owner.perms or cd not in owner.exile: return     # a token, a commander, or saved
    owner.exile.remove(cd)
    if src not in o.perms:                         # the Taker already left: the card returns at once
        enter(g, owner, cd); return
    src.data = dict(src.data or {}, hostage=(cd, owner))
    take_card(g, o, cd, owner, 'Hostage Taker')


def taker_cands(g, o, src):
    """another creature or artifact Hostage Taker can target (blue and black: protection from either stops it)"""
    return [x for q in g.players if q.alive for x in q.perms if x is not src and not x.phased
            and (x.creature or (x.cd is not None and 'A' in x.cd.types))
            and not (x.owner is not o and untargetable(g, x)) and not protected_from(g, x, 'U') and not protected_from(g, x, 'B')]


def taker_target(g, o, src):
    """the AI's target: the best opposing creature or artifact"""
    opp = [x for x in taker_cands(g, o, src) if x.owner is not o]
    best = max(opp, key=lambda x: pval(g, x) + (1.5 if not x.token and x.cd is not None and x.cd.creature else 0), default=None)
    return best if best is not None and pval(g, best) >= 1.5 else None


@on('Hostage Taker', 'leaves')
def _taker_leaves(g, m):
    """the exiled card returns to the battlefield under its owner's control, if it hasn't been cast"""
    h = (m.data or {}).get('hostage')
    if not h: return
    cd, owner = h
    m.data['hostage'] = None
    o = m.owner
    stolen = getattr(o, 'stolen', None) or {}
    if cd in o.hand and stolen.get(id(cd)) is owner and owner.alive:
        o.hand.remove(cd); stolen.pop(id(cd), None); o.agent_ids.discard(id(cd))
        log(f'    {cd.name} returns to the battlefield under {NAME(owner)}\'s control (Hostage Taker left)', g)
        enter(g, owner, cd)


card('Hostage Taker', 'human pow=2 tgh=3', dsl=[])
full('Hostage Taker', 'enters: exiles the best other creature or artifact until it leaves; you may cast that card with '
     'mana of any type (if you haven\'t when the Taker leaves, it returns to the battlefield)')


# ================================================================== Jester's Cap
@on("Jester's Cap", 'options')
def _cap(g, src, p, s, post):
    """{2}, {T}, sacrifice: search target player's library for three cards and exile them (main 2, or an opponent's
    end step: the opponent whose best three cards matter most)"""
    if p is not src.owner or human(g, p) is not None or src.tapped or not can_pay(g, p, 2, ''): return []
    if post is False or (post is None and g.active is p): return []
    q, picks, v = cap_plan(g, p)
    if q is None or v < 6: return []

    def go():
        if src not in p.perms or src.tapped or not can_pay(g, p, 2, '') or not q.alive: return False
        pay(g, p, 2, '')
        cap_use(g, p, src, q)
        return True
    return [(1.0 + v / 10.0, f"Jester's Cap on {NAME(q)}", go)]


def cap_picks(g, p, q):
    """the three cards of q's library p exiles: combo pieces, wished cards and bombs first"""
    from commander_sim.cards.impl import combos
    from commander_sim.ai import pool_ai
    wish = set(pool_ai.wish_list(g, q)) if q.key in E.SEATS else set()
    seen, out = set(), []
    def worth(c):
        return (8 if c.name in combos.PIECES else 0) + (6 if c.name in wish else 0) + 2 * c.bomb + \
            (card_worth(g, q, c) / 10.0 if not c.land else 0)
    for c in sorted(q.library, key=lambda c: -worth(c)):
        if c.name in seen: continue
        seen.add(c.name); out.append(c)
        if len(out) == 3: break
    return out, sum(worth(c) for c in out)


def cap_plan(g, p):
    best = (None, [], 0.0)
    for q in g.opps(p):
        if not q.library: continue
        picks, v = cap_picks(g, p, q)
        v *= 1.0 + 0.05 * max(0, threat(g, p, q))
        if v > best[2]: best = (q, picks, v)
    return best


def cap_use(g, p, src, q):
    log(f"  {NAME(p)} activates Jester's Cap on {NAME(q)}", g)
    src.tapped = True
    die(g, src, 'sac')
    if not ability_window(g, p, src.cd, f"search {NAME(q)}'s library for three cards and exile them", imp=5): return
    hc = human(g, p)
    if hc is not None:
        picks = []
        for i in range(3):
            pool = sorted({c.name: c for c in q.library if c not in picks}.values(), key=lambda c: c.name)
            if not pool: break
            k = hc.choose(g, p, 'search', f"Jester's Cap: exile which card from {NAME(q)}'s library? ({i + 1} of 3)",
                          [c.name for c in pool], cancel='stop')
            if k is None: break
            picks.append(pool[k])
    else:
        picks = cap_picks(g, p, q)[0]
    for c in picks:
        if c in q.library: q.library.remove(c); q.exile.append(c)
    g.rng.shuffle(q.library)
    log(f"    Jester's Cap exiles {', '.join(c.name for c in picks) or 'nothing'} from {NAME(q)}'s library", g)


card("Jester's Cap", '', types='A', dsl=[])
full("Jester's Cap", "{2}, {T}, sacrifice: exile three cards from a player's library (the AI: the most threatening "
     "opponent's combo pieces and best cards)")


# ================================================================== tutors
@on('Dark Petition', 'resolve')
def _petition(g, p, c, ctx):
    """search for any card; with two or more instants and sorceries in your graveyard, add {B}{B}{B}"""
    tutor(g, p, 'any')
    if sum(1 for x in p.gy if x.instant or x.sorcery) >= 2:
        if human(g, p) is not None: importlib.import_module('commander_sim.play.mana').pool_of(p).add('B', 3)
        else: p.floatB = getattr(p, 'floatB', 0) + 3
        log('    spell mastery: Dark Petition adds {B}{B}{B}', g)
    return 'gy'


card('Dark Petition', 'tut=any', types='S', dsl=[])
full('Dark Petition', 'search for any card; spell mastery (two or more instants and sorceries in your graveyard) adds '
     '{B}{B}{B}')


@on('Plea for Guidance', 'resolve')
def _plea(g, p, c, ctx):
    hc = human(g, p)
    if hc is not None:
        from commander_sim import ais
        for x in hc.search(g, p, ais.TUTOR_OK['ench'], 2, 'Plea for Guidance: search for up to two enchantment cards'):
            p.hand.append(x); p.stats['tutored'] += 1; p.seen_names.add(x.name)
        return 'gy'
    tutor(g, p, 'ench'); tutor(g, p, 'ench')
    return 'gy'


card('Plea for Guidance', 'tut=ench', types='S', dsl=[])
full('Plea for Guidance', 'search for up to two enchantment cards (the AI: its two best for the board)')


def tutor_pick(g, p, kind, okn):
    """the card a tutor finds: Sanguine Bond to start the drain engine, a finisher once it kills, Necropotence early,
    a lock or pillowfort piece when under pressure"""
    def first(ns):
        return next((n for n in ns if n in okn), None)
    have = {m.cd.name for m in p.perms if m.cd is not None}
    hand = {c.name for c in p.hand}
    opps = g.opps(p)
    mana = len(p.lands) + sum(1 for m in p.perms if m.cd is not None and 'rock' in m.cd.tags)
    order = []
    if kind == 'any':
        low = sum(q.life for q in opps)
        if mana >= 7 and opps and (low <= 2 * (mana - 2) * len(opps) or min(q.life for q in opps) <= mana - 2):
            order += ['Debt to the Deathless', 'Exsanguinate']
        if ysh_perm(p) is None and p.cmd_in_zone is False and p.tax >= 4: order += ['Lightning Greaves']
    if 'Sanguine Bond' not in have | hand: order.append('Sanguine Bond')
    if p.turns <= 6 and p.life >= 25 and 'Necropotence' not in have | hand: order.append('Necropotence')
    if under_attack(g, p) and not have & {'Propaganda', 'Windborn Muse'}: order.append('Propaganda')
    if 'Marauding Blight-Priest' not in have | hand and kind == 'any': order.append('Marauding Blight-Priest')
    if ysh_perm(p) is not None and 'Curiosity' not in have | hand: order.append('Curiosity')
    order += ['Ill-Gotten Inheritance', 'Prison Sentence', 'Arrest', 'Propaganda']
    if kind == 'any': order += ['Exsanguinate', 'Debt to the Deathless']
    return first(order)


def under_attack(g, p):
    """the table's creatures hit hard enough to matter (total power at 12 or more)"""
    return sum(epow(g, m) for q in g.opps(p) for m in q.perms if m.creature and not m.phased) >= 12


CI.yshtola_tutor = tutor_pick


# ================================================================== Take Up the Shield
@on('Take Up the Shield', 'resolve')
def _shield(g, p, c, ctx):
    t = ctx.get('target')
    if t is not None and t in t.owner.perms: shield(g, t)
    return 'gy'


def shield(g, m):
    CI.add_counters(g, m, 1)
    s = g.eot_kw.setdefault(id(m), set())
    s.add('indestructible'); s.add('lifelink')
    log(f'    Take Up the Shield: {m.name} gets a +1/+1 counter, lifelink and indestructible', g)


card('Take Up the Shield', 'prot=shield', types='I', dsl=[])
full('Take Up the Shield', 'a +1/+1 counter, lifelink and indestructible until end of turn: the AI casts it to save '
     'Y\'shtola or another key creature from destruction or damage')


# ================================================================== Champion's Helm (the AI equips it on Y'shtola)
@on("Champion's Helm", 'options')
def _helm(g, src, p, s, post):
    if p is not src.owner or p.key != 'yshtola' or human(g, p) is not None or post is None or not can_pay(g, p, 1, ''): return []
    from commander_sim import ais
    t = ais.helm_target(g, p) if any(m.creature for m in p.perms) else None
    if t is None or (src.attached is t): return []
    return [(2.5 + 0.3 * pval(g, t), f"equip Champion's Helm to {t.name}", lambda: ais.helm_equip(g, p, t))]


# ================================================================== Zur in the 99: what to fetch
def fetch_value(g, p, c, zur):
    """an enchantment with mana value 3 or less from Zur's attack, for this deck"""
    n = c.name
    Z = _zur()
    have = {m.cd.name for m in p.perms if m.cd is not None and not m.phased}
    if n in have and n not in IC.AURA and n not in Z.LOCKS: return 0
    turn = p.turns
    if n == 'Necropotence': return (8.5 - 0.3 * turn) if p.life >= 20 else 1.0
    if n == 'Mystic Remora': return max(0.0, 5.0 - 1.2 * (turn - 1))
    if n == 'Propaganda': return 2.0 + (5.0 if under_attack(g, p) else 0) + 0.15 * sum(
        1 for q in g.opps(p) for m in q.perms if m.creature and not m.phased)
    if n == 'Curiosity':
        y = ysh_perm(p)
        on_y = y is not None and getattr(g, 'auras', None) and any(a.cd.name == n for a in IC.auras_on(g, y))
        return 3.0 + 1.5 * len(g.opps(p)) if y is not None and not on_y else 1.5
    if n in Z.LOCKS:
        kind = next(k for k, v in Z.LOCK_KINDS.items() if frozenset(v) == Z.LOCKS[n])
        t = Z.lock_host(g, p, n, targeted=False)
        return Z.lock_worth(g, p, t, kind) if t is not None else 0
    return 0.5


CI.yshtola_fetch_value = fetch_value


# ================================================================== the AI: priorities
def trigger_bonus(g, p, c):
    """what Y'shtola's cast trigger adds to casting c now (2 to each opponent, 2 life, and what that sets off)"""
    if c.creature or c.land or c.cmc < 3 or ysh_perm(p) is None: return 0
    opps = g.opps(p)
    per = 2 + drain_extra(g, p)
    cur = sum(1 for a in (IC.auras_on(g, ysh_perm(p)) if getattr(g, 'auras', None) else []) if a.cd.name == 'Curiosity')
    v = 0.5 * len(opps) * per + (1.0 if bond_on(p) else 0) + 1.5 * cur * len(opps)
    if any(q.life <= per for q in opps): v += 6
    return min(20, int(4 * v))


def yshtola_prio(g, p, c):
    t, n = c.tags, c.name
    ysh = ysh_perm(p)
    stolen = getattr(p, 'stolen', None) or {}
    if id(c) in stolen:                                              # a card taken from an opponent
        from commander_sim.ai import pool_ai
        v = pool_ai.generic_prio(g, p, c)
        if not v and c.dsl and E.DSLMOD is not None: v = E.DSLMOD.card_value(g, p, c) * 10
        return v
    if c is p.cmd: return 90 if p.turns >= 2 else 60
    b = trigger_bonus(g, p, c)
    if 'rock' in t: return 82 if p.turns <= 5 else 30 + b
    if n == 'The One Ring': return 66 + b                             # draw engine, a turn of protection, and it triggers her
    if 'necro' in t: return 74 if p.life >= 20 else 20
    if 'remora' in t: return 66 if p.turns <= 4 else 8
    if n == 'Esper Sentinel': return 62 if p.turns <= 4 else 30
    if 'xdrain' in t or 'tokx' in t: return 0                         # X spells: their own options
    if n == 'Sanguine Bond': return 76 + b
    if n == 'Marauding Blight-Priest': return 62
    if n == 'Ill-Gotten Inheritance': return 54 + b
    if n in ('Propaganda', 'Windborn Muse'):
        if any(m.cd is not None and m.cd.name in ('Propaganda', 'Windborn Muse') for m in p.perms):
            return 30 + b
        return min(72, 44 + (18 if under_attack(g, p) else 0)) + b
    Z = _zur()
    if n in Z.LOCKS:
        kind = next(k for k, v in Z.LOCK_KINDS.items() if frozenset(v) == Z.LOCKS[n])
        tg = Z.lock_host(g, p, n, targeted=True)
        return min(80, int(42 + 7 * Z.lock_worth(g, p, tg, kind))) + b if tg is not None else 0
    if n == 'Curiosity':
        return 50 if ysh is not None and not Z.untargetable_by_you(g, ysh) else 18
    if n == 'Enslave':
        v = E.CI.marchesa_steal_value(g, p)
        return min(80, int(50 + 3 * v)) + b if v >= 4 else 0
    if n == 'Bribery': return 56 + b
    if n == 'Hostage Taker':
        tg = taker_target(g, p, None) if human(g, p) is None else None
        return 45 + min(30, int(6 * pval(g, tg))) if tg is not None else 25
    if n == 'Massacre Wurm':
        k = sum(1 for q in g.opps(p) for m in q.perms if m.creature and not m.phased and etgh(g, m) <= 2)
        return min(80, 46 + 6 * k)
    if n == 'Gonti, Lord of Luxury': return 50
    if n == 'Thief of Sanity': return 52
    if n == 'Urborg Syphon-Mage': return 46
    if n == 'Zur the Enchanter': return 54
    if n == "Champion's Helm": return (46 if ysh is not None else 22) + b
    if t.get('prot') == 'boots': return 58 if ysh is not None else 25
    if n == 'Bastion Protector': return 50 if ysh is not None else 30
    if n == 'Notion Thief': return 50
    if n == 'Skyclave Apparition':
        tg = E.legal_targets(g, p, 'exile', 'nl', True, spell=c)
        return 45 + min(30, int(6 * max((pval(g, m) for m in tg), default=0))) if tg else 25
    if n == 'Restoration Angel': return 25                            # held for flash
    if n == 'The Eternal Wanderer': return 58 + b
    if n in ('Prayer of Binding', 'Static Net', 'Memory Trap'):      # exile an opponent's best nonland permanent
        best = max((pval(g, m) for q in g.opps(p) for m in q.perms if not m.phased and not untargetable(g, m)), default=0)
        return min(75, int(40 + 6 * best)) + b if best >= 3 else 0
    if n == "Jester's Cap": return 34 + b
    if t.get('tut'): return 52 + b
    if n == 'Triplicate Spirits': return 46 + b
    if 'draw' in t and (c.instant or c.sorcery): return 46 + b
    if c.creature: return 40
    return 0


CI.yshtola_prio = yshtola_prio


# ------------------------------------------------------------------ the rigid AI's extra plays (ais.yshtola_main)
def yshtola_x_spell(g, p, post):
    """a drain X spell worth casting now"""
    opts = [o for c in list(p.hand) if 'xdrain' in c.tags for o in [x_drain_option(g, p, c)] if o]
    if not opts: return False
    u, lbl, fn = max(opts, key=lambda o: o[0])
    return u >= 4.0 and fn()


def yshtola_abilities(g, p, post):
    """the best activated ability with a clear use (Syphon-Mage, Ill-Gotten Inheritance, Jester's Cap, the Helm)"""
    from commander_sim.ai import brain
    s = brain.Situation(g, p)
    opts = []
    for src, fn in CI.hooked(g, 'options'):
        if src.owner is p: opts += fn(g, src, p, s, post) or []
    opts = [o for o in opts if o[0] >= 2.5]
    if not opts: return False
    return max(opts, key=lambda o: o[0])[2]()


CI.yshtola_x_spell = yshtola_x_spell
CI.yshtola_abilities = yshtola_abilities


# ------------------------------------------------------------------ protection and wipes
def yshtola_protect(g, owner, m, kind, actor, spell):
    """removal at Y'shtola or another key creature: Take Up the Shield against destroy and damage, then the Zur deck's
    answers (Clever Concealment, Rootborn Defenses, Momentary Blink, Restoration Angel)"""
    from commander_sim import ais
    if not m.creature or pval(g, m) < 4: return False
    if kind == 'destroy' or kind.startswith('dmg'):
        for c in list(owner.hand):
            if c.tags.get('prot') == 'shield' and can_pay(g, owner, c.generic, c.pips) and not untargetable_by_you(g, m):
                if not ais.pay_card(g, owner, c): return False
                log(f'    {NAME(owner)} casts Take Up the Shield on {m.name}', g)
                shield(g, m)
                return True
    return _zur().zur_protect(g, owner, m, kind, actor, spell)


def untargetable_by_you(g, m):
    return _zur().untargetable_by_you(g, m)


def yshtola_wipe_response(g, q, kind):
    return _zur().zur_wipe_response(g, q, kind)


CI.yshtola_protect = yshtola_protect
CI.yshtola_wipe_response = yshtola_wipe_response
note('Statute of Denial', 'Full', 'counter target spell; if you control a blue creature, draw a card, then discard a card')
