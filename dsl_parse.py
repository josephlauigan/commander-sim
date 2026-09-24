"""Card ability language (DSL) -- part 1: compiling Oracle text into ability data.

A card is a list of abilities; every ability is plain JSON:

  {"type": "spell",     "effects": [...]}                                   instants / sorceries
  {"type": "triggered", "event": "etb", "source": "self", "effects": [...]}  "When ~ enters, ..."
  {"type": "activated", "cost": {"mana": "2B", "tap": true, "sac": "self"}, "effects": [...]}
  {"type": "loyalty",   "loyalty": -2, "effects": [...]}                      planeswalkers
  {"type": "static",    "static": "anthem", "filter": {...}, "pow": 1, "tgh": 1}
  {"type": "replacement", "replace": "tokens", "multiplier": 2}

Effects are small building blocks:
  {"do": "draw", "n": 2, "who": "you"}
  {"do": "damage", "n": 3, "to": {"sel": "target", "filter": {"type": "creature", "controller": "opp"}}}
  {"do": "destroy" | "exile" | "bounce" | "tuck", "what": <selector>}
  {"do": "token", "n": 1, "pow": 2, "tgh": 2, "keywords": ["flying"], "attacking": false}
  {"do": "counters", "n": 1, "what": <selector>}     {"do": "pump", "pow": 2, "tgh": 0, "what": <selector>}
  {"do": "search", "filter": {...}, "to": "hand" | "battlefield" | "graveyard" | "top", "tapped": true}
  {"do": "reanimate", "filter": {...}, "from": "yours" | "any"}   {"do": "regrow", "filter": {...}}
  {"do": "gain_life" | "lose_life", "n": 3, "who": "you" | "each_opponent" | "target_player" ...}
  {"do": "discard" | "mill", "n": 1, "who": ...}     {"do": "sacrifice", "who": "each_opponent", "filter": {...}}
  {"do": "counter_spell", "filter": {...}, "unless": 2}   {"do": "grant", "keyword": "indestructible", "what": ...}
  {"do": "treasure" | "clue", "n": 1}   {"do": "add_mana", "n": 3}   {"do": "extra_combat"}   {"do": "proliferate"}
  {"do": "amass", "n": 1}   {"do": "untap", "what": ...}   {"do": "phase_out", "what": ...}   {"do": "blink", "what": ...}
  {"do": "modal", "choose": 1, "modes": [[effects], [effects], ...]}
Selectors: {"sel": "target" | "all" | "self" | "each", "filter": {"type", "controller", "other", "max_mv", "nontoken", ...}}
Numbers may be ints or names: "X", "creatures_you_control", "cards_in_hand", "devotion_B", "opponents", ...
"""
import re

NUMW = {'a': 1, 'an': 1, 'one': 1, 'two': 2, 'three': 3, 'four': 4, 'five': 5, 'six': 6, 'seven': 7,
        'eight': 8, 'nine': 9, 'ten': 10, 'x': 'X', 'that many': 'X', 'twice': 2}
KW = ('flying', 'trample', 'haste', 'vigilance', 'lifelink', 'deathtouch', 'hexproof', 'indestructible',
      'menace', 'reach', 'first strike', 'double strike', 'shroud')


def n_(s):
    s = (s or '').strip().lower()
    if s.isdigit(): return int(s)
    return NUMW.get(s, 1)


def mana_str(s):
    gen = 0; pips = ''
    for sym in re.findall(r'\{([^}]+)\}', s):
        sym = sym.upper()
        if sym.isdigit(): gen += int(sym)
        elif sym in ('X', 'T', 'Q', 'E'): continue
        elif sym == 'C': gen += 1
        else:
            c = sym.split('/')[0]
            if c in 'WUBRG': pips += c
    return (str(gen) if gen else '') + pips


# ------------------------------------------------------------------ filters and selectors
TYPE_WORDS = [('nonland permanent', 'nonland'), ('artifact, creature, or planeswalker', 'acp'),
              ('artifact or creature', 'ac'), ('creature or planeswalker', 'cp'), ('creature or enchantment', 'ce'),
              ('artifact or enchantment', 'ae'), ('nonartifact creature', 'nonartifact_creature'),
              ('noncreature', 'noncreature'), ('creature', 'creature'), ('planeswalker', 'planeswalker'),
              ('artifact', 'artifact'), ('enchantment', 'enchantment'), ('land', 'land'), ('permanent', 'permanent'),
              ('spell', 'spell'), ('player', 'player'), ('opponent', 'opponent')]


def filt(phrase):
    p = phrase.lower()
    f = {}
    for w, t in TYPE_WORDS:
        if w in p: f['type'] = t; break
    if "you don't control" in p or 'an opponent controls' in p or 'your opponents control' in p or 'opponents control' in p:
        f['controller'] = 'opp'
    elif 'you control' in p or 'you own' in p or 'your' in p.split()[:1]: f['controller'] = 'you'
    if 'another' in p or 'other' in p.split(): f['other'] = True
    m = re.search(r'mana value (\d+) or less', p)
    if m: f['max_mv'] = int(m.group(1))
    m = re.search(r'mana value (\d+) or greater', p)
    if m: f['min_mv'] = int(m.group(1))
    m = re.search(r'power (\d+) or less', p)
    if m: f['max_pow'] = int(m.group(1))
    m = re.search(r'power (\d+) or greater', p)
    if m: f['min_pow'] = int(m.group(1))
    if 'nontoken' in p: f['nontoken'] = True
    if 'nonlegendary' in p: f['nonlegendary'] = True
    if 'legendary' in p and 'nonlegendary' not in p: f['legendary'] = True
    if 'tapped' in p: f['tapped'] = True
    for kw in ('flying',):
        if 'with flying' in p: f['kw'] = 'flying'
    m = re.search(r'(instant or sorcery|instant|sorcery)', p)
    if m and 'type' not in f: f['type'] = 'instant_or_sorcery'
    return f


def selector(phrase):
    p = phrase.strip().lower()
    if p in ('~', 'it', 'this creature', 'this permanent'): return {'sel': 'self'}
    if p.startswith('each ') or p.startswith('all '): return {'sel': 'all', 'filter': filt(p)}
    if 'target' in p: return {'sel': 'target', 'filter': filt(p), 'upto': 'up to' in p}
    return {'sel': 'all', 'filter': filt(p)}


