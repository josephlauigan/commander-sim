"""Who makes a seat's decisions.

The engine asks `controller_of(g, p)` at each decision point. A seat with no controller (every seat in a simulation)
is played by the AI exactly as before. The human seat's controller turns each decision into a Request, hands it to
whoever is watching (the text client, later the browser) and waits for the answer on the engine thread.
"""
import queue
import threading


class Request:
    """one decision the human must make: kind (e.g. 'continue', 'mulligan'), a prompt, and the legal choices"""
    def __init__(s, kind, prompt, choices=None, data=None):
        s.kind, s.prompt, s.choices, s.data = kind, prompt, list(choices or []), dict(data or {})

    def __repr__(s):
        return f'Request({s.kind!r}, {s.prompt!r}, {len(s.choices)} choices)'


class Cancelled(Exception):
    """the session was closed while the engine waited for an answer"""


class HumanController:
    """the human seat: each decision is posted to `requests` and the engine thread blocks until `answer` is called"""
    human = True

    def __init__(s, notify=None):
        s.requests = queue.Queue()
        s._answers = queue.Queue()
        s._closed = threading.Event()
        s.notify = notify                        # the session's event stream: notify(event_dict)

    def tell(s, kind, text):
        """a message for the human that needs no answer (an illegal move, an automatic choice)"""
        if s.notify is not None: s.notify({'kind': kind, 'text': text})

    def ask(s, req):
        if s._closed.is_set(): raise Cancelled()
        if s.notify is not None: s.notify({'kind': 'request', 'request': req})
        s.requests.put(req)
        while True:
            try:
                ans = s._answers.get(timeout=0.2)
            except queue.Empty:
                if s._closed.is_set(): raise Cancelled()
                continue
            if isinstance(ans, Cancelled): raise ans
            return ans

    def answer(s, value):
        s._answers.put(value)

    def close(s):
        s._closed.set()
        s._answers.put(Cancelled())


class ScriptController(HumanController):
    """tests: answers requests from a list (or a function of the request) instead of a person"""
    def __init__(s, answers):
        super().__init__()
        s.answers = answers if callable(answers) else list(answers)
        s.asked, s.told = [], []

    def ask(s, req):
        s.asked.append(req)
        if callable(s.answers): return s.answers(req)
        if not s.answers: raise Cancelled()
        return s.answers.pop(0)

    def tell(s, kind, text):
        s.told.append((kind, text))


def controller_of(g, p):
    """p's controller in game g, or None (the AI plays it)"""
    ctl = getattr(g, 'controllers', None)
    return ctl.get(p.key) if ctl else None


def is_human(g, p):
    c = controller_of(g, p)
    return c is not None and getattr(c, 'human', False)
