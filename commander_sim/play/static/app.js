// The browser table: the setup screen, the loading screen, the table (table.js), the log, and your decisions.
import { el, card as cardOf, renderTable as drawTable, renderSteps } from './table.js';
const $ = (sel) => document.querySelector(sel);
let lastId = 0, pending = null, source = null;
let liveFrom = 0;           // events up to this id are history replayed on load: no pop-up messages for them
let images = {};            // card name -> image files in /images/ (the table uses them from step 2d)

function loading(done, total) {
  const box = $('#loading');
  if (done === null) { box.hidden = true; return; }
  box.hidden = false;
  const pct = total ? Math.round((100 * done) / total) : 0;
  box.querySelector('.bar > div').style.width = pct + '%';
  box.querySelector('.bar').setAttribute('aria-valuenow', pct);
}

async function api(path, body) {
  const r = await fetch(path, body === undefined ? {} : { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  return { ok: r.ok, data: await r.json() };
}

// ------------------------------------------------------------------ the table
let lastView = null;
function renderTable(view) {
  lastView = view;
  drawTable($('#table'), view, images);
  renderSteps($('#steps'), view);
}
const card = (name, o = {}) => cardOf(images, name, Object.assign({ size: 'sm' }, o));

// ------------------------------------------------------------------ decisions
async function answer(value) {
  if (!pending) return;
  const id = pending.id;
  pending = null; renderPrompt(null); markChoices(null);
  const r = await api('/api/answer', { id, answer: value });
  if (!r.ok) logLine(r.data.error, 'invalid');
}

const btn = (label, value, cls = '') => el('button', { class: cls, onclick: () => answer(value) }, label);

function you(view) { return view && view.players.find((p) => p.you); }

function renderPrompt(ev) {
  const box = $('#prompt'); box.replaceChildren(); closeMenu();
  if (ev && tryBanner) {
    box.append(el('div', { class: 'tryit' }, tryBanner.applied ? `Trying the AI's choice: ${tryBanner.ai}. Carry on from here.`
      : `The AI would have: ${tryBanner.ai}. Make that play, then carry on.`));
    tryBanner = null;
  }
  $('#hintbox').hidden = true;                // a hint is for the decision it was asked at
  for (const x of document.querySelectorAll('.stackbar')) x.remove();
  $('#pass').hidden = !(ev && ev.request.kind === 'priority');
  $('#table').classList.toggle('can-act', !!(ev && ev.request.kind === 'priority'));
  if (!ev) { box.append(el('div', { class: 'stats' }, 'Waiting for the other players…')); return; }
  const req = ev.request;
  box.append(el('h3', {}, req.prompt));
  if (req.kind === 'priority') {
    const me = you(ev.view);
    if (req.data.stack && req.data.stack.length) showStack(req);
    box.append(el('p', { class: 'help' }, 'Click a land or mana rock to tap it, a card in your hand to cast or play it, a permanent to use its abilities, your commander in the command zone or a card in your graveyard to cast it.'));
    const row = (title, items) => items.length && list.append(el('div', { class: 'row' }, el('b', {}, title), items));
    const list = el('details', { class: 'actions' }, el('summary', {}, 'The same as a list'));
    row('Tap', (me.mana_sources || []).flatMap((x) => tapButtons(x)));
    row('Hand', me.hand.flatMap((n, i) => me.hand_land[i]
      ? [btn(`Play ${n}`, { do: 'land', card: i })].concat(me.hand_special[i] ? [btn(`Cast ${n.split(' // ')[0]}`, { do: 'cast', card: i })] : [])
      : [btn(`Cast ${n}`, { do: 'cast', card: i })]));
    if (me.commander_in_zone) row('Command zone', [btn(`${me.commander} (tax ${me.tax})`, { do: 'cast', zone: 'cmd' })]);
    row('Activate', me.battlefield.map((m) => btn(m.name, { do: 'use', perm: m.i })));
    row('Land abilities', me.lands.filter((L) => L.ability).map((L) => btn(L.name, { do: 'use', land: L.i })));
    row('Graveyard', (me.graveyard_playable || []).map((x) => btn(`${x.name} (${x.how})`, { do: x.how === 'land' ? 'land' : 'cast', zone: 'gy', card: x.i })));
    box.append(list);
  } else if (req.kind === 'attack') {
    box.append(el('p', { class: 'help' }, 'Click your creatures to attack with them (click again to take one back).'));
    const boxes = req.choices.map((c, i) => el('label', {}, el('input', Object.assign({ type: 'checkbox', value: i,
      onchange: (e) => { if (e.target.checked) attackSel.add(i); else attackSel.delete(i); syncAttack(); } },
      attackSel.has(i) ? { checked: '' } : {})), ' ', c));
    box.append(el('details', { class: 'actions' }, el('summary', {}, 'The same as a list'), el('div', { class: 'row' }, boxes)));
    box.append(el('div', { class: 'row' },
      el('button', { id: 'attack-go', class: 'primary', onclick: () => answer([...attackSel].sort((x, y) => x - y)) }, attackLabel()),
      btn('Attack with everything', req.choices.map((_, i) => i)), btn('No attack', [])));
  } else if (req.kind === 'mulligan') {
    box.append(el('div', { class: 'row' }, (req.data.hand || []).map((n) => card(n))));
    box.append(el('div', { class: 'row' }, btn('Keep', 'keep', 'primary'), btn('Mulligan', 'mulligan')));
  } else if (req.kind === 'continue') {
    box.append(el('div', { class: 'row' }, btn('Continue', true, 'primary')));
  } else {
    box.append(el('div', { class: 'row' }, req.choices.map((c, i) => btn(c, i))));
  }
}

// the stack, between the opponents and your battlefield, with who gets priority on it in turn order
function showStack(req) {
  const opps = document.querySelector('#table .opps');
  if (!opps) return;
  const me = (req.data.order || []).findIndex((x) => you(pending.view) && x.key === you(pending.view).key);
  const order = (req.data.order || []).map((x, i) => el('span', { class: i < me ? 'passed' : i === me ? 'you' : '' },
    i < me ? `${x.name} · passed` : i === me ? 'You' : x.name));
  opps.after(el('section', { class: 'stackbar', 'aria-label': 'The stack' },
    el('div', { class: 'cards' }, req.data.stack.map((x) => cardOf(images, x.name, { size: 'md' }))),
    el('div', {}, el('h3', {}, req.data.caster ? `${req.data.caster} casts ${req.data.stack[0].name}` : 'On the stack'),
      order.length ? el('div', { class: 'order' }, el('b', {}, 'Priority: '), order.flatMap((o, i) => (i ? [' → ', o] : [o]))) : null)));
}

// ------------------------------------------------------------------ acting on the table (your priority)
function tapButtons(x) {
  return x.colours.length > 1 ? [...x.colours].map((c) => btn(`${x.name} {${c}}`, { do: 'tap', source: x.id, colour: c }))
    : [btn(`${x.name} {${x.colours}}`, { do: 'tap', source: x.id })];
}

function toast(text) {
  const t = $('#toast');
  t.textContent = text; t.hidden = false;
  clearTimeout(toast.timer); toast.timer = setTimeout(() => { t.hidden = true; }, 6000);
}

function closeMenu() { const m = $('#menu'); if (m) m.remove(); }

function menu(e, title, items) {
  closeMenu();
  const m = el('div', { id: 'menu', role: 'menu' }, el('div', { class: 'title' }, title),
    items.map(([label, value]) => el('button', { role: 'menuitem', onclick: () => { closeMenu(); answer(value); } }, label)));
  document.body.append(m);
  const r = m.getBoundingClientRect();
  m.style.left = `${Math.min(e.clientX, window.innerWidth - r.width - 8)}px`;
  m.style.top = `${Math.min(e.clientY, window.innerHeight - r.height - 8)}px`;
  m.querySelector('button').focus();
}

function act(e, title, items, otherwise) {
  if (!items.length) { toast(otherwise); return; }
  if (items.length === 1) { answer(items[0][1]); return; }
  menu(e, title, items);
}

const tapItems = (srcs) => srcs.flatMap((x) => x.colours.length > 1
  ? [...x.colours].map((c) => [`Tap for {${c}}`, { do: 'tap', source: x.id, colour: c }])
  : [[`Tap for {${x.colours}}`, { do: 'tap', source: x.id }]]);

// declaring attackers: the creatures picked so far (indices into the request's choices)
let attackSel = new Set();
const attackLabel = () => (attackSel.size ? `Attack with ${attackSel.size}` : 'Attack with none');
function syncAttack() {
  for (const x of document.querySelectorAll('[data-choice]')) x.classList.toggle('attacking', attackSel.has(+x.dataset.choice));
  const go = document.getElementById('attack-go'); if (go) go.textContent = attackLabel();
  for (const b of document.querySelectorAll('#prompt input[type=checkbox]')) b.checked = attackSel.has(+b.value);
}

// a choice whose answer is on the table (a target, a card to discard): those cards and players light up and a click
// picks them; the list in the panel still works
function markChoices(ev) {
  for (const x of document.querySelectorAll('.targetable, .attacking, .attacker-now')) {
    x.classList.remove('targetable', 'attacking', 'attacker-now'); delete x.dataset.choice;
  }
  if (!ev || ev.request.kind === 'priority') return;
  const at = (r) => r && document.querySelector(r.player ? `[data-player="${CSS.escape(r.player)}"]`
    : `[data-seat="${CSS.escape(r.seat)}"][data-i="${r.perm}"]`);
  for (const r of ev.request.data.attackers || []) { const x = at(r); if (x) x.classList.add('attacking'); }
  { const x = at(ev.request.data.attacker); if (x) x.classList.add('attacker-now'); }
  if (!ev.request.data.refs) return;
  ev.request.data.refs.forEach((r, k) => {
    if (!r) return;
    const sel = r.player ? `[data-player="${CSS.escape(r.player)}"]` : r.hand !== undefined ? `[data-hand="${r.hand}"]`
      : `[data-seat="${CSS.escape(r.seat)}"][data-i="${r.perm}"]`;
    const x = document.querySelector(sel);
    if (x && x.dataset.choice === undefined) { x.classList.add('targetable'); x.dataset.choice = k; }
  });
}

function onTableClick(e) {
  const pick = e.target.closest('[data-choice]');
  if (pick && pending && pending.request.kind === 'attack') {
    e.preventDefault(); const i = +pick.dataset.choice;
    if (attackSel.has(i)) attackSel.delete(i); else attackSel.add(i);
    syncAttack(); return;
  }
  if (pick && pending && pending.request.kind !== 'priority') {
    e.preventDefault(); e.stopPropagation(); answer(+pick.dataset.choice); return;
  }
  const t = e.target.closest('[data-land],[data-perm],[data-hand],[data-cmd],[data-gy],[data-treasure]');
  if (!t) return;
  if (!pending || pending.request.kind !== 'priority') { toast("You don't have priority right now."); return; }
  e.preventDefault(); hidePreviewSoon();
  const me = you(pending.view); const srcs = me.mana_sources || [];
  const name = t.dataset.name || '';
  if (t.dataset.land !== undefined) {
    const i = +t.dataset.land, L = me.lands[i];
    const items = tapItems(srcs.filter((x) => x.land === i));
    if (L.ability) items.push(['Activate an ability', { do: 'use', land: i }]);
    act(e, name, items, L.tapped ? `${name} is tapped.` : `${name} can't be tapped for mana right now.`);
  } else if (t.dataset.perm !== undefined) {
    const i = +t.dataset.perm;
    const items = tapItems(srcs.filter((x) => x.perm === i));
    items.push(['Activate an ability', { do: 'use', perm: i }]);
    act(e, name, items, '');
  } else if (t.dataset.treasure !== undefined) {
    act(e, 'Treasure', tapItems(srcs.filter((x) => x.treasure).slice(0, 1)), 'No untapped Treasure.');
  } else if (t.dataset.hand !== undefined) {
    const i = +t.dataset.hand;
    if (!me.hand_land[i]) answer({ do: 'cast', card: i });
    else if (me.hand_special[i]) menu(e, name, [['Play it as your land', { do: 'land', card: i }], [`Cast ${name.split(' // ')[0]}`, { do: 'cast', card: i }]]);
    else answer({ do: 'land', card: i });
  } else if (t.dataset.cmd !== undefined) {
    answer({ do: 'cast', zone: 'cmd' });
  } else if (t.dataset.gy !== undefined) {
    const i = +t.dataset.gy;
    const playable = (me.graveyard_playable || []).find((x) => x.i === i);
    answer({ do: playable && playable.how === 'land' ? 'land' : 'cast', zone: 'gy', card: i });
  }
}

function hidePreviewSoon() { const p = document.getElementById('preview'); if (p) p.hidden = true; }

// ------------------------------------------------------------------ the event stream
let logQueue = [];           // log lines waiting to be added (a burst of events adds them in one go)
function logLine(text, cls = '') {
  logQueue.push(el('div', { class: cls }, text));
  if (logQueue.length === 1) setTimeout(flushLog, 0);
}

function flushLog() {
  if (!logQueue.length) return;
  const log = $('#log');
  const atEnd = log.scrollTop + log.clientHeight >= log.scrollHeight - 4;
  log.append(...logQueue); logQueue = [];
  if (atEnd) log.scrollTop = log.scrollHeight;
}

let stale = null;            // a view not drawn yet (history catching up draws only the last of a run of views)
let drawTimer = null;
function scheduleDraw() {
  if (drawTimer) return;
  drawTimer = setTimeout(() => { drawTimer = null; if (stale) { renderTable(stale); stale = null; markChoices(pending); } }, 40);
}
function onEvent(ev, draw = true) {
  if (ev.view) { if (draw || ev.kind === 'request') { renderTable(ev.view); stale = null; } else stale = ev.view; }
  else if (stale && (ev.kind === 'request' || ev.kind === 'over')) { renderTable(stale); stale = null; }
  if (ev.kind === 'log') logLine(ev.text, ev.text.startsWith('---') ? 'turn' : '');
  else if (ev.kind === 'invalid') { logLine('Not allowed: ' + ev.text, 'invalid'); if (ev.id > liveFrom && !ev.replay) toast(ev.text); }
  else if (ev.kind === 'reset') {           // Undo or Try it: the game is replayed from its seed up to a decision
    $('#log').replaceChildren(); logQueue = []; pending = null; renderPrompt(null); markChoices(null);
    if (ev.tryit) {
      tryBanner = ev.tryit;
      logLine(`(Try it: the game is replayed to that decision, with the AI's choice: ${ev.tryit.ai})`, 'auto');
      screen('game');
    } else logLine(`(Undo: ${ev.undone} action${ev.undone > 1 ? 's' : ''} taken back; the game so far is replayed)`, 'auto');
  }
  else if (ev.kind === 'reviewing') {
    $('#prompt').replaceChildren(el('h3', {}, 'Game over'), el('p', {}, `Working out the AI comparison: ${ev.done} of ${ev.total} decisions…`));
  }
  else if (ev.kind === 'auto') logLine('(automatic) ' + ev.text, 'auto');
  else if (ev.kind === 'loading') loading(ev.done, ev.total);
  else if (ev.kind === 'images') { images = ev.images; loading(null); if (lastView) renderTable(lastView); }
  else if (ev.kind === 'request') {
    if (!ev.view) ev.view = lastView;          // caught-up history carries only the latest table
    pending = ev; attackSel = new Set(); renderPrompt(ev); markChoices(ev);
  }
  else if (ev.kind === 'over') {
    pending = null; renderPrompt(null); tryBanner = null;
    $('#prompt').replaceChildren(el('h3', {}, `Game over: ${ev.winner || 'no winner'}${ev.how ? ` (${ev.how})` : ''}`),
      el('div', { class: 'row' }, tools.compare ? el('button', { class: 'primary', onclick: showReview }, 'Review the game') : null,
        el('button', { onclick: sameSeed }, 'Play this seed again'), el('button', { onclick: () => screen('setup') }, 'New game')));
    if ((location.hash === '#review' || new URLSearchParams(location.search).has('review')) && tools.compare) showReview();   // a direct link
  }
  else if (ev.kind === 'error') logLine(ev.text, 'invalid');
}

// ------------------------------------------------------------------ playback of opponents' turns
// Events queue up here. An action during another player's turn (a log line with its view of the table) is shown,
// then the next one waits BASE / speed ms; paused, only "Next action" moves on. Everything else (your own turn,
// history replayed when the page loads) shows at once. The engine is waiting on your next decision meanwhile, so
// a decision is reached only after the actions before it have played.
const BASE = 1100;
let inbox = [], paused = false, speed = 1, stepOnce = false, skipping = false, pumping = false;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const theirTurn = (view) => view && view.players.some((p) => p.key === view.active && !p.you);
const paced = (ev) => ev.id > liveFrom && !ev.replay && ev.kind === 'log' && ev.view && theirTurn(ev.view);

function receive(ev) { inbox.push(ev); pump(); }

async function pump() {
  if (pumping) return;
  pumping = true;
  try {
    while (inbox.length) {
      const ev = inbox[0];
      if (paced(ev) && !skipping) {
        if (paused && !stepOnce) break;
        stepOnce = false;
        inbox.shift(); onEvent(ev); showControls();
        if (!paused) await sleep(BASE / speed);
      } else {
        inbox.shift();
        onEvent(ev, false);                  // drawn once the burst is over (scheduleDraw), not once per event
        if (ev.kind === 'request' || ev.kind === 'over') skipping = false;
      }
    }
  } finally {
    pumping = false; showControls(); scheduleDraw();
  }
}

function showControls() {
  const bar = $('#playback');
  const waiting = inbox.some(paced);
  bar.hidden = $('#game').hidden || !(theirTurn(lastView) || waiting);
  $('#pb-pause').textContent = paused ? '▶ Play' : '❚❚ Pause';
  $('#pb-next').disabled = !paused || !waiting;
  for (const b of bar.querySelectorAll('[data-speed]')) b.classList.toggle('on', +b.dataset.speed === speed);
}

function setupPlayback() {
  $('#pb-pause').addEventListener('click', () => { paused = !paused; showControls(); pump(); });
  $('#pb-next').addEventListener('click', () => { stepOnce = true; pump(); });
  $('#pb-skip').addEventListener('click', () => { skipping = true; pump(); });
  for (const b of $('#playback').querySelectorAll('[data-speed]')) b.addEventListener('click', () => { speed = +b.dataset.speed; showControls(); });
}

function connect() {
  if (source) source.close();
  source = new EventSource(`/api/events?since=${lastId}`);
  source.onmessage = (m) => { const ev = JSON.parse(m.data); lastId = Math.max(lastId, ev.id); receive(ev); };
}

// ------------------------------------------------------------------ the review (after the game)
let tryBanner = null, reviewData = null, diffsOnly = true;

async function showReview() {
  const r = await api('/api/review');
  if (!r.ok) { toast(r.data.error); return; }
  reviewData = r.data; renderReview(); screen('review');
}

const fmt = (x) => (x == null ? '–' : (x > 0 ? '+' : '') + x.toFixed(1));

function renderReview() {
  const { summary: m, decisions } = reviewData;
  const rows = decisions.filter((d) => !diffsOnly || d.differs);
  $('#review').replaceChildren(
    el('h2', {}, m.won ? `You won in round ${m.rounds}` : m.winner ? `${m.winner} won in round ${m.rounds}${m.how ? ` (${m.how})` : ''}` : `No winner after ${m.rounds} rounds`),
    el('p', { class: 'summary' }, `${m.answers} answers in all; ${m.decisions} decisions the AI weighs, ${m.compared} compared: `,
      el('b', {}, `${m.matched} matched the AI`), `, `, el('b', { class: 'worse' }, `${m.worse} clearly worse`),
      ` (the AI's choice scored ${m.worse_margin} or more higher).`),
    el('p', { class: 'help' }, 'Scores: where the game stood at the end of your next turn, averaged over six playouts, from −100 (lost) to +100 (won).'),
    el('div', { class: 'row' }, el('label', {}, el('input', Object.assign({ type: 'radio', name: 'rv', onchange: () => { diffsOnly = true; renderReview(); } }, diffsOnly ? { checked: '' } : {})), ' Differences only'),
      el('label', {}, el('input', Object.assign({ type: 'radio', name: 'rv', onchange: () => { diffsOnly = false; renderReview(); } }, diffsOnly ? {} : { checked: '' })), ' All decisions')),
    rows.length ? el('table', { class: 'review' },
      el('thead', {}, el('tr', {}, ['When', 'Situation', 'You', 'The AI', 'Your score', "AI's score", ''].map((h) => el('th', {}, h)))),
      el('tbody', {}, rows.map((d) => el('tr', { class: d.scored && d.score_ai - d.score_yours >= m.worse_margin ? 'worse' : '' },
        el('td', { class: 'when' }, `Round ${d.round} · ${d.step_text}`), el('td', {}, d.situation), el('td', {}, d.yours),
        el('td', {}, d.ai_text || (d.note ? el('em', {}, d.note) : '–')),
        el('td', { class: 'num' }, fmt(d.score_yours)), el('td', { class: 'num' }, fmt(d.score_ai)),
        el('td', {}, d.can_try && d.differs ? el('button', { class: 'try', onclick: () => tryIt(d.n) }, 'Try it') : null)))))
      : el('p', {}, diffsOnly ? 'You and the AI made the same choices everywhere they were compared.' : 'No decisions recorded.'),
    el('div', { class: 'row' }, el('button', { onclick: () => screen('game') }, 'Back to the table'),
      el('button', { onclick: sameSeed }, 'Play this seed again'), el('button', { class: 'primary', onclick: () => screen('setup') }, 'New game')));
}

async function tryIt(n) {
  const r = await api('/api/tryit', { n });
  if (!r.ok) toast(r.data.error);
}

async function sameSeed() {
  const st = (await api('/api/state')).data;
  if (!st.game) { screen('setup'); return; }
  const g = st.game;
  const body = { deck: g.deck, tier: g.tier, seed: g.seed, ai: g.ai, profile: g.profile, tools: g.tools,
    seats: g.seats };
  $('#log').replaceChildren(); logQueue = []; pending = null; inbox = []; skipping = false; renderTable(null); renderPrompt(null); loading(0, 0);
  const r = await api('/api/new', body);
  if (!r.ok) { loading(null); toast(r.data.error); return; }
  setStatus(r.data.seed, r.data.seats); setTools(body.tools); screen('game');
}

// ------------------------------------------------------------------ setup
let tools = { hint: true, undo: true, compare: true };
function setTools(t) {
  tools = Object.assign({ hint: true, undo: true, compare: true }, t || {});
  $('#undo').hidden = !tools.undo;
  $('#hint').hidden = !tools.hint;
}

async function hint() {
  if (!pending) { toast('There is no decision waiting for you.'); return; }
  const box = $('#hintbox');
  box.hidden = false; box.replaceChildren(el('h3', {}, 'Hint'), el('p', {}, 'The AI is thinking…'));
  const r = await api('/api/hint', {});
  if (!r.ok) { box.replaceChildren(el('h3', {}, 'Hint'), el('p', {}, r.data.error)); return; }
  box.replaceChildren(el('h3', {}, 'The AI would: ' + r.data.text),
    r.data.detail.length ? el('ul', {}, r.data.detail.map((d, i) => (i === 0 ? el('li', { class: 'lead' }, d) : el('li', {}, d)))) : null,
    el('button', { type: 'button', onclick: () => { box.hidden = true; } }, 'Hide'));
}

async function undo() {
  const r = await api('/api/undo', { n: 1 });
  if (!r.ok) toast(r.data.error);
}

function setStatus(seed, seats) {
  const st = $('#status');
  st.textContent = `Seed ${seed}`;
  st.title = `Seats in turn order: ${seats.join(', ')}`;
}

let catalog = null, chosen = { deck: null, tier: 't3' };

function screen(name) {
  $('#setup').hidden = name !== 'setup';
  $('#game').hidden = name !== 'game';
  $('#review').hidden = name !== 'review';
  if (name !== 'game') $('#playback').hidden = true;
  $('#to-setup').hidden = name !== 'game';
  $('#undo').hidden = name !== 'game' || !tools.undo;
  $('#hint').hidden = name !== 'game' || !tools.hint;
}

function cardImage(name, cls = '') {
  const files = (catalog && catalog.images[name]) || images[name];
  return files ? el('img', { src: `/images/${files[0]}`, alt: name, class: cls, loading: 'lazy' }) : el('span', { class: 'noimg ' + cls });
}

function pct(x) { return x == null ? '–' : `${x.toFixed(1)}%`; }

function renderSetup() {
  const deck = catalog.decks.find((d) => d.key === chosen.deck);
  $('#decks').replaceChildren(...catalog.decks.map((d) => el('button', {
    type: 'button', class: 'deck', role: 'radio', 'aria-checked': String(d.key === chosen.deck),
    onclick: () => { chosen.deck = d.key; renderSetup(); },
  }, cardImage(d.commander), el('span', {}, el('b', {}, d.name),
    el('small', {}, `Bracket ${d.bracket} · ${d.game_changers.length} Game Changers`),
    el('small', {}, d.average == null ? 'No simulation results yet' : `Simulations: ${pct(d.average)} average`)))));
  $('#tiers').replaceChildren(...catalog.tiers.map((t) => el('button', {
    type: 'button', class: 'tier', role: 'radio', 'aria-checked': String(t.key === chosen.tier),
    onclick: () => { chosen.tier = t.key; renderSetup(); },
  }, el('span', {}, el('b', {}, t.label)),
    el('span', { class: 'names' }, t.decks.map((x) => el('span', { title: x.commander }, cardImage(x.commander), x.name))),
    el('span', { class: 'rate' }, deck && deck.win_rates ? pct(deck.win_rates[t.key]) : '', el('small', {}, deck && deck.win_rates ? 'your win rate in sims' : '')))));
  const tier = catalog.tiers.find((t) => t.key === chosen.tier);
  const prev = new Set([...document.querySelectorAll('#picks input:checked')].map((b) => b.value));
  $('#picks').replaceChildren(...tier.decks.map((x) => el('label', {},
    el('input', Object.assign({ type: 'checkbox', value: x.key }, prev.has(x.key) ? { checked: '' } : {})), ' ', x.name)));
  $('#picks').hidden = $('#newgame').opp.value !== 'pick';
}

async function startGame(e) {
  e.preventDefault();
  const f = $('#newgame');
  const body = { deck: chosen.deck, tier: chosen.tier, ai: f.ai.value, profile: f.profile.value,
    tools: { hint: f.hint.checked, undo: f.undo.checked, compare: f.compare.checked } };
  if (f.seed.value) body.seed = +f.seed.value;
  if (f.seat.value) body.seat = +f.seat.value;
  if (f.opp.value === 'pick') {
    body.opponents = [...document.querySelectorAll('#picks input:checked')].map((b) => b.value);
    if (body.opponents.length !== 3) { $('#setup-error').textContent = 'Pick exactly three opponents.'; return; }
  }
  $('#setup-error').textContent = '';
  $('#log').replaceChildren(); logQueue = []; pending = null; inbox = []; skipping = false; renderTable(null); renderPrompt(null); loading(0, 0);
  const r = await api('/api/new', body);
  if (!r.ok) { loading(null); $('#setup-error').textContent = r.data.error; return; }
  setStatus(r.data.seed, r.data.seats); setTools(body.tools);
  screen('game');
}

async function init() {
  catalog = (await api('/api/options')).data;
  chosen.deck = catalog.decks[0].key;
  const f = $('#newgame');
  f.addEventListener('submit', startGame);
  for (const r of f.opp) r.addEventListener('change', renderSetup);
  $('#to-setup').addEventListener('click', () => screen('setup'));
  $('#table').addEventListener('click', onTableClick);
  $('#undo').addEventListener('click', undo);
  $('#hint').addEventListener('click', hint);
  setupPlayback();
  $('#pass').addEventListener('click', () => answer({ do: 'pass' }));
  document.addEventListener('keydown', (e) => { if (e.key === 'Escape') closeMenu(); });
  document.addEventListener('click', (e) => { if (!e.target.closest('#menu') && !e.target.closest('#table')) closeMenu(); });
  renderSetup();
  if (Object.keys(catalog.images).length < catalog.decks.length) {      // the commanders' images arrive shortly after start-up
    setTimeout(async () => { catalog = (await api('/api/options')).data; renderSetup(); }, 6000);
  }
  const st = (await api('/api/state')).data;
  images = st.images || {};
  liveFrom = st.last_event || 0;
  if (st.game) {
    setStatus(st.game.seed, st.game.seats); setTools(st.game.tools);
    renderTable(st.view); screen('game');
  } else screen('setup');
  connect();
}
init();