def who(phrase):
    p = (phrase or 'you').strip().lower()
    for k, v in (('each other player', 'each_opponent'), ('each opponent', 'each_opponent'), ('target opponent', 'target_opponent'),
                 ('target player', 'target_player'), ('each player', 'each_player'), ('its controller', 'target_controller'),
                 ('that player', 'event_player'), ('defending player', 'defending_player'), ('you', 'you')):
        if p.startswith(k) or p == k: return v
    return 'you'


# ------------------------------------------------------------------ effect clauses
def num_expr(s):
    s = s.strip().lower()
    if s in NUMW or s.isdigit(): return n_(s)
    if 'creature' in s and 'you control' in s: return 'creatures_you_control'
    tm = re.match(r'^(?:the number of )?(elves|goblins|zombies|humans|ninjas|soldiers|warriors|knights|vampires|spirits|faeries)'
                  r' (you control|on the battlefield)$', s)
    if tm: return ('type_you:' if tm.group(2) == 'you control' else 'type_all:') + singular(tm.group(1))
    if 'card' in s and 'hand' in s: return 'cards_in_hand'
    if 'devotion to black' in s: return 'devotion_B'
    if 'opponent' in s: return 'opponents'
    if 'its power' in s: return 'target_power'
    if 'its toughness' in s: return 'target_toughness'
    if 'its mana value' in s: return 'target_mv'
    return 'X'


EFFECT_RULES = []
CTX = {}          # the full line being parsed (some clauses need context from neighbours)


def rule(pattern):
    def deco(fn):
        EFFECT_RULES.append((re.compile(pattern), fn)); return fn
    return deco


@rule(r"^(?:you )?draws? (a|an|one|two|three|four|five|six|seven|x|\d+) (?:additional )?cards?$")
def _draw(m): return [{'do': 'draw', 'n': n_(m.group(1)), 'who': 'you'}]


@rule(r"^(target player|each player|each opponent|target opponent|its controller|that player) draws? (a|one|two|three|x|\d+) cards?$")
def _draw2(m): return [{'do': 'draw', 'n': n_(m.group(2)), 'who': who(m.group(1))}]


@rule(r"^(?:you )?draw a card for each (.+)$")
def _draw3(m): return [{'do': 'draw', 'n': num_expr(m.group(1)), 'who': 'you'}]


@rule(r"^(?:you )?draw (a|one|two|three|\d+) cards?(?:,)? (?:and|then) (?:you )?(?:lose|discard) (a|one|two|three|\d+) (life|cards?)$")
def _draw_then(m):
    e = [{'do': 'draw', 'n': n_(m.group(1)), 'who': 'you'}]
    e.append({'do': 'lose_life', 'n': n_(m.group(2)), 'who': 'you'} if m.group(3) == 'life'
             else {'do': 'discard', 'n': n_(m.group(2)), 'who': 'you'})
    return e


@rule(r"^(?:~ )?deals? (\d+|x) damage to (any target|any other target|target creature or planeswalker|target creature|target player or planeswalker|target player|target opponent|each opponent|each creature your opponents control|each other creature|each creature|each player|each creature and each player)$")
def _damage(m):
    n = n_(m.group(1)); t = m.group(2)
    if t in ('any target', 'any other target', 'target player or planeswalker'): to = {'sel': 'any_target'}
    elif t in ('target player', 'target opponent', 'each opponent', 'each player'): to = {'sel': 'player', 'who': who(t)}
    elif t == 'each creature and each player':
        return [{'do': 'damage', 'n': n, 'to': {'sel': 'all', 'filter': {'type': 'creature'}}},
                {'do': 'damage', 'n': n, 'to': {'sel': 'player', 'who': 'each_player'}}]
    else: to = selector(t)
    return [{'do': 'damage', 'n': n, 'to': to}]


@rule(r"^(destroy|exile) (target|all|each|up to one target|another target|up to one other target|up to (?:one|two|three) target) ([^.]+?)(?:\. (?:they|it) can't be regenerated)?$")
def _destroy(m):
    what = selector(m.group(2) + ' ' + m.group(3))
    if 'card' in m.group(3) and 'graveyard' in m.group(3):
        return [{'do': 'exile_graveyard', 'who': 'target_player' if 'target' in m.group(3) else 'each_player'}]
    if 'graveyard' in m.group(3): return [{'do': 'exile_graveyard', 'who': 'each_player'}]
    return [{'do': m.group(1), 'what': what}]


@rule(r"^return (target|all|each|up to one target|another target) ([^.]+?) to (?:its|their) owners?'s? hands?$")
def _bounce(m):
    ph = m.group(2)
    if 'card' in ph and 'graveyard' in ph:
        return [{'do': 'regrow', 'filter': filt(ph.split('card')[0]), 'from': 'yours'}]
    return [{'do': 'bounce', 'what': selector(m.group(1) + ' ' + ph)}]


@rule(r"^(?:return|put) (?:target |up to one target |another target )?([^.]*?)cards? from (your|a|an opponent's|target player's) graveyard (?:to|onto) the battlefield(?: under your control)?(?: tapped)?(?: with (?:a|two|three) (?:additional )?\+1/\+1 counters? on it)?$")
def _reanimate(m):
    return [{'do': 'reanimate', 'filter': filt(m.group(1) or 'creature'), 'from': 'yours' if m.group(2) == 'your' else 'any'}]


@rule(r"^return (?:target |up to one target |up to two target )?([^.]*?)cards? from your graveyard to your hand$")
def _regrow(m): return [{'do': 'regrow', 'filter': filt(m.group(1) or 'card'), 'from': 'yours'}]


