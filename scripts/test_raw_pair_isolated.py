#!/usr/bin/env python3
"""Run production RAW-pair command functions in an isolated throwaway HOME.

This is a Rust/SQLite integration test, not a Tauri WebView IPC test.
Optionally supply two sample paths: sources are only read and copies are indexed.
No user app configuration, library, source file, or operating-system trash is modified.
"""
import argparse
import json
import shutil
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent.parent
TAURI = ROOT / 'src-tauri'
TEST = 't_cmds::raw_pair_isolation_tests::raw_pair_isolated_command_workflow'


def main():
    if sys.platform == 'win32':
        raise RuntimeError('This isolated runner currently supports macOS and Linux only')
    parser = argparse.ArgumentParser()
    parser.add_argument('--sample-raw', type=Path)
    parser.add_argument('--sample-jpeg', type=Path)
    args = parser.parse_args()
    if (args.sample_raw is None) != (args.sample_jpeg is None):
        parser.error('supply both --sample-raw and --sample-jpeg')
    if args.sample_raw is not None:
        if args.sample_raw.suffix.lower() != '.nef' or args.sample_jpeg.suffix.lower() not in {'.jpg', '.jpeg'}:
            parser.error('sample files must be NEF and JPEG')
        if args.sample_raw.stem.casefold() != args.sample_jpeg.stem.casefold():
            parser.error('sample filenames must have matching stems')
    # Build only: cargo must not execute the other tests under the real HOME.
    result = subprocess.run(
        ['cargo', 'test', '--bin', 'Lap', '--no-run', '--message-format=json'],
        cwd=TAURI, stdout=subprocess.PIPE, text=True,
    )
    if result.returncode:
        for line in result.stdout.splitlines():
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            rendered = event.get('message', {}).get('rendered')
            if rendered:
                print(rendered, file=sys.stderr)
        raise subprocess.CalledProcessError(result.returncode, result.args)
    executables = []
    for line in result.stdout.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get('reason') == 'compiler-artifact' and event.get('profile', {}).get('test') and event.get('executable'):
            if event.get('target', {}).get('name') == 'Lap':
                executables.append(Path(event['executable']))
    if len(executables) != 1:
        raise RuntimeError(f'Expected one Lap test executable, got {executables!r}')
    with tempfile.TemporaryDirectory(prefix='lap-raw-isolated-') as temporary:
        root = Path(temporary).resolve(strict=True)
        (root / '.lap-raw-test-root').write_text('isolated test root\n', encoding='utf-8')
        env = os.environ.copy()
        env.update({
            'HOME': str(root), 'LAP_RAW_TEST_ROOT': str(root),
            'XDG_DATA_HOME': str(root / 'xdg-data'), 'XDG_CONFIG_HOME': str(root / 'xdg-config'),
            'XDG_CACHE_HOME': str(root / 'xdg-cache'),
        })
        for name in ('XDG_DATA_HOME', 'XDG_CONFIG_HOME', 'XDG_CACHE_HOME'):
            Path(env[name]).mkdir()
        if args.sample_raw is not None:
            samples = root / 'real-samples'
            samples.mkdir()
            for source in (args.sample_raw, args.sample_jpeg):
                # Never open the source for writing; all downstream work uses these copies.
                with source.open('rb') as src, (samples / source.name).open('xb') as dest:
                    shutil.copyfileobj(src, dest, length=1024 * 1024)
            env['LAP_RAW_SAMPLE_DIR'] = str(samples)
        # Exact filter + single test thread: never run unrelated tests in this process.
        test = subprocess.run([str(executables[0]), '--exact', TEST, '--ignored', '--nocapture', '--test-threads=1'],
                              cwd=TAURI, env=env, text=True, stdout=subprocess.PIPE)
        print(test.stdout, end='')
        if test.returncode or 'test result: ok. 1 passed; 0 failed;' not in test.stdout:
            raise RuntimeError('Isolated integration test did not run and pass exactly once')
    return 0


if __name__ == '__main__':
    sys.exit(main())
