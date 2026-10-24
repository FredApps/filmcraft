"""GitHub-hosted build receipt and package staging; no network or publishing."""
import hashlib
import json
import os
import pathlib
import platform
import shutil
import subprocess
import sys

kind = sys.argv[1]
root = pathlib.Path.cwd()
out = root / 'rc-artifacts'
out.mkdir(exist_ok=True)
def command(*args):
    return subprocess.check_output(args, text=True).strip()
sha = command('git', 'rev-parse', 'HEAD')
assert sha == os.environ['GITHUB_SHA']
tracked_delta = command('git', 'status', '--porcelain', '--untracked-files=no')
receipt = {'source_commit': sha, 'source_tree': command('git', 'rev-parse', 'HEAD^{tree}'),
           'workflow_run': os.environ['GITHUB_RUN_ID'], 'platform_job': kind,
           'job_status_before_receipt': os.environ.get('RC_JOB_STATUS'),
           'tracked_delta': tracked_delta, 'rustc': command('rustc', '-Vv'),
           'cargo': command('cargo', '-V'), 'host': platform.platform(),
           'fonts_commit': command('git', '-C', '.rc-fonts', 'rev-parse', 'HEAD'),
           'lock_sha256': hashlib.sha256((root / 'Cargo.lock').read_bytes()).hexdigest(),
           'runtime_scope': 'Hosted tests/CLI/smoke only; native graphical and Premiere cases remain NOT_RUN.'}
if os.environ.get('RC_JOB_STATUS') == 'success':
    assert not tracked_delta, tracked_delta
packages = out / 'packages'
packages.mkdir(exist_ok=True)
for file in (root / 'dist/release').glob('*'):
    if file.is_file() and file.suffix not in ['.wixpdb']:
        shutil.copy2(file, packages / file.name)
for directory in ['logs', 'smoke']:
    if (root / directory).is_dir():
        shutil.copytree(root / directory, out / directory, dirs_exist_ok=True)
shutil.copytree(root / 'validation/rc-20261024', out / 'validation', dirs_exist_ok=True)
# Bind the acceptance kit to this build without a self-referential source commit.
for path in (out / 'validation').rglob('*'):
    if path.is_file() and path.suffix in ['.json', '.md']:
        contents = path.read_text(encoding='utf8')
        contents = contents.replace('FROM_FINAL_ARTIFACT_SOURCE_COMMIT_TXT', sha)
        contents = contents.replace('PENDING_ROOT_FINAL_SHA', sha)
        contents = contents.replace('RECORD_EXACT_BASE_SHA', 'bfc23840b603e69c782302c6ed04d4a9f6ec26a8')
        path.write_text(contents, encoding='utf8')
for name in ['LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE']:
    shutil.copy2(root / name, out / name)
(out / 'source-commit.txt').write_text(sha + '\n')
(out / 'source-tree.txt').write_text(receipt['source_tree'] + '\n')
(out / 'build-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
(out / 'README-RC.txt').write_text(
    'FilmCraft community release candidate. Not an official release.\n'
    'Only artifacts from an entirely successful run qualify as the RC build.\n'
    'Windows packages are unsigned; macOS packages are ad-hoc signed, not notarized.\n'
    'Linux AppImage/tar require glibc 2.35+ and system graphics/windowing libraries.\n'
    'Exact source, build evidence, package hashes and acceptance protocol are included.\n'
    'Native graphical and Premiere parity acceptance has not been executed by this build.\n')
lines = []
for path in sorted(out.rglob('*')):
    if path.is_file() and path.name != 'SHA256SUMS.txt':
        lines.append(hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + path.relative_to(out).as_posix())
(out / 'SHA256SUMS.txt').write_text('\n'.join(lines) + '\n')
print(json.dumps(receipt, indent=2))
