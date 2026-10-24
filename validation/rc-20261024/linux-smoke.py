"""Run only on the GitHub-hosted Linux runner; private per-session credentials."""
import json
import os
import pathlib
import secrets
import socket
import struct
import subprocess
import tempfile
import time

out = pathlib.Path('smoke').resolve()
out.mkdir(exist_ok=True)
app = next(pathlib.Path('dist/release').glob('*.AppImage')).resolve()
cli = pathlib.Path('target/linux-package/root/usr/bin/filmcraft-cli').resolve()
version = subprocess.check_output([str(cli), '--version'], text=True).strip()
assert '0.12.0-rc.20261024.1' in version
(out / 'cli-version.txt').write_text(version + '\n')
assert '0.12.0-rc.20261024.1' in subprocess.check_output([str(app), '--version'], text=True)
with tempfile.TemporaryDirectory(prefix='filmcraft-rc-') as private:
    token = secrets.token_hex(32)
    env = dict(os.environ, FILMCRAFT_CONTROL_TOKEN=token)
    with (out / 'gui.log').open('wb') as log:
        process = subprocess.Popen([str(app), '--demo', '--no-recover', '--graphics', 'software',
                                    '--data-dir', private, '--control', '7878'], env=env, stdout=log, stderr=log)
        try:
            deadline = time.monotonic() + 120
            while True:
                if process.poll() is not None:
                    raise RuntimeError(f'GUI exited during startup: {process.returncode}')
                try:
                    connection = socket.create_connection(('127.0.0.1', 7878), timeout=1)
                    break
                except OSError:
                    if time.monotonic() >= deadline:
                        raise TimeoutError('control server not ready')
                    time.sleep(.25)
            connection.settimeout(65)
            with connection, connection.makefile('rwb') as stream:
                serial = 0
                def rpc(method, params):
                    global serial
                    serial += 1
                    stream.write((json.dumps({'id': serial, 'method': method, 'params': params, 'token': token}) + '\n').encode())
                    stream.flush()
                    result = json.loads(stream.readline())
                    assert result.get('ok'), result
                    return result.get('result')
                state = rpc('ui.inspect', {})
                assert state
                (out / 'ui-state.json').write_text(json.dumps(state, indent=2) + '\n')
                time.sleep(3)
                rpc('ui.screenshot', {'path': str(out / 'gui.png')})
                data = (out / 'gui.png').read_bytes()
                assert data[:8] == b'\x89PNG\r\n\x1a\n'
                width, height = struct.unpack_from('>II', data, 16)
                assert width >= 320 and height >= 240
                rpc('app.quit', {})
            assert process.wait(timeout=45) == 0
            (out / 'result.json').write_text(json.dumps({'status': 'PASS', 'source': os.environ['GITHUB_SHA'],
                'display': 'Xvfb X11; forced CPU compositor; software Vulkan UI', 'screenshot': [width, height],
                'scope': 'Packaged AppImage and CLI versions, authenticated GUI startup, demo capture and clean exit; not native GPU or Premiere parity.'}, indent=2) + '\n')
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
    # Source logging should not print credentials; redact defensively before uploading.
    path = out / 'gui.log'
    path.write_text(path.read_text(errors='replace').replace(token, '[redacted]'))
