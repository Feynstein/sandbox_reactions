#!/usr/bin/env node
/* web/smoke.mjs - the web smoke (G-WEB; m0_contrat.md §5.4, §6.2.3). Project code, M0-T15.
 *
 *   node web/smoke.mjs <ready|no-webgpu> [--base http://127.0.0.1:47812] [--timeout-s 30]
 *
 * Launches the box's installed Chrome through playwright-core, headless, with §6.2.3's flags (never `--no-sandbox`):
 *   linux-pc   /usr/bin/google-chrome   --headless=new --use-angle=vulkan --enable-features=Vulkan
 *                                       --disable-vulkan-surface --enable-unsafe-webgpu
 *   win-laptop C:\Program Files\Google\Chrome\Application\chrome.exe   --headless=new --enable-unsafe-webgpu
 * playwright-core appends `--no-sandbox` unless `chromiumSandbox` is on, so the launch turns it on and every case reads
 * the launched Chrome's command line back from chrome://version: `sandbox: on` in the log, or NO-GO naming `--no-sandbox`
 * (M0-D17). SR_CHROME overrides the executable. The page is served by `launch.py start web` (tools/pb/launch.json), never by this file.
 * Cases (the `web` scope's, tests/web/check.sh):
 *   ready      `/` sets document.body.dataset.srState to "ready" within 30 s (srError, "error" or "no-webgpu" ends it early);
 *              srAdapter is printed as `SR-ADAPTER <description>`, and a software adapter is named in a NOTE (it reaches
 *              "ready" without proving the GPU path). The preset case (`?preset=sun&steps=200`, canvas pixels) is M0-T94's.
 *   no-webgpu  `/?no-webgpu=1` sets srState "no-webgpu", shows the three web.no_webgpu_* texts exactly as
 *              crates/sr-app/src/strings.rs holds them (G-STR keeps that file equal to §4.1) and never fetches the wasm.
 * Ends in `=== GO ===` (exit 0) or `=== NO-GO: <reason> ===` (exit 1).
 */
import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..');
const CASES = ['ready', 'no-webgpu'];

const args = process.argv.slice(2);
const flag = (name, dflt) => { const i = args.indexOf(name); return i >= 0 && args[i + 1] ? args[i + 1] : dflt; };
const kase = args.find((a, i) => !a.startsWith('--') && !(i > 0 && args[i - 1].startsWith('--')));
const base = flag('--base', 'http://127.0.0.1:47812').replace(/\/$/, '');
const timeoutMs = Number(flag('--timeout-s', '30')) * 1000;

const nogo = (why) => { console.log(`=== NO-GO: ${why} ===`); process.exit(1); };
if (!CASES.includes(kase)) nogo(`usage: node web/smoke.mjs <${CASES.join('|')}> [--base <url>] [--timeout-s <n>]`);

let chromium;
try {
  ({ chromium } = createRequire(import.meta.url)('playwright-core'));
} catch (e) {
  nogo('playwright-core is not installed (npm ci in web/)');
}

