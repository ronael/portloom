"""Check a real pseudo-terminal using only processes started by this script."""
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import signal
import struct
import subprocess
import termios
import time

ROOT = Path(__file__).resolve().parents[2]


def run_case(exit_kind):
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 40, 140, 0, 0))
    original = termios.tcgetattr(slave)
    start = time.monotonic()
    args = [str(ROOT / 'bin/portloom'), '--demo', '--lang', 'en']
    if exit_kind == 'keyboard-still':
        args.append('--no-motion')
    proc = subprocess.Popen(args, stdin=slave,
                            stdout=slave, stderr=slave, start_new_session=True,
                            env={**os.environ, 'TERM': 'xterm-256color', 'COLORTERM': 'truecolor'})
    data = bytearray()

    def expect(needle, timeout=5):
        deadline = time.monotonic() + timeout
        while needle not in data and time.monotonic() < deadline:
            readable, _, _ = select.select([master], [], [], 0.1)
            if readable:
                try:
                    chunk = os.read(master, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                data.extend(chunk)
        if needle not in data:
            raise AssertionError(f'Missing terminal output: {needle!r}; process status {proc.poll()}; recent output {bytes(data[-3000:])!r}')

    try:
        expect(b'WEB/API')
        first_frame_ms = round((time.monotonic() - start) * 1000, 2)
        process = subprocess.check_output(['ps', '-p', str(proc.pid), '-o', 'rss=,comm='], text=True).strip().split(None, 1)
        assert Path(process[1]).name == 'portloom', process
        if exit_kind.startswith('keyboard'):
            data.clear()
            os.write(master, b'k')
            expect(b'CONFIRM STOP')
            os.write(master, b'y')
            # Ratatui sends only changed cells, so unchanged letters are absent.
            expect(b'Simulat')
            expect(b'9 ports ')
            os.write(master, b'q ' if exit_kind == 'keyboard-skip' else b'q')
        elif exit_kind == 'ctrl-c':
            os.write(master, b'\x03')
        elif exit_kind == 'signal-during-farewell':
            os.write(master, b'q')
            time.sleep(0.15)
            proc.send_signal(signal.SIGTERM)
        else:
            proc.send_signal(signal.SIGTERM)
        # Keep draining: a PTY writer otherwise blocks on its finite output buffer.
        deadline = time.monotonic() + 5
        while proc.poll() is None and time.monotonic() < deadline:
            readable, _, _ = select.select([master], [], [], 0.05)
            if readable:
                try:
                    data.extend(os.read(master, 65536))
                except OSError:
                    break
        proc.wait(timeout=max(0.1, deadline - time.monotonic()))
        expect(b'\x1b[?1049l')
        assert proc.returncode == 0, proc.returncode
        restored = termios.tcgetattr(slave)
        assert restored == original, 'Terminal flags were not restored'
        return {'exit': exit_kind, 'firstDemoFrameMs': first_frame_ms, 'rssKbAtFirstFrame': int(process[0]), 'nativeProcess': True, 'terminalRestored': True}
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait(timeout=5)
        os.close(master)
        os.close(slave)


if __name__ == '__main__':
    results = [run_case(case) for case in [
        'keyboard', 'keyboard-still', 'keyboard-skip', 'ctrl-c', 'sigterm', 'signal-during-farewell'
    ]]
    print(json.dumps(results, indent=2))
