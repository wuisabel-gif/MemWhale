"""Regression tests for the release version gate; all mutations use temp files."""

import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


class ReleaseVersions(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="mw-version-gate-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for name in (
            "scripts/check-release-version.sh", "crates/mw-cli/Cargo.toml",
            "crates/memorywhale-core/Cargo.toml", "src-tauri/Cargo.toml",
            "src-tauri/tauri.conf.json", "package.json", "package-lock.json",
        ):
            destination = self.root / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, destination)
        self.version = json.loads((self.root / "package.json").read_text())["version"]

    def run_gate(self, *args):
        return subprocess.run(
            ["bash", str(self.root / "scripts/check-release-version.sh"), *args],
            capture_output=True, text=True, timeout=20,
        )

    def test_current_metadata_and_matching_tag(self):
        for args in ((), ("v" + self.version,)):
            result = self.run_gate(*args)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_wrong_tag_fails(self):
        result = self.run_gate("v999.0.0")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("tag mismatch", result.stderr)

    def test_desktop_crate_cannot_silently_lag(self):
        path = self.root / "src-tauri/Cargo.toml"
        path.write_text(path.read_text().replace(f'version = "{self.version}"', 'version = "0.0.1"', 1))
        result = self.run_gate()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("src-tauri/Cargo.toml", result.stderr)

    def test_nested_npm_lock_version_must_match(self):
        path = self.root / "package-lock.json"
        payload = json.loads(path.read_text())
        payload["packages"][""]["version"] = "0.0.1"
        path.write_text(json.dumps(payload))
        result = self.run_gate()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("root package", result.stderr)

    def test_core_dependency_must_match_workspace_api(self):
        path = self.root / "crates/memorywhale-core/Cargo.toml"
        lines = path.read_text().splitlines(keepends=True)
        for index, line in enumerate(lines):
            if line.startswith("version = "):
                lines[index] = 'version = "999.0.0"\n'
                break
        path.write_text("".join(lines))
        result = self.run_gate()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("core dependency mismatch", result.stderr)


if __name__ == "__main__":
    unittest.main()