const win = process.platform === 'win32';
const chrome = process.env.SR_CHROME || (win ? 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe' : '/usr/bin/google-chrome');
if (!fs.existsSync(chrome)) nogo(`Chrome not found at ${chrome} (contract §6.2.3)`);
const chromeArgs = win
  ? ['--headless=new', '--enable-unsafe-webgpu']
  : ['--headless=new', '--use-angle=vulkan', '--enable-features=Vulkan', '--disable-vulkan-surface', '--enable-unsafe-webgpu'];

// The three texts come from the string table G-STR holds equal to §4.1.
function noWebgpuTexts() {
  const src = fs.readFileSync(path.join(ROOT, 'crates', 'sr-app', 'src', 'strings.rs'), 'utf8');
  const get = (key) => {
    const m = src.match(new RegExp(`\\("${key.replace('.', '\\.')}",\\s*"([^"]*)"\\)`));
    if (!m) nogo(`crates/sr-app/src/strings.rs has no ${key}`);
    return m[1];
  };
  return { title: get('web.no_webgpu_title'), body: get('web.no_webgpu_body'), browsers: get('web.no_webgpu_browsers') };
}

const browser = await chromium.launch({ executablePath: chrome, headless: true, args: chromeArgs, chromiumSandbox: true });
let verdict;
try {
  const version = browser.version();
  console.log(`web smoke ${kase}: ${chrome} (${version}) on ${process.platform}, flags ${chromeArgs.join(' ')}, ${base}`);
  // §6.2.3 leaves `--no-sandbox` out: the smoke proves WebGPU in the browser a player runs, so Chrome's own command line decides.
  const probe = await browser.newPage();
  await probe.goto('chrome://version');
  const cmdline = await probe.evaluate(() => document.querySelector('#command_line')?.textContent ?? null);
  await probe.close();
  if (cmdline === null) throw new Error('chrome://version shows no #command_line: the sandbox state cannot be read');
  const sandboxOff = /(^|\s)--no-sandbox(\s|$)/.test(cmdline);
  console.log(`sandbox: ${sandboxOff ? 'OFF (--no-sandbox in' : 'on (--no-sandbox absent from'} Chrome's command line)`);
  if (sandboxOff) throw new Error('Chrome runs with --no-sandbox, which contract §6.2.3 leaves out (playwright-core adds it unless chromiumSandbox is on)');
  const page = await browser.newPage();
  const fetched = [];
  const problems = [];
  page.on('request', (r) => fetched.push(new URL(r.url()).pathname));
  page.on('pageerror', (e) => problems.push(`pageerror: ${String(e).split('\n')[0]}`));
  page.on('console', (m) => { if (m.type() === 'error' || m.type() === 'warning') problems.push(`console.${m.type()}: ${m.text().split('\n')[0].slice(0, 200)}`); });
  page.on('requestfailed', (r) => problems.push(`requestfailed: ${new URL(r.url()).pathname} ${r.failure()?.errorText ?? ''}`));
  page.on('response', (r) => { if (r.status() >= 400) problems.push(`HTTP ${r.status()}: ${new URL(r.url()).pathname}`); });
  const states = kase === 'ready' ? ['ready', 'error', 'no-webgpu'] : ['no-webgpu', 'ready', 'error'];
  const url = kase === 'ready' ? `${base}/` : `${base}/?no-webgpu=1`;
  const resp = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: timeoutMs });
  if (!resp || !resp.ok()) throw new Error(`GET ${url} answered ${resp ? resp.status() : 'nothing'}`);
  const started = Date.now();
  let state;
  try {
    await page.waitForFunction((s) => s.includes(document.body.dataset.srState), states, { timeout: timeoutMs, polling: 100 });
    state = await page.evaluate(() => document.body.dataset.srState);
  } catch (e) {
    state = await page.evaluate(() => document.body.dataset.srState ?? '(unset)');
    throw new Error(`srState is "${state}" after ${timeoutMs / 1000} s; never ${states[0]}${problems.length ? ' - ' + problems.join(' | ') : ''}`);
  }
  const took = ((Date.now() - started) / 1000).toFixed(1);
  const info = await page.evaluate(() => ({
    adapter: document.body.dataset.srAdapter ?? null, error: document.body.dataset.srError ?? document.body.dataset.srErrorDetail ?? null,
  }));
  if (info.adapter) console.log(`SR-ADAPTER ${info.adapter}`);
  // wgpu on the web reads little of the adapter (Chrome hides it), so Chrome's own answer is recorded too.
  const chromeAdapter = await page.evaluate(async () => {
    try {
      const a = await navigator.gpu.requestAdapter({ powerPreference: 'high-performance' });
      if (!a) return null;
      const i = a.info ?? {};
      return { fallback: !!a.isFallbackAdapter, vendor: i.vendor ?? '', architecture: i.architecture ?? '', device: i.device ?? '', description: i.description ?? '' };
    } catch (e) { return null; }
  });
  if (chromeAdapter) {
    console.log(`SR-CHROME-ADAPTER vendor=${chromeAdapter.vendor} architecture=${chromeAdapter.architecture} device=${chromeAdapter.device} description=${chromeAdapter.description} fallback=${chromeAdapter.fallback}`);
  }

  if (kase === 'ready') {
    if (state !== 'ready') throw new Error(`srState "${state}"${info.error ? ` (${info.error})` : ''} - not "ready"${problems.length ? '; ' + problems.join(' | ') : ''}`);
    if (!info.adapter) throw new Error('srState "ready" but srAdapter is not set');
    const named = `${info.adapter} ${chromeAdapter ? Object.values(chromeAdapter).join(' ') : ''}`;
    if (!chromeAdapter || chromeAdapter.fallback || /swiftshader|llvmpipe|software|basic render|warp/i.test(named)) {
      console.log(`NOTE the adapter reads as software or unknown (${named.trim()}): "ready" does not prove the GPU path`);
    }
    console.log(`ready: srState "ready" after ${took} s`);
  } else {
    if (state !== 'no-webgpu') throw new Error(`srState "${state}", not "no-webgpu"`);
    const want = noWebgpuTexts();
    const got = await page.evaluate(() => {
      const box = document.getElementById('sr-no-webgpu');
      const visible = !!box && !box.hidden && box.getBoundingClientRect().height > 0;
      return { visible, h1: [...box.querySelectorAll('h1')].map((n) => n.textContent), p: [...box.querySelectorAll('p')].map((n) => n.textContent),
               canvas: !!document.getElementById('sr-canvas'), loadingHidden: document.getElementById('sr-loading').hidden };
    });
    const bad = [];
    if (!got.visible) bad.push('the no-WebGPU page is not shown');
    if (got.h1.length !== 1 || got.h1[0] !== want.title) bad.push(`heading ${JSON.stringify(got.h1)} is not web.no_webgpu_title`);
    if (got.p.length !== 2 || got.p[0] !== want.body) bad.push(`first paragraph ${JSON.stringify(got.p[0])} is not web.no_webgpu_body`);
    if (got.p.length !== 2 || got.p[1] !== want.browsers) bad.push(`second paragraph ${JSON.stringify(got.p[1])} is not web.no_webgpu_browsers`);
    if (!got.loadingHidden) bad.push('the loading line is still shown');
    const loaded = fetched.filter((p) => /\.wasm$|sandbox-reactions\.js$/.test(p));
    if (loaded.length) bad.push(`the wasm side was fetched: ${loaded.join(', ')}`);
    if (bad.length) throw new Error(bad.join('; '));
    console.log(`no-webgpu: srState "no-webgpu" after ${took} s, the three web.no_webgpu_* texts exact, no wasm fetched`);
  }
  verdict = null;
} catch (e) {
  verdict = e.message || String(e);
} finally {
  await browser.close();
}
if (verdict) nogo(`${kase}: ${verdict}`);
console.log('=== GO ===');
