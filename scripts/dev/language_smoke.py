"""Exercise language onboarding and stop confirmations in disposable PTYs.

Real scans are read-only. Every stop uses the in-memory demo adapter.
"""
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time

ROOT = Path(__file__).resolve().parents[2]


class Terminal:
    def __init__(self, config, *args):
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack('HHHH', 40, 140, 0, 0))
        self.original = termios.tcgetattr(self.slave)
        env = {key: value for key, value in os.environ.items()
               if key not in ('PORTLOOM_LANG', 'CI', 'LC_ALL', 'LC_MESSAGES')}
        env.update(PORTLOOM_CONFIG_DIR=str(config), LANG='fr_FR.UTF-8', TERM='xterm-256color')
        self.proc = subprocess.Popen([str(ROOT / 'bin/portloom'), *args],
                                     stdin=self.slave, stdout=self.slave, stderr=self.slave,
                                     start_new_session=True, env=env)
        self.data = bytearray()

    def read(self):
        if select.select([self.master], [], [], 0.05)[0]:
            try:
                self.data.extend(os.read(self.master, 65536))
            except OSError:
                pass

    def expect(self, text):
        needle = text.encode()
        deadline = time.monotonic() + 8
        while needle not in self.data and time.monotonic() < deadline:
            self.read()
        assert needle in self.data, f'Missing {text!r}: {bytes(self.data[-3000:])!r}'

    def send(self, text):
        os.write(self.master, text.encode())

    def finish(self, code=0):
        deadline = time.monotonic() + 8
        while self.proc.poll() is None and time.monotonic() < deadline:
            self.read()
        assert self.proc.poll() is not None, f'Terminal did not exit: {bytes(self.data[-1500:])!r}'
        assert self.proc.wait(timeout=1) == code, bytes(self.data[-2000:])
        assert termios.tcgetattr(self.slave) == self.original, 'Terminal was not restored'

    def __enter__(self):
        return self

    def __exit__(self, *_):
        if self.proc.poll() is None:
            self.proc.kill()
            self.proc.wait(timeout=5)
        os.close(self.master)
        os.close(self.slave)


def run():
    with tempfile.TemporaryDirectory(prefix='portloom-language-') as directory:
        root = Path(directory)
        saved = root / 'saved'
        with Terminal(saved) as terminal:
            terminal.expect('LANGUAGE / LANGUE')
            terminal.send('\r')  # French is suggested from LANG; Enter explicitly saves.
            deadline = time.monotonic() + 8
            settings = saved / 'settings.json'
            while not settings.exists() and time.monotonic() < deadline:
                terminal.read()
            assert json.loads(settings.read_text())['language'] == 'fr'
            terminal.send('q')
            terminal.finish()
        with Terminal(saved) as terminal:
            terminal.expect('PORTS EN ÉCOUTE')
            assert b'LANGUAGE / LANGUE' not in terminal.data
            terminal.send('q')
            terminal.finish()
        with Terminal(saved, '--lang', 'en') as terminal:
            terminal.expect('LISTENING PORTS')
            assert b'LANGUAGE / LANGUE' not in terminal.data
            terminal.send('q')
            terminal.finish()
            assert json.loads(settings.read_text())['language'] == 'fr'
        once = root / 'once'
        with Terminal(once) as terminal:
            terminal.expect('LANGUAGE / LANGUE')
            terminal.expect('\x1b[?25l')  # End of the initial frame.
            terminal.data.clear()
            terminal.send('\x1b')
            # Wait for dismissal to render before q: ESC+q in one input packet
            # is an Alt+q key sequence, not two independent keystrokes.
            terminal.expect('\x1b[?25l')
            terminal.send('q')
            terminal.finish()
            assert not (once / 'settings.json').exists()
        cases = [
            ('en', '3000', 'Y', 'Process stopped.', 0),
            ('fr', '3000', 'O', 'Processus arrêté.', 0),
            ('en', '3000', 'O', 'Cancelled. No signal was sent.', 1),
            ('en', '5432', 'Y', 'Cancelled. No signal was sent.', 1),
            ('fr', '5432', '1206', 'Processus arrêté.', 0),
        ]
        for language, port, answer, outcome, code in cases:
            with Terminal(root / 'demo', '--demo', '--lang', language, 'stop', port) as terminal:
                terminal.expect('cancel: ' if language == 'en' else 'annuler : ')
                terminal.expect('TCP:3000, TCP:8080' if port == '3000' else 'TCP:5432')
                terminal.send(answer + '\n')
                terminal.expect(outcome)
                terminal.finish(code)
        assert not (root / 'demo' / 'settings.json').exists()
    print('Language PTY: first launch, locale suggestion, persistence, override, Escape, '
          'and 5 demo stop confirmations passed; terminal flags restored.')


if __name__ == '__main__':
    run()
