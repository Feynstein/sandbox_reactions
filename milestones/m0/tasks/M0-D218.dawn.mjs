// M0-D218's probe: where Chrome's GPU process spends a sim frame's submit-to-done time, from a Chrome trace (Dawn's events).
//   node milestones/m0/tasks/M0-D218.dawn.mjs http://127.0.0.1:<port> [path, default /?preset=sun&steps=200] [rAF delay ms, default 1500] [trace out]
// requestAnimationFrame is delayed so the frames do not overlap; the trace's complete events on the GPU process are summed by
// name (self-nesting not removed: a parent's time includes its children), between the 4th and the 10th second.
import { createRequire } from 'node:module';
import fs from 'node:fs';
const req = createRequire('/home/ybelanger/private/sandbox_reactions/web/package.json');
const { chromium } = req('playwright-core');
const base = process.argv[2], q = process.argv[3] || '/?preset=sun&steps=200', gap = Number(process.argv[4] || 1500);
const out = process.argv[5] || '/tmp/m0-d218-dawn-trace.json';
const browser = await chromium.launch({ executablePath: '/usr/bin/google-chrome', headless: true, chromiumSandbox: true,
  args: ['--headless=new', '--use-angle=vulkan', '--enable-features=Vulkan', '--disable-vulkan-surface', '--enable-unsafe-webgpu'] });
const page = await browser.newPage();
await page.addInitScript((gap) => {
  if (gap > 0) { const raf = window.requestAnimationFrame.bind(window); window.requestAnimationFrame = (cb) => { setTimeout(() => raf(cb), gap); return 0; }; }
}, gap);
await browser.startTracing(page, { path: out, categories: ['gpu', 'gpu.dawn', 'disabled-by-default-gpu.dawn', 'toplevel', 'gpu.service', 'webgpu'] });
await page.goto(base + q, { waitUntil: 'domcontentloaded' });
await page.waitForTimeout(11000);
await browser.stopTracing();
await browser.close();
const ev = JSON.parse(fs.readFileSync(out, 'utf8')).traceEvents;
const procs = new Map(ev.filter((e) => e.ph === 'M' && e.name === 'process_name').map((e) => [e.pid, e.args.name]));
const t0 = Math.min(...ev.filter((e) => e.ts > 0).map((e) => e.ts));
const sums = new Map();
const open = new Map();
for (const e of ev) {
  const proc = procs.get(e.pid) || String(e.pid);
  if (!/GPU/i.test(proc)) continue;
  const s = (e.ts - t0) / 1e6;
  let dur = null, at = s;
  if (e.ph === 'X') dur = (e.dur || 0) / 1e3;
  else if (e.ph === 'B') { open.set(`${e.tid}:${e.name}`, e.ts); continue; }
  else if (e.ph === 'E') { const b = open.get(`${e.tid}:${e.name}`); if (b == null) continue; dur = (e.ts - b) / 1e3; at = (b - t0) / 1e6; }
  else continue;
  if (at < 4 || at > 10) continue;
  const k = `${e.cat} ${e.name}`; const v = sums.get(k) || [0, 0, 0]; v[0] += dur; v[1]++; v[2] = Math.max(v[2], dur); sums.set(k, v);
}
console.log(`GPU process events between 4 s and 10 s (trace: ${out}):`);
[...sums].sort((a, b) => b[1][0] - a[1][0]).slice(0, 25)
  .forEach(([k, [ms, n, mx]]) => console.log(`${ms.toFixed(1).padStart(9)} ms  ×${String(n).padEnd(6)} max ${mx.toFixed(1).padStart(7)} ms  ${k}`));
