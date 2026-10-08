#!/usr/bin/env python3
"""launch.py — start, stop, status, smoke, without ever touching someone else's process.

PLAYBOOK annex §A.3. Python 3.9+, standard library only, Windows and POSIX.

    launch.py start [service...] [--asked]  spawn, record {pid, start_time, cmd}, wait for ready
    launch.py stop  [service...]            signal ONLY pids recorded here whose start time still matches
    launch.py status                        one line per service
    launch.py smoke [--deadline <s>]        start -> ready -> stop -> counts line -> verdict
    launch.py list | selftest

Every command ends in one counts line, failures and skips only when non-zero, a count-derived
`=== GO ===` / `=== NO-GO: <reason> ===` and exit 0 / 1; `--json` behind a flag (§A.0). `start` and
`smoke` write a log named by the task, so they refuse to run without `--task <ID>` (or $PB_TASK):
there is no shared default log (§A.0).

THE GUARD, AND THE WHOLE REASON THIS TOOL EXISTS. A pid is not an identity: the number is reused
the moment its process dies. Every entry records the process's START TIME, and `stop` re-reads it
live and signals only on a match. When the start time cannot be read at all, the tool REFUSES to
signal — it fails CLOSED, because the failure it guards against is killing someone else's process.
`start` refuses a port that is already listening rather than adopting the stack behind it. And
nothing runs UNRECORDED: a child whose record the pid file refuses (held open, on Windows) is
stopped at once through the Popen that spawned it, before the NO-GO naming the pid file.
`smoke` stops only what it started itself: a service already running from an earlier `start` is a
refusal, and that instance is left running and recorded — a smoke never stops the lead's own stack.

Three readiness kinds, declared per service by `ready` (inferred from `ready_url` / `port`):
  `url`   `ready_url` answers JSON whose `ready_key` carries one of `ready_values`   (§A.3)
  `port`  a TCP connect on `port` succeeds                                           (§A.3)
  `exit`  the process RUNS TO COMPLETION: it exits 0 within the deadline and its log carries no
          `ready_forbid` line — for a service that opens no port and serves no URL, such as a
          desktop app or a game whose boot smoke is a real headless boot that quits on its own flag.
          Shutdown noise a clean run prints is not a boot failure unless `ready_forbid` lists it.

A PRIVATE X DISPLAY, for a windowed app on a box with no display to spare (`xvfb`, Linux by
default): the tool starts ITS OWN Xvfb on the first display in `displays` whose lock file and socket
are both absent, and it is ready once the socket exists and the lock names that Xvfb's pid. The app
alone gets `DISPLAY=:<n>`, loses WAYLAND_DISPLAY (on a Wayland session DISPLAY alone does not keep a
window off the desktop) and the recipe's `env_drop`, and takes the recipe's `args` after its program
name. The Xvfb is recorded beside the app in the pid file and stopped with it, under the same
start-time guard. An X server this tool did not start is never reused or signalled; no Xvfb, or no
free display, is a refusal naming which, and nothing is started.

Manifest `launch.json` (§A.3, widened):
  services.<name>.cmd          argv list, or {"posix": [...], "nt": [...]}
  services.<name>.cwd/env      cwd relative to the root; env entries ADD to the parent environment
  services.<name>.port/host/ready_url/ready_key/ready_values/ready/ready_forbid/deadline_s
  services.<name>.ask          true -> a windowed or irreversible start is asked EVERY time, so the
                               tool refuses without --asked (PLAYBOOK §13 OPT-C)
  services.<name>.xvfb         true, or {"displays": [99, 109], "screen": "1920x1080x24", "args": [],
                               "env_drop": [], "platforms": ["linux"], "exe": "Xvfb", "x_dir": "/tmp"};
                               a platform off `platforms` runs the service as declared; `exe` is a
                               program on PATH or an argv list; `x_dir` is where every X server keeps
                               its lock and socket (the selftest points it at its own fixture)
  smoke                        the service names `smoke` runs (default: every service without `ask`)
  pid_file / deadline_s / logs_dir / root (default: the folder two above this file)
In `cmd` and `env`, `${NAME}` and `${NAME:-default}` expand from the environment and `{tmpdir}`
expands to a throwaway directory this run creates and removes — which is how a boot smoke gets a
throwaway user-data folder (XDG_DATA_HOME on Linux, %APPDATA% on Windows) and never reads or writes
a real user's saved state.
"""
import argparse, contextlib, io, json, os, re, shutil, signal, socket, subprocess, sys, tempfile, time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = Path(__file__).resolve().with_name('launch.json')
ENVVAR = re.compile(r'\$\{([A-Za-z_][A-Za-z0-9_]*)(?::-([^}]*))?\}')
REPLACE_TRIES, REPLACE_WAIT_S = 6, 0.05  # a held pid file: the wait doubles 0.05 -> 0.8 s, 1.55 s in all (plan.py's)
PROC = Path('/proc/self/stat').is_file()  # Linux; a box without it (macOS, BSD) reads ps
XVFB = {'displays': [99, 109], 'screen': '1920x1080x24', 'args': [], 'env_drop': [], 'platforms': ['linux'],
        'exe': 'Xvfb', 'x_dir': '/tmp'}


class Unreadable(Exception):
    """The manifest cannot be read as a JSON object of services: a verdict, never a traceback."""


# ------------------------------------------------------------------ identity
def _win_start_time(pid):
    """The process creation FILETIME, via ctypes — no subprocess, ~0 ms (a PowerShell CIM query costs 1.7 s)."""
    try:
        import ctypes, ctypes.wintypes as wt

        class FILETIME(ctypes.Structure):
            _fields_ = [('lo', wt.DWORD), ('hi', wt.DWORD)]

        k32 = ctypes.WinDLL('kernel32', use_last_error=True)
        k32.OpenProcess.restype, k32.OpenProcess.argtypes = wt.HANDLE, [wt.DWORD, wt.BOOL, wt.DWORD]
        k32.GetProcessTimes.argtypes = [wt.HANDLE] + [ctypes.POINTER(FILETIME)] * 4
        k32.GetExitCodeProcess.argtypes = [wt.HANDLE, ctypes.POINTER(wt.DWORD)]
        handle = k32.OpenProcess(0x1000, False, pid)  # PROCESS_QUERY_LIMITED_INFORMATION
        if not handle:
            return None
        try:
            # A process someone still holds a handle to survives its own death as an object, and
            # OpenProcess keeps succeeding on it [measured on a Windows box: the selftest's own Popen
            # handle made a killed child read as alive]. STILL_ACTIVE is what separates the two, so
            # it is asked first.
            code = wt.DWORD()
            if not k32.GetExitCodeProcess(handle, ctypes.byref(code)) or code.value != 259:
                return None
            times = [FILETIME() for _ in range(4)]
            if not k32.GetProcessTimes(handle, *[ctypes.byref(t) for t in times]):
                return None
            return str((times[0].hi << 32) | times[0].lo)
        finally:
            k32.CloseHandle(handle)
    except Exception:  # no ctypes, no access, a hardened box: unreadable means we do not signal
        return None


def start_time(pid):
    """A value that changes when a pid is reused; None when it cannot be read -> refuse to signal."""
    if not isinstance(pid, int) or pid <= 0:
        return None
    if os.name != 'posix':
        return _win_start_time(pid)
    try:  # /proc/<pid>/stat, read past the comm field's own parentheses: state is 3, starttime 22
        raw = Path('/proc/%d/stat' % pid).read_text(encoding='utf-8', errors='replace')
        fields = raw[raw.rindex(')') + 2:].split()
        return None if fields[0] == 'Z' else fields[19]  # a reaped-but-not-waited zombie is gone
    except (OSError, ValueError, IndexError):
        if PROC:  # a box with /proc answers from it alone: one format, never compared against ps's
            return None
    try:
        out = subprocess.run(['ps', '-o', 'lstart=', '-p', str(pid)], capture_output=True, text=True)
        return out.stdout.strip() or None
    except OSError:
        return None


