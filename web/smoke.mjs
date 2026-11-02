#!/usr/bin/env node
/* web/smoke.mjs - the web smoke (G-WEB; m0_contrat.md §5.4, §6.2.3). Project code, M0-T15.
 *
 *   node web/smoke.mjs <ready|no-webgpu|preset> [--base http://127.0.0.1:47840] [--timeout-s 30]
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
 *              "ready" without proving the GPU path).
 *   preset     `/?preset=sun&steps=200` reaches "ready" within 30 s and the canvas's central 100 x 100 px are not all black (M0-T94).
 *              The pixels come from a Chrome screenshot of that clip. "Not all black" is true of an empty canvas too (bg.space
 *              #05070D is not black), so the log also counts the central pixels that differ from bg.space and the case
 *              NOTEs when none do; a stricter case is the lead's to grant (M0-V13 names the gap).
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
const CASES = ['ready', 'no-webgpu', 'preset'];

const args = process.argv.slice(2);
const flag = (name, dflt) => { const i = args.indexOf(name); return i >= 0 && args[i + 1] ? args[i + 1] : dflt; };
const kase = args.find((a, i) => !a.startsWith('--') && !(i > 0 && args[i - 1].startsWith('--')));
const base = flag('--base', 'http://127.0.0.1:47840').replace(/\/$/, '');
const timeoutMs = Number(flag('--timeout-s', '30')) * 1000;

// The verdict stays one line (verify.py reads it): a multi-line reason (playwright's call log) prints its tail first, as log lines.
const nogo = (why) => {
  const [head, ...rest] = String(why).split(/\r?\n/);
  for (const line of rest) if (line.trim()) console.log(line);
  console.log(`=== NO-GO: ${head.trim()} ===`);
  process.exit(1);
};
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
  const states = kase !== 'no-webgpu' ? ['ready', 'error', 'no-webgpu'] : ['no-webgpu', 'ready', 'error'];
  const url = kase === 'ready' ? `${base}/` : kase === 'preset' ? `${base}/?preset=sun&steps=200` : `${base}/?no-webgpu=1`;
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

  if (kase !== 'no-webgpu') {
    if (state !== 'ready') throw new Error(`srState "${state}"${info.error ? ` (${info.error})` : ''} - not "ready"${problems.length ? '; ' + problems.join(' | ') : ''}`);
    if (!info.adapter) throw new Error('srState "ready" but srAdapter is not set');
    const named = `${info.adapter} ${chromeAdapter ? Object.values(chromeAdapter).join(' ') : ''}`;
    if (!chromeAdapter || chromeAdapter.fallback || /swiftshader|llvmpipe|software|basic render|warp/i.test(named)) {
      console.log(`NOTE the adapter reads as software or unknown (${named.trim()}): "ready" does not prove the GPU path`);
    }
    console.log(`${kase}: srState "ready" after ${took} s`);
    if (kase === 'preset') {
      // The 200 steps run after "ready": sample the canvas's central 100 x 100 px (a screenshot clip, decoded in the page)
      // until a pixel is not black, up to the case's timeout.
      const box = await page.evaluate(() => { const r = document.getElementById('sr-canvas').getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height }; });
      const clip = { x: Math.round(box.x + box.w / 2 - 50), y: Math.round(box.y + box.h / 2 - 50), width: 100, height: 100 };
      const sample = async () => {
        const png = await page.screenshot({ clip, type: 'png' });
        return page.evaluate(async (b64) => {
          const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
          const bmp = await createImageBitmap(new Blob([bytes], { type: 'image/png' }));
          const cv = new OffscreenCanvas(bmp.width, bmp.height);
          const ctx = cv.getContext('2d');
          ctx.drawImage(bmp, 0, 0);
          const d = ctx.getImageData(0, 0, bmp.width, bmp.height).data;
          let lit = 0, offBg = 0;
          for (let i = 0; i < d.length; i += 4) {
            if (d[i] + d[i + 1] + d[i + 2] > 0) lit++;
            if (d[i] !== 0x05 || d[i + 1] !== 0x07 || d[i + 2] !== 0x0d) offBg++;
          }
          return { w: bmp.width, h: bmp.height, lit, offBg };
        }, png.toString('base64'));
      };
      const until = Date.now() + timeoutMs;
      let px = await sample();
      while (px.offBg === 0 && Date.now() < until) { await page.waitForTimeout(500); px = await sample(); }
      console.log(`SR-PIXELS central ${px.w}x${px.h}: ${px.lit} not black, ${px.offBg} differ from bg.space #05070D`);
      if (px.w !== 100 || px.h !== 100) throw new Error(`the central clip read ${px.w}x${px.h}, not 100x100 (canvas ${Math.round(box.w)}x${Math.round(box.h)})`);
      if (px.lit === 0) throw new Error('the central 100 x 100 px of the canvas are all black');
      if (px.offBg === 0) console.log('NOTE every central pixel is bg.space: "not all black" holds, but no star is drawn there');
    }
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