@rule(r"^search your library for (an?|up to (?:one|two|three|x))\s?([^,]*?)\s?cards?(?: with [^,]*)?(?:,| and)? ?(?:reveal (?:it|them|those cards)(?:,| and)? ?)?(?:put (?:it|that card|them|those cards|one of them|one) (into your hand|onto the battlefield(?: tapped)?|into your graveyard|on top of your library)|then shuffle and put that card on top)?(.*)$")
def _search(m):
    what = m.group(2); rest = m.group(4) or ''
    dest = m.group(3) or ('on top of your library' if 'on top' in (m.group(0) + CTX.get('line', '')) else 'into your hand')
    to = 'hand' if 'hand' in dest else 'battlefield' if 'battlefield' in dest else 'graveyard' if 'graveyard' in dest else 'top'
    n = 2 if 'two' in m.group(1) else 1
    eff = {'do': 'search', 'filter': filt(what) if what not in ('', 'a') else {}, 'to': to, 'n': n,
           'tapped': 'tapped' in dest or 'tapped' in rest}
    if 'basic land' in what or re.search(r'\b(forest|plains|island|swamp|mountain|land)\b', what):
        eff['filter'] = {'type': 'land', 'basic': 'basic' in what}
    out = [eff]
    if 'the other into your hand' in rest:
        eff['n'] = 1; out.append({'do': 'search', 'filter': {'type': 'land', 'basic': True}, 'to': 'hand', 'n': 1})
    return out


@rule(r"^create (a|an|one|two|three|four|five|x|that many|\d+) (\d+|x)/(\d+|x) ([^.]*?)creature tokens?(?: with ([a-z, ]+?))?(?: that'?s? tapped and attacking[^.]*| named [^.]*)?(?:\. they gain [a-z]+ until end of turn)?$")
def _token(m):
    kws = [k for k in KW if k in (m.group(4) + ' ' + (m.group(5) or ''))]
    types = [w for w in m.group(4).split() if w not in ('white', 'blue', 'black', 'red', 'green', 'colorless', 'and',
                                                        'artifact', 'enchantment', 'legendary', 'snow')]
    return [{'do': 'token', 'n': n_(m.group(1)), 'pow': 1 if m.group(2) == 'x' else int(m.group(2)),
             'tgh': 1 if m.group(3) == 'x' else int(m.group(3)), 'keywords': kws,
             'attacking': 'attacking' in m.group(0), 'warrior': 'warrior' in m.group(4), 'types': types}]


@rule(r"^create (a|an|one|two|three|x|that many|\d+) (treasure|clue|food|blood) tokens?$")
def _treasure(m):
    k = m.group(2)
    if k in ('treasure', 'clue'): return [{'do': k, 'n': n_(m.group(1))}]
    return [{'do': 'noop'}]


@rule(r"^(?:you )?gain (\d+|x|that much) life$")
def _gain(m): return [{'do': 'gain_life', 'n': n_(m.group(1)), 'who': 'you'}]


@rule(r"^(each opponent|target opponent|target player|each player|you|that player|its controller) (?:loses|lose) (\d+|x) life(?: and you gain (?:\d+|x|that much) life)?$")
def _lose(m):
    e = [{'do': 'lose_life', 'n': n_(m.group(2)), 'who': who(m.group(1))}]
    if 'you gain' in m.group(0): e.append({'do': 'gain_life', 'n': n_(m.group(2)), 'who': 'you', 'per_opponent': m.group(1) == 'each opponent'})
    return e


@rule(r"^(?:its controller|that player|target player|you) gains? life equal to (?:its|that creature's) (power|toughness)$")
def _gain_pow(m): return [{'do': 'gain_life', 'n': 'target_' + m.group(1), 'who': 'target_controller'}]


@rule(r"^put (a|an|one|two|three|x|\d+) \+1/\+1 counters? on (each creature you control|each other creature you control|target creature|~|target creature you control|each of up to two target creatures|it)$")
def _counters(m): return [{'do': 'counters', 'n': n_(m.group(1)), 'what': selector(m.group(2))}]


@rule(r"^(creatures you control|other creatures you control|target creature|~|target creature you control|each creature you control|up to one target creature) gets? \+(\d+|x)/\+(\d+|x)(?: and gains? ([a-z, ]+?))? until end of turn$")
def _pump(m):
    e = [{'do': 'pump', 'pow': n_(m.group(2)), 'tgh': n_(m.group(3)), 'what': selector(m.group(1))}]
    if m.group(4):
        for k in KW:
            if k in m.group(4): e.append({'do': 'grant', 'keyword': k, 'what': selector(m.group(1))})
    return e


@rule(r"^(creatures you control|target creature|target creature you control|~|each creature you control|permanents you control|other permanents you control) gains? ([a-z, ]+?) until end of turn$")
def _grant(m):
    return [{'do': 'grant', 'keyword': k, 'what': selector(m.group(1))} for k in KW if k in m.group(2)] or None


@rule(r"^counter target ([a-z, ]*?)\s?spell(?: with mana value (\d+) or greater| that targets [^.]*)?(?: unless its controller pays \{(\d+)\})?$")
def _counter(m):
    f = filt(m.group(1) + ' spell'); f['type'] = ('noncreature' if 'noncreature' in m.group(1) else
                                                  'creature' if m.group(1).strip() == 'creature' else
                                                  'instant_or_sorcery' if ('instant' in m.group(1) or 'sorcery' in m.group(1)) else 'any')
    if m.group(2): f['min_mv'] = int(m.group(2))
    e = {'do': 'counter_spell', 'filter': f}
    if m.group(3): e['unless'] = int(m.group(3))
    return [e]


@rule(r"^(each opponent|target opponent|target player|each player|you|that player) (discards?|mills?) (a|one|two|three|x|\d+) cards?$")
def _discard(m): return [{'do': 'discard' if 'discard' in m.group(2) else 'mill', 'n': n_(m.group(3)), 'who': who(m.group(1))}]


@rule(r"^(?:you )?(discard|mill) (a|one|two|three|\d+) cards?$")
def _discard_self(m): return [{'do': m.group(1), 'n': n_(m.group(2)), 'who': 'you'}]


@rule(r"^(each other player|each opponent|target opponent|target player|each player|defending player) sacrifices? (a|an|two) ([a-z ,]+?)(?: of their choice| with the (?:least|greatest) power[^.]*)?$")
def _edict(m): return [{'do': 'sacrifice', 'who': who(m.group(1)), 'filter': filt(m.group(3)), 'n': n_(m.group(2))}]