def alive(pid):
    return start_time(pid) is not None


def is_ours(entry):
    """The one predicate every signal goes through: recorded start time still matches the live one."""
    if not entry:
        return False
    live = start_time(entry.get('pid'))
    return live is not None and live == entry.get('start_time')


def pause(n):
    """The n-th wait of a poll: 10 ms doubling to 200 ms, so a fast child is seen at once and a slow
    one costs no busy loop."""
    time.sleep(min(0.2, 0.01 * 2 ** n))


def settle(entry, seconds):
    """True once `entry` is no longer ours (stopped, or its pid reused), polled for at most `seconds`."""
    began, n = time.monotonic(), 0
    while is_ours(entry):
        if time.monotonic() - began >= seconds:
            return False
        pause(n)
        n += 1
    return True


# ------------------------------------------------------------------ manifest
def pick_os(value):
    """{"posix": ..., "nt": ...} is read per-OS; anything else is returned as it stands."""
    if isinstance(value, dict) and value and set(value) <= {'posix', 'nt'}:
        return value.get(os.name)
    return value


def expand(text, tmpdir):
    text = str(text).replace('{tmpdir}', tmpdir or '')
    return ENVVAR.sub(lambda m: os.environ.get(m.group(1)) or (m.group(2) or ''), text)


def argv_for(svc, tmpdir):
    cmd = pick_os(svc.get('cmd'))
    return [expand(part, tmpdir) for part in cmd] if cmd else None


def env_for(svc, tmpdir):
    extra = pick_os(svc.get('env')) or {}
    env = dict(os.environ)
    env.update({k: expand(v, tmpdir) for k, v in extra.items()})
    return env


def ready_kind(svc):
    return svc.get('ready') or ('url' if svc.get('ready_url') else 'port' if svc.get('port') else 'exit')


def xvfb_recipe(svc):
    """The service's private-display recipe with its defaults filled in, or None: none declared, or
    this platform is off its list (the service then runs as declared — `cmd` per OS)."""
    raw = svc.get('xvfb')
    if not raw:
        return None
    recipe = dict(XVFB, **(raw if isinstance(raw, dict) else {}))
    return recipe if sys.platform in recipe['platforms'] else None


def own_group():
    """Popen keywords: POSIX, a session of its own (PGID == PID, so `stop`'s group signal is exact);
    Windows, a new process group."""
    if os.name == 'posix':
        return {'start_new_session': True}
    return {'creationflags': subprocess.CREATE_NEW_PROCESS_GROUP}


# ------------------------------------------------------------------ pid file
class WriteRefused(Exception):
    """A pid-file write that did not land: the file as it was, its temp gone. `Context.save()` keeps
    it for the verdict, which names the pid file — never a traceback."""


def insist(op, *args):
    """`op(*args)`, retried while it raises PermissionError — on Windows, another process holding the
    file without share-delete: an editor, an antivirus scan of the last write (a POSIX rename over an
    open file is never refused). REPLACE_TRIES tries, the wait doubling from REPLACE_WAIT_S — bounded,
    so a file that stays held gets a verdict, not a hang. The last refusal is raised. plan.py's retry."""
    for n in range(REPLACE_TRIES):
        try:
            return op(*args)
        except PermissionError:
            if n == REPLACE_TRIES - 1:
                raise
            time.sleep(REPLACE_WAIT_S * 2 ** n)


def dropped(tmp):
    """Remove a temp that never landed; False only when even `insist` cannot."""
    try:
        insist(os.unlink, tmp)
    except FileNotFoundError:
        pass
    except OSError:
        return False
    return True


def read_pids(path):
    try:
        return json.loads(Path(path).read_text(encoding='utf-8'))
    except (OSError, ValueError):
        return {}


def write_pids(path, data):
    """`data` to the pid file whole or not at all: `<pid_file>.tmp`, then one os.replace through
    `insist`. On any failure the temp is removed before the error leaves, and an OSError leaves as
    WriteRefused naming the pid file — a bare replace onto a held pid file was a traceback, no
    verdict, a leaked `.tmp` and the child just spawned left running unrecorded."""
    path = Path(path)
    tmp, replacing, began = path.with_name(path.name + '.tmp'), False, time.monotonic()
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        tmp.write_text(json.dumps(data, indent=2, sort_keys=True) + '\n', encoding='utf-8')
        replacing, began = True, time.monotonic()
        insist(os.replace, tmp, path)
    except BaseException as e:
        left = not dropped(tmp)
        if not isinstance(e, OSError):
            raise
        why = '%s: %s' % ('WinError %s' % e.winerror if getattr(e, 'winerror', None) else 'errno %s' % e.errno,
                          e.strerror or e)
        if replacing and isinstance(e, PermissionError):
            what = 'held open by another process: the replace was refused %d times over %.2f s (%s)' % (
                REPLACE_TRIES, time.monotonic() - began, why)
        else:
            what = '%s failed (%s)' % ('the replace' if replacing else 'writing its temp', why)
        raise WriteRefused('pid file %s not written - %s; it is unchanged, %s' % (
            path, what, 'its temp LEFT: %s' % tmp if left else 'no temp left')) from None


# ------------------------------------------------------------------ probes
def port_busy(port, host='127.0.0.1'):
    with socket.socket() as sock:
        sock.settimeout(0.5)
        return sock.connect_ex((host, int(port))) == 0


def url_ready(svc):
    import urllib.request
    try:
        with urllib.request.urlopen(svc['ready_url'], timeout=2) as response:
            body = json.loads(response.read().decode('utf-8', 'replace'))
    except Exception:
        return False
    key = svc.get('ready_key')
    if not key:
        return True
    if not isinstance(body, dict):  # a list or a bare value carries no key: not ready, never a crash
        return False
    return str(body.get(key)) in [str(v) for v in (svc.get('ready_values') or [])]


def forbidden(svc, log_path):
    """The `exit` kind's second half: a line the boot must not have printed."""
    patterns = svc.get('ready_forbid') or []
    if not patterns:
        return None
    try:
        text = Path(log_path).read_text(encoding='utf-8', errors='replace')
    except OSError:
        return None
    for line in text.splitlines():
        for pattern in patterns:
            if re.search(pattern, line):
                return line.strip()[:160]
    return None


def terminate(pid, hard=False):
    """Signal the pid's whole group (POSIX) or tree (Windows), or a child outlives it — but NEVER a
    group the pid does not lead. `killpg(getpgid(pid))` signals whatever group the pid sits in, and a
    pid in its caller's group (a `stop` run bare from a script, `bash -c`, ssh) takes the caller with
    it: the selftest once SIGTERMed itself that way on Linux. So a leader's group, else the pid alone —
    the shell's `kill -- -PID || kill PID`, the leadership asked outright because killpg(pid) also
    reaches a group the pid has left. Every pid start_one records leads a session of its own."""
    try:
        if os.name == 'posix':
            sig = signal.SIGKILL if hard else signal.SIGTERM
            if os.getpgid(pid) == pid:
                os.killpg(pid, sig)
            else:
                os.kill(pid, sig)
        else:
            subprocess.run(['taskkill', '/PID', str(pid), '/T'] + (['/F'] if hard else []),
                           capture_output=True)
    except (OSError, ProcessLookupError):
        pass


def end(entry):
    """Stop one identified process: its group, wait, then kill -> (still running, needed the kill)."""
    terminate(entry['pid'])
    hard = not settle(entry, 5)
    if hard:
        terminate(entry['pid'], hard=True)
        settle(entry, 3)
    return is_ours(entry), hard


# ------------------------------------------------------------------ a private X display
def lock_pid(x_dir, n):
    """The pid an X server wrote into display n's lock file, or None."""
    with contextlib.suppress(OSError, ValueError):
        return int(Path(x_dir, '.X%d-lock' % n).read_text(encoding='ascii').strip())
    return None


