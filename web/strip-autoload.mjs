// Trunk post_build hook (M0-T7): trunk's rust pipeline always injects a module script that fetches and starts the wasm
// the moment the page loads, plus preload links for it. The page's inline script must run the WebGPU check first and
// load the wasm only on success (m0_contrat.md §3.5, §6.2.4), so this removes what trunk injected from the staged
// index.html. It fails loudly, rather than shipping a page that loads the wasm unchecked, if the shape is not found.
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const file = join(process.env.TRUNK_STAGING_DIR ?? "dist", "index.html");
let html = readFileSync(file, "utf8");
const before = html;
html = html.replace(/<script type="module">(?:(?!<\/script>)[\s\S])*?TrunkApplicationStarted(?:(?!<\/script>)[\s\S])*?<\/script>/, "");
html = html.replace(/<link rel="(?:modulepreload|preload)"[^>]*>/g, "");
if (html === before || /TrunkApplicationStarted|import init/.test(html) || /rel="(?:modulepreload|preload)"/.test(html)) {
  console.error(`strip-autoload: trunk's injected loader was not found or not fully removed in ${file}`);
  process.exit(1);
}
writeFileSync(file, html);