@rule(r"^(?:scry|surveil) (\d+|x)$|^investigate$|^proliferate$|^you may pay \{\d+\}$|^shuffle$|^then shuffle$|^shuffle and put that card on top$|^reveal (?:it|them)$")
def _misc(m):
    t = m.group(0)
    if t == 'investigate': return [{'do': 'clue', 'n': 1}]
    if t == 'proliferate': return [{'do': 'proliferate'}]
    return [{'do': 'noop'}]


@rule(r"^amass (?:orcs|zombies) (\d+|x)$")
def _amass(m): return [{'do': 'amass', 'n': n_(m.group(1))}]


@rule(r"^untap (all creatures you control|target creature|all lands you control|~|all attacking creatures|up to (?:two|three|four|five) target lands|target permanent)$")
def _untap(m): return [{'do': 'untap', 'what': selector(m.group(1))}]


@rule(r"^add ((?:\{[wubrgc]\})+|one mana of any color|two mana of any one color|three mana of any one color|x mana of any one color)$")
def _add(m):
    s = m.group(1)
    n = len(re.findall(r'\{', s)) if '{' in s else n_(s.split()[0])
    return [{'do': 'add_mana', 'n': n}]


@rule(r"^(any number of target nonland permanents you control|target creature you control|target permanent you control|target creature|permanents you control) phases? out$")
def _phase(m): return [{'do': 'phase_out', 'what': selector(m.group(1))}]


@rule(r"^exile (target creature you control|another target creature you control|up to one target creature you control|target nonland permanent you control)(?:,)? then return (?:it|that card) to the battlefield under (?:its owner's|your) control$")
def _blink(m): return [{'do': 'blink', 'what': selector(m.group(1))}]


@rule(r"^blink (up to one target creature you (?:control|own)|target creature you (?:control|own)|another target creature you (?:control|own))$")
def _blink2(m): return [{'do': 'blink', 'what': selector(m.group(1).replace('you own', 'you control'))}]


@rule(r"^after this (?:main )?phase, there is an additional combat phase(?: followed by an additional main phase)?$|^there is an additional combat phase after this phase$")
def _xcombat(m): return [{'do': 'extra_combat'}]


@rule(r"^the owner of target ([a-z ]+) shuffles it into their library$")
def _tuck(m): return [{'do': 'tuck', 'what': {'sel': 'target', 'filter': filt(m.group(1))}}]


@rule(r"^tap (target creature|up to one target creature|target permanent|all creatures your opponents control)$")
def _tap(m): return [{'do': 'tap', 'what': selector(m.group(1))}]


@rule(r"^(?:its controller|that player) may search their library for a basic land card, put it onto the battlefield tapped$")
def _path(m): return [{'do': 'opp_ramp'}]


@rule(r"^(?:its controller|that player) creates? a (\d+)/(\d+) [a-z ]+ creature token$")
def _pong(m): return [{'do': 'opp_token', 'pow': int(m.group(1))}]


@rule(r"^(?:you )?lose (\d+|x) life$")
def _selflose(m): return [{'do': 'lose_life', 'n': n_(m.group(1)), 'who': 'you'}]


@rule(r"^all creatures get -(\d+|x)/-(\d+|x) until end of turn$")
def _minus(m): return [{'do': 'pump', 'pow': -9 if m.group(1) == 'x' else -int(m.group(1)),
                        'tgh': -9 if m.group(2) == 'x' else -int(m.group(2)), 'what': {'sel': 'all', 'filter': {'type': 'creature'}}}]


@rule(r"^creatures your opponents control get -(\d+)/-(\d+) until end of turn$")
def _oppminus(m): return [{'do': 'pump', 'pow': -int(m.group(1)), 'tgh': -int(m.group(2)),
                           'what': {'sel': 'all', 'filter': {'type': 'creature', 'controller': 'opp'}}}]


@rule(r"^(?:its controller|that player) may search their library for a basic land card, put it onto the battlefield(?: tapped)?$")
def _path2(m): return [{'do': 'opp_ramp'}]


@rule(r"^(?:its controller|that player) creates? (a|two|three) treasure tokens?$")
def _opptreas(m): return [{'do': 'opp_treasure', 'n': n_(m.group(1))}]


@rule(r"^(?:its controller|that player) (?:may )?draws? (?:up to )?(a|one|two|three) cards?$")
def _oppdraw(m): return [{'do': 'draw', 'n': n_(m.group(1)), 'who': 'target_controller'}]


@rule(r"^(?:you )?gain life equal to the life lost this way$")
def _gainlost(m): return [{'do': 'gain_life', 'n': 'last_lost', 'who': 'you'}]


@rule(r"^(?:you )?gain (\d+|x) life for each (.+)$")
def _gainfor(m): return [{'do': 'gain_life', 'n': num_expr(m.group(2)), 'who': 'you', 'mult': n_(m.group(1))}]


@rule(r"^(?:you )?lose life equal to (?:its|that permanent's|that creature's|the sacrificed creature's) (mana value|power)$")
def _loseeq(m): return [{'do': 'lose_life', 'n': 'target_mv' if 'mana' in m.group(1) else 'target_power', 'who': 'you'}]


@rule(r"^~ deals damage equal to (?:that spell's mana value|its power|the number of [^.]+) to (any target|each opponent|target creature|target opponent)$")
def _dmgeq(m):
    n = 'event_spell_mv' if 'spell' in m.group(0) else 'source_power' if 'its power' in m.group(0) else 'X'
    to = {'sel': 'any_target'} if m.group(1) == 'any target' else {'sel': 'player', 'who': who(m.group(1))} if 'opponent' in m.group(1) else selector(m.group(1))
    return [{'do': 'damage', 'n': n, 'to': to}]


@rule(r"^(?:until end of turn, )?creatures you control have base power and toughness x/x(?: and gain all creature types)?(?: until end of turn)?$")
def _setpt(m): return [{'do': 'set_pt', 'n': 'X'}]


@rule(r"^double the number of each kind of counter on (?:any number of target permanents|each permanent you control|target permanent)$")
def _dbl(m): return [{'do': 'double_counters'}]