def start_xvfb(name, recipe):
    """((display, Popen), None) for an Xvfb THIS call started, or (None, the refusal). A display with a
    lock file or a socket belongs to an X server we did not start, or to its leftovers: it is skipped,
    never reused, never signalled. Ready once the socket exists and the lock names our Xvfb's pid."""
    exe = recipe['exe']
    if isinstance(exe, list):
        argv = [str(part) for part in exe]
    else:
        found = shutil.which(str(exe))
        if not found:
            return None, '%s: no %s on PATH - the private display needs one (not started)' % (name, exe)
        argv = [found]
    x_dir, (lo, hi) = Path(recipe['x_dir']), recipe['displays']
    for n in range(int(lo), int(hi) + 1):
        sock = x_dir / '.X11-unix' / ('X%d' % n)
        if (x_dir / ('.X%d-lock' % n)).exists() or sock.exists():
            continue
        try:
            proc = subprocess.Popen(argv + [':%d' % n, '-screen', '0', str(recipe['screen']), '-nolisten', 'tcp'],
                                    stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                    stderr=subprocess.DEVNULL, **own_group())
        except OSError as exc:
            return None, '%s: could not start the X server (%s) (not started)' % (name, exc)
        began, k = time.monotonic(), 0
        while time.monotonic() - began < 5:
            if proc.poll() is not None:
                break  # another server took the display between our look and its start: the next one
            if sock.exists() and lock_pid(x_dir, n) == proc.pid:
                return (n, proc), None
            pause(k)
            k += 1
        terminate(proc.pid, hard=True)
        proc.wait()
    return None, '%s: no free X display in :%s-:%s (each has a lock or a socket) (not started)' % (name, lo, hi)


# ------------------------------------------------------------------ context
class Context:
    def __init__(self, args):
        try:
            self.manifest = json.loads(Path(args.manifest).read_text(encoding='utf-8-sig'))
        except (OSError, ValueError) as exc:
            raise Unreadable('manifest %s unreadable - %s' % (args.manifest, exc)) from None
        if not isinstance(self.manifest, dict) or not isinstance(self.manifest.get('services', {}), dict):
            raise Unreadable('manifest %s holds no object of services' % args.manifest)
        self.root = Path(self.manifest.get('root', ROOT))
        self.services = self.manifest.get('services', {})
        self.pid_file = self.root / self.manifest.get('pid_file', 'tools/pb/.launch.pid.json')
        self.logs = Path(args.logs_dir) if args.logs_dir else self.root / self.manifest.get('logs_dir', 'logs')
        self.task = args.task or os.environ.get('PB_TASK')
        self.deadline = args.deadline or self.manifest.get('deadline_s', 90)
        self.pids = read_pids(self.pid_file)
        self.refused = []  # every pid-file write that did not land

    def log_path(self, name):
        return self.logs / ('%s.launch.%s.log' % (self.task, name))

    def save(self):
        """True when the pid file took `self.pids`. A refusal is kept, never raised: each caller
        decides what it means for its own process, and the verdict names the file."""
        try:
            write_pids(self.pid_file, self.pids)
            return True
        except WriteRefused as e:
            self.refused.append(str(e))
            return False

    def reason(self, otherwise=None):
        """A verb's NO-GO reason: the first refused pid-file write, which names the file."""
        return self.refused[0] if self.refused else otherwise

    def names(self, wanted, for_smoke=False):
        if wanted:
            return [n for n in wanted if n in self.services], [n for n in wanted if n not in self.services]
        if for_smoke:
            declared = self.manifest.get('smoke')
            if declared:  # a name the manifest does not declare is a failure, never silently skipped
                return [n for n in declared if n in self.services], [n for n in declared if n not in self.services]
            return [n for n, s in self.services.items() if not s.get('ask')], []
        return list(self.services), []


NO_TASK = ('no --task: start and smoke write a log named by the task, so there is no shared one '
           '(`--task <ID>`, or $PB_TASK)')


# ------------------------------------------------------------------ the verbs
def start_one(ctx, name, asked, deadline=None):
    """Returns (ok, message). Nothing is spawned until every refusal arm has passed."""
    svc = ctx.services[name]
    if is_ours(ctx.pids.get(name)):
        return False, '%s: already running as pid %s - not restarting' % (name, ctx.pids[name]['pid'])
    if svc.get('ask') and not asked:
        return False, ('%s: windowed or irreversible, so the lead is asked EVERY time '
                       '(PLAYBOOK 13, OPT-C) - re-run with --asked once the go is given' % name)
    if not argv_for(svc, None):
        return False, '%s: no cmd for os %s' % (name, os.name)
    port = svc.get('port')
    if port and port_busy(port, svc.get('host', '127.0.0.1')):
        return False, '%s: port %s held by a process we did not start - not reusing' % (name, port)

    recipe = xvfb_recipe(svc)
    tmpdir = tempfile.mkdtemp(prefix='pb-launch-%s-' % name) if '{tmpdir}' in json.dumps(svc) else None
    argv, env, log_path = argv_for(svc, tmpdir), env_for(svc, tmpdir), ctx.log_path(name)
    deadline = deadline or svc.get('deadline_s') or ctx.deadline
    popen = dict(cwd=str((ctx.root / svc.get('cwd', '.')).resolve()), env=env,
                 stdin=subprocess.DEVNULL, stderr=subprocess.STDOUT, **own_group())
    display = xproc = None
    ctx.logs.mkdir(parents=True, exist_ok=True)
    with open(log_path, 'w', encoding='utf-8') as log:  # opened BEFORE anything is spawned (§4 Rule 4)
        log.write('# %s | launch %s | %s\n' % (ctx.task, name, time.strftime('%Y-%m-%dT%H:%M:%S%z')))
        if recipe:
            got, why = start_xvfb(name, recipe)
            if not got:
                log.write('# %s\n' % why)
                _clean_tmp({'tmpdir': tmpdir})
                return False, why
            display, xproc = got
            for key in ['WAYLAND_DISPLAY'] + [str(k) for k in recipe['env_drop']]:
                env.pop(key, None)
            env['DISPLAY'] = ':%d' % display
            argv[1:1] = [expand(a, tmpdir) for a in recipe['args']]
            log.write('# private X display :%d, its X server pid %d\n' % (display, xproc.pid))
        log.write('$ %s\n' % ' '.join(argv))
        log.flush()
        try:
            proc = subprocess.Popen(argv, stdout=log, **popen)
        except OSError as exc:
            if xproc:
                terminate(xproc.pid, hard=True)
                xproc.wait()
            _clean_tmp({'tmpdir': tmpdir})
            return False, '%s: could not spawn (%s)' % (name, exc)
    entry = {'pid': proc.pid, 'start_time': start_time(proc.pid), 'cmd': argv, 'service': name,
             'log': str(log_path), 'tmpdir': tmpdir, 'started': time.strftime('%Y-%m-%dT%H:%M:%S%z')}
    if xproc:
        entry['xvfb'] = {'pid': xproc.pid, 'start_time': start_time(xproc.pid), 'display': display}
    ctx.pids[name] = entry
    recorded = False
    try:
        recorded = ctx.save()
    finally:  # refused, or interrupted mid-retry: no `stop` could ever find it, so it dies here
        if not recorded:
            _unspawn(ctx, name, proc, entry, xproc)
    if not recorded:
        return False, '%s: pid %s spawned but its record was refused - stopped, nothing left running' % (
            name, proc.pid)
    # Read the start time BEFORE asking whether it finished, and the two answers cannot contradict:
    # a service of the `exit` kind can be done before we look, and "no start time" then means
    # FINISHED, not unidentifiable. Only a process that is still running and cannot be identified is
    # the dangerous one - it could never be stopped safely, so it is not left behind. The X server
    # has no such excuse: it was ready a moment ago, so an unreadable start time is unidentifiable.
    if (entry['start_time'] is None and proc.poll() is None) or (xproc and entry['xvfb']['start_time'] is None):
        _unspawn(ctx, name, proc, entry, xproc)
        ctx.save()
        return False, '%s: start time unreadable, so it could never be stopped safely - killed' % name

    kind, began, n = ready_kind(svc), time.monotonic(), 0
    while time.monotonic() - began < deadline:
        if kind == 'exit':
            if proc.poll() is not None:
                entry['exit'] = proc.returncode
                if not ctx.save():
                    return False, '%s: exited %d, but the pid file refused its record (log %s)' % (
                        name, proc.returncode, log_path)
                if proc.returncode != 0:
                    return False, '%s: exited %d (log %s)' % (name, proc.returncode, log_path)
                bad = forbidden(svc, log_path)
                if bad:
                    return False, '%s: forbidden line %r in its output (log %s)' % (name, bad, log_path)
                return True, '%s: ready - exit 0, %.1fs (log %s)' % (name, time.monotonic() - began, log_path)
        elif proc.poll() is not None:
            return False, '%s: died before ready, exit %s (log %s)' % (name, proc.returncode, log_path)
        elif url_ready(svc) if kind == 'url' else port_busy(port, svc.get('host', '127.0.0.1')):
            return True, '%s: ready on %s, %.1fs (log %s)' % (
                name, svc.get('ready_url') or 'port %s' % port, time.monotonic() - began, log_path)
        pause(n)
        n += 1
    return False, '%s: not ready within %ss (log %s)' % (name, deadline, log_path)


