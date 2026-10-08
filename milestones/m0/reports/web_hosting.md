# Web hosting and ads — a quick check for the web release (2026-10-08)

**Summary.** The web build can be served as static files from a small Amazon Lightsail instance.
Each player's own GPU runs the simulation, so the server only ships the download. As traffic
grows, the cost that grows is outbound bandwidth, not server CPU. So the scaling step is a CDN in
front of the box first, and a bigger box only if needed. Revenue: AdSense's H5 Games Ads, which
places ads at game breaks (loading, pause, rewarded). The lead approved this direction. It is
**routed past M0**: the web *release* is a later milestone (reports/bootstrap.md §3), and M0-TZ
places it. Written to PLAYBOOK §4 Rule 7: each claim with its URL and access date; opinion
labelled; UNVERIFIED kept apart.

## The lead's words (verbatim, 2026-10-08)
- "do that choice means I could serve the game on a website and the players own hardware kicks in? Keeping what I swerve light?"
- "nice can you quickly check if I could serve it on something like a small amazon lightsail? and move to a larger box when traffic justifies it? If so were gonna have to add some google adsense ad serving to the website, to generate passive revenue"
- After the check below: "great record all that, the CDN and the adsense stuff, its perfect!"

## Why the server stays light
- The ruled stack (reports/engine_stack.md, Ruling: Rust + wgpu) runs the simulation's compute
  shaders in the browser through WebGPU — engine_stack.md E1, E2. The web build is static files:
  a WebAssembly program, its shaders, a page.
- Browsers with WebGPU today: Chrome on all desktops (on Linux: Intel Gen12+, or NVIDIA on
  Wayland), Safari 26, Firefox on Windows and macOS. Firefox on Linux is not there yet — engine_stack.md E3.
- **Opinion:** the server does no per-player work, so a request costs only bytes sent.

## Hosting — Amazon Lightsail
- Linux bundles: $5/mo (2 vCPU, 0.5 GB RAM, 20 GB SSD, 1 TB transfer), $7 (1 GB, 2 TB), $12 (2 GB, 3 TB), $24 (4 GB, 4 TB), $44 (8 GB, 5 TB), up to $1,764 (64 vCPU, 10 TB) — https://aws.amazon.com/lightsail/pricing/ (accessed 2026-10-08)
- Outbound transfer above the bundle's allowance: $0.09/GB — https://aws.amazon.com/lightsail/pricing/ (accessed 2026-10-08)
- Moving to a larger box: snapshot the instance, then create a new instance from the snapshot on a larger plan; only the same size or larger is allowed — https://docs.aws.amazon.com/lightsail/latest/userguide/how-to-create-larger-instance-from-snapshot-using-console.md (accessed 2026-10-08)
- **Arithmetic (opinion, the download size not yet measured):** at an assumed 5 MB per visit, the
  $5 bundle's 1 TB covers about 200,000 visits a month; each further 1,000 visits is about 5 GB,
  or about $0.45 at the overage rate.

## The CDN — the scaling step before a bigger box
- Lightsail CDN distributions: $2.50/mo with 50 GB, $10/mo with 200 GB, $35/mo with 500 GB — https://aws.amazon.com/lightsail/pricing/ (accessed 2026-10-08)
- **Opinion:** bigger Lightsail bundles mostly add CPU and RAM a static site does not use. Their
  transfer allowance rises only from 1 TB to 10 TB. A CDN caches the files near the players, takes
  the load off the box and cuts latency, so it comes before a bigger box.
- **Opinion, alternatives worth pricing at the release milestone:** object storage plus a CDN
  (with no server at all), or a game portal hosting the build. Not researched here.

## Ads — AdSense H5 Games Ads
- H5 Games Ads show ads inside HTML5 games through the Ad Placement API, at game moments such as loading, pause, or a reward offer — https://adsense.google.com/start/h5-games-ads/ (accessed 2026-10-08)
- Formats: interstitials at natural breaks, and rewarded ads that a player opts into for an in-game reward; ad frequency is configurable — https://adsense.google.com/start/h5-games-ads/ (accessed 2026-10-08)
- Eligibility: a publisher with an H5 games website may apply; AdSense program policies apply (copyright, replicated content, more ads than content, ad placement, inappropriate content, misrepresentation) — https://support.google.com/adsense/answer/9959170 (accessed 2026-10-08)
- **Opinion:** M0's pause and the automatic slow-down at big events are natural break points; a
  rewarded ad could unlock a preset. Game design to rule at the release milestone, with the lead.

## The header that would break ads — keep it off the game page
- `Cross-Origin-Embedder-Policy: require-corp` lets a page load cross-origin `no-cors` resources only when they send a `Cross-Origin-Resource-Policy` header allowing it, and it applies to iframes too; `credentialless` loads them but strips cookies — https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Embedder-Policy (accessed 2026-10-08)
- Browser threads (SharedArrayBuffer) need that cross-origin isolation — https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer (accessed 2026-10-08)
- **Consequence:** the web build stays without browser threads, as M0's design already has it
  (engine_stack.md, Recommendation), so the page needs no COEP and ad scripts load normally.

## UNVERIFIED (refuted by default)
- That AdSense ads specifically fail under COEP `require-corp`. This is inferred from MDN's rule
  plus ads being cross-origin; no AdSense page says it.
- The Lightsail CDN's overage rate and which origins it accepts (the AWS docs page did not load).
- Privacy consent for ads (Europe/UK, Quebec's Law 25): what the page must show before ads
  run — not researched; the lead's to rule.
- Whether H5 Games Ads is open to all or by application only beyond "may apply". AdSense
  approval is not guaranteed.
- The web build's download size — unmeasured until the first web build (M0's build lots).

## Routed to
- **M0-TZ** (next-plan authoring): place the web release — hosting, CDN, ads, consent — in a
  milestone after M0, with the lead (its Carried flags points here).
- **The lead's worklist** (milestones.md): the AWS account and the AdSense application, before
  the web release.
