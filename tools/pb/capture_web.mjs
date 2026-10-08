#!/usr/bin/env node
/* capture_web.mjs - deterministic captures for web stacks (PLAYBOOK annex §A.5).
 *
 *   node tools/pb/capture_web.mjs --routes tools/pb/routes.json --base <url> --out <dir>
 *        [--viewports 1280x800,390x844] [--fail-on-overflow] [--only <route>[/<state>],...]
 *        [--langs <code>,...] [--timeout-ms 20000] [--browser <exe>] [--channel <name>]
 *        [--playwright <dir>] [--task <ID>] [--json]
 *   node tools/pb/capture_web.mjs --routes ... --boot-seed --task <ID> --out <dir>    this run owns the stack
 *   node tools/pb/capture_web.mjs browsers [--playwright <dir>]           which browser a capture would launch
 *   node tools/pb/capture_web.mjs selftest [--live] [--verbose] [--json]
 *
 * The ONE Node file in tools/pb (§A.0): node builtins, and the Playwright the project already has -
 * resolved from the manifest's "playwright" folder (default: the project root), never installed.
 * Per route x state x language x viewport: navigate, wait for networkidle and the ready selector,
 * run the actions, full-page PNG to `<id>[__<state>][__<lang>]__<w>x<h>.png`, measure overflow,
 * then `captions.json` for review_page.py (§A.4): a list of {id, file, route, path, state, lang,
 * viewport, overflow, overflow_doc, overflow_px, overflow_at, caption} - the TV adds flag and note.
 * Every command prints one counts line, failures only when non-zero, then `=== GO ===` /
 * `=== NO-GO: <reason> ===` (exit 0 / 1); `--json` for agents.
 *
 * Over §A.5, declared:
 * - routes.json is §A.5's LIST, or an object carrying it as "routes" beside the project's values:
 *   base, viewports, locale, timezone, lang_key, langs, playwright, boot. A route with no "states"
 *   captures one unnamed state; with states, exactly those. Ids - route, state, language - match
 *   [A-Za-z0-9][A-Za-z0-9.-]*: no "_", so a stem cut on "__" gives its parts back. An unknown key is
 *   refused (a misspelt key is a value silently dropped); a key starting with "_" is a note.
 * - Overflow is measured at EVERY viewport, two ways: §A.5's document metric (documentElement's or
 *   body's scrollWidth past innerWidth), and each live element's right edge - outside aria-hidden,
 *   inert and visibility:hidden, a non-zero box, not inside a container that scrolls sideways - more
 *   than 1 px past the viewport. A screen rendered inside a container that clips sideways (the way
 *   react-native-web renders every screen) never scrolls, so the document metric alone cannot fire
 *   there. A capture overflows when either does; its caption carries both, the worst element's px
 *   and its selector.
 * - A route is NO-GO when its response is >= 400, when the ready selector never appears, when an
 *   action throws, or when the page renders BLANK; networkidle timing out is a NOTE (§A.5 puts the
 *   NO-GO on ready). A `--base` that does not answer is refused before anything is written.
 * - `--only` captures the named routes (each with every state) and route/state rows; every name
 *   must be declared. `--langs` (else the manifest's "langs") captures each language: "lang_key"
 *   names the localStorage key set before the page's first script; languages without it are refused.
 * - `--boot-seed`: this run owns the stack. The manifest's "boot" names its base and the project's
 *   start and stop commands (launch.py's, §A.3, as a rule: seeding is the launcher's business). The
 *   base is probed first - one that already answers is NOT RUN, with nothing started - then start,
 *   capture, and stop in every case; a stack still answering after the stop is a failure.
 * - Nothing is ever deleted: a stale PNG in --out is named in a NOTE. Writes are atomic, the replace
 *   retried while another process still holds the file; printed paths are relative under the
 *   project root and start with ~ under the home folder.
 */
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import http from 'node:http';
import net from 'node:net';
import os from 'node:os';
import path from 'node:path';

const SELF = fileURLToPath(import.meta.url);
const HERE = path.dirname(SELF);
const ROOT = path.resolve(HERE, '..', '..');
const ID_RE = /^[A-Za-z0-9][A-Za-z0-9.-]*$/;          // no `_`: every part of a stem survives a cut on `__`
const VP_RE = /^(\d{2,5})x(\d{2,5})$/;
const ACTIONS = ['click', 'fill', 'press', 'wait', 'waitMs'];
const TOP_KEYS = ['routes', 'base', 'viewports', 'locale', 'timezone', 'lang_key', 'langs', 'playwright', 'boot'];
const ROUTE_KEYS = ['id', 'path', 'ready', 'actions', 'states', 'viewports', 'caption'];
const STATE_KEYS = ['id', 'ready', 'actions', 'caption'];
const BOOT_KEYS = ['base', 'start', 'stop', 'deadline_s'];
const DEFAULTS = { viewports: '1280x800,390x844', locale: 'en-US', timezone: 'UTC', timeoutMs: 20000, deadlineS: 120 };
const PW_PACKAGES = ['playwright', '@playwright/test', 'playwright-core'];
const FREEZE = '*,*::before,*::after{animation:none!important;transition:none!important;' +
               'caret-color:transparent!important;scroll-behavior:auto!important}';
const NO_BROWSER = "Playwright browser absent - the install is the lead's";
const NOT_RUN = 'NOT RUN (a stack this run did not start)';
const TUNE = { rename: fs.renameSync, tries: 6, delayMs: 50, pollMs: 100, graceMs: 10000 };   // the selftest's seams

const VALUE_FLAGS = ['routes', 'base', 'out', 'viewports', 'only', 'langs', 'task', 'timeout-ms', 'browser', 'channel', 'playwright'];
const BOOL_FLAGS = ['json', 'fail-on-overflow', 'boot-seed', 'live', 'verbose', 'help'];
const COMMANDS = ['capture', 'browsers', 'selftest'];
const USAGE = [
  'usage: node tools/pb/capture_web.mjs --routes tools/pb/routes.json --base <url> --out <dir>',
  '         [--viewports 1280x800,390x844] [--fail-on-overflow] [--only <route>[/<state>],...] [--langs <code>,...]',
  '         [--timeout-ms 20000] [--browser <exe>] [--channel <name>] [--playwright <dir>] [--task <ID>] [--json]',
  '       node tools/pb/capture_web.mjs --routes ... --boot-seed --task <ID> --out <dir>   (the manifest\'s "boot")',
  '       node tools/pb/capture_web.mjs browsers [--routes <file>] [--playwright <dir>] [--browser <exe>] [--json]',
  '       node tools/pb/capture_web.mjs selftest [--live] [--verbose] [--playwright <dir>] [--json]',
].join('\n');

class Refuse extends Error {}

// ------------------------------------------------------------------ output, files, arguments

/** A printed path: relative under the project root, `~` under the home folder. */
function shown(text) {
  let s = String(text);
  const esc = (p) => p.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  for (const [from, to] of [[ROOT, ''], [os.homedir(), '~']]) {
    if (!from || from === path.parse(from).root) continue;
    s = s.replace(new RegExp(`(?<![\\w./\\\\-])${esc(from)}(?:[\\\\/]|(?=$|[\\s"'\`:,;)\\]]))`, 'g'),
      (m) => (/[\\/]$/.test(m) ? (to ? `${to}${path.sep}` : '') : to || '.'));
  }
  return s;
}

function verdict(a, cmd, counts, fails = [], notes = [], reason = null) {
  const ok = !fails.length && !reason;
  const why = reason || (fails.length ? `${fails.length} failure(s)` : '');
  if (a.json) {
    console.log(shown(JSON.stringify({ cmd, counts, failures: fails, notes, verdict: ok ? 'GO' : 'NO-GO', reason: why })));
  } else {
    for (const n of notes) console.log(shown(`NOTE ${n}`));
    console.log(`${cmd}: ${Object.entries(counts).map(([k, v]) => `${k}=${v}`).join(' · ')}`);
    for (const f of fails) console.log(shown(`FAIL ${f}`));
    console.log(ok ? '=== GO ===' : shown(`=== NO-GO: ${why} ===`));
  }
  return ok ? 0 : 1;
}

function sleepMs(ms) { Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms); }

/** Temp file, then replace - retried while another process still holds the target; the temp is ours to remove. */
function writeAtomic(file, data) {
  const tmp = `${file}.tmp${process.pid}`;
  fs.writeFileSync(tmp, data);
  for (let n = 1, wait = TUNE.delayMs; ; n++, wait *= 2) {
    try {
      TUNE.rename(tmp, file);
      return;
    } catch (e) {
      if (n >= TUNE.tries || !['EPERM', 'EACCES', 'EBUSY'].includes(e.code)) {
        try { fs.unlinkSync(tmp); } catch { /* already gone */ }
        throw e;
      }
      sleepMs(wait);
    }
  }
}

function readJson(file, what) {
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch (e) {
    throw new Refuse(`${what} ${shown(file)}: ${e.code || e.message}`);
  }
  if (text.charCodeAt(0) === 0xfeff) text = text.slice(1);
  try {
    return JSON.parse(text);
  } catch (e) {
    throw new Refuse(`${what} ${shown(file)}: ${e.message}`);
  }
}

function camel(k) { return k.replace(/-([a-z])/g, (_m, c) => c.toUpperCase()); }

/** §A.5's flag interface, strictly: an unknown option or a value flag with no value is refused, never guessed. */
function parseArgs(argv) {
  const a = { _: [] };
  for (let i = 0; i < argv.length; i++) {
    const t = argv[i];
    if (t === '-h') { a.help = true; continue; }
    if (!t.startsWith('--')) { a._.push(t); continue; }
    const eq = t.indexOf('=');
    const k = eq > 0 ? t.slice(2, eq) : t.slice(2);
    if (BOOL_FLAGS.includes(k)) {
      if (eq > 0) throw new Refuse(`--${k} takes no value`);
      a[camel(k)] = true;
    } else if (VALUE_FLAGS.includes(k)) {
      const v = eq > 0 ? t.slice(eq + 1) : argv[++i];
      if (v === undefined || v === '' || (eq < 0 && v.startsWith('--'))) throw new Refuse(`--${k} needs a value`);
      a[camel(k)] = v;
    } else {
      throw new Refuse(`unknown option --${k} (known: --${[...VALUE_FLAGS, ...BOOL_FLAGS].join(' --')})`);
    }
  }
  if (a._.length > 1) throw new Refuse(`one command at a time, got: ${a._.join(' ')}`);
  a.cmd = a._[0] || 'capture';
  if (!COMMANDS.includes(a.cmd)) throw new Refuse(`unknown command ${JSON.stringify(a.cmd)} (${COMMANDS.join(' | ')})`);
  return a;
}

// ------------------------------------------------------------------ the manifest, and the plan

