// M0-D218's probe: GPU time per pass of every submission the web page makes, from WebGPU timestamps the probe writes itself.
//   node milestones/m0/tasks/M0-D218.passes.mjs http://127.0.0.1:<port> [path, default /] [rAF delay ms, default 1500] [submissions, default 12]
// Every compute and render pass gets the probe's own timestampWrites (the app's own pass timestamps are overridden, so its
// c_step reads nothing while the probe runs — the first frames plan one step whatever it reads); each submission's queries
// are resolved and mapped after it, and the probe prints, per submission: its passes, its GPU span, and its costliest passes
// with the pipelines they dispatched. requestAnimationFrame is delayed so the submissions do not overlap (M0-D216's gpucost).
import { createRequire } from 'node:module';
const req = createRequire('/home/ybelanger/private/sandbox_reactions/web/package.json');
const { chromium } = req('playwright-core');
const base = process.argv[2], q = process.argv[3] || '/', gap = Number(process.argv[4] || 1500), keep = Number(process.argv[5] || 12);
const browser = await chromium.launch({ executablePath: '/usr/bin/google-chrome', headless: true, chromiumSandbox: true,
  args: ['--headless=new', '--use-angle=vulkan', '--enable-features=Vulkan', '--disable-vulkan-surface', '--enable-unsafe-webgpu'] });
const page = await browser.newPage();
page.on('console', (m) => { if (m.text().startsWith('PP ')) console.log(m.text().slice(3)); });
await page.addInitScript(([gap, keep]) => {
  const t0 = performance.now(); const t = () => ((performance.now() - t0) / 1000).toFixed(2);
  const log = (s) => console.log(`PP ${t()} ${s}`);
  if (gap > 0) { const raf = window.requestAnimationFrame.bind(window); window.requestAnimationFrame = (cb) => { setTimeout(() => raf(cb), gap); return 0; }; }
  const rd = GPUAdapter.prototype.requestDevice;
  GPUAdapter.prototype.requestDevice = function (desc = {}) {
    const f = new Set(desc.requiredFeatures || []);
    if (this.features.has('timestamp-query')) f.add('timestamp-query'); else log('adapter has no timestamp-query');
    return rd.call(this, { ...desc, requiredFeatures: [...f] });
  };
  const MAXQ = 4096; const enc = new WeakMap(); const cmd = new WeakMap(); const pass = new WeakMap();
  const ce = GPUDevice.prototype.createCommandEncoder;
  GPUDevice.prototype.createCommandEncoder = function (d) { const e = ce.call(this, d); enc.set(e, { dev: this, label: d?.label || '', passes: [], qs: null }); return e; };
  const wrapBegin = (name) => {
    const orig = GPUCommandEncoder.prototype[name];
    GPUCommandEncoder.prototype[name] = function (d = {}) {
      const s = enc.get(this);
      if (!s || !s.dev.features.has('timestamp-query') || 2 * (s.passes.length + 1) > MAXQ) return orig.call(this, d);
      s.qs ??= s.dev.createQuerySet({ type: 'timestamp', count: MAXQ });
      const i = s.passes.length; const rec = { label: d.label || '', pipes: new Set() }; s.passes.push(rec);
      const p = orig.call(this, { ...d, timestampWrites: { querySet: s.qs, beginningOfPassWriteIndex: 2 * i, endOfPassWriteIndex: 2 * i + 1 } });
      pass.set(p, rec); return p;
    };
  };
  wrapBegin('beginComputePass'); wrapBegin('beginRenderPass');
  for (const P of [GPUComputePassEncoder, GPURenderPassEncoder]) {
    const sp = P.prototype.setPipeline;
    P.prototype.setPipeline = function (pl) { pass.get(this)?.pipes.add(pl.label || '?'); return sp.call(this, pl); };
  }
  const fin = GPUCommandEncoder.prototype.finish;
  GPUCommandEncoder.prototype.finish = function (d) {
    const s = enc.get(this);
    if (s?.qs && s.passes.length) {
      const n = 2 * s.passes.length, bytes = n * 8;
      const res = s.dev.createBuffer({ size: bytes, usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC });
      const out = s.dev.createBuffer({ size: bytes, usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST });
      this.resolveQuerySet(s.qs, 0, n, res, 0); this.copyBufferToBuffer(res, 0, out, 0, bytes);
      const b = fin.call(this, d); cmd.set(b, { ...s, out }); return b;
    }
    return fin.call(this, d);
  };
  let k = 0;
  const sub = GPUQueue.prototype.submit;
  GPUQueue.prototype.submit = function (bufs) {
    const n = ++k; sub.call(this, bufs);
    if (n > keep) return;
    for (const b of bufs) {
      const s = cmd.get(b); if (!s) continue;
      s.out.mapAsync(GPUMapMode.READ).then(() => {
        const v = new BigUint64Array(s.out.getMappedRange().slice(0)); s.out.unmap();
        const rows = s.passes.map((p, i) => ({ ms: Number(v[2 * i + 1] - v[2 * i]) / 1e6, label: p.label, pipes: [...p.pipes].join(',') }));
        const span = Number(v[2 * s.passes.length - 1] - v[0]) / 1e6, sum = rows.reduce((a, r) => a + r.ms, 0);
        log(`submit #${n} encoder="${s.label}" passes=${rows.length} span=${span.toFixed(1)} ms sum=${sum.toFixed(1)} ms state=${document.body.dataset.srState}`);
        const byPipe = new Map();
        for (const r of rows) byPipe.set(r.pipes || r.label, (byPipe.get(r.pipes || r.label) || [0, 0]).map((x, j) => x + (j ? 1 : r.ms)));
        [...byPipe].sort((a, b) => b[1][0] - a[1][0]).slice(0, 8).forEach(([p, [ms, c]]) => log(`    ${ms.toFixed(1).padStart(8)} ms  ×${c}  ${p}`));
      }).catch((e) => log(`submit #${n} map failed: ${e}`));
    }
  };
}, [gap, keep]);
const s0 = Date.now();
await page.goto(base + q, { waitUntil: 'domcontentloaded' });
await page.waitForFunction(() => ['ready', 'error'].includes(document.body.dataset.srState), null, { timeout: 120000, polling: 200 }).catch(() => {});
console.log('state', await page.evaluate(() => document.body.dataset.srState), 'after', (Date.now() - s0) / 1000, 's');
await page.waitForTimeout(Math.max(4000, 2 * gap));
await browser.close();
