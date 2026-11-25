import { createRequire } from 'node:module';
const req = createRequire('/home/ybelanger/private/sandbox_reactions/web/package.json');
const { chromium } = req('playwright-core');
const base = process.argv[2], q = process.argv[3] || '/', gap = Number(process.argv[4] || 1500);
const browser = await chromium.launch({ executablePath: '/usr/bin/google-chrome', headless: true, chromiumSandbox: true,
  args: ['--headless=new', '--use-angle=vulkan', '--enable-features=Vulkan', '--disable-vulkan-surface', '--enable-unsafe-webgpu'] });
const page = await browser.newPage();
page.on('console', (m) => { if (m.text().startsWith('TR ')) console.log(m.text()); });
await page.addInitScript((gap) => {
  const t0 = performance.now(); const t = () => ((performance.now() - t0) / 1000).toFixed(2);
  const log = (s) => console.log(`TR ${t()} ${s}`);
  const raf = window.requestAnimationFrame.bind(window);
  window.requestAnimationFrame = (cb) => { setTimeout(() => raf(cb), gap); return 0; };
  let n = 0, maps = 0;
  const sub = GPUQueue.prototype.submit;
  GPUQueue.prototype.submit = function (bufs) { const k = ++n; const s = performance.now(); sub.call(this, bufs); this.onSubmittedWorkDone().then(() => log(`submit #${k} (${bufs.length} cmd) done in ${((performance.now() - s) / 1000).toFixed(3)} s; maps=${maps} state=${document.body.dataset.srState}`)); };
  const ma = GPUBuffer.prototype.mapAsync;
  GPUBuffer.prototype.mapAsync = function (...a) { maps++; return ma.apply(this, a); };
}, gap);
const s0 = Date.now();
await page.goto(base + q, { waitUntil: 'domcontentloaded' });
await page.waitForFunction(() => ['ready', 'error'].includes(document.body.dataset.srState), null, { timeout: 120000, polling: 200 }).catch(() => {});
console.log('state', await page.evaluate(() => document.body.dataset.srState), 'after', (Date.now() - s0) / 1000, 's');
await page.waitForTimeout(8000);
await browser.close();
