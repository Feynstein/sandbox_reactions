// M0-D218's probe: how long Chrome's GPU process takes to work through each burst of sync createComputePipeline calls.
//   node milestones/m0/tasks/M0-D218.gpuproc.mjs http://127.0.0.1:<port> [path, default /] [rAF delay ms, default 1500]
// A burst is the sync creations of one JS task (the warm-up's dry build, then SandboxApp::new's real build): right after it
// the probe asks queue.onSubmittedWorkDone() — it resolves once the GPU process has handled every command sent before it,
// the burst's pipeline creations included. Each submission's submit-to-done time is printed beside it, so a submission
// queued behind a burst reads the burst's cost. requestAnimationFrame is delayed as in M0-D216's gpucost.
import { createRequire } from 'node:module';
const req = createRequire('/home/ybelanger/private/sandbox_reactions/web/package.json');
const { chromium } = req('playwright-core');
const base = process.argv[2], q = process.argv[3] || '/', gap = Number(process.argv[4] || 1500);
const browser = await chromium.launch({ executablePath: '/usr/bin/google-chrome', headless: true, chromiumSandbox: true,
  args: ['--headless=new', '--use-angle=vulkan', '--enable-features=Vulkan', '--disable-vulkan-surface', '--enable-unsafe-webgpu'] });
const page = await browser.newPage();
page.on('console', (m) => { if (m.text().startsWith('GP ')) console.log(m.text().slice(3)); });
await page.addInitScript((gap) => {
  const t0 = performance.now(); const t = (x = performance.now()) => ((x - t0) / 1000).toFixed(2);
  const log = (s) => console.log(`GP ${t()} ${s}`);
  if (gap > 0) { const raf = window.requestAnimationFrame.bind(window); window.requestAnimationFrame = (cb) => { setTimeout(() => raf(cb), gap); return 0; }; }
  let burst = null, bursts = 0, asyncs = 0;
  const cp = GPUDevice.prototype.createComputePipeline;
  GPUDevice.prototype.createComputePipeline = function (d) {
    const s = performance.now(); const r = cp.call(this, d); const js = performance.now() - s;
    if (!burst) {
      burst = { n: 0, invalid: 0, js: 0, first: s, dev: this, k: ++bursts };
      setTimeout(() => {
        const b = burst; burst = null; const asked = performance.now();
        log(`burst ${b.k}: ${b.n} sync createComputePipeline (${b.invalid} the warm-up's invalid ones) in ${((asked - b.first) / 1000).toFixed(2)} s of JS (${(b.js / 1000).toFixed(2)} s inside the calls)`);
        b.dev.queue.onSubmittedWorkDone().then(() => log(`burst ${b.k}: the GPU process done with it ${((performance.now() - asked) / 1000).toFixed(2)} s after`));
      }, 0);
    }
    burst.n++; burst.js += js; if (d?.compute?.entryPoint === 'sr_warm_no_entry') burst.invalid++;
    return r;
  };
  const ca = GPUDevice.prototype.createComputePipelineAsync;
  GPUDevice.prototype.createComputePipelineAsync = function (d) {
    const s = performance.now(), n = ++asyncs; const p = ca.call(this, d);
    p.then(() => { if (n === 1 || n % 20 === 0) log(`createComputePipelineAsync #${n} resolved after ${((performance.now() - s) / 1000).toFixed(2)} s`); }, () => {});
    return p;
  };
  let k = 0;
  const sub = GPUQueue.prototype.submit;
  GPUQueue.prototype.submit = function (bufs) {
    const n = ++k, s = performance.now(); sub.call(this, bufs);
    if (n <= 14) this.onSubmittedWorkDone().then(() => log(`submit #${n} (sent ${t(s)}) done ${((performance.now() - s) / 1000).toFixed(3)} s after; state=${document.body.dataset.srState}`));
  };
}, gap);
const s0 = Date.now();
await page.goto(base + q, { waitUntil: 'domcontentloaded' });
await page.waitForFunction(() => ['ready', 'error'].includes(document.body.dataset.srState), null, { timeout: 120000, polling: 200 }).catch(() => {});
console.log('state', await page.evaluate(() => document.body.dataset.srState), 'after', (Date.now() - s0) / 1000, 's');
await page.waitForTimeout(Math.max(4000, 2 * gap));
await browser.close();
