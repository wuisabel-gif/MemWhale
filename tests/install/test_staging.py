"""Offline installer tests: signing/copy failures must preserve installed files."""

import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
BINARIES = ("mw", "mw-remember", "mw-serve", "mw-view", "mw-recover", "mw-run", "mw-screenshot", "mw-mcp")


class InstallerStaging(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="mw-install-staging-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.mocks = self.root / "mocks"
        self.mocks.mkdir()
        self.files = self.root / "responses"
        self.files.mkdir()
        self.bin_dir = self.root / "prefix" / "bin"
        self.bin_dir.mkdir(parents=True)
        for name in BINARIES:
            (self.bin_dir / name).write_text("old " + name)
        (self.bin_dir / "mw-unrelated").write_text("leave unchanged")
        self.env = dict(os.environ)
        self.env.pop("GITHUB_TOKEN", None)
        self.env.update(PATH=str(self.mocks) + os.pathsep + os.defpath,
                        PREFIX=str(self.bin_dir.parent), MOCK_FILES=str(self.files))
        self.fake("uname", 'import sys\nprint("Darwin" if sys.argv[1] == "-s" else "arm64")\n')
        self.fake("curl", '''import os, pathlib, shutil, sys
args = sys.argv[1:]
url = next(value for value in args if value.startswith("https://"))
name = "latest.json" if url.endswith("/releases/latest") else url.rsplit("/", 1)[1]
shutil.copyfile(pathlib.Path(os.environ["MOCK_FILES"]) / name, args[args.index("-o") + 1])
''')
        self.fake("codesign", '''import os, pathlib, sys
path = pathlib.Path(sys.argv[-1])
assert path.parent.name.startswith(".memorywhale-install."), "must sign staged files"
if path.name == os.environ.get("FAIL_SIGN_BINARY"):
    sys.exit(2)
path.write_text(path.read_text() + " signed")
''')
        (self.files / "latest.json").write_text(json.dumps({"tag_name": "v0.10.0"}))
        self.archive()

    def fake(self, name, code):
        path = self.mocks / name
        path.write_text(f"#!{sys.executable}\n" + code)
        path.chmod(0o755)

    def archive(self, omitted=None):
        name = "memorywhale-0.10.0-aarch64-apple-darwin"
        path = self.files / (name + ".tar.gz")
        with tarfile.open(path, "w:gz") as archive:
            for binary in BINARIES:
                if binary == omitted:
                    continue
                data = ("new " + binary).encode()
                member = tarfile.TarInfo(f"{name}/bin/{binary}")
                member.size = len(data)
                member.mode = 0o755
                archive.addfile(member, io.BytesIO(data))
        self.checksum = self.files / (path.name + ".sha256")
        self.checksum.write_text(hashlib.sha256(path.read_bytes()).hexdigest() + "  " + path.name + "\n")

    def install(self):
        return subprocess.run(["sh", str(ROOT / "install.sh")], env=self.env,
                              cwd=self.root, capture_output=True, text=True, timeout=30)

    def assert_unchanged(self):
        for name in BINARIES:
            self.assertEqual((self.bin_dir / name).read_text(), "old " + name)
        self.assertEqual((self.bin_dir / "mw-unrelated").read_text(), "leave unchanged")
        self.assertFalse(list(self.bin_dir.glob(".memorywhale-install.*")))

    def test_signing_failure_does_not_replace_any_binary(self):
        self.env["FAIL_SIGN_BINARY"] = "mw-view"
        self.assertNotEqual(self.install().returncode, 0)
        self.assert_unchanged()

    def test_missing_archive_binary_does_not_replace_any_binary(self):
        self.archive(omitted="mw-mcp")
        self.assertNotEqual(self.install().returncode, 0)
        self.assert_unchanged()

    def test_checksum_mismatch_does_not_replace_any_binary(self):
        self.checksum.write_text("0" * 64 + "  wrong.tar.gz\n")
        self.assertNotEqual(self.install().returncode, 0)
        self.assert_unchanged()

    def test_complete_set_is_signed_and_installed(self):
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        for name in BINARIES:
            self.assertEqual((self.bin_dir / name).read_text(), "new " + name + " signed")
            self.assertTrue(os.access(self.bin_dir / name, os.X_OK))
        self.assertEqual((self.bin_dir / "mw-unrelated").read_text(), "leave unchanged")
        self.assertFalse(list(self.bin_dir.glob(".memorywhale-install.*")))


if __name__ == "__main__":
    unittest.main()
