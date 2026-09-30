"""One practice game: seats the table, runs the engine on a worker thread, and streams what happens as events.

Events (dicts on `Session.events`, read by the text client or the server):
  {'kind': 'log', 'text': ...}            a line of the game log
  {'kind': 'turn', 'player': key, 'view': ...}   a turn begins (view: the table as the human seat sees it)
  {'kind': 'request', 'request': Request} the engine waits for the human (answer with Session.answer)
  {'kind': 'over', 'view': ..., 'winner': key, 'how': ...}
  {'kind': 'error', 'text': traceback}
Only the engine thread touches the game; views are built there and handed over.
"""
import queue
import random
import threading
import traceback

from commander_sim import ais, pools, poolmode
from commander_sim.play.controller import HumanController, Request, Cancelled
from commander_sim.play.view import build_view

MY_DECKS = poolmode.MINE
TIERS = ('t1', 't2', 't3', 't4', 't5')


class EventLog(list):
    """the game log (g.log): every line is also sent to the session as it happens"""
    def __init__(s, session):
        super().__init__()
        s.session = session

    def append(s, line):
        super().append(line)
        s.session._on_log(line)

    def __deepcopy__(s, memo):                     # a look-ahead copy of the game keeps no link to the session
        return None


class Session:
    def __init__(s, deck, tier, seed=None, seat=None, opponents=None, profile='loose', ai='lookahead',
                 step=False, max_rounds=30):
        if deck not in MY_DECKS: raise ValueError(f'unknown deck {deck!r}: one of {", ".join(MY_DECKS)}')
        if tier not in TIERS: raise ValueError(f'unknown tier {tier!r}: one of {", ".join(TIERS)}')
        s.deck, s.tier, s.profile, s.ai, s.step, s.max_rounds = deck, tier, profile, ai, step, max_rounds
        s.seed = seed if seed is not None else random.SystemRandom().randrange(1, 10 ** 6)
        s.events = queue.Queue()
        s.human = HumanController(notify=s.events.put)
        s.game = None
        s._thread = None
        s._seats = s._choose_seats(opponents, seat)

    # ------------------------------------------------------------------ setup
    def _choose_seats(s, opponents, seat):
        poolmode._setup(s.profile, s.ai, 1.0)
        keys = poolmode.pool_keys(s.tier)
        if opponents:
            bad = [k for k in opponents if k not in keys]
            if bad or len(opponents) != 3: raise ValueError(f'pick three of: {", ".join(keys)}')
            r = random.Random(f'seats:{s.seed}')
            seats = list(opponents) + [s.deck]; r.shuffle(seats)
        else:
            seats = pools.draw_seats(s.seed, keys, s.deck, 3)
        if seat is not None:
            if not 1 <= seat <= 4: raise ValueError('seat is 1 to 4')
            seats.remove(s.deck); seats.insert(seat - 1, s.deck)
        return seats

    @property
    def seats(s):
        return list(s._seats)

    # ------------------------------------------------------------------ running
    def start(s):
        s._thread = threading.Thread(target=s._run, name='practice-game', daemon=True)
        s._thread.start()
        return s

    def _run(s):
        try:
            g = ais.setup_pool_game(s.seed, [poolmode.seat_spec(k) for k in s._seats], human=s.deck)
            g.log = EventLog(s)
            g.controllers = {s.deck: s.human}
            s.game = g
            g.log.append('Seat order: ' + ', '.join(ais.NAME(p) for p in g.players))
            me = next(p for p in g.players if p.key == s.deck)
            from commander_sim.play import choices
            choices.mulligan(g, me, me.mull_rng)
            ais._run_rounds(g, g.players, s.max_rounds)
            s.events.put({'kind': 'over', 'view': build_view(g, s.deck),
                          'winner': g.winner.key if g.winner else None, 'how': getattr(g, 'wintype', '')})
        except Cancelled:
            s.events.put({'kind': 'over', 'view': None, 'winner': None, 'how': 'closed'})
        except Exception:
            s.events.put({'kind': 'error', 'text': traceback.format_exc()})

    def _on_log(s, line):
        s.events.put({'kind': 'log', 'text': line})
        if '--- ' in line and ' turn ' in line and s.game is not None and s.game.active is not None:
            g = s.game
            s.events.put({'kind': 'turn', 'player': g.active.key, 'view': build_view(g, s.deck)})
            if s.step: s.ask(Request('continue', f'{ais.NAME(g.active)}: turn {g.active.turns}'))

    def ask(s, req):
        """engine thread: post a request and wait for the human's answer"""
        return s.human.ask(req)

    def answer(s, value):
        s.human.answer(value)

    def close(s):
        s.human.close()

    def join(s, timeout=None):
        if s._thread is not None: s._thread.join(timeout)