@rule(r"^(?:they|those creatures|it) gains? ([a-z, ]+?) until end of turn$")
def _theygain(m):
    return [{'do': 'grant', 'keyword': k, 'what': {'sel': 'all', 'filter': {'type': 'creature', 'controller': 'you'}}}
            for k in KW if k in m.group(1)] or None


@rule(r"^untap all attacking creatures$")
def _untapatk(m): return [{'do': 'untap', 'what': {'sel': 'all', 'filter': {'type': 'creature', 'controller': 'you'}}}]


@rule(r"^(each player|each opponent|target player) discards? (?:their|his or her) hand(?:,)? (?:then|and) draws? (?:seven|that many|\w+) cards?$")
def _wheel(m): return [{'do': 'wheel', 'who': who(m.group(1))}]


@rule(r"^if a creature is destroyed this way, you gain life equal to its toughness$")
def _gearhulk(m): return [{'do': 'gain_life', 'n': 'target_toughness', 'who': 'you'}]


@rule(r"^exile the top (card|two cards|three cards) of your library$")
def _impulse(m): return [{'do': 'draw', 'n': {'card': 1, 'two cards': 2, 'three cards': 3}[m.group(1)], 'who': 'you'}]


@rule(r"^(?:you may |until (?:the )?end of (?:your next )?turn, you may )?(?:cast|play) (?:that card|those cards|it|them|the exiled cards?)(?: this turn| until the end of your next turn)?$")
def _impulse2(m): return [{'do': 'noop'}]


@rule(r"^(?:you may )?put (?:a|up to one) land card from your hand onto the battlefield(?: tapped)?$")
def _putland(m): return [{'do': 'put_land'}]


@rule(r"^if you don't, ~ deals (\d+) damage to each opponent$")
def _ifnot(m): return [{'do': 'noop'}]


def split_clauses(sentence):
    s = sentence.strip().strip('.').strip()
    parts = re.split(r'(?:,? then |\. |; )', s)
    out = []
    for p in parts:
        p = p.strip().strip(',').strip()
        if p.startswith('you may '): p = p[8:]
        if p.startswith('then '): p = p[5:]
        if p: out.append(p)
    return out


SUBJ = r'^(each other player|target opponent|each opponent|target player|each player|that player|its controller|defending player) '


def parse_effects(text):
    """returns (effects, unparsed_clauses)"""
    effs, bad = [], []
    text = re.sub(r" at the beginning of the next turn's upkeep", '', text)
    text = re.sub(r"exile ((?:up to one |another )?target creature you (?:control|own)),? then return (?:it|that card) to the battlefield under (?:its owner's|your) control",
                  r"blink \1", text)
    text = re.sub(r"exile ((?:up to one |another )?target creature you (?:control|own)),? then search your library for (an?|up to one) (basic land|land) card\.? put both cards onto the battlefield(?: tapped)?(?: under your control)?",
                  r"blink \1. search your library for a \3 card, put it onto the battlefield", text)
    xm = re.search(r',? where x is (.+?)(?=\.|$)', text)
    xexpr = num_expr(xm.group(1)) if xm else None
    if xm: text = text[:xm.start()] + text[xm.end():]
    # "target opponent sacrifices a creature, discards a card, and loses 3 life" -> one subject, several verbs
    sents = text.split('. ')
    for k, sen in enumerate(sents):
        sm = re.match(SUBJ + r'(.+)$', sen)
        if sm and ', ' in sm.group(2):
            verbs = [v.strip() for v in re.split(r',? and |, ', sm.group(2)) if v.strip()]
            sents[k] = '. '.join(sm.group(1) + ' ' + v for v in verbs)
    text = '. '.join(sents)
    per_opp = False
    if text.startswith('for each opponent, '):
        text = text[len('for each opponent, '):]; per_opp = True
    for cl in split_clauses(text):
        got = None
        # "X and Y" when both halves parse on their own
        for rx, fn in EFFECT_RULES:
            m = rx.match(cl)
            if m:
                got = fn(m)
                if got: break
        if not got and ' and ' in cl:
            a, b = cl.split(' and ', 1)
            ea, ba = parse_effects(a); eb, bb = parse_effects(b)
            if ea and eb and not ba and not bb: got = ea + eb
        if got:
            for e in got:
                if xexpr is not None and e.get('n') == 'X': e['n'] = xexpr
                if per_opp and e.get('do') == 'token': e['n'] = 'opponents'
            effs += [e for e in got if e.get('do') != 'noop']
        elif re.match(r'^(it|they) (?:can\'t be regenerated|gains? haste until end of turn)$', cl): pass
        else: bad.append(cl)
    return effs, bad