function parseViewports(spec) {
  const out = [];
  const list = Array.isArray(spec) ? spec : String(spec).split(',');
  for (const raw of list.map((s) => String(s).trim()).filter(Boolean)) {
    const m = VP_RE.exec(raw);
    if (!m) throw new Refuse(`viewport ${JSON.stringify(raw)}: expected <w>x<h>`);
    out.push({ w: Number(m[1]), h: Number(m[2]), label: `${m[1]}x${m[2]}` });
  }
  if (!out.length) throw new Refuse('no viewport declared');
  return out;
}

function parseLangs(spec, where) {
  const list = Array.isArray(spec) ? spec : String(spec).split(',').map((s) => s.trim()).filter(Boolean);
  if (!list.length) throw new Refuse(`${where} names no language`);
  for (const l of list) {
    if (typeof l !== 'string' || !ID_RE.test(l)) throw new Refuse(`${where}: language ${JSON.stringify(l)} must match ${ID_RE}`);
  }
  if (new Set(list).size !== list.length) throw new Refuse(`${where}: a language named twice`);
  return list;
}

function knownKeys(where, obj, keys) {
  const bad = Object.keys(obj).filter((k) => !keys.includes(k) && !k.startsWith('_'));
  if (bad.length) {
    throw new Refuse(`${where}: unknown key ${bad.join(', ')} (known: ${keys.join(', ')}) - a misspelt key would be a value silently dropped`);
  }
}

function checkUrl(where, v) {
  let u = null;
  try { u = new URL(v); } catch { /* refused below */ }
  if (typeof v !== 'string' || !u || !['http:', 'https:'].includes(u.protocol)) {
    throw new Refuse(`${where} ${JSON.stringify(v)}: expected an http(s) URL`);
  }
  return u;
}

/** A command: a list of strings, or {"posix": [...], "nt": [...]} - the one for this box. */
function argvOf(spec, where) {
  const v = spec && typeof spec === 'object' && !Array.isArray(spec) ? spec[process.platform === 'win32' ? 'nt' : 'posix'] : spec;
  if (!Array.isArray(v) || !v.length || !v.every((x) => typeof x === 'string' && x)) {
    throw new Refuse(`${where}: a command is a non-empty list of strings, or {"posix": [...], "nt": [...]} (this box: ${process.platform === 'win32' ? 'nt' : 'posix'})`);
  }
  return v;
}

function checkActions(where, actions) {
  if (actions === undefined) return [];
  if (!Array.isArray(actions)) throw new Refuse(`${where}: actions must be a list`);
  for (const act of actions) {
    if (!act || typeof act !== 'object' || Array.isArray(act)) throw new Refuse(`${where}: an action is an object`);
    const keys = Object.keys(act);
    const bad = keys.filter((k) => !ACTIONS.includes(k));
    if (bad.length) throw new Refuse(`${where}: unknown action ${bad.join(', ')} (known: ${ACTIONS.join(', ')})`);
    if (keys.length !== 1) throw new Refuse(`${where}: one action per object, got ${keys.length}`);
    if ('fill' in act && !(Array.isArray(act.fill) && act.fill.length === 2)) throw new Refuse(`${where}: fill takes [selector, text]`);
  }
  return actions;
}

function loadManifest(file) {
  const raw = readJson(file, 'routes');
  const doc = Array.isArray(raw) ? { routes: raw } : raw;
  const where = `routes ${shown(file)}`;
  if (!doc || typeof doc !== 'object' || !Array.isArray(doc.routes)) {
    throw new Refuse(`${where}: expected a list of routes, or an object carrying "routes"`);
  }
  knownKeys(where, doc, TOP_KEYS);
  for (const k of ['locale', 'timezone', 'lang_key', 'playwright']) {
    if (doc[k] !== undefined && (typeof doc[k] !== 'string' || !doc[k].trim())) throw new Refuse(`${where}: "${k}" is a non-empty string`);
  }
  if (doc.base !== undefined) checkUrl(`${where}: "base"`, doc.base);
  if (doc.langs !== undefined) {
    parseLangs(doc.langs, `${where}: "langs"`);
    if (!doc.lang_key) throw new Refuse(`${where}: "langs" are declared but no "lang_key" names the localStorage key to set them with`);
  }
  if (doc.boot !== undefined) {
    const b = doc.boot;
    if (!b || typeof b !== 'object' || Array.isArray(b)) throw new Refuse(`${where}: "boot" is an object {base, start, stop}`);
    knownKeys(`${where}: "boot"`, b, BOOT_KEYS);
    checkUrl(`${where}: "boot.base"`, b.base);
    argvOf(b.start, `${where}: "boot.start"`);
    argvOf(b.stop, `${where}: "boot.stop"`);
    if (b.deadline_s !== undefined && !(typeof b.deadline_s === 'number' && b.deadline_s > 0)) {
      throw new Refuse(`${where}: "boot.deadline_s" is a number of seconds`);
    }
  }
  const seen = new Set();
  for (const r of doc.routes) {
    if (!r || typeof r !== 'object' || Array.isArray(r)) throw new Refuse(`${where}: a route is an object`);
    if (!ID_RE.test(String(r.id || ''))) throw new Refuse(`route id ${JSON.stringify(r.id)}: ${ID_RE} (the PNG stem is cut on "__")`);
    if (seen.has(r.id)) throw new Refuse(`two routes share the id ${JSON.stringify(r.id)}`);
    seen.add(r.id);
    knownKeys(`route ${r.id}`, r, ROUTE_KEYS);
    if (typeof r.path !== 'string' || !r.path.startsWith('/')) throw new Refuse(`route ${r.id}: path must be a string starting with "/"`);
    if (r.ready !== undefined && typeof r.ready !== 'string') throw new Refuse(`route ${r.id}: ready is a selector`);
    if (r.caption !== undefined && typeof r.caption !== 'string') throw new Refuse(`route ${r.id}: caption is a text`);
    checkActions(`route ${r.id}`, r.actions);
    if (r.viewports !== undefined) parseViewports(r.viewports);
    if (r.states !== undefined) {
      if (!Array.isArray(r.states) || !r.states.length) throw new Refuse(`route ${r.id}: states is a non-empty list`);
      const sseen = new Set();
      for (const s of r.states) {
        if (!s || typeof s !== 'object' || Array.isArray(s)) throw new Refuse(`route ${r.id}: a state is an object`);
        if (!ID_RE.test(String(s.id || ''))) throw new Refuse(`route ${r.id}: state id ${JSON.stringify(s.id)} must match ${ID_RE}`);
        if (sseen.has(s.id)) throw new Refuse(`route ${r.id}: two states share the id ${JSON.stringify(s.id)}`);
        sseen.add(s.id);
        knownKeys(`route ${r.id} state ${s.id}`, s, STATE_KEYS);
        if (s.ready !== undefined && typeof s.ready !== 'string') throw new Refuse(`route ${r.id} state ${s.id}: ready is a selector`);
        if (s.caption !== undefined && typeof s.caption !== 'string') throw new Refuse(`route ${r.id} state ${s.id}: caption is a text`);
        checkActions(`route ${r.id} state ${s.id}`, s.actions);
      }
    }
  }
  if (doc.viewports !== undefined) parseViewports(doc.viewports);
  return doc;
}

/** `--only <route>[/<state>],...` -> the names kept, or null for all; every name must be declared. */
function onlyCells(doc, raw) {
  if (raw === undefined || raw === null) return null;
  const known = new Set();
  for (const r of doc.routes) {
    known.add(r.id);
    for (const s of r.states || []) known.add(`${r.id}/${s.id}`);
  }
  const picked = new Set();
  for (const tok of String(raw).split(',').map((s) => s.trim())) {
    if (!tok || !known.has(tok)) throw new Refuse(`--only ${JSON.stringify(tok)}: no such route or route/state in the manifest - nothing captured`);
    picked.add(tok);
  }
  return picked;
}

/** The languages to capture: --langs, else the manifest's; [''] when none (a stem with no language). */
function pickLangs(doc, flag) {
  const where = flag !== undefined ? '--langs' : '"langs"';
  const spec = flag !== undefined ? flag : doc.langs;
  if (spec === undefined) return [''];
  const langs = parseLangs(spec, where);
  if (!doc.lang_key) throw new Refuse(`${where}: languages are declared but the manifest names no "lang_key" to set them with`);
  return langs;
}

function langScript(key, lang) {
  return `try { localStorage.setItem(${JSON.stringify(key)}, ${JSON.stringify(lang)}); } catch (e) {}`;
}

/** route x state x language x viewport, in manifest order - one shot per PNG. */
function plan(doc, viewports, langs = [''], only = null) {
  const shots = [];
  for (const r of doc.routes) {
    const vps = r.viewports ? parseViewports(r.viewports) : viewports;
    const states = r.states && r.states.length ? r.states : [{ id: '' }];
    for (const s of states) {
      if (only && !only.has(r.id) && !only.has(`${r.id}/${s.id}`)) continue;
      for (const lang of langs) {
        for (const vp of vps) {
          const stem = [r.id, s.id, lang, vp.label].filter(Boolean).join('__');
          shots.push({
            route: r.id, path: r.path, state: s.id || '', lang, vp, stem, file: `${stem}.png`,
            ready: s.ready || r.ready || 'body',
            actions: [...(r.actions || []), ...(s.actions || [])],
            caption: s.caption || r.caption || '',
          });
        }
      }
    }
  }
  if (!shots.length) throw new Refuse('the manifest declares no route');
  return shots;
}

/** One captions.json entry, as review_page.py reads it; the TV adds `flag` and `note`. */
function captionOf(shot, m) {
  return {
    id: shot.stem, file: shot.file, route: shot.route, path: shot.path,
    state: [shot.state, shot.lang].filter(Boolean).join(' · '), lang: shot.lang, viewport: shot.vp.label,
    overflow: !!(m.document || m.px > 0), overflow_doc: !!m.document, overflow_px: m.px, overflow_at: m.at,
    caption: shot.caption,
  };
}

function tally(shots, caps, fails, failOnOverflow) {
  const over = caps.filter((c) => c.overflow).length;
  const counts = { declared: shots.length, captured: caps.length, failed: fails.length, overflow: over };
  let reason = null;
  if (!shots.length) reason = 'no route declared';
  else if (!caps.length) reason = 'zero captures';
  else if (caps.length < shots.length) reason = `${shots.length - caps.length} of ${shots.length} shot(s) not captured`;
  else if (failOnOverflow && over) reason = `${over} capture(s) overflow at their viewport`;
  return { counts, reason };
}

// ------------------------------------------------------------------ what the page is asked (run in the page)
// Both stand alone - no reference outside their own text - because the page gets them as source.

