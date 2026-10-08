/* switch.mjs - the switch: a Claude Code plugin that sets every request's model and level from the
 * block's heading (PLAYBOOK annex §A.10).
 *
 *   node tools/pb/switch.mjs build [--out <dir>] [--json]        the plugin folder (default: outside every checkout), ready to install
 *   node tools/pb/switch.mjs explain --prompt <text> [--opened <file>] [--root <dir>] [--json]
 *   node tools/pb/switch.mjs selftest [--json]
 *
 * Two lives, one file. Installed (`build`, then the client's `plugin marketplace add` and `plugin
 * install`, once per box, user scope), it is the plugin's hooks module: Claude Code loads it in an
 * environment of its own - no Node, no import of any kind, every read through `$` - and on each model
 * request of the main loop (a `turn.step` without `agentId`) it
 *   1. finds the block: the session's first typed prompt, tags stripped - `execute <id>` (§14.5) or
 *      `do task <id>`, a whole id or the plan's local one; the plan is the one .md path that prompt
 *      names, else the file the IDE had open (`<ide_opened_file>`); no id, two ids, no plan -> none;
 *   2. reads that plan's heading for the id - outside code fences, exactly one: the segment after the
 *      type tag is the rating, and `switch` must be the segment after it (§3.2);
 *   3. maps the rating through RUNGS to a model id and a level and sends the request on with them.
 * Nothing found, no switch segment, a rating RUNGS lacks, a sub-agent's request, any read that fails ->
 * the request goes on untouched: the lead's pick. The answer is kept per session id, so the plan is
 * read once and a later prompt never moves it; a new session (`/clear` makes one) reads again.
 * Run by Node (>= 20.16, for process.getBuiltinModule), it is a tool in the §4 Rule 4 shape: one
 * counts line, failures only when non-zero, `=== GO ===` / `=== NO-GO: <reason> ===`, exit 0 / 1.
 * Nothing here writes outside `build`'s folder and the selftest's own temp folder.
 */

const VERSION = '12.2.1'
const PLUGIN = 'pb-switch'
const MARKET = 'pb'

// The ladder's rungs, in the heading's words -> [model id, level] (the ids the rung skills named).
// A rung the ladder adds is added here; a rating not listed passes untouched.
const RUNGS = {
  'Fable 5.1, high': ['claude-fable-5-1', 'high'],
  'Opus 5.5, max': ['claude-opus-5-5', 'max'],
  'Opus 5.5, xhigh': ['claude-opus-5-5', 'xhigh'],
  'Opus 5.5, high': ['claude-opus-5-5', 'high'],
  'Sonnet 5.5, high': ['claude-sonnet-5-5', 'high'],
  'Sonnet 5.5, medium': ['claude-sonnet-5-5', 'medium'],
}
const TRANSCRIPT_CAP = 4096          // messages() returns the newest 4096: at the cap the first prompt may be cut

