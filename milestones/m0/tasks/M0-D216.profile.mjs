import { createRequire } from 'node:module';
import fs from 'node:fs';
const req = createRequire('/home/ybelanger/private/sandbox_reactions/web/package.json');
const { chromium } = req('playwright-core');
const base = process.argv[2], out = process.argv[3];
const browser = await chromium.launch({ executablePath: '/usr/bin/google-chrome', headless: true, chromiumSandbox: true,
  args: ['--headless=new', '--use-angle=vulkan', '--enable-features=Vulkan', '--disable-vulkan-surface', '--enable-unsafe-webgpu'] });
const page = await browser.newPage();
const cdp = await page.context().newCDPSession(page);
await cdp.send('Profiler.enable');
await cdp.send('Profiler.setSamplingInterval', { interval: 1000 });
await cdp.send('Profiler.start');
const t0 = Date.now();
await page.goto(base + '/', { waitUntil: 'domcontentloaded' });
await page.waitForFunction(() => ['ready', 'error'].includes(document.body.dataset.srState), null, { timeout: 240000, polling: 200 });
console.log('state', await page.evaluate(() => document.body.dataset.srState), 'after', (Date.now() - t0) / 1000, 's');
const { profile } = await cdp.send('Profiler.stop');
fs.writeFileSync(out, JSON.stringify(profile));
// self time per function
const dt = {}; const byId = new Map(profile.nodes.map((n) => [n.id, n]));
for (let i = 0; i < profile.samples.length; i++) { const n = byId.get(profile.samples[i]); const k = `${n.callFrame.functionName || '(anon)'} @${n.callFrame.url.split('/').pop()}`; dt[k] = (dt[k] || 0) + (profile.timeDeltas[i] || 0); }
const top = Object.entries(dt).sort((a, b) => b[1] - a[1]).slice(0, 25);
for (const [k, v] of top) console.log((v / 1e6).toFixed(2).padStart(8), 's', k);
// inclusive time for JS-side functions (glue imports)
const parent = new Map(); for (const n of profile.nodes) for (const c of n.children || []) parent.set(c, n.id);
const incl = {};
for (let i = 0; i < profile.samples.length; i++) { const seen = new Set(); let id = profile.samples[i]; while (id) { const n = byId.get(id); const k = n.callFrame.functionName || '(anon)'; if (!seen.has(k)) { seen.add(k); incl[k] = (incl[k] || 0) + (profile.timeDeltas[i] || 0); } id = parent.get(id); } }
console.log('-- inclusive, JS-named --');
for (const [k, v] of Object.entries(incl).filter(([k]) => !k.startsWith('$') && !/^wasm-function/.test(k)).sort((a, b) => b[1] - a[1]).slice(0, 25)) console.log((v / 1e6).toFixed(2).padStart(8), 's', k);
await browser.close();