def stop_one(ctx, name):
    """Returns (ok, message). The ONLY place a RECORDED pid is signalled (`_unspawn` signals a Popen);
    the app first, then its private X server, each only when its start time still matches."""
    entry = ctx.pids.get(name)
    if not entry:
        return True, '%s: nothing recorded' % name
    ok, parts = True, []
    for label, rec in [('pid', entry)] + ([('its X server :%s, pid' % entry['xvfb'].get('display'),
                                            entry['xvfb'])] if entry.get('xvfb') else []):
        pid = rec.get('pid')
        if not is_ours(rec):
            parts.append('%s %s not signalled - %s' % (label, pid, 'gone' if start_time(pid) is None else
                                                       'a DIFFERENT process now (start time moved)'))
            continue
        still, hard = end(rec)
        ok = ok and not still
        parts.append('%s %s %s' % (label, pid, 'is STILL RUNNING after SIGKILL' if still else
                                   'stopped' + (' (needed SIGKILL)' if hard else '')))
    ctx.pids.pop(name, None)
    _clean_tmp(entry)
    if not ctx.save():
        return False, '%s: %s; the pid file refused to drop its entry' % (name, '; '.join(parts))
    return ok, '%s: %s' % (name, '; '.join(parts))


def _unspawn(ctx, name, proc, entry, xproc=None):
    """Kill the children this process spawned and still holds, and forget them. An unwaited Popen's
    pid cannot have been reused, so this is the one signal that needs no start-time match — the way to
    stop a child that could never be identified, or never be found (its record refused)."""
    for child in (proc, xproc):
        if child is not None:
            terminate(child.pid, hard=True)
            child.wait()
    ctx.pids.pop(name, None)
    _clean_tmp(entry)


def _clean_tmp(entry):
    """Only ever the throwaway directory this tool created for that service (§A.0: never a file)."""
    tmpdir = entry.get('tmpdir')
    if tmpdir and os.path.basename(tmpdir).startswith('pb-launch-'):
        shutil.rmtree(tmpdir, ignore_errors=True)


def finish(args, counts, failures=(), skips=(), lines=(), reason=None, extra=None):
    failures = list(failures)
    if failures and not reason:
        reason = '%d failure(s)' % len(failures)
    if args.json:
        print(json.dumps(dict(extra or {}, counts=counts, failures=failures, skips=list(skips),
                              verdict='NO-GO' if reason else 'GO', reason=reason)))
    else:
        for line in lines:
            print(line)
        print(', '.join('%s: %s' % kv for kv in counts.items()))
        for item in failures:
            print('FAIL ' + item)
        for item in skips:
            print('SKIP ' + item)
        print('=== NO-GO: %s ===' % reason if reason else '=== GO ===')
    return 1 if reason else 0


def cmd_start(args):
    ctx = Context(args)
    if not ctx.task:
        return finish(args, {'services': 0, 'ready': 0}, reason=NO_TASK)
    names, unknown = ctx.names(args.services)
    lines, failures = [], ['unknown service %r' % n for n in unknown]
    for name in names:
        ok, message = start_one(ctx, name, args.asked)
        lines.append(message)
        if not ok:
            failures.append(message)
    total = len(names) + len(unknown)
    return finish(args, {'services': total, 'ready': total - len(failures)}, failures, (), lines,
                  ctx.reason(None if total else 'no service selected'))


def cmd_stop(args):
    ctx = Context(args)
    names, unknown = ctx.names(args.services)
    lines, failures = [], ['unknown service %r' % n for n in unknown]
    for name in names:
        ok, message = stop_one(ctx, name)
        lines.append(message)
        if not ok:
            failures.append(message)
    total = len(names) + len(unknown)
    return finish(args, {'services': total, 'stopped': total - len(failures)}, failures, (), lines,
                  ctx.reason(None if total else 'no service selected'))


def cmd_status(args):
    ctx = Context(args)
    lines, running = [], 0
    for name in ctx.services:
        entry = ctx.pids.get(name)
        if is_ours(entry):
            running += 1
            xv = entry.get('xvfb')
            lines.append('%s running pid %s since %s%s' % (
                name, entry['pid'], entry.get('started'),
                ' on private X display :%s' % xv.get('display') if is_ours(xv) else ''))
        elif entry:
            lines.append('%s STALE entry pid %s - that pid is not ours any more' % (name, entry.get('pid')))
        else:
            lines.append('%s stopped' % name)
    return finish(args, {'services': len(ctx.services), 'running': running}, (), (), lines,
                  None if ctx.services else 'no service declared')


def cmd_list(args):
    ctx = Context(args)
    lines = []
    for name, svc in ctx.services.items():
        argv = argv_for(svc, '<tmpdir>')
        lines.append('%s | ready=%s%s%s | %s' % (name, ready_kind(svc), ' | ask' if svc.get('ask') else '',
                                                 ' | private X display' if xvfb_recipe(svc) else '',
                                                 ' '.join(argv) if argv else 'NOT RUN on os %s' % os.name))
    return finish(args, {'services': len(ctx.services)}, (), (), lines,
                  None if ctx.services else 'no service declared')


def cmd_smoke(args):
    """start -> ready -> stop -> counts -> verdict, and no child is left behind (§A.3). Only what this
    smoke started is stopped: a refusal spawned nothing, and an instance an earlier `start` began is
    left running and recorded."""
    ctx = Context(args)
    if not ctx.task:
        return finish(args, {'services': 0, 'ready': 0, 'failures': 1}, reason=NO_TASK)
    names, unknown = ctx.names(args.services, for_smoke=True)
    lines, failures, ready = [], ['unknown service %r' % n for n in unknown], 0
    for name in names:
        before = ctx.pids.get(name)
        ok, message = start_one(ctx, name, args.asked, args.deadline)
        lines.append(message)
        if ok:
            ready += 1
        else:
            failures.append(message)
        entry = ctx.pids.get(name)
        if entry is not None and entry is not before:  # this smoke's own spawn, and nothing else
            # stop_one fails for any process of the entry still ours after the kill: a survivor is its NO-GO
            stopped, stop_message = stop_one(ctx, name)
            lines.append(stop_message)
            if not stopped:
                failures.append(stop_message)
        if ctx.log_path(name).is_file():
            lines.append('%s: log %s' % (name, ctx.log_path(name)))
    return finish(args, {'services': len(names), 'ready': ready, 'failures': len(failures)},
                  failures, (), lines, ctx.reason(None if names else 'no smoke service declared'))