// ------------------------------------------------------------------ the pure steps (the selftest's seams)
const F = {
  // The first user message with text once its tags are stripped; the IDE's opened file up to it.
  launchOf(messages) {
    let opened = null
    for (const m of messages || []) {
      if (!m || m.role !== 'user') continue
      const raw = String(m.text || '')
      const o = /<ide_opened_file>[^<]*?opened the file (.+?) in the IDE/.exec(raw)
      if (o && !opened) opened = o[1].trim()
      const prompt = stripTags(raw).trim()
      if (prompt) return { prompt, opened, raw }
    }
    return null
  },
  // Every id the prompt names after `task` or `execute`; the caller wants exactly one.
  idsOf(prompt) {
    const ids = new Set()
    for (const m of prompt.matchAll(/\b(?:[Tt]ask|[Ee]xecute)\s+([A-Z0-9][\w.-]*)/g)) ids.add(m[1].replace(/[.-]+$/, ''))
    return [...ids]
  },
  // The plan: the one .md path the prompt names (one implementation plan among several), else the opened file.
  planOf(launch) {
    const mds = [...new Set(launch.prompt.match(/[^\s'"`«»()<>]+\.md\b/g) || [])]
    if (mds.length === 1) return mds[0]
    if (mds.length > 1) {
      const plans = mds.filter(p => /implementation_plan\.md$/.test(p))
      return plans.length === 1 ? plans[0] : null
    }
    return launch.opened && /\.md$/.test(launch.opened) ? launch.opened : null
  },
  // The one heading of that id outside code fences; a local id matches the part after the milestone prefix.
  headingFor(text, id) {
    const whole = /^\d*M[\d.]+-/.test(id)
    const hits = []
    let fence = null
    for (const line of text.split(/\r?\n/)) {
      const f = /^ {0,3}(`{3,}|~{3,})(.*)$/.exec(line)
      if (f) {
        if (!fence) fence = f[1]
        else if (f[1][0] === fence[0] && f[1].length >= fence.length && !f[2].trim()) fence = null
        continue
      }
      if (fence) continue
      const h = /^## (\d*M[\d.]+)-(\S+) · /.exec(line)
      if (h && (whole ? `${h[1]}-${h[2]}` === id : h[2] === id)) hits.push(line)
    }
    return hits.length === 1 ? hits[0] : null
  },
  // The rating after the type tag, only when ` · switch` follows it. The tag is the first segment after
  // the id and the title that is wholly bold, whatever it holds (`**BUILD + LEAD go**`, §3.2); a bold
  // word inside the title is no tag.
  rungOf(heading) {
    const seg = heading.split(' · ').map(s => s.trim())
    const t = seg.findIndex((s, i) => i > 1 && /^\*\*[^*]+\*\*$/.test(s))
    return t > 0 && seg[t + 2] === 'switch' && seg[t + 1] ? seg[t + 1] : null
  },
  // `build`'s default folder: the client's own, never a checkout's (`.claude/` is tracked in some projects).
  defaultOut(env, home, join) {
    return join(env.CLAUDE_CONFIG_DIR || join(home, '.claude'), PLUGIN + '-src')
  },
  mapRung(rating) {
    return Object.prototype.hasOwnProperty.call(RUNGS, rating) ? { model: RUNGS[rating][0], effort: RUNGS[rating][1] } : null
  },
}

function stripTags(s) {
  return s.replace(/<([a-z][\w-]*)\b[^>]*>[\s\S]*?<\/\1>/g, ' ')
}

function joinPath(root, p) {
  if (/^([\\/]|[A-Za-z]:[\\/]|~)/.test(p)) return p
  return String(root || '').replace(/[\\/]+$/, '') + '/' + p.replace(/^\.[\\/]/, '')
}

// launch -> { id, plan, heading, rating, model, effort } or { why } (untouched). `read` may reject.
async function decide(launch, root, read) {
  if (!launch) return { why: 'no typed prompt' }
  const ids = F.idsOf(launch.prompt)
  if (ids.length !== 1) return { why: ids.length ? `the prompt names ${ids.length} ids: ${ids.join(', ')}` : 'the prompt names no block' }
  const plan = F.planOf(launch)
  if (!plan) return { why: 'no plan: the prompt names none and the IDE had no .md open', id: ids[0] }
  const path = joinPath(root, plan)
  let text
  try { text = await read(path) } catch (e) { return { why: `the plan does not read: ${path}`, id: ids[0] } }
  const heading = F.headingFor(String(text), ids[0])
  if (!heading) return { why: `no single heading for ${ids[0]} in ${path}`, id: ids[0], plan: path }
  const rating = F.rungOf(heading)
  if (!rating) return { why: 'no rating followed by `· switch` in its heading', id: ids[0], plan: path, heading }
  const to = F.mapRung(rating)
  if (!to) return { why: `the rating "${rating}" is not a rung of RUNGS`, id: ids[0], plan: path, heading, rating }
  return { id: ids[0], plan: path, heading, rating, model: to.model, effort: to.effort }
}

// ------------------------------------------------------------------ the plugin
const CACHE = new Map()

async function decision($) {
  const sid = await $.session.id()
  if (CACHE.has(sid)) return CACHE.get(sid)
  const messages = await $.session.messages()
  if (messages.length >= TRANSCRIPT_CAP) return { why: 'the transcript is past the cap: its first prompt may be cut' }
  const launch = F.launchOf(messages)
  if (!launch) return { why: 'no typed prompt yet' }                 // not kept: the next request looks again
  const d = await decide(launch, await $.session.root(), p => $.fs.read(p))
  CACHE.set(sid, d)
  return d
}

export const register = (on) => {
  on('turn.step', async function* ($, e, next) {
    if (e.agentId !== undefined && e.agentId !== null) return yield* next(e)
    let d = null
    try { d = await decision($) } catch (err) { d = null }
    if (!d || !d.model) return yield* next(e)
    return yield* next({ ...e, model: d.model, effort: d.effort })
  })
}

// ------------------------------------------------------------------ the tool (Node only)
function out(P, json, obj, lines, ok, why) {
  if (json) P.stdout.write(JSON.stringify({ ...obj, verdict: ok ? 'GO' : 'NO-GO: ' + why }) + '\n')
  else P.stdout.write([...lines, ok ? '=== GO ===' : `=== NO-GO: ${why} ===`].join('\n') + '\n')
  P.exitCode = ok ? 0 : 1
}

function pluginFiles(source) {
  const desc = 'PLAYBOOK switch: every main-loop request of a session on the model and level its block\'s heading rates (· switch)'
  const j = o => JSON.stringify(o, null, 2) + '\n'
  return {
    '.claude-plugin/marketplace.json': j({ name: MARKET, owner: { name: 'PLAYBOOK' }, description: 'The PLAYBOOK toolkit\'s Claude Code plugins',
      plugins: [{ name: PLUGIN, source: `./plugins/${PLUGIN}`, description: desc }] }),
    [`plugins/${PLUGIN}/.claude-plugin/plugin.json`]: j({ name: PLUGIN, version: VERSION, description: desc, author: { name: 'PLAYBOOK' } }),
    [`plugins/${PLUGIN}/hooks/hooks.json`]: j({ modules: ['./switch.mjs'] }),
    [`plugins/${PLUGIN}/hooks/switch.mjs`]: source,
  }
}

function writeAtomic(fs, path, text) {
  const tmp = `${path}.tmp${Date.now()}`
  fs.writeFileSync(tmp, text, 'utf8')
  for (let i = 0; ; i++) {
    try { fs.renameSync(tmp, path); return } catch (e) { if (i >= 5) throw e }
  }
}

function build(P, fs, pathm, outDir) {
  const source = fs.readFileSync(P.argv[1], 'utf8')
  const files = pluginFiles(source)
  for (const [rel, text] of Object.entries(files)) {
    const dest = pathm.join(outDir, rel)
    fs.mkdirSync(pathm.dirname(dest), { recursive: true })
    writeAtomic(fs, dest, text)
  }
  return Object.keys(files)
}

function args(argv) {
  const a = { cmd: argv[0], json: false }
  for (let i = 1; i < argv.length; i++) {
    const t = argv[i]
    if (t === '--json') a.json = true
    else if (/^--(out|prompt|opened|root)$/.test(t) && i + 1 < argv.length) a[t.slice(2)] = argv[++i]
    else a.bad = t
  }
  return a
}

async function main(P) {
  const a = args(P.argv.slice(2))
  if (typeof P.getBuiltinModule !== 'function') return out(P, a.json, { cmd: a.cmd }, ['switch: node >= 20.16 needed (process.getBuiltinModule)'], false, 'node too old')
  const fs = P.getBuiltinModule('node:fs'), pathm = P.getBuiltinModule('node:path'), os = P.getBuiltinModule('node:os')
  const usage = 'usage: node tools/pb/switch.mjs build [--out <dir>] | explain --prompt <text> [--opened <file>] [--root <dir>] | selftest  [--json]'
  if (a.bad || !['build', 'explain', 'selftest'].includes(a.cmd)) return out(P, a.json, { cmd: a.cmd }, [usage], false, a.bad ? `unknown argument ${a.bad}` : 'no command')
  if (a.cmd === 'build') {
    const dir = pathm.resolve(a.out || F.defaultOut(P.env || {}, os.homedir(), pathm.join))
    const files = build(P, fs, pathm, dir)
    return out(P, a.json, { cmd: 'build', out: dir, files }, [
      `build: files=${files.length} out=${dir}`,
      `install, once per box (claude = the client's own binary; in VS Code the extension's resources/native-binary/claude):`,
      `  claude plugin marketplace add "${dir}" && claude plugin install ${PLUGIN}@${MARKET} --scope user`], true)
  }
  if (a.cmd === 'explain') {
    if (!a.prompt) return out(P, a.json, { cmd: 'explain' }, [usage], false, 'no --prompt')
    const opened = a.opened ? `<ide_opened_file>The user opened the file ${pathm.resolve(a.opened)} in the IDE.</ide_opened_file>` : ''
    const d = await decide(F.launchOf([{ role: 'user', text: opened + a.prompt }]), pathm.resolve(a.root || P.cwd()), p => fs.readFileSync(p, 'utf8'))
    return out(P, a.json, { cmd: 'explain', ...d }, [
      `explain: switched=${d.model ? 1 : 0}`,
      d.model ? `switch: ${d.id} in ${d.plan} · ${d.rating} · switch -> model=${d.model} level=${d.effort}` : `untouched: ${d.why}`], true)
  }
  return selftest(P, fs, pathm, os, a.json)
}

// ------------------------------------------------------------------ selftest, red-armed
async function selftest(P, fs, pathm, os, json) {
  const checks = [], plants = []
  const check = (name, ok) => checks.push([name, !!ok])
  const ROOT = '/w', M9 = 'milestones/m9/m9_implementation_plan.md', M8 = 'milestones/m8/m8_implementation_plan.md'
  const FILES = {
    [`${ROOT}/${M9}`]: [
      '# M9 - fixture plan', '',
      '## M9-T12 · a neighbour · **BUILD** · Opus 5.5, max · switch · (FIRST)', '- Status: TODO', '',
      '```markdown', '## M9-T1 · a template inside a fence · **BUILD** · Fable 5.1, high · switch · (FIRST)', '```', '',
      '## M9-T1 · the block · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M9-T12)', '- Status: TODO', '',
      '## M9-T2 · switch deleted by the lead · **BUILD** · Sonnet 5.5, high · (AFTER M9-T1)', '- Status: TODO', '',
      '## M9-T3 · a rung the map lacks · **BUILD** · Haiku 4.5, low · switch · (AFTER M9-T2)', '- Status: TODO', '',
      '## M9-T4 · another switched block · **BUILD** · Opus 5.5, xhigh · switch · (LAST)', '- Status: TODO', '',
      '## M9-T5 · a gate where the lead plays a part · **BUILD + LEAD go** · Opus 5.5, max · switch · (AFTER M9-T4)', '- Status: TODO', '',
      '## M9-T6 · Closing live walk — **the USER GATE** · Sonnet 5.5, high · switch · (AFTER M9-T5)', '- Status: TODO', '',
      '## M9-T7 · a bold word in the title and a real tag — **the USER GATE** · **JUDGE (+ one JUDGE scribe)** · Opus 5.5, max · switch · (LAST)', '- Status: TODO', ''].join('\n'),
    [`${ROOT}/${M8}`]: ['# M8', '', '## M8-T1 · the other plan · **BUILD** · Fable 5.1, high · switch · (FIRST)', '- Status: TODO', ''].join('\n'),
  }
  const user = text => ({ role: 'user', text, toolUses: [] })
  const asst = text => ({ role: 'assistant', text, toolUses: [] })
  const opened = p => `<ide_opened_file>The user opened the file ${p} in the IDE. This may or may not be related to the current task.</ide_opened_file>`
  const remind = `<system-reminder>gitStatus:  M ${M8}</system-reminder>`
  const clear = [user('<local-command-caveat>The command below was run directly.</local-command-caveat>'),
    user('<command-name>/clear</command-name> <command-message>clear</command-message> <command-args></command-args>')]
  const S = {
    launch: [user(`Read ${M9} and execute M9-T1 yourself — you are the task agent, not an orchestrator.`)],
    lead: [...clear, user(remind + opened(`${ROOT}/${M9}`) + 'read this and do task T1')],
    whole: [user(opened(`${ROOT}/${M9}`) + 'read this and do task M9-T1')],
    noswitch: [user(opened(`${ROOT}/${M9}`) + 'read this and do task T2')],
    unmapped: [user(opened(`${ROOT}/${M9}`) + 'read this and do task T3')],
    nolaunch: [user(opened(`${ROOT}/${M9}`) + 'what does this plan say about tests?'), asst('It says…'), user('ok, do task T1')],
    followup: [user(opened(`${ROOT}/${M9}`) + 'read this and do task T4'), asst('Done.'), user('now do task T1')],
    leadgo: [user(opened(`${ROOT}/${M9}`) + 'read this and do task T5')],
    boldtitle: [user(opened(`${ROOT}/${M9}`) + 'read this and do task T6')],
    boldtitletag: [user(opened(`${ROOT}/${M9}`) + 'read this and do task T7')],
    twoids: [user(opened(`${ROOT}/${M9}`) + 'do task T1 then task T4')],
    noplan: [user(opened(`${ROOT}/nope.md`) + 'read this and do task T1')],
    capped: Array.from({ length: TRANSCRIPT_CAP }, (_, i) => (i ? asst('…') : user(`Read ${M9} and execute M9-T1`))),
  }
  let n = 0
  const fake = messages => {
    const reads = [], sid = `s${++n}`
    return { reads, $: { session: { id: async () => sid, root: async () => ROOT, messages: async () => messages },
      fs: { read: async p => { reads.push(p); if (!(p in FILES)) throw new Error(`ENOENT ${p}`); return FILES[p] } } } }
  }
  let hook = null
  register((ev, h) => { if (ev === 'turn.step') hook = h })
  check('register hooks turn.step', typeof hook === 'function')
  const E = { turnId: 't', index: 0, model: 'claude-opus-5-5', effort: 'high', messageCount: 3 }
  const step = async ($, e = E) => {
    let sent = null
    const next = async function* (x) { sent = x; return { done: true } }
    const g = hook($, e, next)
    for (let r = await g.next(); !r.done; r = await g.next()) { /* no chunks */ }
    return sent
  }
  const on = (s, model, effort) => s && s.model === model && s.effort === effort
  const untouched = s => on(s, E.model, E.effort)
  const probe = async (name, want) => step(fake(S[name]).$).then(s => want ? on(s, ...want) : untouched(s))
  const SONNET_MED = ['claude-sonnet-5-5', 'medium']

  check('§14.5 launch prompt -> the rung of M9-T1', await probe('launch', SONNET_MED))
  check("lead's form (IDE plan, local id, after /clear) -> M9-T1's rung, not the fenced copy's or M9-T12's", await probe('lead', SONNET_MED))
  check('a whole id in the lead\'s form', await probe('whole', SONNET_MED))
  check('no ` · switch` -> untouched', await probe('noswitch'))
  check('a rung RUNGS lacks -> untouched', await probe('unmapped'))
  check('a type tag that names the lead\'s part (`BUILD + LEAD go`) -> the rung', await probe('leadgo', ['claude-opus-5-5', 'max']))
  check('a tag with a parenthesis and a plus (`JUDGE (+ one JUDGE scribe)`) after a bold title word -> the rung', await probe('boldtitletag', ['claude-opus-5-5', 'max']))
  check('a bold word in the title and no tag -> untouched', await probe('boldtitle'))
  check('a first prompt naming no block -> untouched, a later one ignored', await probe('nolaunch'))
  check('a follow-up naming another task -> the launch block holds', await probe('followup', ['claude-opus-5-5', 'xhigh']))
  check('two ids in the launch prompt -> untouched', await probe('twoids'))
  check('a plan that does not read -> untouched, no throw', await probe('noplan'))
  check('a transcript at the cap -> untouched', await probe('capped'))
  {
    const f = fake(S.launch)
    const sub = await step(f.$, { ...E, agentId: 'a1' })
    const main1 = await step(f.$), main2 = await step(f.$, { ...E, index: 1, effort: 'xhigh' })
    check("a sub-agent's request -> untouched", untouched(sub))
    check('every main-loop request of the session switched', on(main1, ...SONNET_MED) && on(main2, ...SONNET_MED))
    check('the plan read once per session', f.reads.length === 1)
  }
  check('explain reads the same decision', (await decide(F.launchOf(S.lead), ROOT, p => FILES[p])).model === 'claude-sonnet-5-5')

  const plant = async (name, fn, bug, probeFn) => {
    const saved = F[fn]
    F[fn] = bug
    let ok
    try { ok = await probeFn() } finally { F[fn] = saved }
    plants.push([name, !ok])
    check('plant red: ' + name, !ok)
  }
  await plant('the rating read off another block', 'headingFor',
    (text, id) => text.split('\n').find(l => l.startsWith('## ') && l.includes(id)) || null, () => probe('lead', SONNET_MED))
  await plant('a heading without the switch segment switched', 'rungOf',
    h => { const s = h.split(' · '); const t = s.findIndex(x => /^\*\*/.test(x)); return t > 0 ? s[t + 1] : null }, () => probe('noswitch'))
  await plant('a LEAD go gate left untouched', 'rungOf',
    h => { const seg = h.split(' · ').map(x => x.trim()); const t = seg.findIndex((x, i) => i > 0 && /^\*\*[A-Z][A-Za-z-]*\*\*$/.test(x)); return t > 0 && seg[t + 2] === 'switch' && seg[t + 1] ? seg[t + 1] : null },
    () => probe('leadgo', ['claude-opus-5-5', 'max']))
  await plant('a bold title word read as the tag', 'rungOf',
    h => { const seg = h.split(' · ').map(x => x.trim()); const t = seg.findIndex((x, i) => i > 0 && /\*\*[^*]+\*\*/.test(x)); return t > 0 && seg[t + 2] === 'switch' && seg[t + 1] ? seg[t + 1] : null },
    () => probe('boldtitle'))
  await plant('an unmapped rung switched', 'mapRung',
    r => { const m = /^(.+), (\w+)$/.exec(r); return m ? { model: 'claude-' + m[1].toLowerCase().replace(/[\s.]+/g, '-'), effort: m[2] } : null },
    () => probe('unmapped'))
  await plant('a session with no launch prompt switched', 'launchOf',
    msgs => { let o = null; for (const m of msgs) { const x = /opened the file (.+?) in the IDE/.exec(m.text); if (x) o = o || x[1]; const p = stripTags(m.text).trim(); if (m.role === 'user' && F.idsOf(p).length) return { prompt: p, opened: o, raw: m.text } } return null },
    () => probe('nolaunch'))
  await plant('the local id resolved in another plan than the opened one', 'planOf',
    l => (/(\S+_implementation_plan\.md)/.exec(l.raw) || [])[1] || l.opened, () => probe('lead', SONNET_MED))

  const inside = (dir, root) => { const r = pathm.relative(root, dir); return r === '' || (!r.startsWith('..') && !pathm.isAbsolute(r)) }
  const defOut = () => !inside(pathm.resolve(F.defaultOut({}, '/home/u', pathm.join)), pathm.resolve('/w/proj')) &&
    F.defaultOut({ CLAUDE_CONFIG_DIR: '/cfg' }, '/home/u', pathm.join) === pathm.join('/cfg', PLUGIN + '-src') &&
    F.defaultOut({}, '/home/u', pathm.join) === pathm.join('/home/u', '.claude', PLUGIN + '-src')
  check("build's default folder is the client's own, beside no checkout", defOut())
  await plant('build\'s folder inside the checkout', 'defaultOut', (env, home, join) => join('/w/proj', '.claude', PLUGIN), defOut)

  // the file itself: what the client refuses to load, and the folder `build` writes
  const self = fs.readFileSync(P.argv[1], 'utf8')
  check('no import of any kind (the client loads none)', !/^\s*import[\s{*'"]/m.test(self) && !new RegExp('\\bimport\\s*\\(').test(self) && !self.includes('import' + '.meta'))
  const tmp = fs.mkdtempSync(pathm.join(os.tmpdir(), 'switch_'))
  try {
    const files = build(P, fs, pathm, tmp)
    const rd = r => fs.readFileSync(pathm.join(tmp, r), 'utf8')
    const mk = JSON.parse(rd('.claude-plugin/marketplace.json')), pj = JSON.parse(rd(`plugins/${PLUGIN}/.claude-plugin/plugin.json`))
    const hk = JSON.parse(rd(`plugins/${PLUGIN}/hooks/hooks.json`))
    check('build writes the marketplace, the manifest, hooks.json and the module', files.length === 4 && mk.plugins[0].name === pj.name &&
      hk.modules[0] === './switch.mjs' && rd(`plugins/${PLUGIN}/hooks/switch.mjs`) === self)
  } finally { fs.rmSync(tmp, { recursive: true, force: true }) }

  const failed = checks.filter(c => !c[1])
  const red = plants.filter(p => p[1]).length
  const ok = checks.length > 0 && !failed.length && red === plants.length && plants.length === 8
  out(P, json, { cmd: 'selftest', checks: checks.length, failed: failed.map(c => c[0]), plants_red: red, plants: plants.length }, [
    `selftest: checks=${checks.length} passed=${checks.length - failed.length} failed=${failed.length} · plants red ${red}/${plants.length}`,
    ...failed.map(c => `  FAIL ${c[0]}`)], ok, failed.length ? `${failed.length} failed` : 'a plant stayed green')
}

const NODE = globalThis.process
if (NODE && NODE.versions && NODE.versions.node && /(^|[\\/])switch\.mjs$/.test(String(NODE.argv[1] || ''))) {
  main(NODE).catch(e => { NODE.stdout.write(`=== NO-GO: switch.mjs stopped: ${e && e.message} ===\n`); NODE.exitCode = 1 })
}