# ------------------------------------------------------------------ triggers, costs, statics
TRIGGERS = [
    (r"^~ enters(?: the battlefield)?(?: or attacks)?$", lambda m, t: [{'event': 'etb', 'source': 'self'}] +
     ([{'event': 'attack', 'source': 'self'}] if 'attacks' in t else [])),
    (r"^~ attacks$", lambda m, t: [{'event': 'attack', 'source': 'self'}]),
    (r"^~ dies$", lambda m, t: [{'event': 'dies', 'source': 'self'}]),
    (r"^~ or another creature (you control )?dies$", lambda m, t: [{'event': 'dies', 'source': 'you_creature' if m.group(1) else 'any_creature'}]),
    (r"^(a|another|one or more other|one or more) (nontoken )?creatures? you control (?:enters?|enter)$", lambda m, t: [{'event': 'etb', 'source': 'you_creature', 'other': 'other' in m.group(1) or 'another' in m.group(1)}]),
    (r"^(a|another) creature (enters)$", lambda m, t: [{'event': 'etb', 'source': 'any_creature', 'other': m.group(1) == 'another'}]),
    (r"^a creature an opponent controls enters$|^a creature enters under an opponent's control$", lambda m, t: [{'event': 'etb', 'source': 'opp_creature'}]),
    (r"^a land (?:you control )?enters(?: under your control)?$", lambda m, t: [{'event': 'landfall'}]),
    (r"^(a|another) creature (you control )?dies$", lambda m, t: [{'event': 'dies', 'source': 'you_creature' if m.group(2) else 'any_creature', 'other': m.group(1) == 'another'}]),
    (r"^a creature an opponent controls dies$|^a creature your opponents control dies$", lambda m, t: [{'event': 'dies', 'source': 'opp_creature'}]),
    (r"^you cast (?:or copy )?(a|an|your first|your second) ?(instant or sorcery|noncreature|creature|artifact|enchantment|legendary|historic|multicolored|)? ?spell(?: each turn)?$",
     lambda m, t: [{'event': 'cast', 'who': 'you', 'spell': (m.group(2) or 'any').replace(' ', '_'),
                    'nth': 2 if 'second' in (m.group(1) or '') else None}]),
    (r"^(an opponent|a player|another player) casts (a|an) ?(noncreature|creature|instant or sorcery|)? ?spell$",
     lambda m, t: [{'event': 'cast', 'who': 'opponent' if 'opponent' in m.group(1) or 'another' in m.group(1) else 'any',
                    'spell': (m.group(3) or 'any').replace(' ', '_')}]),
    (r"^you draw a card$|^you draw your second card each turn$", lambda m, t: [{'event': 'draw', 'who': 'you'}]),
    (r"^an opponent draws a card$", lambda m, t: [{'event': 'draw', 'who': 'opponent'}]),
    (r"^the beginning of your upkeep$", lambda m, t: [{'event': 'upkeep', 'who': 'you'}]),
    (r"^the beginning of each opponent's upkeep$", lambda m, t: [{'event': 'upkeep', 'who': 'opponent'}]),
    (r"^the beginning of each upkeep$", lambda m, t: [{'event': 'upkeep', 'who': 'any'}]),
    (r"^the beginning of your end step$", lambda m, t: [{'event': 'end_step', 'who': 'you'}]),
    (r"^the beginning of each end step$", lambda m, t: [{'event': 'end_step', 'who': 'any'}]),
    (r"^the beginning of combat on your turn$|^the beginning of your precombat main phase$|^the beginning of your draw step$",
     lambda m, t: [{'event': 'upkeep', 'who': 'you'}]),
    (r"^you attack(?: with one or more creatures)?$|^one or more creatures you control attack$", lambda m, t: [{'event': 'attack', 'source': 'you_any'}]),
    (r"^a creature you control attacks$|^(?:a|another) (?:nontoken )?creature you control attacks$", lambda m, t: [{'event': 'attack', 'source': 'you_each'}]),
    (r"^~ deals combat damage to a player$|^~ deals combat damage to a player or planeswalker$", lambda m, t: [{'event': 'combat_damage', 'source': 'self'}]),
    (r"^(?:a|one or more) creatures? you control deals? combat damage to a player$", lambda m, t: [{'event': 'combat_damage', 'source': 'you_creature'}]),
    (r"^equipped creature deals combat damage to a player$", lambda m, t: [{'event': 'combat_damage', 'source': 'equipped'}]),
    (r"^equipped creature attacks$", lambda m, t: [{'event': 'attack', 'source': 'equipped'}]),
    (r"^equipped creature dies$", lambda m, t: [{'event': 'dies', 'source': 'equipped'}]),
    (r"^you gain life$", lambda m, t: [{'event': 'gain_life', 'who': 'you'}]),
    (r"^~ becomes tapped$", lambda m, t: [{'event': 'attack', 'source': 'self'}]),
    (r"^you cast your (third|second) spell each turn$", lambda m, t: [{'event': 'cast', 'who': 'you', 'spell': 'any', 'nth': 3 if m.group(1) == 'third' else 2}]),
    (r"^you cast a spell$", lambda m, t: [{'event': 'cast', 'who': 'you', 'spell': 'any'}]),
    (r"^a creature you control with power (\d+) or greater enters$", lambda m, t: [{'event': 'etb', 'source': 'you_creature', 'min_pow': int(m.group(1))}]),
    (r"^a warrior attacks$|^a (\w+) you control attacks$", lambda m, t: [{'event': 'attack', 'source': 'you_each', 'subtype': (m.group(1) or 'warrior')}]),
    (r"^you attack with (?:two|three) or more creatures$", lambda m, t: [{'event': 'attack', 'source': 'you_any'}]),
]
TRIGGERS = [(re.compile(a), b) for a, b in TRIGGERS]


def parse_trigger(clause):
    c = clause.strip()
    c = re.sub(r', if [^,]*$', '', c)
    for rx, fn in TRIGGERS:
        m = rx.match(c)
        if m: return fn(m, c)
    return None


def parse_cost(s):
    cost = {}
    parts = [x.strip() for x in s.split(',')]
    for x in parts:
        if re.fullmatch(r'(\{[^}]+\})+', x):
            ms = mana_str(x)
            if '{t}' in x: cost['tap'] = True
            if ms: cost['mana'] = ms
            if '{e}' in x: cost['energy'] = True
        elif x == '{t}': cost['tap'] = True
        elif x.startswith('sacrifice ~') or x.startswith('sacrifice this'): cost['sac'] = 'self'
        elif x.startswith('sacrifice'): cost['sac'] = filt(x).get('type', 'permanent')
        elif x.startswith('pay') and 'life' in x:
            m = re.search(r'pay (\w+) life', x)
            if m is None: return None                  # variable life cost (War Room): leave the line unmodeled
            cost['life'] = n_(m.group(1))
        elif x.startswith('discard'): cost['discard'] = n_(x.split()[1])
        elif x.startswith('exile ~ from your graveyard') or x.startswith('remove'): cost['other'] = x
        elif x.startswith('tap an untapped') or x.startswith('tap '): cost['tap_other'] = x
        else: return None
    return cost