# ------------------------------------------------------------------ selftest
# The selftest's stand-in for Xvfb: `-c` code whose argv is <x_dir> :<n> -screen 0 <geometry>
# -nolisten tcp. Like an X server it refuses a display whose lock is held, writes the lock (its pid)
# and the socket, and removes both when it is signalled; it also writes `argv.<n>` (its pid and its
# arguments) so the selftest can check what it was given and that it is gone afterwards.
FAKE_X = '''import os, pathlib, signal, sys, time
x_dir, n = pathlib.Path(sys.argv[1]), int(sys.argv[2].lstrip(":"))
lock, sock = x_dir / (".X%d-lock" % n), x_dir / ".X11-unix" / ("X%d" % n)
if lock.exists():
    sys.exit(1)
sock.parent.mkdir(parents=True, exist_ok=True)
(x_dir / ("argv.%d" % n)).write_text("%d %s" % (os.getpid(), " ".join(sys.argv[2:])))
lock.write_text("%10d\\n" % os.getpid())
sock.write_text("")
def bye(*_):
    for path in (lock, sock):
        try:
            path.unlink()
        except OSError:
            pass
    os._exit(0)
signal.signal(signal.SIGTERM, bye)
time.sleep(60)
bye()
'''

# The throwaway web service: §A.3's `http.server` (its own `test()` entry, which `-m http.server` runs),
# serving its cwd on the port it is handed, after writing its own pid to the file it is handed — so a
# teardown is proven by the pid THIS run started, never by a port another run may hold. Like every
# stray here it ends by itself after 60 s, so a broken tool under a red arm leaves nothing for long.
SERVE = ('import http.server as h, os, pathlib, sys, threading; pathlib.Path(sys.argv[2]).write_text(str(os.getpid())); '
         'threading.Timer(60, os._exit, (0,)).start(); '
         'h.test(HandlerClass=h.SimpleHTTPRequestHandler, port=int(sys.argv[1]), bind="127.0.0.1")')