function measureOverflow(win, doc) {
  const vw = win.innerWidth;
  const de = doc.documentElement;
  const body = doc.body;
  const docW = Math.max(de ? de.scrollWidth : 0, body ? body.scrollWidth : 0);
  const hidden = (el) => {
    for (let n = el; n && n.nodeType === 1; n = n.parentElement) {
      if (n.getAttribute('aria-hidden') === 'true' || n.hasAttribute('inert')) return true;
    }
    return win.getComputedStyle(el).visibility === 'hidden';
  };
  const insideSidewaysScroller = (el) => {
    for (let n = el.parentElement; n && n !== body && n !== de; n = n.parentElement) {
      const ox = win.getComputedStyle(n).overflowX;
      if ((ox === 'auto' || ox === 'scroll') && n.scrollWidth > n.clientWidth) return true;
    }
    return false;
  };
  let px = 0;
  let at = '';
  for (const el of body ? body.querySelectorAll('*') : []) {
    const r = el.getBoundingClientRect();
    if (!(r.width > 0 && r.height > 0)) continue;
    const past = r.right - vw;
    if (past <= 1 || past <= px || hidden(el) || insideSidewaysScroller(el)) continue;
    px = past;
    const cls = typeof el.className === 'string' ? el.className.trim().split(/\s+/).filter(Boolean).slice(0, 2) : [];
    at = el.tagName.toLowerCase() + (el.id ? `#${el.id}` : '') + cls.map((c) => `.${c}`).join('');
  }
  return { document: docW > vw, px: Math.round(px), at };
}

function aliveProbe(win, doc) {
  const b = doc.body;
  return !!b && b.childElementCount > 0
    && ((b.innerText || '').trim().length > 0 || !!b.querySelector('img,svg,canvas,input,button,video'));
}

const OVERFLOW_JS = `(${measureOverflow})(window, document)`;
const ALIVE_JS = `(${aliveProbe})(window, document)`;

// ------------------------------------------------------------------ the browser, never installed

function loadPlaywright(dir) {
  const req = createRequire(pathToFileURL(path.join(dir, 'package.json')));
  for (const name of PW_PACKAGES) {
    try {
      const pw = req(name);
      if (pw && pw.chromium) return { pw, from: dir, name };
    } catch { /* the next name */ }
  }
  return { pw: null, from: dir, name: null };
}

function browserCandidates(a, pw) {
  const out = [];
  const push = (exe, label) => { if (exe && fs.existsSync(exe)) out.push({ executablePath: exe, label }); };
  push(a.browser, `--browser ${a.browser}`);
  push(process.env.PB_CAPTURE_BROWSER, 'PB_CAPTURE_BROWSER');
  if (a.channel) out.push({ channel: a.channel, label: `--channel ${a.channel}` });
  try { push(pw.chromium.executablePath(), 'playwright pinned chromium'); } catch { /* not downloaded */ }
  const home = os.homedir();
  const cache = process.env.PLAYWRIGHT_BROWSERS_PATH || (process.platform === 'win32'
    ? path.join(home, 'AppData', 'Local', 'ms-playwright')
    : process.platform === 'darwin' ? path.join(home, 'Library', 'Caches', 'ms-playwright')
      : path.join(home, '.cache', 'ms-playwright'));
  let dirs = [];
  try { dirs = fs.readdirSync(cache).filter((d) => d.startsWith('chromium')); } catch { /* no cache */ }
  const rev = (d) => Number((/-(\d+)$/.exec(d) || [0, 0])[1]);
  for (const d of dirs.sort((x, y) => rev(y) - rev(x))) {
    for (const rel of ['chrome-linux64/chrome', 'chrome-linux/chrome', 'chrome-win/chrome.exe',
      'chrome-headless-shell-linux64/chrome-headless-shell', 'chrome-linux/headless_shell',
      'chrome-mac/Chromium.app/Contents/MacOS/Chromium']) {
      push(path.join(cache, d, rel), `ms-playwright/${d}`);
    }
  }
  if (!a.channel) for (const ch of ['chrome', 'chromium', 'msedge']) out.push({ channel: ch, label: `channel ${ch}` });
  return out;
}

async function openBrowser(a, pw) {
  const tried = [];
  for (const c of browserCandidates(a, pw)) {
    try {
      const browser = await pw.chromium.launch({ headless: true, ...(c.channel ? { channel: c.channel } : { executablePath: c.executablePath }) });
      return { browser, how: c.label };
    } catch (e) { tried.push(`${c.label}: ${String(e.message || e).split('\n')[0]}`); }
  }
  throw new Refuse(`${NO_BROWSER} (tried ${tried.length}: ${tried.slice(0, 3).join(' | ')})`);
}

// ------------------------------------------------------------------ the stack: someone else's, or this run's

/** Does anything accept a connection at the URL's host and port? A probe only - nothing is sent. */
function answering(base, ms = 1000) {
  const u = new URL(base);
  const port = Number(u.port) || (u.protocol === 'https:' ? 443 : 80);
  const host = u.hostname.replace(/^\[(.*)\]$/, '$1');
  return new Promise((ok) => {
    const s = net.connect({ host, port, autoSelectFamily: true });
    const done = (v) => { s.destroy(); ok(v); };
    s.setTimeout(ms, () => done(false));
    s.once('connect', () => done(true));
    s.once('error', () => done(false));
  });
}

/** Wait until the base answers (want = true) or goes quiet (false); false when the time runs out first. */
async function settle(base, want, ms) {
  const end = Date.now() + ms;
  for (;;) {
    if ((await answering(base, Math.min(1000, ms))) === want) return true;
    if (Date.now() >= end) return false;
    await new Promise((ok) => setTimeout(ok, TUNE.pollMs));
  }
}

function runCmd(argv, env, ms) {
  return new Promise((done) => {
    let out = '';
    let kid;
    let settled = false;
    const finish = (r) => { if (!settled) { settled = true; done(r); } };
    try {
      kid = spawn(argv[0], argv.slice(1), { cwd: ROOT, env, stdio: ['ignore', 'pipe', 'pipe'], windowsHide: true });
    } catch (e) {
      finish({ status: null, why: `could not start (${e.code || e.message})`, tail: '' });
      return;
    }
    const keep = (b) => { out = (out + b).slice(-2000); };
    kid.stdout.on('data', keep);
    kid.stderr.on('data', keep);
    const timer = setTimeout(() => kid.kill(), ms);
    kid.on('error', (e) => { clearTimeout(timer); finish({ status: null, why: `could not start (${e.code || e.message})`, tail: '' }); });
    // `exit`, not `close`: a launcher's detached children may hold its pipes open for as long as they run
    kid.on('exit', (code, sig) => {
      clearTimeout(timer);
      kid.stdout.destroy();
      kid.stderr.destroy();
      finish({ status: code, why: code === null ? `was stopped (${sig}) after ${ms / 1000} s` : `exited ${code}`,
        tail: out.trim().split('\n').slice(-3).join(' | ') });
    });
  });
}

/** --boot-seed: start the manifest's stack, run fn against it, stop it in every case, prove it quiet. */
async function ownedStack(boot, task, fn) {
  const env = { ...process.env, PB_TASK: task };
  const ms = (boot.deadline_s || DEFAULTS.deadlineS) * 1000;
  const start = argvOf(boot.start, '"boot.start"');
  const stop = argvOf(boot.stop, '"boot.stop"');
  const fails = [];
  let crash = null;
  try {
    const s = await runCmd(start, env, ms);
    if (s.status !== 0) fails.push(`--boot-seed: the start command ${s.why} - nothing captured${s.tail ? `: ${s.tail}` : ''}`);
    else if (!(await settle(boot.base, true, ms))) fails.push(`--boot-seed: ${boot.base} never answered after the start command (${ms / 1000} s)`);
    else await fn();
  } catch (e) { crash = e; }
  const t = await runCmd(stop, env, ms);        // every case: a start that failed may have left half a stack
  if (t.status !== 0) fails.push(`--boot-seed: the stop command ${t.why}${t.tail ? `: ${t.tail}` : ''}`);
  if (!(await settle(boot.base, false, TUNE.graceMs))) fails.push(`a stack this run started still answers after the stop command: ${boot.base}`);
  const notes = [`stack: started and stopped by this run (start: ${start.join(' ')} · stop: ${stop.join(' ')})`];
  if (crash instanceof Refuse) throw new Refuse([crash.message, ...fails].join(' · '));
  if (crash) throw crash;
  return { fails, notes };
}

// ------------------------------------------------------------------ the capture

async function shoot(ctx, shot, base, outDir, timeout, caps, fails, notes) {
  const page = await ctx.newPage();
  try {
    const url = new URL(shot.path, base).toString();
    const resp = await page.goto(url, { waitUntil: 'load', timeout });
    if (resp && resp.status() >= 400) throw new Error(`HTTP ${resp.status()} at ${url}`);
    try {
      await page.waitForLoadState('networkidle', { timeout });
    } catch { notes.push(`${shot.stem}: networkidle never settled - captured after load`); }
    await page.waitForSelector(shot.ready, { timeout, state: 'attached' });
    for (const act of shot.actions) {
      if ('click' in act) await page.click(act.click, { timeout });
      else if ('fill' in act) await page.fill(act.fill[0], act.fill[1], { timeout });
      else if ('press' in act) await page.press(Array.isArray(act.press) ? act.press[0] : 'body', Array.isArray(act.press) ? act.press[1] : act.press, { timeout });
      else if ('wait' in act) await page.waitForSelector(act.wait, { timeout });
      else if ('waitMs' in act) await page.waitForTimeout(act.waitMs);
    }
    await page.addStyleTag({ content: FREEZE });
    if (!(await page.evaluate(ALIVE_JS))) throw new Error('the page rendered BLANK (ready matched, nothing drawn)');
    const m = await page.evaluate(OVERFLOW_JS);
    writeAtomic(path.join(outDir, shot.file), await page.screenshot({ fullPage: true, animations: 'disabled' }));
    caps.push(captionOf(shot, m));
  } catch (e) {
    fails.push(`${shot.stem} at ${shot.path}: ${String(e.message || e).split('\n')[0]}`);
  } finally {
    await page.close().catch(() => {});
  }
}

async function capture(a, doc, shots, outDir, base, browser, log = () => {}) {
  fs.mkdirSync(outDir, { recursive: true });
  const caps = [];
  const fails = [];
  const notes = [];
  const groups = new Map();                      // one context per viewport x language
  for (const s of shots) {
    const k = `${s.vp.label} ${s.lang}`;
    if (!groups.has(k)) groups.set(k, []);
    groups.get(k).push(s);
  }
  for (const group of groups.values()) {
    const { vp, lang } = group[0];
    const ctx = await browser.newContext({
      viewport: { width: vp.w, height: vp.h }, deviceScaleFactor: 1, reducedMotion: 'reduce',
      locale: doc.locale || DEFAULTS.locale, timezoneId: doc.timezone || DEFAULTS.timezone,
    });
    try {
      if (lang) await ctx.addInitScript(langScript(doc.lang_key, lang));
      for (const shot of group) {
        const before = caps.length;
        await shoot(ctx, shot, base, outDir, a.timeoutMs, caps, fails, notes);
        log(`  ${caps.length > before ? 'ok  ' : 'FAIL'} ${shot.stem}`);
      }
    } finally { await ctx.close().catch(() => {}); }
  }
  const order = new Map(shots.map((s, i) => [s.stem, i]));
  caps.sort((x, y) => order.get(x.id) - order.get(y.id));
  writeAtomic(path.join(outDir, 'captions.json'), `${JSON.stringify(caps, null, 1)}\n`);
  const mine = new Set(caps.map((c) => c.file));
  const stale = fs.readdirSync(outDir).filter((f) => /\.png$/i.test(f) && !mine.has(f));
  if (stale.length) {
    notes.push(`${stale.length} PNG(s) in ${outDir} are not this run's and were left untouched (review_page.py refuses an image no entry names - use a fresh --out): ${stale.slice(0, 5).join(', ')}`);
  }
  return { caps, fails, notes };
}