STATICS = [
    (r"^(other )?([a-z ]*?)creatures you control get \+(\d+)/\+(\d+)(?: and have ([a-z, ]+))?$",
     lambda m: [{'type': 'static', 'static': 'anthem', 'filter': dict({'type': 'creature', 'controller': 'you', 'other': bool(m.group(1))},
                **({'subtype': m.group(2).strip()} if m.group(2).strip() else {})), 'pow': int(m.group(3)), 'tgh': int(m.group(4))}] +
     [{'type': 'static', 'static': 'keyword', 'keyword': k, 'filter': {'type': 'creature', 'controller': 'you'}}
      for k in KW if m.group(5) and k in m.group(5)]),
    (r"^(other )?(elves|goblins|zombies|humans|ninjas|soldiers|warriors|knights|angels|demons|dragons|merfolk|vampires|spirits|faeries) you control get \+(\d+)/\+(\d+)(?: and have ([a-z, ]+))?$",
     lambda m: [{'type': 'static', 'static': 'anthem', 'filter': {'type': 'creature', 'controller': 'you', 'other': bool(m.group(1)),
                'subtype': singular(m.group(2))}, 'pow': int(m.group(3)), 'tgh': int(m.group(4))}] +
     [{'type': 'static', 'static': 'keyword', 'keyword': k, 'filter': {'type': 'creature', 'controller': 'you', 'subtype': singular(m.group(2))}}
      for k in KW if m.group(5) and k in m.group(5)]),
    (r"^other (elf|goblin|zombie|human|ninja|soldier|warrior|knight|angel|demon|dragon|vampire|spirit|faerie) creatures get \+(\d+)/\+(\d+)(?: and have [a-z ]+)?$",
     lambda m: [{'type': 'static', 'static': 'anthem', 'filter': {'type': 'creature', 'other': True, 'subtype': m.group(1)},
                 'pow': int(m.group(2)), 'tgh': int(m.group(3))}]),
    (r"^creatures your opponents control get -(\d+)/-(\d+)$",
     lambda m: [{'type': 'static', 'static': 'anthem', 'filter': {'type': 'creature', 'controller': 'opp'},
                 'pow': -int(m.group(1)), 'tgh': -int(m.group(2))}]),
    (r"^(other )?creatures you control have ([a-z, ]+)$",
     lambda m: [{'type': 'static', 'static': 'keyword', 'keyword': k, 'filter': {'type': 'creature', 'controller': 'you', 'other': bool(m.group(1))}}
                for k in KW if k in m.group(2)] or None),
    (r"^equipped creature gets \+(\d+)/\+(\d+)(?: and has ([a-z, ]+))?$",
     lambda m: [{'type': 'static', 'static': 'equip_bonus', 'pow': int(m.group(1)), 'tgh': int(m.group(2))}] +
     [{'type': 'static', 'static': 'equip_keyword', 'keyword': k} for k in KW if m.group(3) and k in m.group(3)] +
     ([{'type': 'static', 'static': 'equip_protection', 'colors': prot_colors(m.group(3))}]
      if m.group(3) and 'protection from' in m.group(3) else [])),
    (r"^equipped creature has ([a-z, ]+)$",
     lambda m: [{'type': 'static', 'static': 'equip_keyword', 'keyword': k} for k in KW if k in m.group(1)] or None),
    (r"^(instant and sorcery|instant|sorcery|creature|noncreature|artifact|enchantment|legendary|)? ?spells you cast cost \{(\d+)\} less to cast$",
     lambda m: [{'type': 'static', 'static': 'cost', 'who': 'you', 'spell': (m.group(1) or 'any').replace(' ', '_').replace('instant_and_sorcery', 'instant_or_sorcery'), 'amount': -int(m.group(2))}]),
    (r"^(noncreature )?spells your opponents cast cost \{(\d+)\} more to cast$",
     lambda m: [{'type': 'static', 'static': 'cost', 'who': 'opponents', 'spell': 'noncreature' if m.group(1) else 'any', 'amount': int(m.group(2))}]),
    (r"^your opponents can't gain life$", lambda m: [{'type': 'static', 'static': 'no_lifegain', 'who': 'opponents'}]),
    (r"^you have no maximum hand size$", lambda m: [{'type': 'static', 'static': 'no_max_hand'}]),
    (r"^~ can't be blocked$", lambda m: [{'type': 'static', 'static': 'unblockable'}]),
    (r"^~ can't block$", lambda m: [{'type': 'static', 'static': 'cant_block'}]),
    (r"^if an effect would create one or more tokens under your control, it creates twice that many of those tokens instead$",
     lambda m: [{'type': 'replacement', 'replace': 'tokens', 'multiplier': 2}]),
    (r"^if one or more \+1/\+1 counters would be put on (?:a creature|a permanent|an artifact or creature) you control, twice that many [^.]*instead$",
     lambda m: [{'type': 'replacement', 'replace': 'counters', 'multiplier': 2}]),
    (r"^(?:~|this creature) gets \+(\d+)/\+(\d+) for each (.+)$",
     lambda m: [{'type': 'static', 'static': 'self_scaling', 'pow': int(m.group(1)), 'per': num_expr(m.group(3))}]),
    (r"^~'s power is equal to the number of (.+)$|^~'s power and toughness are each equal to the number of (.+)$",
     lambda m: [{'type': 'static', 'static': 'self_scaling', 'pow': 1, 'per': num_expr(m.group(1) or m.group(2)), 'base0': True}]),
]
STATICS = [(re.compile(a), b) for a, b in STATICS]


def prot_colors(text):
    return ''.join(c for w, c in (('white', 'W'), ('blue', 'U'), ('black', 'B'), ('red', 'R'), ('green', 'G'))
                   if w in text.split('protection from', 1)[1])


SINGULAR = {'elves': 'elf', 'dwarves': 'dwarf', 'merfolk': 'merfolk', 'faeries': 'faerie'}


def singular(w):
    return SINGULAR.get(w, w[:-1] if w.endswith('s') else w)


def strip_name(text, name):
    t = re.sub(r'\([^)]*\)', '', text)
    front = name.split(' // ')[0]
    for nm in (front, front.split(',')[0]):
        if len(nm) > 3: t = t.replace(nm, '~')
    t = re.sub(r'\bthis (creature|spell|artifact|enchantment|land|permanent|card|planeswalker)\b', '~', t, flags=re.I)
    return t


