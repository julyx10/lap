"""Exercise the real downloader CLIs; only curl's transport is substituted."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]


class DownloadIntegrityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="lap-download-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        scripts = self.root / "scripts"
        scripts.mkdir()
        for name in ("download_models.sh", "download_ffmpeg_sidecar.sh"):
            shutil.copy2(SCRIPTS / name, scripts / name)
        bin_dir = self.root / "bin"
        bin_dir.mkdir()
        curl = bin_dir / "curl"
        curl.write_text(
            "#!/bin/bash\n"
            "while [ $# -gt 0 ]; do\n"
            "  if [ \"$1\" = -o ]; then\n"
            "    printf 'corrupt response\\n' > \"$2\"\n"
            "    exit \"${CURL_STATUS:-0}\"\n"
            "  fi\n"
            "  shift\n"
            "done\n"
            "exit 99\n"
        )
        curl.chmod(0o755)
        self.env = {**os.environ, "PATH": f"{bin_dir}:{os.environ['PATH']}"}

    def run_downloader(self, script, *args):
        return subprocess.run(
            ["bash", str(self.root / "scripts" / script), *args],
            env=self.env,
            capture_output=True,
            text=True,
            timeout=15,
        )

    def test_corrupt_cached_inputs_are_rejected(self):
        cases = (
            ("download_models.sh", (), "models/tokenizer.json"),
            (
                "download_ffmpeg_sidecar.sh",
                ("linux", "x86_64"),
                "ffmpeg/ffmpeg-x86_64-unknown-linux-gnu",
            ),
            (
                "download_ffmpeg_sidecar.sh",
                ("linux", "aarch64"),
                "ffmpeg/ffmpeg-aarch64-unknown-linux-gnu",
            ),
        )
        for script, args, destination in cases:
            with self.subTest(destination=destination):
                path = self.root / "src-tauri/resources" / destination
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"corrupt cached input\n")
                result = self.run_downloader(script, *args)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertEqual(path.read_bytes(), b"corrupt cached input\n")
                path.unlink()

    def test_corrupt_downloads_never_publish_destination_files(self):
        cases = (
            ("download_models.sh", (), "models/tokenizer.json"),
            (
                "download_ffmpeg_sidecar.sh",
                ("linux", "x86_64"),
                "ffmpeg/ffmpeg-x86_64-unknown-linux-gnu",
            ),
            (
                "download_ffmpeg_sidecar.sh",
                ("linux", "aarch64"),
                "ffmpeg/ffmpeg-aarch64-unknown-linux-gnu",
            ),
        )
        for script, args, destination in cases:
            with self.subTest(destination=destination):
                result = self.run_downloader(script, *args)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                path = self.root / "src-tauri/resources" / destination
                self.assertFalse(path.exists())
                self.assertEqual(list(path.parent.iterdir()), [])

    def test_failed_transfers_never_publish_partial_files(self):
        self.env["CURL_STATUS"] = "22"
        for script, args, resource_dir in (
            ("download_models.sh", (), "models"),
            ("download_ffmpeg_sidecar.sh", ("linux", "x86_64"), "ffmpeg"),
        ):
            with self.subTest(script=script):
                result = self.run_downloader(script, *args)
                self.assertEqual(result.returncode, 22, result.stderr)
                path = self.root / "src-tauri/resources" / resource_dir
                self.assertEqual(list(path.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
