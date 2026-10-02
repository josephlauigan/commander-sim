"""One practice game: seats the table, runs the engine on a worker thread, and streams what happens as events.

One or two people play (two: each on their own computer, through the server; `partner` is the second person's
deck). Each person's seat has its own controller, hint and AI comparison; their answers go into one list in the order
the game asked for them, so Undo and saved games replay both.

Events (dicts on `Session.events`, read by the text client or the server):
  {'kind': 'log', 'text': ...}            a line of the game log
  {'kind': 'turn', 'player': key, 'view': ...}   a turn begins (view: the table as the first person's seat sees it)
  {'kind': 'request', 'request': Request, 'seat': key} the engine waits for that seat (answer with Session.answer)
  {'kind': 'over', 'view': ..., 'winner': key, 'how': ...}
  {'kind': 'error', 'text': traceback}
An event with a view also has 'views': each person's seat -> the table as that seat sees it. An event meant for one
seat only (its requests and messages, a card only it may see) has 'seat'; one every seat but that one sees has 'hide'.
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


WORSE = 1.0        # the review counts a decision "clearly worse" when the AI's choice scored this much higher


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
    def __init__(s, session, seat=None):
        s.seat = seat or session.deck
        super().__init__(notify=lambda ev: session._emit(dict(ev, seat=s.seat)))
        s.session = session

    def ask(s, req):
        ss = s.session
        ss.marks.append(len(ss.tape.entries))      # how far the AI's tape had got at this decision
        if ss.replay:
            ans = ss.replay.pop(0)
        else:
            ss.replaying = False                   # caught up: this decision is live
            ss.current, ss.current_seat = req, s.seat
            try:
                ans = super().ask(req)
            finally:
                ss.current, ss.current_seat = None, None
            if ss.compare and ss.game is not None:      # the AI comparison: copy the game before the answer applies
                me = next(p for p in ss.game.players if p.key == s.seat)
                ss.shadows[s.seat].record(ss.game, me, req, ans, len(ss.answers))
        ss.answers.append(ans)
        ss.answer_seats.append(s.seat)
        return ans

    def idle(s):
        ss = s.session
        if not ss.compare or s._closed.is_set() or not ss.hint_lock.acquire(blocking=False): return False
        try:
            return any(sh.step() for sh in ss.shadows.values())
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

    def secret(s, key, public, line):
        """a line naming a card only seat `key` may see (a card tutored to hand): the game's log keeps it whole; that
        seat is shown `line`, the others `public`"""
        super().append(line)
        s.session._on_log(public, hide=key)
        if key in s.session.humans: s.session._on_log(line, seat=key)

    def __deepcopy__(s, memo):                     # a look-ahead copy of the game keeps no link to the session
        return None


class Session:
    def __init__(s, deck, tier, seed=None, seat=None, opponents=None, profile='loose', ai='lookahead',
                 step=False, max_rounds=30, views=False, compare=False, seats=None, partner=None):
        if deck not in MY_DECKS: raise ValueError(f'unknown deck {deck!r}: one of {", ".join(MY_DECKS)}')
        if tier not in TIERS: raise ValueError(f'unknown tier {tier!r}: one of {", ".join(TIERS)}')
        if partner is not None and partner not in MY_DECKS:
            raise ValueError(f'unknown deck {partner!r}: one of {", ".join(MY_DECKS)}')
        if partner == deck: raise ValueError('the two players need different decks')
        s.deck, s.tier, s.profile, s.ai, s.step, s.max_rounds = deck, tier, profile, ai, step, max_rounds
        s.partner = partner
        s.humans = [deck] + ([partner] if partner else [])       # the seats people play
        s.views = views                    # a view of the table with every action (the browser plays them back)
        s.compare = compare                # the AI comparison log (shadow.py), for the review after the game
        from commander_sim.play.shadow import Shadow
        s.shadows = {k: Shadow() for k in s.humans}
        s.seed = seed if seed is not None else random.SystemRandom().randrange(1, 10 ** 6)
        s.events = queue.Queue()
        from commander_sim.ai import search
        s.tape = search.Tape()             # the look-ahead AI's decisions, for replaying
        s.answers, s.marks, s.replay, s.replaying, s._restarting = [], [], [], False, False
        s.answer_seats = []                # whose answer each one is
        s.current = None                   # the decision waiting for you (the engine thread is blocked on it)
        s.current_seat = None              # whose decision it is
        s.finished = False                 # the game has ended (won, lost, or the round limit)
        s.hint_lock = threading.Lock()     # a hint reads the game: your answer waits until it's done
        s.ctls = {k: RecordingController(s, k) for k in s.humans}
        s.names = {}                       # deck key -> the name the table shows
        s.game = None
        s._thread = None
        s._seats = s._exact_seats(seats) if seats else s._choose_seats(opponents, seat)
        from types import SimpleNamespace
        s.names = {k: ais.NAME(SimpleNamespace(key=k)) for k in s._seats}

    @property
    def human(s):
        """the first person's controller"""
        return s.ctls[s.deck]

    @human.setter
    def human(s, ctl):
        s.ctls[s.deck] = ctl

    @property
    def shadow(s):
        """the first person's AI comparison"""
        return s.shadows[s.deck]

    # ------------------------------------------------------------------ setup
    def _exact_seats(s, seats):
        """a seating given in full (playing the same seed again)"""
        poolmode._setup(s.profile, s.ai, 1.0)
        keys = poolmode.pool_keys(s.tier)
        n = 4 - len(s.humans)
        if (len(seats) != 4 or any(seats.count(k) != 1 for k in s.humans) or len(set(seats)) != 4
                or any(k not in keys for k in seats if k not in s.humans)):
            raise ValueError(f'seats: {"your decks" if s.partner else "your deck"} and {n} of {", ".join(keys)}, in turn order')
        return list(seats)

    def _choose_seats(s, opponents, seat):
        poolmode._setup(s.profile, s.ai, 1.0)
        keys = poolmode.pool_keys(s.tier)
        if s.partner:                                  # two people: two opponents, everyone seated at random
            if opponents and (len(opponents) != 2 or any(k not in keys for k in opponents)):
                raise ValueError(f'pick two of: {", ".join(keys)}')
            if seat is not None: raise ValueError('with two players the seats are drawn at random')
            seats = list(opponents) if opponents else random.Random(f'pool:{s.seed}').sample(sorted(keys), 2)
            seats += s.humans
            random.Random(f'seats:{s.seed}').shuffle(seats)
            return seats
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
            g = ais.setup_pool_game(s.seed, [poolmode.seat_spec(k) for k in s._seats],
                                    human=s.deck if not s.partner else s.humans)
            g.log = EventLog(s)
            g.controllers = dict(s.ctls)
            g.search_tape = s.tape
            s.game = g
            g.log.append('Seat order: ' + ', '.join(ais.NAME(p) for p in g.players))
            from commander_sim.play import choices
            for me in [p for p in g.players if p.key in s.ctls]:        # in seat order
                choices.mulligan(g, me, me.mull_rng)
            ais._run_rounds(g, g.players, s.max_rounds)
            jobs = [sh for sh in s.shadows.values() if sh.pending()]
            if s.compare and jobs:                         # the comparisons not yet worked out, before the review
                total = sum(len(sh.pending()) for sh in jobs)
                done = [0]
                def progress(d, t):
                    s.events.put({'kind': 'reviewing', 'done': done[0] + d, 'total': total})
                for sh in jobs:
                    k = len(sh.pending())
                    sh.finish(progress)
                    done[0] += k
            s.finished = True
            s.events.put(s._with_views({'kind': 'over', 'winner': g.winner.key if g.winner else None,
                                        'how': getattr(g, 'wintype', '')}, g))
        except Cancelled:
            if not s._restarting: s.events.put({'kind': 'over', 'view': None, 'winner': None, 'how': 'closed'})
        except Exception:
            s.events.put({'kind': 'error', 'text': traceback.format_exc()})

    def _with_views(s, ev, g):
        """ev with the table as each person's seat sees it ('view': the first person's)"""
        ev['views'] = {k: build_view(g, k) for k in s.humans}
        ev['view'] = ev['views'][s.deck]
        return ev

    def _on_log(s, line, seat=None, hide=None):
        if is_private(line): return                  # stays in the game's log (for the review), never shown in play
        ev = {'kind': 'log', 'text': line}
        if seat is not None: ev['seat'] = seat
        if hide is not None: ev['hide'] = hide
        if is_action(line) and s.game is not None and s.views and not s.replaying:
            s._with_views(ev, s.game)                # the table after each action, for the browser's playback
        s._emit(ev)
        if '--- ' in line and ' turn ' in line and s.game is not None and s.game.active is not None:
            g = s.game
            s._emit(s._with_views({'kind': 'turn', 'player': g.active.key}, g))
            if s.step: s.ask(Request('continue', f'{ais.NAME(g.active)}: turn {g.active.turns}'))

    def _emit(s, ev):
        if s.replaying: ev['replay'] = True        # re-played history after an Undo: shown at once, not animated
        s.events.put(ev)

    def ask(s, req):
        """engine thread: post a request and wait for the human's answer"""
        return s.human.ask(req)

    def undo(s, n=1, seat=None):
        """take back your last n answers (with two players, any of the other person's made since go too): the game
        restarts from its seed and replays the rest (the AI's look-ahead decisions come off the tape), then asks the
        decision before them again. Returns None, or why not"""
        to = s.undo_point(n, seat)
        if to is None: return 'Nothing to undo yet.'
        return s.rewind(to, info={'undone': n, 'by': seat or s.deck})

    def undo_point(s, n=1, seat=None):
        """where Undo of seat's last n answers rewinds to (an index into answers), or None"""
        mine = [i for i, k in enumerate(s.answer_seats) if k == (seat or s.deck)]
        if n < 1 or len(mine) < n: return None
        return mine[-n]

    def rewind(s, to, then=None, info=None):
        """restart the game and replay your first `to` answers (then `then`, if given, as your answer to decision
        `to`); the decisions after are live again. Returns None, or why not"""
        if not isinstance(to, int) or not 0 <= to <= len(s.answers): return 'No such decision.'
        keep, cut = s.answers[:to], s.marks[to] if to < len(s.marks) else len(s.tape.entries)
        s._restarting = True
        for c in s.ctls.values(): c.close()
        s.join(30)
        if s._thread is not None and s._thread.is_alive(): return 'The game is busy; try again in a moment.'
        s.tape.cut(cut)
        for sh in s.shadows.values(): sh.forget_from(len(keep))
        s.answers, s.marks, s.answer_seats = [], [], []
        s.replay = list(keep) + ([then] if then is not None else [])
        s.finished = False
        s.replaying = False
        s.events.put(dict({'kind': 'reset', 'undone': 0}, **(info or {})))
        s.replaying = bool(s.replay)
        s.ctls = {k: RecordingController(s, k) for k in s.humans}
        s._restarting = False
        s.start()
        return None

    def try_it(s, n, seat=None):
        """the review's "Try it": back to decision n, with the AI's choice there instead of yours (when it maps onto
        one action; otherwise you're back at the decision and the AI's choice is shown for you to make)"""
        e = s.try_entry(n, seat)
        if e is None: return 'That decision has no AI choice to try.'
        then = e.ai_answer()
        return s.rewind(n, then=then, info={'tryit': {'n': n, 'ai': e.ai_text(), 'applied': then is not None,
                                                      'by': seat or s.deck}})

    def try_entry(s, n, seat=None):
        e = next((x for x in s.shadows[seat or s.deck].entries if x.n == n), None)
        return e if e is not None and e.scored and e.ai is not None else None

    def answer(s, value, seat=None):
        with s.hint_lock:
            s.ctls[seat or s.current_seat or s.deck].answer(value)

    # ------------------------------------------------------------------ saving and loading
    SAVE_VERSION = 1

    def saved(s):
        """the game as a file: the seed and table, your answers, the AI's look-ahead decisions and the comparison.
        Loading it replays the same game (on the same code: a change to card rules or the AI can change a game).
        A two-player game also keeps the second deck, whose answer each one was, and the second comparison"""
        data = {'version': s.SAVE_VERSION, 'deck': s.deck, 'tier': s.tier, 'seed': s.seed, 'seats': s.seats,
                'profile': s.profile, 'ai': s.ai, 'max_rounds': s.max_rounds, 'answers': list(s.answers),
                'tape': [list(e) for e in s.tape.entries],
                'shadow': [e.saved() for e in s.shadow.entries if e.job is None],   # unfinished comparisons don't save
                'finished': s.finished, 'round': s.game.round if s.game is not None else 0}
        if s.partner:
            data['partner'] = s.partner
            data['answer_seats'] = list(s.answer_seats)
            data['partner_shadow'] = [e.saved() for e in s.shadows[s.partner].entries if e.job is None]
        return data

    @classmethod
    def load(cls, data, views=False, compare=True):
        """a session that replays a saved game to where it was saved (a finished game to its end)"""
        if data.get('version') != cls.SAVE_VERSION: raise ValueError('a saved game from a different version')
        s = cls(data['deck'], data['tier'], seed=data['seed'], profile=data['profile'], ai=data['ai'],
                max_rounds=data.get('max_rounds', 30), views=views, compare=compare, seats=data['seats'],
                partner=data.get('partner'))
        from commander_sim.play.shadow import Entry
        s.tape.entries = [(k, tuple(r) if isinstance(r, list) else r, w) for k, r, w in data['tape']]
        s.shadow.entries = [Entry.from_saved(d) for d in data['shadow']]
        if s.partner: s.shadows[s.partner].entries = [Entry.from_saved(d) for d in data.get('partner_shadow', [])]
        s.replay = list(data['answers'])
        s.replaying = bool(s.replay)
        return s

    def review(s, seat=None):
        """the AI comparison for a seat (yours), once the game is over (it stays hidden while you play):
        {'summary', 'decisions'}"""
        g = s.game
        if g is None or not s.finished: return 'The review opens when the game ends.'
        seat = seat or s.deck
        ds = s.shadows[seat].review()
        scored = [d for d in ds if d['scored']]
        me = next(p for p in g.players if p.key == seat)
        won = g.winner is me
        summary = {'won': won, 'winner': ais.NAME(g.winner) if g.winner else None, 'how': getattr(g, 'wintype', '') or '',
                   'rounds': g.round, 'answers': s.answer_seats.count(seat), 'decisions': len(ds), 'compared': len(scored),
                   'matched': sum(1 for d in scored if not d['differs']),
                   'worse': sum(1 for d in scored if d['score_ai'] - d['score_yours'] >= WORSE),
                   'worse_margin': WORSE}
        return {'summary': summary, 'decisions': ds}

    def hint(s, seat=None):
        """the AI's advice for the decision waiting for you: {'text', 'detail', 'choice'}, or a string saying why not"""
        with s.hint_lock:
            req = s.current
            if req is None or s.game is None or s.current_seat != (seat or s.deck):
                return 'There is no decision waiting for you.'
            me = next(p for p in s.game.players if p.key == s.current_seat)
            from commander_sim.play import advisor
            return advisor.hint(s.game, me, req)

    def close(s):
        for c in s.ctls.values(): c.close()

    def join(s, timeout=None):
        if s._thread is not None: s._thread.join(timeout)