def compile_card(name, oracle, types, loyalty=None):
    """Oracle text -> (abilities, unparsed lines). types: engine letters (C, I, S, E, A, P, L)."""
    spell = 'I' in types or 'S' in types
    text = strip_name(oracle or '', name)
    abil, bad = [], []
    lines = [l.strip() for l in text.split('\n') if l.strip()]
    i = 0
    while i < len(lines):
        raw = lines[i]; L = raw.lower().rstrip('.'); i += 1
        L = re.sub(r'^[a-z ]+ — (?=(when|whenever|at|choose|\{))', '', L)             # ability words
        # keyword lines and keyword abilities
        m = re.match(r'^equip \{(\d+)\}$', L)
        if m:
            abil.append({'type': 'static', 'static': 'equip_cost', 'mana': int(m.group(1))}); continue
        words = [w.strip() for w in L.split(',')]
        if all(w in KW or re.match(r'^(ward|equip|cycling|kicker|flashback|convoke|overload|foretell|spectacle|prowess|protection|landcycling|basic landcycling|devour|undying|persist|flash|rebound|dredge|partner|crew|cumulative upkeep|affinity|delve|improvise|storm|cascade|buyback|madness|evoke|escape|disturb|channel|companion|hideaway|toxic|mentor|exalted|intimidate|defender|changeling|split second|enchant)\b', w) for w in words):
            continue
        if re.match(r'^\{t\}(?:, [^:]*)?: add ', L) or re.match(r'^~ deals 1 damage to you$', L):
            continue                                            # mana abilities -> mana system
        if 'L' in types and re.match(r'^(\{[^}]+\}, )?\{t\}, sacrifice ~: search your library for a basic land', L):
            continue                                            # fetch lands -> land tags
        if re.match(r"^~ can't be countered$", L): abil.append({'type': 'static', 'static': 'uncounterable'}); continue
        if re.match(r"^you may pay 1 life and exile a blue card from your hand rather than pay", L):
            abil.append({'type': 'static', 'static': 'free_counter'}); continue
        if re.match(r'^you may cast sorcery spells as though they had flash$', L):
            abil.append({'type': 'static', 'static': 'sorcery_flash'}); continue
        if re.match(r'^(~ enters prepared|as ~ enters, choose)', L):
            abil.append({'type': 'static', 'static': 'note', 'text': L}); continue
        if L.startswith('as an additional cost to cast'):
            abil.append({'type': 'additional_cost', 'text': L}); continue
        if re.match(r'^~ costs \{\d+\} less to cast for each', L) or re.match(r'^~ enters (tapped|with)', L):
            abil.append({'type': 'static', 'static': 'note', 'text': L}); continue
        # modal spells / abilities
        m = re.match(r'^choose (one|two|three|one or both|one or more|any number)(?: that hasn\'t been chosen)?(?:\. you may choose the same mode more than once)?\.? ?—?$', L)
        if m:
            modes = []
            while i < len(lines) and lines[i].lstrip().startswith('•'):
                e, b = parse_effects(lines[i].lstrip('• ').lower().rstrip('.'))
                if e: modes.append(e)
                if b: bad += ['• ' + x for x in b]
                i += 1
            k = {'one': 1, 'two': 2, 'three': 3}.get(m.group(1), 2)
            if modes: abil.append({'type': 'spell' if spell else 'triggered_modal', 'effects': [{'do': 'modal', 'choose': k, 'modes': modes}]})
            continue
        # loyalty abilities
        m = re.match(r'^([+−-]?\d+|0): (.+)$', L)
        if m and 'P' in types:
            e, b = parse_effects(m.group(2))
            if e: abil.append({'type': 'loyalty', 'loyalty': int(m.group(1).replace('−', '-')), 'effects': e})
            bad += b; continue
        # triggered abilities
        m = re.match(r'^(when|whenever|at) (.+?), (.+)$', L)
        if m and re.match(r'^choose (one|two) —$', m.group(3)):
            trig = parse_trigger(m.group(2)); modes = []
            while i < len(lines) and lines[i].lstrip().startswith('•'):
                e, b = parse_effects(lines[i].lstrip('• ').lower().rstrip('.'))
                if e: modes.append(e)
                i += 1
            if trig and modes:
                for t in trig: abil.append(dict({'type': 'triggered', 'effects': [{'do': 'modal', 'choose': 1, 'modes': modes}]}, **t))
            else: bad.append(raw)
            continue
        if m:
            CTX['line'] = L
            trig = parse_trigger(m.group(2))
            e, b = parse_effects(m.group(3))
            if trig and e:
                for t in trig: abil.append(dict({'type': 'triggered', 'effects': e}, **t))
                if b: bad.append(raw + '   [partly: ' + '; '.join(b) + ']')
            else: bad.append(raw)
            continue
        # activated abilities
        m = re.match(r'^([^:"]+): (.+)$', L)
        if m and ('{' in m.group(1) or 'sacrifice' in m.group(1) or 'pay' in m.group(1) or 'discard' in m.group(1)):
            cost = parse_cost(m.group(1))
            body = re.sub(r'\.? ?activate (only|this ability only)[^.]*', '', m.group(2))
            e, b = parse_effects(body)
            if cost is not None and e and all(x.get('do') == 'add_mana' for x in e) and set(cost) <= {'tap', 'mana'}:
                continue                                   # mana abilities are handled by the mana system
            if cost is not None and e:
                abil.append({'type': 'activated', 'cost': cost, 'effects': e,
                             'sorcery': 'as a sorcery' in m.group(2), 'once': 'once each turn' in m.group(2)})
                if b: bad.append(raw + '   [partly: ' + '; '.join(b) + ']')
            else: bad.append(raw)
            continue
        # statics
        got = None
        for rx, fn in STATICS:
            mm = rx.match(L)
            if mm:
                got = fn(mm)
                if got: break
        if got: abil += got; continue
        # spell text
        if spell:
            CTX['line'] = L
            e, b = parse_effects(L)
            if e: abil.append({'type': 'spell', 'effects': e})
            if b: bad.append(raw if not e else raw + '   [partly: ' + '; '.join(b) + ']')
            continue
        if re.search(r'\benters tapped\b|\{t\}: add|pay 2 life', L): continue
        if 'L' in types: continue                                  # other land text: utility abilities not modeled
        bad.append(raw)
    # merge several spell lines into one spell ability
    sp = [a for a in abil if a['type'] == 'spell']
    if len(sp) > 1:
        merged = {'type': 'spell', 'effects': [e for a in sp for e in a['effects']]}
        abil = [a for a in abil if a['type'] != 'spell'] + [merged]
    return abil, bad