def selftest(args):
    """Red-armed (§A.0). Every refusal arm is planted, and every plant is proven STILL ALIVE after."""
    checks, failures, skips = [], [], []
    strays, kin, held = [], [], {}  # kin: plants' own children, no Popen here holds them; held: reserved ports

    def check(name, ok):
        checks.append(name)
        if not ok:
            failures.append(name)

    # Read a result the way a BROKEN tool might have left it. A selftest that indexes its way into
    # a missing key dies with a traceback and no verdict line - which is exactly what happened on
    # two plants before these existed, and what §4 Rule 4 forbids a runner to do. A missing answer
    # must become a NAMED failed check, never an exception.
    counts = lambda out: (out.get('counts') or {})
    failed = lambda out: ' '.join(out.get('failures') or [])
    recorded = lambda name: (read_pids(tmp / pid_file).get(name) or {})

    def spawn(code, *argv, cwd=None):
        # POSIX: a session of its own, PGID == PID, as every pid start_one records. A plant sharing
        # this run's group once made CONTROL's `stop` SIGTERM the selftest itself.
        proc = subprocess.Popen([sys.executable, '-c', code] + [str(a) for a in argv], stdout=subprocess.DEVNULL,
                                stderr=subprocess.DEVNULL, stdin=subprocess.DEVNULL, cwd=cwd,
                                start_new_session=os.name == 'posix')
        strays.append(proc)
        return proc

    def lives(proc):
        """`proc` is still running 0.3 s on — a group member only DYING must not read alive."""
        try:
            proc.wait(timeout=0.3)
        except subprocess.TimeoutExpired:
            return True
        return False

    def waited(read, seconds=5):
        """`read()`'s first answer that is not None, polled for at most `seconds`; else None."""
        began, n = time.monotonic(), 0
        while time.monotonic() - began < seconds:
            with contextlib.suppress(OSError, ValueError):
                got = read()
                if got is not None:
                    return got
            pause(n)
            n += 1
        return None

    def ended(pid):
        """`pid` reads gone within 2 s: a process the tool just stopped can still be exiting, or waiting
        to be reaped, for a beat after the tool returns; a leak (every stray here lives 60 s) does not end."""
        return pid is not None and waited(lambda: True if start_time(pid) is None else None, 2) is True

    # A port NUMBER is not an identity. reserve() KEEPS the bound socket, so no concurrent caller -
    # here or in another selftest - is handed the same number. Two runs handed one port saw each other's
    # child: a false NO-GO under a parallel loop, never alone. On Linux the reservation is SO_REUSEADDR
    # and never listens, so the service's own server (http.server sets SO_REUSEADDR too) binds and
    # listens beside it while the number stays taken for the whole run; elsewhere, where SO_REUSEADDR
    # means something else, release() closes it at the instant the port goes to a child.
    share = sys.platform.startswith('linux')

    def reserve():
        sock = socket.socket()
        if share:
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        sock.bind(('127.0.0.1', 0))
        held[sock.getsockname()[1]] = sock
        return sock.getsockname()[1]

    def release(port):
        sock = None if share else held.pop(port, None)
        if sock is not None:
            sock.close()
        return port

    def served(port):
        return {'cmd': [sys.executable, '-c', SERVE, str(port), str(tmp / ('web.%d.pid' % port))],
                'port': port, 'deadline_s': 20}

    def server_pid(port):
        return waited(lambda: int((tmp / ('web.%d.pid' % port)).read_text(encoding='utf-8')), 2)

    sleep = 'import time; time.sleep(%d)'
    say = lambda out, code=0: [sys.executable, '-c',
                               'import sys; print(%r); sys.exit(%d)' % (out, code)]
    # NOT TemporaryDirectory: a plant that breaks the guard leaves a child alive, that child holds
    # its own log open, and on Windows the context manager's teardown then raises PermissionError —
    # so the tool died with a traceback and NO verdict line, which is the one thing §4 Rule 4 does
    # not allow a runner to do [measured on a Windows box, by a planted-bug campaign, on the
    # port-probe and never-signal plants]. The fixture is torn down by hand instead, after every
    # process is accounted for, and a leftover is a NAMED check failure.
    tmp, pid_file = Path(tempfile.mkdtemp(prefix='launch_selftest_')), 'pids.json'
    try:
        echoed = {'cmd': [sys.executable, '-c', 'import os, sys; print(os.environ["PB_TEST_DIR"]);'
                                                ' sys.exit(0)'],
                  'ready': 'exit', 'ready_forbid': ['SCRIPT ERROR'],
                  'env': {'PB_TEST_DIR': '{tmpdir}'}, 'deadline_s': 20}
        windowed = {'cmd': say('never'), 'ready': 'exit', 'ask': True}

        def cli(manifest, *argv, env=None, task='ST'):
            """The real CLI, in its own process: the pid file and the guard are the subject."""
            try:
                done = subprocess.run([sys.executable, __file__, '--manifest', str(manifest)]
                                      + (['--task', task] if task else []) + list(argv),
                                      capture_output=True, text=True, timeout=60, env=env)
            except subprocess.TimeoutExpired:
                return None, ''
            return done.returncode, done.stdout + done.stderr[-300:]

        def run(services, *argv, env=None, task='ST', **extra):
            path = tmp / 'launch.json'
            path.write_text(json.dumps(dict({'services': services, 'pid_file': pid_file,
                                             'logs_dir': 'logs', 'root': str(tmp)}, **extra)))
            code, text = cli(path, *argv, '--json', env=env, task=task)
            try:
                return code, json.loads(text.splitlines()[0])
            except (ValueError, IndexError):
                return code, {'counts': {}, 'failures': [text[-300:]], 'reason': 'no json'}

        # --- the clean fixture: §A.3's own http.server service, start -> ready -> stop
        port = release(reserve())
        code, out = run({'web': served(port)}, 'smoke')
        check('clean port service: smoke GO, ready 1/1',
              code == 0 and counts(out) == {'services': 1, 'ready': 1, 'failures': 0})
        pid = server_pid(port)
        check('smoke leaves no child behind (by the pid it started) and no pid entry',
              ended(pid) and read_pids(tmp / pid_file) == {})
        code, plain = cli(tmp / 'launch.json', 'smoke')
        check('smoke prints the log path, and that log exists (A.3)',
              'ST.launch.web.log' in plain and (tmp / 'logs' / 'ST.launch.web.log').is_file()
              and plain.rstrip().endswith('=== GO ==='))
        port = release(reserve())
        code, out = run({'web': served(port)}, 'start')
        check('start -> GO, and the pid file records pid + start_time + cmd',
              code == 0 and set(recorded('web')) >= {'pid', 'start_time', 'cmd'})
        web_pid = recorded('web').get('pid')
        code, out = run({'web': served(port)}, 'status')
        check('status names it running', code == 0 and counts(out).get('running') == 1)
        code, out = run({'web': served(port)}, 'start')
        check('a second start refuses rather than spawning a twin',
              code == 1 and 'already running' in failed(out))
        # --- PLANT (a smoke stops only what it started): the instance an earlier `start` began
        code, out = run({'web': served(port)}, 'smoke')
        check('PLANT a smoke over an instance an earlier start began: NO-GO, that instance STILL RUNNING and recorded',
              code == 1 and 'already running' in failed(out) and is_ours(recorded('web'))
              and recorded('web').get('pid') == web_pid)
        code, out = run({'web': served(port)}, 'stop')
        check('stop -> GO and the process is gone', code == 0 and ended(web_pid))
        code, out = run({'web': served(port)}, 'status')
        check('status after stop says stopped', code == 0 and counts(out).get('running') == 0)

        # --- PLANT 1 (§A.3): a port held by a process this tool did not start. The plant binds an
        # ephemeral port ITSELF and reports it, so its number never passes through a hand-off window
        said = tmp / 'foreign.port'
        foreign = spawn('import socket, sys, time\n'
                        's = socket.socket(); s.bind(("127.0.0.1", 0)); s.listen(5)\n'
                        'open(sys.argv[1], "w").write(str(s.getsockname()[1])); time.sleep(60)', said)
        fport = waited(lambda: int(said.read_text(encoding='utf-8')))
        code, out = run({'web': served(fport or 1)}, 'start')
        check('PLANT foreign port: start REFUSES for that reason',
              fport is not None and code == 1 and 'not reusing' in failed(out))
        check('PLANT foreign port: the foreign process is STILL ALIVE',
              foreign.poll() is None and alive(foreign.pid))
        check('PLANT foreign port: nothing was recorded for it', read_pids(tmp / pid_file) == {})
        foreign.kill()
        foreign.wait()

        # --- PLANT 2 (the guard): a live pid recorded with a DIFFERENT start time
        ghost = spawn(sleep % 60)
        write_pids(tmp / pid_file, {'web': {'pid': ghost.pid, 'start_time': 'a-different-boot',
                                            'cmd': ['ghost'], 'service': 'web'}})
        code, out = run({'web': served(1)}, 'stop')
        check('PLANT pid reuse: stop does NOT signal it, and says why',
              code == 0 and ghost.poll() is None and alive(ghost.pid))
        check('PLANT pid reuse: the stale entry is dropped', read_pids(tmp / pid_file) == {})
        # the control, so the guard is not merely "never stop anything"
        write_pids(tmp / pid_file, {'web': {'pid': ghost.pid, 'start_time': start_time(ghost.pid),
                                            'cmd': ['ghost'], 'service': 'web'}})
        code, out = run({'web': served(1)}, 'stop')
        ghost.poll()  # reap if it died; never wait on it - under a plant that stops nothing, a
        check('CONTROL matching start time: stop DOES kill it',  # blocking wait would stall the run
              code == 0 and ended(ghost.pid))

        # --- PLANT 3: a family — a leader in a session of its own and its child, in the leader's group
        # without leading one. Recording the child is a pid in its CALLER's group, the leader standing
        # in for the script, `bash -c` or ssh shell that ran `stop` bare; the group is not this run's,
        # so a tool that signals it reaches the family, never the selftest, which prints its verdict
        def family():
            """(the leader's Popen, its child's identity) — the child's pid comes back through a file."""
            said = tmp / ('kid.%d' % len(strays))
            leader = spawn('import subprocess, sys, time\n'
                           'kid = subprocess.Popen([sys.executable, "-c", %r])\n'
                           'with open(%r, "w") as f: f.write(str(kid.pid))\n'
                           'time.sleep(60)' % (sleep % 60, str(said)))
            pid = waited(lambda: int(said.read_text(encoding='utf-8')), 10)
            if pid is None:
                return leader, {}
            kin.append({'pid': pid, 'start_time': start_time(pid)})
            return leader, kin[-1]

        leader, kid = family()
        write_pids(tmp / pid_file, {'web': dict(kid, cmd=['kid'], service='web')})
        code, out = run({'web': served(1)}, 'stop')
        check('PLANT a recorded pid that does not lead its group: stop kills it ALONE, its group lives on',
              code == 0 and kid.get('start_time') is not None and settle(kid, 5) and lives(leader))
        leader.kill()
        leader.wait()
        # the control, so the fix is not merely "never signal a group": a started service's children die too
        leader, kid = family()
        write_pids(tmp / pid_file, {'web': {'pid': leader.pid, 'start_time': start_time(leader.pid),
                                            'cmd': ['leader'], 'service': 'web'}})
        code, out = run({'web': served(1)}, 'stop')
        leader.poll()  # reap, never wait: as CONTROL above
        check('CONTROL a recorded group leader: stop kills its WHOLE group, its child too',
              code == 0 and ended(leader.pid) and kid.get('start_time') is not None and settle(kid, 5))

        # --- the `url` kind, clean and each way its answer can fail. The answers come from one server
        # this selftest starts first, so every poll is answered and a plant's short deadline is spent
        # reading the wrong answer, never waiting for a server to come up; the service is a sleeper
        www = tmp / 'www'
        www.mkdir()
        for page, body in (('ok.json', {'status': 'ready'}), ('wrong.json', {'status': 'starting'}),
                           ('list.json', ['ready'])):
            (www / page).write_text(json.dumps(body), encoding='utf-8')
        wport = release(reserve())
        spawn(SERVE, wport, tmp / 'www.pid', cwd=str(www))
        up = waited(lambda: port_busy(wport) or None)

        def url_service(page, **over):
            return dict({'cmd': [sys.executable, '-c', sleep % 60], 'deadline_s': 20, 'ready_key': 'status',
                         'ready_url': 'http://127.0.0.1:%d/%s' % (wport, page), 'ready_values': ['ready']}, **over)

        code, out = run({'web': url_service('ok.json')}, 'smoke')
        check('url kind: the ready key carries an accepted value -> smoke GO',
              up and code == 0 and counts(out) == {'services': 1, 'ready': 1, 'failures': 0})
        for page, why in (('wrong.json', 'another value'), ('list.json', 'a body that is no object')):
            code, out = run({'web': url_service(page, deadline_s=0.15)}, 'smoke')
            check('PLANT url kind, %s: not ready, NO-GO for that reason, nothing left recorded' % why,
                  up and code == 1 and 'not ready within' in failed(out) and read_pids(tmp / pid_file) == {})

        # --- the `exit` kind, clean and each way it can fail
        code, out = run({'boot': echoed}, 'smoke')
        check('exit-kind service: exit 0 with no forbidden line -> ready',
              code == 0 and counts(out) == {'services': 1, 'ready': 1, 'failures': 0})
        boot_log = tmp / 'logs' / 'ST.launch.boot.log'
        log = boot_log.read_text(encoding='utf-8') if boot_log.is_file() else ''
        made = [l.strip() for l in log.splitlines() if 'pb-launch-boot-' in l]
        check('{tmpdir} reached the child env and was removed afterwards',
              bool(made) and not Path(made[-1]).exists())
        plants = {
            'a forbidden line in the output':
                ({'boot': dict(echoed, cmd=say('SCRIPT ERROR: bad'))}, 'forbidden line'),
            'a non-zero exit':
                ({'boot': dict(echoed, cmd=say('fine', 3))}, 'exited 3'),
            'a boot that never quits':  # it writes its pid, so its end is proven by identity, not a message
                ({'boot': dict(echoed, deadline_s=0.15, cmd=[
                    sys.executable, '-c', 'import os, sys, time; open(sys.argv[1], "w").write(str(os.getpid())); '
                    'time.sleep(60)', str(tmp / 'boot.pid')])}, 'not ready within'),
        }
        for name, (services, why) in plants.items():
            code, out = run(services, 'smoke')
            joined = failed(out)
            check('PLANT %s: NO-GO for its own reason' % name,
                  code == 1 and why in joined and counts(out).get('ready') == 0)
            pid = waited(lambda: int((tmp / 'boot.pid').read_text(encoding='utf-8')), 2) \
                if name == 'a boot that never quits' else None
            check('PLANT %s: no child left behind, its entry dropped' % name,
                  'STILL RUNNING' not in joined and read_pids(tmp / pid_file) == {}
                  and (name != 'a boot that never quits' or ended(pid)))

        # --- a forbidden line carrying `≤`, on a cp1252 stdout: the encoding is the process entry's, so
        # only a real child on a forced-cp1252 pipe can see it, never `run()`'s `--json`. The boot
        # writes its line as UTF-8 bytes, hex in its argv, so the log's `$ <cmd>` echo cannot match first
        line = 'SCRIPT ERROR: header 612 ≤ 600\n'.encode('utf-8').hex()
        boot = dict(echoed, cmd=[sys.executable, '-c',
                                 'import sys; sys.stdout.buffer.write(bytes.fromhex(%r))' % line])
        (tmp / 'launch.json').write_text(json.dumps({'services': {'boot': boot}, 'pid_file': pid_file,
                                                     'logs_dir': 'logs', 'root': str(tmp)}))
        child = subprocess.run([sys.executable, __file__, '--manifest', str(tmp / 'launch.json'),
                                '--task', 'ST', 'smoke'], capture_output=True, timeout=60,
                               env=dict(os.environ, PYTHONIOENCODING='cp1252'))
        check('a forbidden line carrying U+2264 on a cp1252 stdout: exit 1, the verdict, the line in UTF-8',
              child.returncode == 1 and child.stdout.rstrip().endswith(b'=== NO-GO: 1 failure(s) ===')
              and '612 ≤ 600'.encode('utf-8') in child.stdout)

        # --- the task names the log (§A.0): no --task and no $PB_TASK -> refused before anything runs
        bare = {k: v for k, v in os.environ.items() if k != 'PB_TASK'}
        lone = {'lone': dict(echoed, cmd=say('lone ran'))}
        for verb in ('smoke', 'start'):
            code, out = run(lone, verb, env=bare, task=None)
            check('PLANT no --task and no $PB_TASK: %s REFUSED naming --task, no log, nothing recorded' % verb,
                  code == 1 and '--task' in (out.get('reason') or '') and read_pids(tmp / pid_file) == {}
                  and not list((tmp / 'logs').glob('*.launch.lone.log')))
        code, out = run(lone, 'smoke', env=dict(bare, PB_TASK='ENV1'), task=None)
        check('$PB_TASK names the log when --task is absent',
              code == 0 and (tmp / 'logs' / 'ENV1.launch.lone.log').is_file())

        # --- the manifest: unreadable -> a verdict, never a traceback; saved with a byte-order mark -> read
        code, text = cli(tmp / 'nope.json', 'status', '--json')
        try:
            verdict = json.loads(text.splitlines()[0])
        except (ValueError, IndexError):
            verdict = {}
        check('an unreadable manifest -> NO-GO naming it, the JSON verdict, no traceback',
              code == 1 and 'nope.json' in (verdict.get('reason') or '') and 'Traceback' not in text)
        (tmp / 'bom.json').write_bytes(b'\xef\xbb\xbf' + json.dumps({'services': lone, 'root': str(tmp)}).encode())
        code, text = cli(tmp / 'bom.json', 'list')
        check('a manifest saved with a byte-order mark is read', code == 0 and text.rstrip().endswith('=== GO ==='))

        # --- the replace Windows refuses while another process holds the pid file, made to fail
        # in-process: a held handle never refuses a POSIX rename, so only an injection bites on both boxes.
        # The spawned child is named by the record the tool TRIED to write, so a stray is seen, not guessed.
        # The retry's wait is scaled down here only: the checks count the tries, never the seconds.
        def smoke_refused(fails, services):
            """`smoke` in this process, os.replace refusing its first `fails` calls -> (rc, out, tried)."""
            tried, real, wait, buf = [], os.replace, REPLACE_WAIT_S, io.StringIO()

            def refuse(src, dst):
                tried.append(read_pids(src))
                if len(tried) <= fails:
                    raise PermissionError(13, 'injected: held open without share-delete')
                return real(src, dst)
            (tmp / 'launch.json').write_text(json.dumps({'services': services, 'pid_file': pid_file,
                                                         'logs_dir': 'logs', 'root': str(tmp)}))
            os.replace = refuse
            globals()['REPLACE_WAIT_S'] = wait / 50
            try:
                with contextlib.redirect_stdout(buf):
                    rc = main(['--manifest', str(tmp / 'launch.json'), '--task', 'ST', 'smoke', '--json'])
            except Exception as exc:  # the bare replace: a traceback, no verdict
                rc, buf = None, io.StringIO('%s: %s' % (type(exc).__name__, exc))
            finally:
                os.replace = real
                globals()['REPLACE_WAIT_S'] = wait
            try:
                return rc, json.loads(buf.getvalue()), tried
            except ValueError:
                return rc, {'counts': {}, 'failures': [buf.getvalue()[-300:]], 'reason': 'no json'}, tried

        sleeper = {'cmd': [sys.executable, '-c', sleep % 60], 'ready': 'exit', 'deadline_s': 20}
        began = time.monotonic()
        code, out, tried = smoke_refused(REPLACE_TRIES, {'boot': sleeper})
        spawned = [e['boot'] for e in tried if e.get('boot')]
        check('PLANT pid file held on every try: NO-GO naming it, bounded, no .tmp, its spawned child NOT running',
              code == 1 and str(tmp / pid_file) in (out.get('reason') or '') and len(tried) == REPLACE_TRIES
              and time.monotonic() - began < 10 and bool(spawned) and spawned[0].get('start_time') is not None
              and not any(is_ours(e) for e in spawned) and not (tmp / (pid_file + '.tmp')).exists()
              and read_pids(tmp / pid_file) == {})
        for entry in spawned:  # a broken tool's stray is that check's failure, never left on the box
            if is_ours(entry):
                terminate(entry['pid'], hard=True)
        code, out, tried = smoke_refused(1, {'boot': echoed})
        check('PLANT pid file held for one try: retried, the smoke GO, no .tmp, nothing left recorded',
              code == 0 and counts(out) == {'services': 1, 'ready': 1, 'failures': 0} and len(tried) >= 2
              and tried[0] == tried[1] and not (tmp / (pid_file + '.tmp')).exists()
              and read_pids(tmp / pid_file) == {})

        # --- a private X display, on a stand-in X server (FAKE_X) in a fixture folder: no Xvfb needed
        if os.name == 'posix':
            xdir = tmp / 'x'

            def recipe(**over):
                return dict({'exe': [sys.executable, '-c', FAKE_X, str(xdir)], 'x_dir': str(xdir),
                             'displays': [99, 101], 'platforms': [sys.platform],
                             'args': ['-X', 'pb_private_display']}, **over)

            shows = {'cmd': [sys.executable, '-c', 'import os, sys; print("DISPLAY=%s WAYLAND=%s XOPT=%s" % ('
                             'os.environ.get("DISPLAY"), os.environ.get("WAYLAND_DISPLAY"), '
                             '"pb_private_display" in sys._xoptions))'], 'ready': 'exit', 'deadline_s': 20}
            xenv = dict(os.environ, DISPLAY=':pb-parent', WAYLAND_DISPLAY='pb-wayland')
            app_log = lambda: (tmp / 'logs' / 'ST.launch.app.log').read_text(encoding='utf-8')

            def xserver(n):
                """(pid, arguments) the stand-in X server on :n wrote, or (None, '')."""
                got = waited(lambda: (xdir / ('argv.%d' % n)).read_text(encoding='utf-8') or None, 2)
                return (int(got.split()[0]), got.split(' ', 1)[1]) if got else (None, '')

            code, out = run({'app': dict(shows, xvfb=recipe())}, 'smoke', env=xenv)
            check('private display: smoke GO, the app alone on its own :99, WAYLAND_DISPLAY dropped, args in',
                  code == 0 and counts(out).get('ready') == 1
                  and 'DISPLAY=:99 WAYLAND=None XOPT=True' in app_log())
            xpid, xargs = xserver(99)
            check('private display: its X server ran -nolisten tcp and is gone after the smoke, nothing recorded',
                  xargs.endswith('-nolisten tcp') and ended(xpid)
                  and read_pids(tmp / pid_file) == {})
            other = spawn(FAKE_X, xdir, ':99', '-screen', '0', '8x8x8', '-nolisten', 'tcp')
            waited(lambda: lock_pid(xdir, 99))
            code, out = run({'app': dict(shows, xvfb=recipe())}, 'smoke', env=xenv)
            check('PLANT a foreign X server on :99: the smoke takes :100, the foreign one STILL ALIVE',
                  code == 0 and 'DISPLAY=:100 ' in app_log() and other.poll() is None and alive(other.pid))
            code, out = run({'app': dict(shows, xvfb=recipe(displays=[99, 99]))}, 'smoke', env=xenv)
            check('PLANT every display held: NO-GO naming it, the app never ran, the foreign X server STILL ALIVE',
                  code == 1 and 'no free X display' in failed(out) and 'DISPLAY=' not in app_log()
                  and other.poll() is None and alive(other.pid) and read_pids(tmp / pid_file) == {})
            code, out = run({'app': dict(shows, xvfb=recipe(exe='pb-no-such-x-server'))}, 'smoke', env=xenv)
            check('PLANT no X server program: NO-GO naming it, the app never ran',
                  code == 1 and 'no pb-no-such-x-server on PATH' in failed(out) and 'DISPLAY=' not in app_log())
            code, out = run({'app': dict(shows, xvfb=recipe(platforms=['pb-no-such-platform']))}, 'smoke', env=xenv)
            check('a platform off the recipe runs the service as declared: no X server, the parent DISPLAY',
                  code == 0 and 'DISPLAY=:pb-parent WAYLAND=pb-wayland XOPT=False' in app_log())
            port = release(reserve())
            code, out = run({'web': dict(served(port), xvfb=recipe())}, 'start', env=xenv)
            web, xrec = recorded('web'), recorded('web').get('xvfb') or {}
            check('start records the X server beside the app: pid, start time, display :100 (:99 foreign)',
                  code == 0 and xrec.get('display') == 100 and is_ours(xrec) and is_ours(web))
            code, out = run({'web': dict(served(port), xvfb=recipe())}, 'stop', env=xenv)
            check('stop ends the app and then its X server, each by its recorded start time',
                  code == 0 and bool(xrec) and settle(web, 5) and settle(xrec, 5)
                  and other.poll() is None and read_pids(tmp / pid_file) == {})
            write_pids(tmp / pid_file, {'app': {'pid': 0, 'start_time': None, 'cmd': ['gone'], 'service': 'app',
                                                'xvfb': {'pid': other.pid, 'start_time': 'a-different-boot',
                                                         'display': 99}}})
            code, out = run({'app': dict(shows, xvfb=recipe())}, 'stop', env=xenv)
            check('PLANT a recorded X server pid with a DIFFERENT start time: stop does NOT signal it',
                  code == 0 and lives(other) and read_pids(tmp / pid_file) == {})
        else:
            skips.append('private X display: an X server is POSIX only - NOT RUN on os %s' % os.name)

        # --- the ask gate, the default smoke set, and the empty arms
        code, out = run({'game': windowed}, 'start')
        check('PLANT windowed start without --asked: REFUSED, naming the ask',
              code == 1 and 'asked EVERY time' in failed(out))
        code, out = run({'game': windowed}, 'start', '--asked')
        check('the same start with --asked runs', code == 0)
        code, out = run({'game': windowed, 'boot': echoed}, 'smoke')
        check("smoke's default set skips an `ask` service",
              code == 0 and counts(out).get('services') == 1)
        code, out = run({'game': windowed}, 'smoke')
        check('a manifest whose only service is `ask` -> smoke NO-GO, never green',
              code == 1 and 'no smoke service declared' in (out.get('reason') or ''))
        code, out = run({}, 'smoke')
        check('no service at all -> NO-GO', code == 1)
        code, out = run({'web': served(1)}, 'smoke', 'nosuch')
        check('an unknown service name -> NO-GO', code == 1)
        code, out = run({'boot': echoed}, 'smoke', smoke=['boot', 'nosuch'])
        check("PLANT the manifest's smoke list names an undeclared service -> NO-GO naming it, never skipped",
              code == 1 and "unknown service 'nosuch'" in failed(out))
        code, out = run({'web': served(1), 'boot': echoed}, 'list')
        check('list prints one line per service', code == 0 and counts(out).get('services') == 2)
        code, out = run({'web': served(1)}, 'wat')
        check('an unknown verb -> NO-GO', code == 1)
        left = [e['pid'] for e in read_pids(tmp / pid_file).values() if is_ours(e)]
        check('the fixture ends with nothing of its own still running', not left)
    except Exception as exc:  # the last line of defence: a verdict is printed no matter what
        check('the selftest itself ran to the end without raising (%s: %s)'
              % (type(exc).__name__, exc), False)
    finally:
        # Only ever what this selftest itself started: its own strays, and anything the tool
        # recorded and failed to stop. A leftover is already a named failure above; killing it
        # here is what keeps the verdict printable and the box clean.
        for entry in read_pids(tmp / pid_file).values():
            for rec in (entry, entry.get('xvfb') or {}):
                if is_ours(rec):
                    terminate(rec['pid'], hard=True)
            _clean_tmp(entry)
        for entry in kin:
            if is_ours(entry):
                terminate(entry['pid'], hard=True)
        for proc in strays:
            if proc.poll() is None:
                proc.kill()
                proc.wait()
        for sock in held.values():
            sock.close()
        if os.name != 'posix':
            time.sleep(0.3)  # Windows releases the children's log handles a beat after they die
        shutil.rmtree(tmp, ignore_errors=True)
    return finish(args, {'checks': len(checks), 'failures': len(failures)}, failures, skips)


