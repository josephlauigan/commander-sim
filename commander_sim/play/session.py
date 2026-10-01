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


def is_private(line):
    """the AI's reasoning traces ('[Tergrid decides: Jet Medallion 56%, Bloodline Keeper 11% ...]'): they name cards
    still in an opponent's hand, so they are not part of what the human seat sees"""
    body = line[4:] if line.startswith('R') else line
    return body.lstrip().startswith('[')


def is_action(line):
    """a top-level action ('R4    Tergrid casts Jet Medallion'), not one of its effects (indented further) or a turn
    header"""
    body = line[4:] if line.startswith('R') else line
    return body.startswith('  ') and not body.startswith('   ')


class RecordingController(HumanController):
    """the human seat, keeping every answer (Undo replays the game from its seed with them). While the session is
    replaying, recorded answers are given back without asking, and what it would tell you is skipped"""
    def __init__(s, session):
        super().__init__(notify=session._emit)
        s.session = session

    def ask(s, req):
        ss = s.session
        ss.marks.append(len(ss.tape.entries))      # how far the AI's tape had got at this decision
        if ss.replay:
            ans = ss.replay.pop(0)
        else:
            ss.replaying = False                   # caught up: this decision is live
            ss.current = req
            try:
                ans = super().ask(req)
            finally:
                ss.current = None
            if ss.compare and ss.game is not None:      # the AI comparison: copy the game before the answer applies
                me = next(p for p in ss.game.players if p.key == ss.deck)
                ss.shadow.record(ss.game, me, req, ans, len(ss.answers))
        ss.answers.append(ans)
        return ans

    def idle(s):
        ss = s.session
        if not ss.compare or not ss.hint_lock.acquire(blocking=False): return False
        try:
            return ss.shadow.step()
        finally:
            ss.hint_lock.release()

    def tell(s, kind, text):
        if not s.session.replaying: super().tell(kind, text)


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
                 step=False, max_rounds=30, views=False, compare=False):
        if deck not in MY_DECKS: raise ValueError(f'unknown deck {deck!r}: one of {", ".join(MY_DECKS)}')
        if tier not in TIERS: raise ValueError(f'unknown tier {tier!r}: one of {", ".join(TIERS)}')
        s.deck, s.tier, s.profile, s.ai, s.step, s.max_rounds = deck, tier, profile, ai, step, max_rounds
        s.views = views                    # a view of the table with every action (the browser plays them back)
        s.compare = compare                # the AI comparison log (shadow.py), for the review after the game
        from commander_sim.play.shadow import Shadow
        s.shadow = Shadow()
        s.seed = seed if seed is not None else random.SystemRandom().randrange(1, 10 ** 6)
        s.events = queue.Queue()
        from commander_sim.ai import search
        s.tape = search.Tape()             # the look-ahead AI's decisions, for replaying
        s.answers, s.marks, s.replay, s.replaying, s._restarting = [], [], [], False, False
        s.current = None                   # the decision waiting for you (the engine thread is blocked on it)
        s.hint_lock = threading.Lock()     # a hint reads the game: your answer waits until it's done
        s.human = RecordingController(s)
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
            g.search_tape = s.tape
            s.game = g
            g.log.append('Seat order: ' + ', '.join(ais.NAME(p) for p in g.players))
            me = next(p for p in g.players if p.key == s.deck)
            from commander_sim.play import choices
            choices.mulligan(g, me, me.mull_rng)
            ais._run_rounds(g, g.players, s.max_rounds)
            if s.compare and s.shadow.pending():           # the comparisons not yet worked out, before the review
                s.shadow.finish(lambda d, t: s.events.put({'kind': 'reviewing', 'done': d, 'total': t}))
            s.events.put({'kind': 'over', 'view': build_view(g, s.deck),
                          'winner': g.winner.key if g.winner else None, 'how': getattr(g, 'wintype', '')})
        except Cancelled:
            if not s._restarting: s.events.put({'kind': 'over', 'view': None, 'winner': None, 'how': 'closed'})
        except Exception:
            s.events.put({'kind': 'error', 'text': traceback.format_exc()})

    def _on_log(s, line):
        if is_private(line): return                  # stays in the game's log (for the review), never shown in play
        ev = {'kind': 'log', 'text': line}
        if is_action(line) and s.game is not None and s.views and not s.replaying:
            ev['view'] = build_view(s.game, s.deck)  # the table after each action, for the browser's playback
        s._emit(ev)
        if '--- ' in line and ' turn ' in line and s.game is not None and s.game.active is not None:
            g = s.game
            s._emit({'kind': 'turn', 'player': g.active.key, 'view': build_view(g, s.deck)})
            if s.step: s.ask(Request('continue', f'{ais.NAME(g.active)}: turn {g.active.turns}'))

    def _emit(s, ev):
        if s.replaying: ev['replay'] = True        # re-played history after an Undo: shown at once, not animated
        s.events.put(ev)

    def ask(s, req):
        """engine thread: post a request and wait for the human's answer"""
        return s.human.ask(req)

    def undo(s, n=1):
        """take back your last n answers: the game restarts from its seed and replays the rest (the AI's look-ahead
        decisions come off the tape), then asks the decision before them again. Returns None, or why not"""
        k = len(s.answers)
        if n < 1 or k < n: return 'Nothing to undo yet.'
        keep, cut = s.answers[:k - n], s.marks[k - n]
        s._restarting = True
        s.human.close()
        s.join(30)
        if s._thread is not None and s._thread.is_alive(): return 'The game is busy; try again in a moment.'
        s.tape.cut(cut)
        s.shadow.forget_from(len(keep))
        s.answers, s.marks, s.replay = [], [], list(keep)
        s.replaying = False
        s.events.put({'kind': 'reset', 'undone': n})
        s.replaying = bool(keep)
        s.human = RecordingController(s)
        s._restarting = False
        s.start()
        return None

    def answer(s, value):
        with s.hint_lock:
            s.human.answer(value)

    def review(s):
        """the AI comparison, once the game is over (it stays hidden while you play): a list of decisions"""
        if s.game is None or not s.game.over: return 'The review opens when the game ends.'
        return s.shadow.review()

    def hint(s):
        """the AI's advice for the decision waiting for you: {'text', 'detail', 'choice'}, or a string saying why not"""
        with s.hint_lock:
            req = s.current
            if req is None or s.game is None: return 'There is no decision waiting for you.'
            me = next(p for p in s.game.players if p.key == s.deck)
            from commander_sim.play import advisor
            return advisor.hint(s.game, me, req)

    def close(s):
        s.human.close()

    def join(s, timeout=None):
        if s._thread is not None: s._thread.join(timeout)