/** The capture command, up to its verdict: {counts, reason, fails, notes}; a refusal throws, having written nothing. */
async function runCapture(a, log = () => {}) {
  const doc = loadManifest(a.routes ? path.resolve(a.routes) : path.join(HERE, 'routes.json'));
  if (a.bootSeed && a.base) throw new Refuse('--base and --boot-seed name two different stacks - pass one (--boot-seed starts and stops its own)');
  if (!a.out) throw new Refuse('--out <dir> is required');
  const task = a.task || process.env.PB_TASK || '';
  if (a.bootSeed && !doc.boot) throw new Refuse('--boot-seed: the manifest declares no "boot" {base, start, stop}');
  if (a.bootSeed && !task) throw new Refuse('--boot-seed needs --task <ID> (or $PB_TASK): the launcher keys its logs by task');
  const base = a.bootSeed ? doc.boot.base : a.base || doc.base;
  if (!base) throw new Refuse('--base <url> is required (or a "base" in the manifest, or --boot-seed)');
  checkUrl('--base', base);
  const timeoutMs = a.timeoutMs === undefined ? DEFAULTS.timeoutMs : Number(a.timeoutMs);
  if (!(Number.isInteger(timeoutMs) && timeoutMs > 0)) throw new Refuse(`--timeout-ms ${JSON.stringify(a.timeoutMs)}: a whole number of milliseconds`);
  const viewports = parseViewports(a.viewports || doc.viewports || DEFAULTS.viewports);
  const shots = plan(doc, viewports, pickLangs(doc, a.langs), onlyCells(doc, a.only));
  const up = await answering(base);
  if (a.bootSeed && up) throw new Refuse(`--boot-seed: ${NOT_RUN} - ${base} already answers; not reused, not stopped, nothing started`);
  if (!a.bootSeed && !up) throw new Refuse(`--base ${base}: not answering - nothing started, nothing written`);
  const { pw, from } = loadPlaywright(a.playwright ? path.resolve(a.playwright) : path.resolve(ROOT, doc.playwright || '.'));
  if (!pw) throw new Refuse(`Playwright is not on disk (${PW_PACKAGES.join(', ')}, resolved from ${shown(from)}) - the install is the lead's`);
  log(`capture ${shots.length} shot(s) at ${base}${task ? ` · ${task}` : ''}`);
  const { browser, how } = await openBrowser(a, pw);
  const notes = [`browser: ${how}`];
  const fails = [];
  let caps = [];
  const run = async () => {
    const r = await capture({ ...a, timeoutMs }, doc, shots, path.resolve(a.out), base, browser, log);
    caps = r.caps;
    fails.push(...r.fails);
    notes.push(...r.notes);
  };
  try {
    if (a.bootSeed) {
      const s = await ownedStack(doc.boot, task, run);
      fails.push(...s.fails);
      notes.push(...s.notes);
    } else {
      await run();
    }
  } finally {
    await browser.close().catch(() => {});
  }
  return { ...tally(shots, caps, fails, a.failOnOverflow), fails, notes };
}

function cmdBrowsers(a) {
  const file = a.routes ? path.resolve(a.routes) : path.join(HERE, 'routes.json');
  const doc = a.routes || fs.existsSync(file) ? loadManifest(file) : {};
  const { pw, from } = loadPlaywright(a.playwright ? path.resolve(a.playwright) : path.resolve(ROOT, doc.playwright || '.'));
  const candidates = pw ? browserCandidates(a, pw) : [];
  const notes = [`playwright: ${pw ? 'found' : 'none'}, resolved from ${from}`,
    ...candidates.map((c) => c.label + (c.executablePath ? ` -> ${c.executablePath}` : ' (probed by launching)'))];
  return verdict(a, 'browsers', { playwright: pw ? 1 : 0, candidates: candidates.length }, [], notes, pw && candidates.length ? null : NO_BROWSER);
}

// ------------------------------------------------------------------ selftest, red-armed
// Offline by default: node builtins, and a synthetic Playwright written into the selftest's own temp
// folder, so every path of this file's own code runs - the manifest, the plan, the flags, the capture
// loop, the owned stack, the overflow probe, the captions - with no browser. `--live` runs the same
// fixtures in a real browser, through the project's Playwright (--playwright <dir>, else the root).

const PAGES = {
  '/ok': '<!doctype html><title>ok</title><body><div id="app"><h1>Hello</h1>'
    + '<button id="more" onclick="document.getElementById(\'panel\').hidden=false">more</button>'
    + '<p id="panel" hidden>open - a much longer text, so that the capture changes</p><p id="lang"></p></div>'
    + '<script>try { document.getElementById("lang").textContent = "language: " + (localStorage.getItem("pb.lang") || "none") } catch (e) {}</script></body>',
  '/wide': '<!doctype html><title>wide</title><body style="margin:0"><div id="app" style="width:900px">wide</div></body>',
  '/clipped': '<!doctype html><title>clipped</title><body style="margin:0"><div id="app" style="width:100vw;height:100vh;overflow:hidden">'
    + '<p id="long" style="white-space:nowrap;width:2000px;margin:0">a line that runs far past its column, clipped by it</p></div></body>',
  '/stacked': '<!doctype html><title>stacked</title><body style="margin:0"><div id="app" style="width:100vw;overflow:hidden">'
    + '<div id="behind" aria-hidden="true" style="width:3000px"><span>a screen stacked behind the one shown</span></div>'
    + '<div id="row" style="width:100vw;overflow-x:auto"><div id="cards" style="width:2400px">cards that scroll sideways</div></div>'
    + '<p id="front">the screen shown</p></div></body>',
  '/blank': '<!doctype html><title>blank</title><body></body>',
};

/** The synthetic Playwright's view of the same pages: the nodes present, and the boxes the probe measures. */
const FAKE_PAGES = {
  '/ok': { ids: ['app', 'more', 'lang'], reveals: { '#more': ['panel'] }, text: 'Hello', docW: 'vw', els: [{ id: 'app', width: 'vw', height: 300 }] },
  '/wide': { ids: ['app'], text: 'wide', docW: 'content', els: [{ id: 'app', width: 900 }] },
  '/clipped': { ids: ['app', 'long'], text: 'a line', docW: 'vw',
    els: [{ id: 'app', width: 'vw', height: 800, overflowX: 'hidden' }, { id: 'long', tag: 'p', parent: 0, width: 2000 }] },
  '/stacked': { ids: ['app', 'behind', 'row', 'cards', 'front'], text: 'the screen shown', docW: 'vw', els: [
    { id: 'app', width: 'vw', height: 400, overflowX: 'hidden' },
    { id: 'behind', parent: 0, width: 3000, aria: true },
    { tag: 'span', parent: 1, width: 3000 },
    { id: 'row', parent: 0, width: 'vw', height: 40, overflowX: 'auto', scrollWidth: 2400 },
    { id: 'cards', parent: 3, width: 2400, height: 40 },
    { id: 'front', tag: 'p', parent: 0, width: 'vw' }] },
  '/blank': { ids: [], blank: true, docW: 'vw', els: [] },
};
const FAKE_404 = { ids: ['app'], text: '404 - nothing here', docW: 'vw', els: [{ id: 'app', width: 'vw' }] };

/** A window and a document with just what the two probes read. Stands alone: the synthetic Playwright carries its text. */
function fakeDom(spec, vw) {
  const px = (v) => (v === 'vw' ? vw : v);
  const body = { nodeType: 1, tagName: 'BODY', id: '', className: '', parentElement: null, attrs: {}, style: {},
    childElementCount: spec.blank ? 0 : Math.max(1, (spec.els || []).length), innerText: spec.text || '' };
  const els = (spec.els || []).map((e) => {
    const w = px(e.width);
    return {
      nodeType: 1, tagName: (e.tag || 'div').toUpperCase(), id: e.id || '', className: e.cls || '', parentElement: null,
      attrs: Object.assign({}, e.aria ? { 'aria-hidden': 'true' } : {}, e.inert ? { inert: '' } : {}),
      style: { visibility: e.visibility || 'visible', overflowX: e.overflowX || 'visible' },
      scrollWidth: px(e.scrollWidth === undefined ? e.width : e.scrollWidth), clientWidth: w,
      rect: { left: e.left || 0, right: (e.left || 0) + w, width: w, height: e.height === undefined ? 20 : e.height },
    };
  });
  (spec.els || []).forEach((e, i) => { els[i].parentElement = e.parent === undefined ? body : els[e.parent]; });
  for (const n of [body, ...els]) {
    n.getAttribute = (k) => (k in n.attrs ? n.attrs[k] : null);
    n.hasAttribute = (k) => k in n.attrs;
    n.getBoundingClientRect = () => n.rect || { left: 0, right: vw, width: vw, height: 0 };
  }
  body.querySelectorAll = () => els;
  body.querySelector = () => (spec.media ? {} : null);
  const width = spec.docW === 'content' ? Math.max(vw, ...els.map((n) => n.rect.right)) : vw;
  body.scrollWidth = width;
  const win = { innerWidth: vw, getComputedStyle: (n) => ({ visibility: 'visible', overflowX: 'visible', ...n.style }) };
  return { win, doc: { documentElement: { scrollWidth: width }, body } };
}

