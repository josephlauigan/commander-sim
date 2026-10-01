// The bare browser table (step 2a): follow the game from your seat and answer its decisions with buttons.
const $ = (sel) => document.querySelector(sel);
let lastId = 0, pending = null, source = null;
let images = {};            // card name -> image files in /images/ (the table uses them from step 2d)

function loading(done, total) {
  const box = $('#loading');
  if (done === null) { box.hidden = true; return; }
  box.hidden = false;
  const pct = total ? Math.round((100 * done) / total) : 0;
  box.querySelector('.bar > div').style.width = pct + '%';
  box.querySelector('.bar').setAttribute('aria-valuenow', pct);
}

function el(tag, attrs = {}, ...kids) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (k === 'class') e.className = v; else if (k.startsWith('on')) e.addEventListener(k.slice(2), v); else e.setAttribute(k, v);
  }
  for (const k of kids.flat()) if (k != null) e.append(k.nodeType ? k : String(k));
  return e;
}

async function api(path, body) {
  const r = await fetch(path, body === undefined ? {} : { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  return { ok: r.ok, data: await r.json() };
}

// ------------------------------------------------------------------ the table
function card(name, extra = {}) {
  const bits = [name];
  if (extra.pt) bits.push(extra.pt);
  if (extra.counters) bits.push(`+${extra.counters}`);
  if (extra.loyalty != null) bits.push(`loyalty ${extra.loyalty}`);
  if (extra.attached_to) bits.push(`on ${extra.attached_to}`);
  return el('span', { class: 'card' + (extra.tapped ? ' tapped' : '') }, bits.join(' · '));
}

function renderTable(view) {
  const t = $('#table'); t.replaceChildren();
  if (!view) return;
  t.append(el('div', { class: 'stats' }, `Round ${view.round} · ${view.step || ''}`));
  for (const p of view.players) {
    const box = el('div', { class: 'player' + (p.you ? ' you' : '') + (p.key === view.active ? ' active' : '') + (p.alive ? '' : ' out') },
      el('h2', {}, p.name + (p.you ? ' (you)' : '')),
      el('div', { class: 'stats' }, `${p.life} life · hand ${p.hand_count} · library ${p.library} · graveyard ${p.graveyard.length}` +
        (p.poison ? ` · ${p.poison} poison` : '') + (p.treasures ? ` · ${p.treasures} Treasure` : '') +
        (p.commander_in_zone ? ` · ${p.commander} in the command zone (tax ${p.tax})` : '')));
    if (p.battlefield.length) box.append(el('div', { class: 'zone' }, el('b', {}, 'Battlefield'), p.battlefield.map((m) => card(m.name, m))));
    if (p.lands.length) box.append(el('div', { class: 'zone' }, el('b', {}, 'Lands'), p.lands.map((L) => card(L.name, L))));
    if (p.you && p.hand) box.append(el('div', { class: 'zone' }, el('b', {}, 'Hand'), p.hand.map((n) => card(n))));
    if (p.you) box.append(el('div', { class: 'zone' }, el('b', {}, 'Mana pool'), p.mana_pool));
    t.append(box);
  }
}

// ------------------------------------------------------------------ decisions
async function answer(value) {
  if (!pending) return;
  const id = pending.id;
  pending = null; renderPrompt(null);
  const r = await api('/api/answer', { id, answer: value });
  if (!r.ok) logLine(r.data.error, 'invalid');
}

const btn = (label, value, cls = '') => el('button', { class: cls, onclick: () => answer(value) }, label);

function you(view) { return view && view.players.find((p) => p.you); }

function renderPrompt(ev) {
  const box = $('#prompt'); box.replaceChildren();
  if (!ev) { box.append(el('div', { class: 'stats' }, 'Waiting for the other players…')); return; }
  const req = ev.request;
  box.append(el('h3', {}, req.prompt));
  if (req.kind === 'priority') {
    const me = you(ev.view);
    if (req.data.stack && req.data.stack.length) box.append(el('div', { class: 'row' }, el('b', {}, 'On the stack'), req.data.stack.map((x) => card(x.name))));
    const row = (title, items) => items.length && box.append(el('div', { class: 'row' }, el('b', {}, title), items));
    row('Tap', (me.mana_sources || []).flatMap((s) => s.colours.length > 1
      ? [...s.colours].map((c) => btn(`${s.name} {${c}}`, { do: 'tap', source: s.id, colour: c }))
      : [btn(`${s.name} {${s.colours}}`, { do: 'tap', source: s.id })]));
    row('Hand', me.hand.flatMap((n, i) => me.hand_land[i]
      ? [btn(`Play ${n}`, { do: 'land', card: i })].concat(me.hand_special[i] ? [btn(`Cast ${n.split(' // ')[0]}`, { do: 'cast', card: i })] : [])
      : [btn(`Cast ${n}`, { do: 'cast', card: i })]));
    if (me.commander_in_zone) row('Command zone', [btn(`${me.commander} (tax ${me.tax})`, { do: 'cast', zone: 'cmd' })]);
    row('Activate', me.battlefield.map((m) => btn(m.name, { do: 'use', perm: m.i })));
    row('Land abilities', me.lands.filter((L) => L.ability).map((L) => btn(L.name, { do: 'use', land: L.i })));
    row('Graveyard', (me.graveyard_playable || []).map((x) => btn(`${x.name} (${x.how})`, { do: x.how === 'land' ? 'land' : 'cast', zone: 'gy', card: x.i })));
    box.append(el('div', { class: 'row' }, btn('Pass priority', { do: 'pass' }, 'primary')));
  } else if (req.kind === 'attack') {
    const boxes = req.choices.map((c, i) => el('label', {}, el('input', { type: 'checkbox', value: i }), ' ', c));
    box.append(el('div', { class: 'row' }, boxes));
    box.append(el('div', { class: 'row' },
      el('button', { class: 'primary', onclick: () => answer(boxes.map((b) => b.firstChild).filter((b) => b.checked).map((b) => +b.value)) }, 'Attack'),
      btn('No attack', [])));
  } else if (req.kind === 'mulligan') {
    box.append(el('div', { class: 'row' }, (req.data.hand || []).map((n) => card(n))));
    box.append(el('div', { class: 'row' }, btn('Keep', 'keep', 'primary'), btn('Mulligan', 'mulligan')));
  } else if (req.kind === 'continue') {
    box.append(el('div', { class: 'row' }, btn('Continue', true, 'primary')));
  } else {
    box.append(el('div', { class: 'row' }, req.choices.map((c, i) => btn(c, i))));
  }
}

// ------------------------------------------------------------------ the event stream
function logLine(text, cls = '') {
  const log = $('#log');
  const atEnd = log.scrollTop + log.clientHeight >= log.scrollHeight - 4;
  log.append(el('div', { class: cls }, text));
  if (atEnd) log.scrollTop = log.scrollHeight;
}

function onEvent(ev) {
  lastId = Math.max(lastId, ev.id);
  if (ev.view) renderTable(ev.view);
  if (ev.kind === 'log') logLine(ev.text, ev.text.startsWith('---') ? 'turn' : '');
  else if (ev.kind === 'invalid') logLine('Not allowed: ' + ev.text, 'invalid');
  else if (ev.kind === 'auto') logLine('(automatic) ' + ev.text, 'auto');
  else if (ev.kind === 'loading') loading(ev.done, ev.total);
  else if (ev.kind === 'images') { images = ev.images; loading(null); }
  else if (ev.kind === 'request') { pending = ev; renderPrompt(ev); }
  else if (ev.kind === 'over') { pending = null; renderPrompt(null); $('#prompt').replaceChildren(el('h3', {}, `Game over: ${ev.winner || 'no winner'} (${ev.how})`)); }
  else if (ev.kind === 'error') logLine(ev.text, 'invalid');
}

function connect() {
  if (source) source.close();
  source = new EventSource(`/api/events?since=${lastId}`);
  source.onmessage = (m) => onEvent(JSON.parse(m.data));
}

// ------------------------------------------------------------------ setup
let catalog = null, chosen = { deck: null, tier: 't3' };

function screen(name) {
  $('#setup').hidden = name !== 'setup';
  $('#game').hidden = name !== 'game';
  $('#to-setup').hidden = name !== 'game';
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
  $('#log').replaceChildren(); pending = null; renderTable(null); renderPrompt(null); loading(0, 0);
  const r = await api('/api/new', body);
  if (!r.ok) { loading(null); $('#setup-error').textContent = r.data.error; return; }
  $('#status').textContent = `Seed ${r.data.seed} · seats: ${r.data.seats.join(', ')}`;
  screen('game');
}

async function init() {
  catalog = (await api('/api/options')).data;
  chosen.deck = catalog.decks[0].key;
  const f = $('#newgame');
  f.addEventListener('submit', startGame);
  for (const r of f.opp) r.addEventListener('change', renderSetup);
  $('#to-setup').addEventListener('click', () => screen('setup'));
  renderSetup();
  if (Object.keys(catalog.images).length < catalog.decks.length) {      // the commanders' images arrive shortly after start-up
    setTimeout(async () => { catalog = (await api('/api/options')).data; renderSetup(); }, 6000);
  }
  const st = (await api('/api/state')).data;
  images = st.images || {};
  if (st.game) {
    $('#status').textContent = `Seed ${st.game.seed} · seats: ${st.game.seats.join(', ')}`;
    renderTable(st.view); screen('game');
  } else screen('setup');
  connect();
}
init();
