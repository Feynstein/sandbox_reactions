// M0-D218's probe: the WebGPU calls the page makes between two submissions (one frame each), counted by name, with the
// bytes writeBuffer sends and the buffers and bind groups created; and each submission's submit-to-done time.
//   node milestones/m0/tasks/M0-D218.census.mjs http://127.0.0.1:<port> [path, default /?preset=sun&steps=200] [rAF delay ms, default 1500]
import { createRequire } from 'node:module';
const req = createRequire('/home/ybelanger/private/sandbox_reactions/web/package.json');
const { chromium } = req('playwright-core');
const base = process.argv[2], q = process.argv[3] || '/?preset=sun&steps=200', gap = Number(process.argv[4] || 1500);
const browser = await chromium.launch({ executablePath: '/usr/bin/google-chrome', headless: true, chromiumSandbox: true,
  args: ['--headless=new', '--use-angle=vulkan', '--enable-features=Vulkan', '--disable-vulkan-surface', '--enable-unsafe-webgpu'] });
const page = await browser.newPage();
page.on('console', (m) => { if (m.text().startsWith('CN ')) console.log(m.text().slice(3)); });
await page.addInitScript((gap) => {
  const t0 = performance.now(); const t = () => ((performance.now() - t0) / 1000).toFixed(2);
  const log = (s) => console.log(`CN ${t()} ${s}`);
  if (gap > 0) { const raf = window.requestAnimationFrame.bind(window); window.requestAnimationFrame = (cb) => { setTimeout(() => raf(cb), gap); return 0; }; }
  let c = {};
  const count = (k, n = 1) => { c[k] = (c[k] || 0) + n; };
  const wrap = (P, names, extra) => { for (const n of names) { const f = P.prototype[n]; if (!f) continue; P.prototype[n] = function (...a) { count(n); extra?.(n, a, this); return f.apply(this, a); }; } };
  wrap(GPUDevice, ['createBuffer', 'createBindGroup', 'createTexture', 'createCommandEncoder', 'createComputePipeline', 'createQuerySet'], (n, a) => { if (n === 'createBuffer') count('createBuffer bytes', Number(a[0].size)); });
  wrap(GPUQueue, ['writeBuffer', 'writeTexture'], (n, a) => { if (n === 'writeBuffer') count('writeBuffer bytes', Number(a[2]?.byteLength ?? a[2]?.length ?? 0) * (a[2]?.BYTES_PER_ELEMENT || 1)); });
  wrap(GPUCommandEncoder, ['beginComputePass', 'beginRenderPass', 'copyBufferToBuffer', 'copyTextureToBuffer', 'copyBufferToTexture', 'copyTextureToTexture', 'clearBuffer', 'resolveQuerySet'],
    (n, a) => { if (n === 'copyBufferToBuffer') count('copyBufferToBuffer bytes', Number(a.length === 5 ? a[4] : a[a.length - 1] ?? 0)); });
  wrap(GPUComputePassEncoder, ['dispatchWorkgroups', 'dispatchWorkgroupsIndirect', 'setBindGroup', 'setPipeline']);
  wrap(GPURenderPassEncoder, ['draw', 'drawIndexed', 'setBindGroup']);
  wrap(GPUBuffer, ['mapAsync']);
  let k = 0;
  const sub = GPUQueue.prototype.submit;
  GPUQueue.prototype.submit = function (bufs) {
    const n = ++k, s = performance.now(), seen = c; c = {};
    sub.call(this, bufs);
    if (n > 12) return;
    this.onSubmittedWorkDone().then(() => log(`submit #${n} done ${((performance.now() - s) / 1000).toFixed(3)} s after; since the last submit: ${JSON.stringify(seen)}`));
  };
}, gap);
await page.goto(base + q, { waitUntil: 'domcontentloaded' });
await page.waitForTimeout(12000);
await browser.close();