/** The synthetic Playwright (CommonJS, written as node_modules/playwright): real HTTP to the fixture, a fake page. */
function stubModule(module, require, dir, fakeDom) {
  const http = require('http');
  const fs = require('fs');
  const path = require('path');
  const FIX = JSON.parse(fs.readFileSync(path.join(dir, 'fixture.json'), 'utf8'));
  const PNG = Buffer.from('89504e470d0a1a0a', 'hex');
  const pause = (ms) => new Promise((ok) => setTimeout(ok, ms));
  const get = (url) => new Promise((ok, no) => {
    const q = http.get(url, (res) => { res.resume(); res.on('end', () => ok(res)); });
    q.on('error', (e) => no(new Error(`page.goto: net::ERR_CONNECTION_REFUSED at ${url} (${e.code})`)));
  });
  const page = (ctx) => {
    const st = { spec: null, url: null, seen: new Set(), actions: [], css: 0, ls: {}, server: '' };
    const need = async (sel, o) => {
      if (st.seen.has(sel)) return;
      const t = (o && o.timeout) || 0;
      await pause(Math.min(t, 30));
      throw new Error(`Timeout ${t}ms exceeded waiting for locator('${sel}')`);
    };
    return {
      async goto(url) {
        const res = await get(url);
        st.url = new URL(url);
        st.server = String(res.headers['x-pb-server'] || '');
        st.spec = FIX.pages[st.url.pathname] || FIX.notFound;
        st.seen = new Set(['body', ...st.spec.ids.map((i) => `#${i}`)]);
        st.ls = {};
        for (const s of ctx.init) new Function('localStorage', s)({ setItem: (k, v) => { st.ls[k] = String(v); } });
        return { status: () => res.statusCode };
      },
      async waitForLoadState() {},
      async waitForSelector(sel, o) { await need(sel, o); },
      async click(sel, o) {
        await need(sel, o);
        st.actions.push(`click ${sel}`);
        for (const i of (st.spec.reveals || {})[sel] || []) st.seen.add(`#${i}`);
      },
      async fill(sel, text, o) { await need(sel, o); st.actions.push(`fill ${sel}=${text}`); },
      async press(sel, key, o) { await need(sel, o); st.actions.push(`press ${sel}:${key}`); },
      async waitForTimeout(ms) { await pause(Math.min(ms, 10)); },
      async addStyleTag() { st.css += 1; },
      async evaluate(expr) {
        const { win, doc } = fakeDom(st.spec, ctx.opts.viewport.width);
        return new Function('window', 'document', `return (${expr});`)(win, doc);
      },
      async screenshot(o) {
        return Buffer.concat([PNG, Buffer.from(JSON.stringify({ path: st.url.pathname, viewport: ctx.opts.viewport,
          locale: ctx.opts.locale, timezone: ctx.opts.timezoneId, ls: st.ls, actions: st.actions, css: st.css,
          server: st.server, fullPage: !!(o && o.fullPage) }))]);
      },
      async close() {},
    };
  };
  const browser = (how) => ({
    how,
    async newContext(opts) {
      const ctx = { opts, init: [] };
      return { async addInitScript(s) { ctx.init.push(String(s)); }, async newPage() { return page(ctx); }, async close() {} };
    },
    async close() {},
  });
  module.exports = {
    chromium: {
      executablePath: () => FIX.exe,
      async launch(o) {
        if ((FIX.refuse || []).includes(o.executablePath || o.channel)) throw new Error('browserType.launch: Failed to launch');
        return browser(o);
      },
    },
  };
}

/** The fixture's answer: its page, or a 404 that RENDERS (a single-page shell or a proxy error page does). */
function answer(pages, tag, req, res) {
  const body = pages[req.url.split('?')[0]];
  res.writeHead(body === undefined ? 404 : 200, { 'content-type': 'text/html; charset=utf-8', 'x-pb-server': tag });
  res.end(body === undefined ? '<!doctype html><body><div id="app">404 - nothing here</div></body>' : body);
}

/** A stack for the owned-stack arms (serve.cjs): the fixture on a given port; it ends by itself after 60 s. */
function serveMain(require, process, answer) {
  const http = require('http');
  const fs = require('fs');
  const [port, tag, pagesFile] = process.argv.slice(2);
  const pages = JSON.parse(fs.readFileSync(pagesFile, 'utf8'));
  http.createServer((req, res) => answer(pages, tag, req, res)).listen(Number(port), '127.0.0.1');
  setTimeout(() => process.exit(0), 60000);
}

/** A launcher for the owned-stack arms (launch.cjs <verb> <port> <state>): start | start-fail | stop | stop-noop. */
function launcherMain(require, process, dir) {
  const { spawn } = require('child_process');
  const fs = require('fs');
  const path = require('path');
  const [verb, port, stateFile] = process.argv.slice(2);
  const st = fs.existsSync(stateFile) ? JSON.parse(fs.readFileSync(stateFile, 'utf8')) : { calls: [], pids: [], port: Number(port) };
  st.calls.push(`${verb}:${process.env.PB_TASK || ''}`);
  let code = 0;
  if (verb === 'start' || verb === 'start-fail') {
    const kid = spawn(process.execPath, [path.join(dir, 'serve.cjs'), port, 'owned', path.join(dir, 'pages.json')],
      { detached: true, stdio: 'ignore', windowsHide: true });
    kid.unref();
    st.pids.push(kid.pid);
    if (verb === 'start-fail') code = 3;
  } else if (verb === 'stop') {
    for (const pid of st.pids) { try { process.kill(pid); } catch (e) { /* gone */ } }
  }                                              // stop-noop stops nothing: the plant
  fs.writeFileSync(stateFile, JSON.stringify(st));
  if (code) process.stderr.write('half the stack started, then the launcher failed\n');
  process.exit(code);
}

function serveFixture(tag) {
  const srv = http.createServer((req, res) => answer(PAGES, tag, req, res));
  return new Promise((ok, no) => {
    srv.once('error', no);
    srv.listen(0, '127.0.0.1', () => ok({ srv, base: `http://127.0.0.1:${srv.address().port}` }));
  });
}

function freePort() {
  return new Promise((ok, no) => {
    const s = net.createServer();
    s.once('error', no);
    s.listen(0, '127.0.0.1', () => { const { port } = s.address(); s.close(() => ok(port)); });
  });
}

function childRun(args, env, cwd) {
  return new Promise((done) => {
    const k = spawn(process.execPath, [SELF, ...args], { cwd, env: { ...process.env, ...env }, windowsHide: true });
    let out = '';
    k.stdout.on('data', (b) => { out += b; });
    k.stderr.on('data', (b) => { out += b; });
    k.on('close', (code) => done({ code, out }));
  });
}