# ------------------------------------------------------------------ CLI
VERBS = {'start': cmd_start, 'stop': cmd_stop, 'status': cmd_status, 'smoke': cmd_smoke,
         'list': cmd_list, 'selftest': selftest}


def build_parser():
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('target', nargs='?', help=' | '.join(VERBS))
    parser.add_argument('services', nargs='*', help='service names (default: every declared service)')
    parser.add_argument('--asked', action='store_true',
                        help='the lead has given the go for this windowed or irreversible start')
    parser.add_argument('--deadline', type=float, help='seconds to wait for ready (overrides the manifest)')
    parser.add_argument('--task', help='names the log files; required by start and smoke (else $PB_TASK)')
    parser.add_argument('--manifest', default=str(MANIFEST))
    parser.add_argument('--logs-dir')
    parser.add_argument('--json', action='store_true')
    return parser


def main(argv=None):
    args = build_parser().parse_args(argv)
    if args.target not in VERBS:
        return finish(args, {'services': 0}, reason='name a verb: %s' % ' | '.join(VERBS))
    try:
        return VERBS[args.target](args)
    except Unreadable as exc:
        return finish(args, {'services': 0}, reason=str(exc))


if __name__ == '__main__':
    for stream in (sys.stdout, sys.stderr):  # a piped Windows stdout is cp1252; a boot line can carry `≤`
        stream.reconfigure(encoding='utf-8', errors='backslashreplace')
    sys.exit(main())
