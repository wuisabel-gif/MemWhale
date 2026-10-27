"""Synthetic stores and a throwaway git repository for the OpenHands SDK checks.

Importing this module points HOME at a fresh temporary directory and drops
MemoryWhale variables from the environment, before the SDK is imported. A
config that forgets MEMORYWHALE_DATA_DIR then falls back to an empty store
under that temporary HOME, never to the real one.
"""
import atexit
import os
import shutil
import sqlite3
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BIN = Path(os.environ.get("MW_BIN_DIR", ROOT / "target/debug")).resolve()
SCRATCH = Path(tempfile.mkdtemp(prefix="mw-openhands-"))
HOME = SCRATCH / "home"
HOME.mkdir()
atexit.register(shutil.rmtree, SCRATCH, True)
for name in [n for n in os.environ if n.startswith(("MEMORYWHALE_", "XDG_"))]:
    del os.environ[name]
os.environ.update(HOME=str(HOME), OPENHANDS_SUPPRESS_BANNER="1", GIT_CONFIG_NOSYSTEM="1")

TOOLS = ["recent_errors", "search_memory", "get_context", "remember", "similar_failures", "stats"]
TEST_CMD = ["python3", "-m", "unittest", "-q"]
MARKER = "calc-add-MWOH7"
UNRELATED = "UNRELATED-ZEBRA-41"
GIT = ["git", "-c", "user.name=Synthetic Fixture", "-c", "user.email=fixture@example.invalid"]


def child_env(store):
    return {"HOME": str(HOME), "PATH": os.environ["PATH"], "MEMORYWHALE_DATA_DIR": str(store)}


def mw(store, *args):
    return subprocess.run([BIN / args[0], *args[1:]], env=child_env(store),
                          capture_output=True, text=True, check=True)


def make_repo(name="calc"):
    """A one-file repo whose only test fails: add() subtracts."""
    repo = SCRATCH / name
    repo.mkdir()
    (repo / "calc.py").write_text("def add(a, b):\n    return a - b\n")
    (repo / "test_calc.py").write_text(
        "import unittest\nfrom calc import add\n\n\n"
        "class AddTest(unittest.TestCase):\n"
        "    def test_add(self):\n"
        f"        self.assertEqual(add(2, 3), 5, {MARKER!r})\n")
    subprocess.run(["git", "init", "-q", "-b", "main"], cwd=repo, check=True)
    commit(repo, "add calc")
    return repo


def commit(repo, message):
    subprocess.run(GIT + ["add", "-A"], cwd=repo, check=True)
    subprocess.run(GIT + ["commit", "-q", "-m", message], cwd=repo, check=True)
    return git(repo, "rev-parse", "HEAD")


def run_test(repo):
    return subprocess.run(TEST_CMD, cwd=repo, capture_output=True, text=True,
                          env={"PATH": os.environ["PATH"], "HOME": str(HOME),
                               "PYTHONDONTWRITEBYTECODE": "1"})


def seed_failure(store, repo):
    """Record the repo's failing test run in a store, the way a capture hook would,
    after a few passing commands: BM25 scores nothing in a one-row store."""
    for filler in (["git", "status"], ["ls"], ["cargo", "fmt"]):
        mw(store, "mw-remember", "--cwd", str(repo), "--exit-code", "0",
           "--notes", "project:synthetic-calc", "--", *filler)
    run = run_test(repo)
    assert run.returncode != 0 and MARKER in run.stderr, run.stderr
    mw(store, "mw-remember", "--cwd", str(repo), "--exit-code", str(run.returncode),
       "--stderr", run.stderr, "--notes", "project:synthetic-calc", "--", *TEST_CMD)
    return run.stderr


def seed_unrelated(store):
    """A second store holding content no calc session should ever see."""
    mw(store, "mw-remember", "--cwd", str(SCRATCH), "--exit-code", "2",
       "--stderr", f"error: {UNRELATED} build exploded", "--notes", "project:unrelated",
       "--", "make", UNRELATED.lower())
    mw(store, "mw", "remember", f"fix: {UNRELATED} needs the vendored toolchain")


def verified_handoff(repo, hints):
    """The handoff note for a fix, or None unless the test passes right now on a
    clean, committed tree. Retrieved memory (`hints`) is only cited, never
    counted as evidence: a lesson that says "verified" proves nothing here."""
    if git(repo, "status", "--porcelain"):
        return None
    run = run_test(repo)
    if run.returncode != 0:
        return None
    # Provenance first: search_memory shows only the first ~160 characters.
    return (f"verified fix at commit {git(repo, 'rev-parse', 'HEAD')}: `{' '.join(TEST_CMD)}` "
            f"exited 0 in repo {repo.name} on {git(repo, 'branch', '--show-current')}, run by the "
            f"harness after the change. Retrieved {', '.join(hints)} were hints only.")


def git(repo, *args):
    return subprocess.run(["git", *args], cwd=repo, check=True,
                          capture_output=True, text=True).stdout.strip()


def scalar(store, sql, *params):
    with sqlite3.connect(Path(store) / "memorywhale.sqlite3") as db:
        return db.execute(sql, params).fetchone()[0]


def approve(store, note_id):
    """What the dashboard's Approve button runs (`approve_pending`)."""
    with sqlite3.connect(Path(store) / "memorywhale.sqlite3") as db:
        db.execute("UPDATE bookmarks SET approved = 1 WHERE id = ?", (note_id,))


def server(store, **extra):
    from openhands.sdk.mcp import MCPServer
    from pydantic import SecretStr

    env = {"MEMORYWHALE_DATA_DIR": SecretStr(str(store))}
    env.update({k: SecretStr(v) for k, v in extra.items()})
    return MCPServer(command=str(BIN / "mw-mcp"), args=[], env=env)