async function selftest(a) {
  const t0 = Date.now();
  const checks = [];
  const fails = [];
  const notes = [];
  let plants = 0;
  const check = (label, cond, detail = '') => {
    checks.push(label);
    if (label.startsWith('PLANT ')) plants += 1;
    if (!cond) fails.push(`${label}${detail ? ` :: ${String(typeof detail === 'string' ? detail : JSON.stringify(detail)).slice(0, 400)}` : ''}`);
    if (a.verbose && !a.json) console.log(`  ${cond ? 'ok  ' : 'FAIL'} ${label}`);
  };
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'pb-capture-selftest-'));
  const stackFiles = [];
  const saved = { ...TUNE };
  const savedEnv = { PB_TASK: process.env.PB_TASK, PB_CAPTURE_BROWSER: process.env.PB_CAPTURE_BROWSER };
  let seq = 0;
  const put = (name, obj) => {
    const p = path.join(tmp, name);
    fs.writeFileSync(p, typeof obj === 'string' ? obj : JSON.stringify(obj));
    return p;
  };
  const refusal = (fn) => { try { fn(); return null; } catch (e) { return e instanceof Refuse ? e.message : `wrong error: ${e}`; } };
  const refuses = (label, obj, want) => {
    const msg = refusal(() => loadManifest(put(`r${seq++}.json`, obj)));
    check(`PLANT ${label}: refused`, !!msg && !msg.startsWith('wrong error') && (!want || msg.includes(want)), msg || 'accepted');
  };
  const refusalOf = async (fn) => { try { await fn(); return null; } catch (e) { return e instanceof Refuse ? e.message : `wrong error: ${e && e.stack}`; } };
  const pngInfo = (dir, file) => {
    const b = fs.readFileSync(path.join(dir, file));
    return b.subarray(0, 8).toString('hex') === '89504e470d0a1a0a' ? JSON.parse(b.subarray(8).toString('utf8')) : null;
  };
  let srv = null;
  let holder = null;
  try {
    TUNE.delayMs = 1;
    TUNE.pollMs = 25;
    TUNE.graceMs = 3000;
    delete process.env.PB_CAPTURE_BROWSER;

    // 1 · the manifest and the plan: §A.5's shapes, the stem, the caption review_page.py reads
    const vps = parseViewports('1280x800,390x844');
    const ok = { id: 'home', path: '/', ready: '#app' };
    const one = loadManifest(put('a5.json', [{ ...ok, actions: [{ click: '#menu' }, { fill: ['#q', 'text'] }], states: [{ id: 'empty', actions: [{ wait: '#app' }] }] }]));
    check('the §A.5 example manifest loads', one.routes.length === 1 && one.routes[0].states.length === 1);
    check('route x state x viewport expands', plan(one, vps).length === 2);
    const two = plan(loadManifest(put('two.json', [ok, { id: 'board', path: '/board', states: [{ id: 'empty' }, { id: 'full' }] }])), parseViewports('390x844'));
    const stems = two.map((s) => s.stem).join(',');
    check('a route with no states captures one unnamed state; with states, exactly those', two.length === 3, stems);
    check('the stem is <id>[__<state>]__<w>x<h>', stems === 'home__390x844,board__empty__390x844,board__full__390x844', stems);
    const cap = captionOf(two[1], { document: false, px: 0, at: '' });
    check("a caption is review_page.py's entry: route = the route id, path = the path, no flag and no note",
      Object.keys(cap).join(',') === 'id,file,route,path,state,lang,viewport,overflow,overflow_doc,overflow_px,overflow_at,caption'
      && cap.route === 'board' && cap.path === '/board' && cap.state === 'empty' && cap.file === 'board__empty__390x844.png'
      && cap.overflow === false, cap);
    const caps = two.map((s) => captionOf(s, { document: false, px: 0, at: '' }));
    check('captured = declared, 0 failed -> GO', !tally(two, caps, [], false).reason);
    check('a missing capture -> NO-GO', /not captured/.test(tally(two, caps.slice(1), [], false).reason || ''));
    check('a failure -> NO-GO even with every PNG on disk', tally(two, caps, ['boom'], false).counts.failed === 1);
    const over = two.map((s) => captionOf(s, { document: false, px: 5, at: 'p' }));
    check('overflow alone is a count; --fail-on-overflow makes it the verdict',
      !tally(two, over, [], false).reason && /overflow/.test(tally(two, over, [], true).reason || ''));
    const bom = refusal(() => { if (loadManifest(put('bom.json', `\ufeff${JSON.stringify([ok])}`)).routes.length !== 1) throw new Refuse('no route read'); });
    check('a manifest saved with a byte-order mark reads the same', bom === null, bom);
    check('a key starting with "_" is a note, not a refusal', loadManifest(put('note.json', { _note: 'why', routes: [{ ...ok, _why: 'x' }] })).routes.length === 1);
    refuses('route with no id', [{ path: '/' }]);
    refuses('route with no path', [{ id: 'home' }]);
    refuses('two routes share an id', [ok, { ...ok, path: '/x' }]);
    refuses('state id carrying an underscore', [{ ...ok, states: [{ id: 'a_b' }] }]);
    refuses('route id carrying a double underscore', [{ ...ok, id: 'a__b' }]);
    refuses('unknown action key', [{ ...ok, actions: [{ tap: '#x' }] }]);
    refuses('malformed viewport', [{ ...ok, viewports: '1280' }]);
    refuses('a manifest that is neither a list nor an object carrying routes', { routes: 'home' });
    refuses('a misspelt top-level key (viewport for viewports)', { viewport: '390x844', routes: [ok] }, 'viewport');
    refuses('a misspelt route key (stats for states)', [{ ...ok, stats: [{ id: 'a' }] }], 'stats');
    refuses('languages with no lang_key', { langs: ['en'], routes: [ok] }, 'lang_key');
    refuses('a language breaking the id rule', { lang_key: 'k', langs: ['en_GB'], routes: [ok] }, 'en_GB');
    refuses('a boot with no stop command', { boot: { base: 'http://127.0.0.1:9', start: ['x'] }, routes: [ok] }, 'boot.stop');
    refuses('a boot whose base is no URL', { boot: { base: '127.0.0.1:9', start: ['x'], stop: ['y'] }, routes: [ok] }, 'boot.base');
    const shipped = path.join(HERE, 'routes.json');
    if (fs.existsSync(shipped)) {
      let r;
      try { r = plan(loadManifest(shipped), vps); } catch (e) { r = e.message; }
      check("the project's routes.json beside the tool loads and plans", Array.isArray(r) && r.length > 0, r);
    } else {
      notes.push(`no ${shown(shipped)} beside the tool: nothing of the project's to validate`);
    }

    // 2 · --only and --langs, on the plan
    const m = loadManifest(put('only.json', { lang_key: 'pb.lang', langs: ['en', 'fr'], routes: [ok, { id: 'board', path: '/board', states: [{ id: 'empty' }, { id: 'full' }] }] }));
    const v1 = parseViewports('390x844');
    const ids = (only, langs) => plan(m, v1, pickLangs(m, langs), onlyCells(m, only)).map((s) => s.stem).join(',');
    check('--only <route> keeps every state of it', ids('board', 'en') === 'board__empty__en__390x844,board__full__en__390x844', ids('board', 'en'));
    check('--only <route>/<state> keeps that one row, in manifest order beside a stateless route',
      ids('board/full,home', 'en') === 'home__en__390x844,board__full__en__390x844', ids('board/full,home', 'en'));
    check('PLANT --only with a typo beside a real name: refused, naming the typo', /"bord"/.test(refusal(() => ids('home,bord', 'en')) || ''), refusal(() => ids('home,bord', 'en')));
    check('PLANT --only naming a state the route does not declare: refused', /board\/nope/.test(refusal(() => ids('board/nope', 'en')) || ''));
    check("the manifest's langs: one capture per language, the language between state and viewport",
      ids(undefined, undefined) === 'home__en__390x844,home__fr__390x844,board__empty__en__390x844,board__empty__fr__390x844,board__full__en__390x844,board__full__fr__390x844',
      ids(undefined, undefined));
    check("--langs overrides the manifest's", ids('home', 'fr') === 'home__fr__390x844', ids('home', 'fr'));
    check('PLANT --langs on a manifest with no lang_key: refused', /lang_key/.test(refusal(() => pickLangs(loadManifest(put('nolk.json', [ok])), 'fr')) || ''));
    check('PLANT --langs naming no language: refused', /no language/.test(refusal(() => pickLangs(m, ',')) || ''));
    const store = {};
    new Function('localStorage', langScript('pb.lang', 'fr'))({ setItem: (k, v) => { store[k] = v; } });
    check('the language script sets lang_key to the language', store['pb.lang'] === 'fr', store);
    const lc = captionOf(plan(m, v1, ['en'], onlyCells(m, 'board/empty'))[0], { document: false, px: 0, at: '' });
    check("a language's caption: state reads <state> · <lang>, lang its own key", lc.state === 'empty · en' && lc.lang === 'en', lc);

    // 3 · the overflow probe, on a synthetic page: §A.5's document metric, and the element metric beside it
    const probe = (spec, vw) => { const { win, doc } = fakeDom(spec, vw); return new Function('window', 'document', `return ${OVERFLOW_JS};`)(win, doc); };
    const w390 = probe(FAKE_PAGES['/wide'], 390);
    const w1280 = probe(FAKE_PAGES['/wide'], 1280);
    check('a document wider than the viewport: flagged by both metrics; at a viewport it fits, by neither',
      w390.document === true && w390.px === 510 && w390.at === 'div#app' && w1280.document === false && w1280.px === 0, { w390, w1280 });
    const clip = probe(FAKE_PAGES['/clipped'], 1280);
    check('PLANT a clipping container with a wider child: §A.5\'s document metric misses it, the element one flags it (720 px, p#long)',
      clip.document === false && clip.px === 720 && clip.at === 'p#long', clip);
    const stack = probe(FAKE_PAGES['/stacked'], 1280);
    check('an aria-hidden stacked screen and a row that scrolls sideways are not overflow', stack.document === false && stack.px === 0, stack);
    const edge = (e) => probe({ docW: 'vw', els: [{ id: 'app', width: 'vw', overflowX: 'hidden' }, { id: 'x', parent: 0, ...e }] }, 1000).px;
    const edges = { onePx: edge({ width: 1001 }), twoPx: edge({ width: 1002 }), flat: edge({ width: 3000, height: 0 }),
      invisible: edge({ width: 3000, visibility: 'hidden' }), inert: edge({ width: 3000, inert: true }), shifted: edge({ width: 500, left: 800 }) };
    check('1 px past is not counted, 2 px is; a zero box, visibility:hidden and inert are not; a box pushed right is',
      edges.onePx === 0 && edges.twoPx === 2 && edges.flat === 0 && edges.invisible === 0 && edges.inert === 0 && edges.shifted === 300, edges);
    const alive = (spec) => new Function('window', 'document', `return ${ALIVE_JS};`)(...Object.values(fakeDom(spec, 800)));
    check('the blank probe: a body with nothing drawn is blank, a page with text is not', alive(FAKE_PAGES['/blank']) === false && alive(FAKE_PAGES['/ok']) === true);

    // 4 · §A.0: the retried replace, printed paths
    const target = path.join(tmp, 'atomic.txt');
    const readOr = (f) => { try { return fs.readFileSync(f, 'utf8'); } catch { return null; } };
    let n = 0;
    TUNE.rename = (x, y) => { n += 1; if (n < 3) { const e = new Error('held'); e.code = 'EBUSY'; throw e; } saved.rename(x, y); };
    try { writeAtomic(target, 'one\n'); } catch (e) { n = -n; }
    check('a replace refused while another process holds the file is retried, then lands', readOr(target) === 'one\n' && n === 3, `${n} tries`);
    n = 0;
    TUNE.rename = () => { n += 1; const e = new Error('held'); e.code = 'EPERM'; throw e; };
    let threw = false;
    try { writeAtomic(target, 'two\n'); } catch { threw = true; }
    TUNE.rename = saved.rename;
    check('PLANT a replace that never lands: bounded, the old file kept, the temp removed',
      threw && n === saved.tries && readOr(target) === 'one\n' && !fs.readdirSync(tmp).some((f) => f.startsWith('atomic.txt.tmp')), `${n} tries`);
    const inRoot = shown(path.join(ROOT, 'out', 'x.png'));
    const sibling = `${ROOT}x${path.sep}y`;
    check('printed paths: relative under the project root; a folder that only starts like the root is not cut',
      inRoot === path.join('out', 'x.png') && shown(ROOT) === '.' && !shown(sibling).startsWith(`x${path.sep}`), { inRoot, sibling: shown(sibling) });
    const home = os.homedir();
    if (home && home !== path.parse(home).root && !ROOT.startsWith(home + path.sep) && ROOT !== home) {
      check('printed paths: ~ under the home folder', shown(path.join(home, 'elsewhere')) === `~${path.sep}elsewhere`, shown(path.join(home, 'elsewhere')));
    } else if (home && home !== path.parse(home).root) {
      check('printed paths: ~ under the home folder, outside the root', shown(`${home}${path.sep}zz-outside`).startsWith('~'), shown(`${home}${path.sep}zz-outside`));
    }

    // 5 · the Playwright the project has, found from the manifest; the browser candidates
    const pwDir = path.join(tmp, 'pw');
    const pwMod = path.join(pwDir, 'node_modules', 'playwright');
    fs.mkdirSync(pwMod, { recursive: true });
    const exe = put('fake-browser', '');
    const other = put('other-browser', '');
    fs.writeFileSync(path.join(pwMod, 'package.json'), '{"name": "playwright", "main": "index.js"}\n');
    fs.writeFileSync(path.join(pwMod, 'fixture.json'), JSON.stringify({ exe, pages: FAKE_PAGES, notFound: FAKE_404 }));
    fs.writeFileSync(path.join(pwMod, 'index.js'), `'use strict';\nconst fakeDom = ${fakeDom};\n(${stubModule})(module, require, __dirname, fakeDom);\n`);
    const found = loadPlaywright(pwDir);
    check("Playwright is resolved from the manifest's folder, never a fixed one", found.name === 'playwright' && !!found.pw, found.name);
    check('PLANT a folder with no Playwright under it: none found', loadPlaywright(path.join(tmp, 'nowhere')).pw === null);
    const first = (env) => {
      if (env) process.env.PB_CAPTURE_BROWSER = env; else delete process.env.PB_CAPTURE_BROWSER;
      const c = browserCandidates({}, found.pw)[0];
      delete process.env.PB_CAPTURE_BROWSER;
      return c;
    };
    check('the first browser tried: PB_CAPTURE_BROWSER when set, else the pinned one',
      first(other).label === 'PB_CAPTURE_BROWSER' && first(other).executablePath === other && first(null).label === 'playwright pinned chromium', [first(other), first(null)]);

    // 6 · the capture, end to end, through the synthetic Playwright
    const fixture = await serveFixture('selftest');
    srv = fixture.srv;
    const { base } = fixture;
    const man = (routes, extra = {}) => put(`m${seq++}.json`, { playwright: pwDir, ...extra, routes });
    const cap6 = (file, extra = {}) => runCapture({ routes: file, base, timeoutMs: 400, ...extra });
    const okRoute = { id: 'ok', path: '/ok', ready: '#app', states: [{ id: 'base' }, { id: 'open', actions: [{ click: '#more' }, { wait: '#panel' }] }] };
    const out1 = path.join(tmp, 'shots');
    const r1 = await cap6(man([okRoute], { lang_key: 'pb.lang', langs: ['en', 'fr'], locale: 'en-GB', timezone: 'Etc/GMT-3' }), { out: out1 });
    const c1 = JSON.parse(fs.readFileSync(path.join(out1, 'captions.json'), 'utf8'));
    const i1 = Object.fromEntries(c1.map((c) => [c.id, pngInfo(out1, c.file)]));
    check('captured = declared = 8 (2 states x 2 languages x 2 viewports), GO', !r1.reason && !r1.fails.length && r1.counts.captured === 8, r1);
    check("captions.json lists every capture, in the plan's order",
      c1.map((c) => c.id).join(',') === plan(loadManifest(man([okRoute], { lang_key: 'pb.lang', langs: ['en', 'fr'] })), vps, ['en', 'fr']).map((s) => s.stem).join(','), c1.map((c) => c.id));
    check('each language reached its own page, set before its first script',
      c1.every((c) => i1[c.id] && i1[c.id].ls['pb.lang'] === c.lang), c1.map((c) => [c.id, i1[c.id] && i1[c.id].ls]));
    check("the manifest's locale and timezone reach the browser, never a value of the code's",
      Object.values(i1).every((x) => x.locale === 'en-GB' && x.timezone === 'Etc/GMT-3'), Object.values(i1)[0]);
    check("a state's actions ran on its own captures only",
      i1['ok__open__en__390x844'].actions.join() === 'click #more' && i1['ok__base__en__390x844'].actions.length === 0, [i1['ok__open__en__390x844'], i1['ok__base__en__390x844']]);
    check('every shot: its viewport, full page, animations frozen, from the stack at --base',
      c1.every((c) => i1[c.id].viewport.width === Number(c.viewport.split('x')[0]) && i1[c.id].fullPage && i1[c.id].css === 1 && i1[c.id].server === 'selftest'), Object.values(i1)[0]);
    const out2 = path.join(tmp, 'defaults');
    await cap6(man([{ id: 'ok', path: '/ok', ready: '#app' }]), { out: out2, viewports: '390x844' });
    const i2 = pngInfo(out2, 'ok__390x844.png');
    check('no locale or timezone in the manifest: en-US and UTC, the same on every box', i2 && i2.locale === 'en-US' && i2.timezone === 'UTC', i2);
    const out3 = path.join(tmp, 'overflow');
    const oroutes = [{ id: 'wide', path: '/wide', ready: '#app' }, { id: 'clipped', path: '/clipped', ready: '#long' }, { id: 'stacked', path: '/stacked', ready: '#front' }];
    const r3 = await cap6(man(oroutes), { out: out3 });
    const c3 = Object.fromEntries(JSON.parse(fs.readFileSync(path.join(out3, 'captions.json'), 'utf8')).map((c) => [c.id, c]));
    check('overflow reaches the captions: the document at 390 (wide), an element at both widths (clipped), nothing where it fits',
      c3['wide__390x844'].overflow && c3['wide__390x844'].overflow_doc && !c3['wide__1280x800'].overflow
      && c3['clipped__1280x800'].overflow && !c3['clipped__1280x800'].overflow_doc && c3['clipped__1280x800'].overflow_px === 720
      && c3['clipped__1280x800'].overflow_at === 'p#long' && !c3['stacked__1280x800'].overflow && !c3['stacked__390x844'].overflow, c3);
    check('overflow counted (3), and GO without --fail-on-overflow', !r3.reason && r3.counts.overflow === 3, r3.counts);
    const r3b = await cap6(man([oroutes[1]]), { out: path.join(tmp, 'overflow-b'), failOnOverflow: true });
    check('PLANT the clipped screen with --fail-on-overflow: NO-GO on the element metric alone', /overflow/.test(r3b.reason || ''), r3b);
    const dead = await cap6(man([{ id: 'dead', path: '/dead', ready: '#app' }]), { out: path.join(tmp, 'dead'), viewports: '390x844' });
    check('PLANT a dead path that RENDERS a 404 page: NO-GO on the status alone', dead.counts.captured === 0 && /HTTP 404/.test(dead.fails.join(' ')), dead);
    const blank = await cap6(man([{ id: 'blank', path: '/blank', ready: 'body' }]), { out: path.join(tmp, 'blank'), viewports: '390x844' });
    check('PLANT a page that renders BLANK: NO-GO (ready matched, nothing drawn)', blank.counts.captured === 0 && /BLANK/.test(blank.fails.join(' ')), blank);
    const never = await cap6(man([{ id: 'nope', path: '/ok', ready: '#nothing-here' }]), { out: path.join(tmp, 'never'), viewports: '390x844' });
    check('PLANT a ready selector that never appears: NO-GO', never.counts.captured === 0 && !!never.reason && /nothing-here/.test(never.fails.join(' ')), never);
    const act = await cap6(man([{ id: 'act', path: '/ok', ready: '#app', actions: [{ click: '#missing' }] }]), { out: path.join(tmp, 'act'), viewports: '390x844' });
    check('PLANT an action on a node that is not there: NO-GO naming it', act.counts.captured === 0 && /#missing/.test(act.fails.join(' ')), act);
    const r4 = await cap6(man([{ id: 'ok', path: '/ok', ready: '#app' }]), { out: out1, viewports: '390x844' });
    check('PLANT a stale PNG is NAMED and never deleted', r4.notes.some((x) => /not this run's/.test(x)) && fs.existsSync(path.join(out1, 'ok__open__fr__1280x800.png')), r4.notes);
    const r5 = await cap6(man([okRoute], { lang_key: 'pb.lang', langs: ['en', 'fr'] }), { out: path.join(tmp, 'only'), only: 'ok/open', langs: 'fr', viewports: '390x844' });
    check('--only and --langs through the capture: one shot, the one named', !r5.reason && r5.counts.declared === 1
      && fs.readdirSync(path.join(tmp, 'only')).filter((f) => f.endsWith('.png')).join() === 'ok__open__fr__390x844.png', r5);
    const deadBase = `http://127.0.0.1:${await freePort()}`;
    const nb = await refusalOf(() => runCapture({ routes: man([ok]), base: deadBase, out: path.join(tmp, 'unwritten') }));
    check('PLANT a --base that does not answer: refused, and --out never created', /not answering/.test(nb || '') && !fs.existsSync(path.join(tmp, 'unwritten')), nb);
    const nopw = await refusalOf(() => runCapture({ routes: put(`m${seq++}.json`, { playwright: path.join(tmp, 'nowhere'), routes: [ok] }), base, out: path.join(tmp, 'unwritten') }));
    check('PLANT no Playwright where the manifest points: refused, naming the folder, nothing written',
      /Playwright is not on disk/.test(nopw || '') && /nowhere/.test(nopw || '') && !fs.existsSync(path.join(tmp, 'unwritten')), nopw);

    // 7 · --boot-seed: the stack this run owns, through a synthetic launcher
    put('pages.json', PAGES);
    put('serve.cjs', `'use strict';\nconst answer = ${answer};\n(${serveMain})(require, process, answer);\n`);
    const launcher = put('launch.cjs', `'use strict';\n(${launcherMain})(require, process, __dirname);\n`);
    const port = await freePort();
    const bootBase = `http://127.0.0.1:${port}`;
    const boot = (startVerb, stopVerb, at = bootBase) => {
      const st = path.join(tmp, `stack${seq}.json`);
      stackFiles.push(st);
      const p = at.split(':').pop();
      const file = put(`b${seq++}.json`, { playwright: pwDir, boot: { base: at, deadline_s: 5,
        start: [process.execPath, launcher, startVerb, p, st], stop: [process.execPath, launcher, stopVerb, p, st] }, routes: [{ id: 'ok', path: '/ok', ready: '#app' }] });
      return { file, st, calls: () => (fs.existsSync(st) ? JSON.parse(fs.readFileSync(st, 'utf8')).calls : []) };
    };
    const own = (b, extra = {}) => runCapture({ routes: b.file, bootSeed: true, task: 'SELFTEST', out: path.join(tmp, `own${seq++}`), viewports: '390x844', timeoutMs: 400, ...extra });
    const b1 = boot('start', 'stop');
    const o1 = await own(b1);
    const o1dir = path.join(tmp, `own${seq - 1}`);
    check('--boot-seed: start, capture against the stack it started, stop - GO', !o1.reason && !o1.fails.length && o1.counts.captured === 1, o1);
    check('the capture came from the owned stack', (pngInfo(o1dir, 'ok__390x844.png') || {}).server === 'owned', pngInfo(o1dir, 'ok__390x844.png'));
    check('start then stop, once each, both keyed by the task', b1.calls().join() === 'start:SELFTEST,stop:SELFTEST', b1.calls());
    check('the stack is quiet after the stop, and the run says it owned it',
      !(await answering(bootBase)) && o1.notes.some((x) => /started and stopped by this run/.test(x)), o1.notes);
    ({ srv: holder } = await serveFixture('holder'));
    const held = `http://127.0.0.1:${holder.address().port}`;
    const b2 = boot('start', 'stop', held);
    const o2 = await refusalOf(() => own(b2, { out: path.join(tmp, 'unwritten') }));
    check("PLANT the stack's base already answers: NOT RUN, the launcher never called, nothing written",
      (o2 || '').includes('NOT RUN (a stack this run did not start)') && b2.calls().length === 0 && !fs.existsSync(path.join(tmp, 'unwritten')), o2);
    check('the holder still answers after the refusal', await answering(held));
    const b3 = boot('start-fail', 'stop');
    const o3 = await own(b3);
    check('PLANT a start that fails after starting half the stack: NO-GO naming it, and the half stopped',
      !!o3.reason && /start command exited 3/.test(o3.fails.join(' ')) && b3.calls().join() === 'start-fail:SELFTEST,stop:SELFTEST' && !(await answering(bootBase)), o3);
    const b4 = boot('start', 'stop-noop');
    TUNE.graceMs = 300;
    const o4 = await own(b4);
    TUNE.graceMs = 3000;
    check('PLANT a stop that leaves the stack running: NO-GO, still answering after the stop', /still answers after the stop/.test(o4.fails.join(' ')), o4);
    for (const pid of JSON.parse(fs.readFileSync(b4.st, 'utf8')).pids) { try { process.kill(pid); } catch { /* gone */ } }
    await settle(bootBase, false, 3000);
    const b5 = boot('start', 'stop');
    const blocker = put('a-file', '');
    const o5 = await refusalOf(() => own(b5, { out: path.join(blocker, 'below-a-file') }));
    check('PLANT a capture that crashes mid-run: the stack it started is still stopped',
      !!o5 && b5.calls().join() === 'start:SELFTEST,stop:SELFTEST' && !(await answering(bootBase)), [o5, b5.calls()]);
    delete process.env.PB_TASK;
    const o6 = await refusalOf(() => own(boot('start', 'stop'), { task: undefined }));
    if (savedEnv.PB_TASK !== undefined) process.env.PB_TASK = savedEnv.PB_TASK;
    check('PLANT --boot-seed with no --task and no $PB_TASK: refused', /needs --task/.test(o6 || ''), o6);
    const o7 = await refusalOf(() => own(boot('start', 'stop'), { base }));
    check('PLANT --boot-seed beside --base: refused', /two different stacks/.test(o7 || ''), o7);

    // 8 · the command line: §A.5's flag interface, run as a child
    const cliMan = man([{ id: 'ok', path: '/ok', ready: '#app' }, { id: 'wide', path: '/wide', ready: '#app' }]);
    const cliOut = path.join(tmp, 'cli');
    const [c1r, c2r, c3r, c4r, c5r, c6r, c7r] = await Promise.all([
      childRun(['--routes', cliMan, '--base', base, '--out', cliOut, '--viewports', '1280x800,390x844'], {}, tmp),
      childRun(['--routes', cliMan, '--base', base, '--out', path.join(tmp, 'cli2'), '--viewports', '390x844', '--fail-on-overflow'], {}, tmp),
      childRun(['--routes', cliMan, '--base', base, '--out', path.join(tmp, 'cli3'), '--viewport', '390x844'], {}, tmp),
      childRun(['--routes', cliMan, '--base', base, '--out'], {}, tmp),
      childRun(['--routes', cliMan, '--base', base], {}, tmp),
      childRun(['-h'], {}, tmp),
      childRun(['--routes', cliMan, '--base', base, '--out', path.join(tmp, 'cli7'), '--only', 'ok', '--json'], {}, tmp),
    ]);
    const last = (r) => r.out.trim().split('\n').pop();
    check("§A.5's literal call: --routes --base --out --viewports -> GO, captions.json beside the PNGs",
      c1r.code === 0 && last(c1r) === '=== GO ===' && /declared=4 · captured=4 · failed=0 · overflow=1/.test(c1r.out)
      && JSON.parse(fs.readFileSync(path.join(cliOut, 'captions.json'), 'utf8')).length === 4, c1r.out);
    check('--fail-on-overflow on the command line: exit 1, NO-GO', c2r.code === 1 && /NO-GO: 1 capture\(s\) overflow/.test(last(c2r)), c2r.out);
    check('PLANT an unknown option (--viewport): exit 1, refused by name, nothing written',
      c3r.code === 1 && /unknown option --viewport /.test(c3r.out) && !fs.existsSync(path.join(tmp, 'cli3')), c3r.out);
    check('PLANT a value flag with no value (--out last): exit 1, refused', c4r.code === 1 && /--out needs a value/.test(c4r.out), c4r.out);
    check('PLANT no --out: exit 1, refused naming it', c5r.code === 1 && /--out <dir> is required/.test(c5r.out), c5r.out);
    check('-h: the usage, exit 0', c6r.code === 0 && ['--routes', '--base', '--out', '--viewports', '--fail-on-overflow', '--boot-seed', '--only', '--langs'].every((f) => c6r.out.includes(f)), c6r.out);
    let j = null;
    try { j = JSON.parse(c7r.out.trim()); } catch { /* checked below */ }
    check('--json: one JSON object, its verdict and counts', c7r.code === 0 && j && j.verdict === 'GO' && j.counts.captured === 2, c7r.out);

    // 9 · the consumer, when it sits beside this file: review_page.py reads the capture set as written
    const consumer = path.join(HERE, 'review_page.py');
    const py = process.env.PB_PYTHON || (process.platform === 'win32' ? 'python' : 'python3');
    const build = () => runCmd([py, consumer, 'build', '--images', cliOut, '--report', path.join(tmp, 'review1.md')], process.env, 60000);
    const rp0 = fs.existsSync(consumer) ? await build() : null;
    if (!rp0 || rp0.status === null) {
      notes.push(`consumer check NOT RUN (${rp0 ? `${py} ${rp0.why}` : `no ${shown(consumer)} beside the tool`})`);
    } else {
      check('review_page.py reads every capture, and shows none until the TV flags one',
        rp0.status === 1 && /captures 4 · flagged 0/.test(rp0.tail) && /nothing flagged/.test(rp0.tail), rp0.tail);
      const cj = path.join(cliOut, 'captions.json');
      const list = JSON.parse(fs.readFileSync(cj, 'utf8'));
      list.find((x) => x.id === 'wide__390x844').flag = true;
      list.find((x) => x.id === 'wide__390x844').note = 'wider than the phone';
      fs.writeFileSync(cj, JSON.stringify(list));
      const rp1 = await build();
      check('review_page.py builds its page once the TV flags a capture with its note', rp1.status === 0 && /flagged 1/.test(rp1.tail), rp1.tail);
    }

    if (a.live) await liveTier(a, tmp, check, { base, put, launcher, freePort, stackFiles });
    else notes.push("live tier NOT RUN (--live: a real browser, through the project's Playwright)");
  } catch (e) {
    fails.push(`the selftest itself crashed: ${e && e.stack ? e.stack.split('\n').slice(0, 3).join(' | ') : e}`);
  } finally {
    Object.assign(TUNE, saved);
    for (const [k, v] of Object.entries(savedEnv)) { if (v === undefined) delete process.env[k]; else process.env[k] = v; }
    for (const f of stackFiles) {
      let st = null;
      try { st = JSON.parse(fs.readFileSync(f, 'utf8')); } catch { continue; }
      if (await answering(`http://127.0.0.1:${st.port}`)) for (const pid of st.pids) { try { process.kill(pid); } catch { /* gone */ } }
    }
    for (const s of [srv, holder]) if (s) await new Promise((done) => s.close(done));
    fs.rmSync(tmp, { recursive: true, force: true });
  }
  const counts = { checks: checks.length, failed: fails.length, plants, live: a.live ? 1 : 0, seconds: ((Date.now() - t0) / 1000).toFixed(2) };
  return verdict(a, 'selftest', counts, fails, notes);
}

/** --live: the same fixtures in a real browser, through the project's Playwright. */
async function liveTier(a, tmp, check, t) {
  const pwDir = a.playwright ? path.resolve(a.playwright) : ROOT;
  const { pw } = loadPlaywright(pwDir);
  check('live: the project\'s Playwright is on disk', !!pw, shown(pwDir));
  if (!pw) return;
  let n = 0;
  const man = (routes, extra = {}) => t.put(`live${n++}.json`, { playwright: pwDir, ...extra, routes });
  const out = path.join(tmp, 'live');
  const run = (routes, extra = {}, mextra = {}) => runCapture({ routes: man(routes, mextra), base: t.base, out, browser: a.browser, channel: a.channel, ...extra });
  const png = (f) => { const b = fs.readFileSync(path.join(out, f)); return b.subarray(0, 8).toString('hex') === '89504e470d0a1a0a' ? b : null; };
  const okRoute = { id: 'ok', path: '/ok', ready: '#app', states: [{ id: 'base' }, { id: 'open', actions: [{ click: '#more' }, { wait: '#panel' }] }] };
  const r = await run([okRoute], { viewports: '390x844' }, { lang_key: 'pb.lang', langs: ['en', 'fr'] });
  const caps = JSON.parse(fs.readFileSync(path.join(out, 'captions.json'), 'utf8'));
  check('live: 4 real PNGs (2 states x 2 languages) and captions.json matches the folder',
    !r.reason && r.counts.captured === 4 && caps.length === 4 && caps.every((c) => png(c.file)), r);
  check("live: the state's actions really ran (its PNG differs from the base state's)",
    !png('ok__base__en__390x844.png').equals(png('ok__open__en__390x844.png')));
  check('live: the language reached the page (the two languages\' PNGs differ)',
    !png('ok__base__en__390x844.png').equals(png('ok__base__fr__390x844.png')));
  const o = await run([{ id: 'wide', path: '/wide', ready: '#app' }, { id: 'clipped', path: '/clipped', ready: '#long' }, { id: 'stacked', path: '/stacked', ready: '#front' }]);
  const c = Object.fromEntries(JSON.parse(fs.readFileSync(path.join(out, 'captions.json'), 'utf8')).map((x) => [x.id, x]));
  check('live: the document metric - wide overflows at 390, not at 1280', c['wide__390x844'].overflow_doc === true && c['wide__1280x800'].overflow === false, c);
  check('PLANT live: a clipping container with a wider child - the document metric misses it, the element one flags it',
    c['clipped__1280x800'].overflow_doc === false && c['clipped__1280x800'].overflow_px > 600 && c['clipped__1280x800'].overflow_at === 'p#long', c['clipped__1280x800']);
  check('live: an aria-hidden stacked screen and a row that scrolls sideways are not overflow',
    c['stacked__1280x800'].overflow === false && c['stacked__390x844'].overflow === false, [c['stacked__1280x800'], c['stacked__390x844']]);
  check('live: --fail-on-overflow turns it NO-GO', /overflow/.test((await run([{ id: 'clipped', path: '/clipped', ready: '#long' }], { failOnOverflow: true })).reason || ''));
  const dead = await run([{ id: 'dead', path: '/dead', ready: '#app' }], { timeoutMs: 3000, viewports: '390x844' });
  check('PLANT live: a dead path that RENDERS a 404 page: NO-GO on the status alone', dead.counts.captured === 0 && /HTTP 404/.test(dead.fails.join(' ')), dead.fails);
  const blank = await run([{ id: 'blank', path: '/blank', ready: 'body' }], { timeoutMs: 3000, viewports: '390x844' });
  check('PLANT live: a page that renders BLANK: NO-GO', blank.counts.captured === 0 && /BLANK/.test(blank.fails.join(' ')), blank.fails);
  const never = await run([{ id: 'nope', path: '/ok', ready: '#nothing-here' }], { timeoutMs: 3000, viewports: '390x844' });
  check('PLANT live: a ready selector that never appears: NO-GO', never.counts.captured === 0 && !!never.reason, never.fails);
  const port = await t.freePort();
  const st = path.join(tmp, 'live-stack.json');
  t.stackFiles.push(st);
  const bootFile = t.put('live-boot.json', { playwright: pwDir, boot: { base: `http://127.0.0.1:${port}`, deadline_s: 10,
    start: [process.execPath, t.launcher, 'start', String(port), st], stop: [process.execPath, t.launcher, 'stop', String(port), st] },
  routes: [{ id: 'ok', path: '/ok', ready: '#app' }] });
  const b = await runCapture({ routes: bootFile, bootSeed: true, task: 'SELFTEST', out: path.join(tmp, 'live-own'), viewports: '390x844', browser: a.browser, channel: a.channel });
  check('live: --boot-seed starts the stack, captures it in a real browser, stops it', !b.reason && b.counts.captured === 1
    && !(await answering(`http://127.0.0.1:${port}`)), b);
}

// ------------------------------------------------------------------ main

async function main(argv) {
  let a;
  try {
    a = parseArgs(argv);
  } catch (e) {
    if (!(e instanceof Refuse)) throw e;
    return verdict({ json: argv.includes('--json') }, 'capture', { declared: 0, captured: 0, failed: 0 }, [], [], `${e.message} - see -h`);
  }
  if (a.help) {
    console.log(USAGE);
    return 0;
  }
  try {
    if (a.cmd === 'selftest') return await selftest(a);
    if (a.cmd === 'browsers') return cmdBrowsers(a);
    const r = await runCapture(a, a.json ? () => {} : (s) => console.log(shown(s)));
    return verdict(a, 'capture', r.counts, r.fails, r.notes, r.reason);
  } catch (e) {
    if (e instanceof Refuse) return verdict(a, a.cmd, { declared: 0, captured: 0, failed: 0 }, [], [], e.message);
    process.stderr.write(`${e && e.stack ? e.stack : e}\n`);
    return verdict(a, a.cmd, { declared: 0, captured: 0, failed: 0 }, [], [], `crashed: ${String((e && e.message) || e).split('\n')[0]}`);
  }
}

process.exitCode = await main(process.argv.slice(2));
