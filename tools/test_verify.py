"""Non-compiling negative plants at the Cargo probe boundary."""

import json
import hashlib
import io
import os
import signal
import subprocess
import sys
import tempfile
import tarfile
import unittest
from pathlib import Path
from unittest.mock import patch

import verify


class ProcessSupervisionTests(unittest.TestCase):
    def test_timeout_terminates_descendant_and_reaps_parent(self):
        parent = """
import os, signal, subprocess, sys
from pathlib import Path
child = subprocess.Popen([sys.executable, '-c',
    'import signal; signal.signal(signal.SIGTERM, signal.SIG_DFL); print("ready", flush=True); signal.pause()'],
    stdout=subprocess.PIPE, text=True)
assert child.stdout.readline() == 'ready\\n'
def stop(signum, frame):
    child.wait(timeout=3)
    Path(sys.argv[2]).write_text('child reaped')
    sys.exit(0)
signal.signal(signal.SIGTERM, stop)
Path(sys.argv[1]).write_text(f'{os.getpid()} {child.pid}')
signal.signal(signal.SIGALRM, lambda *_: None)
signal.setitimer(signal.ITIMER_REAL, 0.05)
while True:
    signal.pause()
"""
        with tempfile.TemporaryDirectory() as directory:
            pids = Path(directory) / "pids"
            reaped = Path(directory) / "reaped"
            try:
                previous = signal.signal(signal.SIGTERM, signal.SIG_IGN)
                try:
                    with self.assertRaises(subprocess.TimeoutExpired):
                        verify.run([sys.executable, "-c", parent, str(pids), str(reaped)], timeout=1)
                finally:
                    signal.signal(signal.SIGTERM, previous)
                self.assertTrue(pids.exists(), "parent and child must be ready before timeout")
                parent_pid, child_pid = map(int, pids.read_text().split())
                with self.assertRaises(ProcessLookupError):
                    os.kill(child_pid, 0)
                with self.assertRaises(ProcessLookupError):
                    os.kill(parent_pid, 0)
                with self.assertRaises(ChildProcessError):
                    os.waitpid(parent_pid, os.WNOHANG)
                self.assertEqual(reaped.read_text(), "child reaped")
            finally:
                if pids.exists():
                    for pid in map(int, pids.read_text().split()):
                        try:
                            os.kill(pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass

    def test_clean_completion_and_distinct_errors(self):
        parent = "import subprocess, sys; subprocess.run([sys.executable, '-c', \"print('complete')\"], check=True)"
        self.assertEqual(verify.run([sys.executable, "-c", parent]), "complete\n")
        with self.assertRaisesRegex(ValueError, r"failed \(7\)"):
            verify.run([sys.executable, "-c", "raise SystemExit(7)"])
        with self.assertRaises(FileNotFoundError):
            verify.run(["/nonexistent/cherry-pit-test-command"])

    def test_term_resistant_parent_is_killed_and_reaped(self):
        parent = "import os, signal; signal.signal(signal.SIGTERM, signal.SIG_IGN); print(os.getpid(), flush=True); signal.pause()"
        with self.assertRaises(subprocess.TimeoutExpired) as result:
            verify.run([sys.executable, "-c", parent], timeout=1)
        pid = int(result.exception.output)
        with self.assertRaises(ProcessLookupError):
            os.kill(pid, 0)
        with self.assertRaises(ChildProcessError):
            os.waitpid(pid, os.WNOHANG)

    def test_timeout_aborts_remaining_commands(self):
        with patch.object(verify, "intake", return_value={"RUSTUP_TOOLCHAIN": "local", "CARGO_HOME": "/unused"}), \
             patch.object(verify, "run", side_effect=subprocess.TimeoutExpired("probe", 1)) as run:
            with self.assertRaises(subprocess.TimeoutExpired):
                verify.execute("supply-chain")
            self.assertEqual(run.call_count, 1)


class GraphPlants(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.metadata = verify.cargo("metadata", "--locked", "--offline", "--no-deps", "--format-version", "1")

    def probe(self, violation=None):
        def cargo(*args):
            if args[0] == "metadata":
                return self.metadata
            edges = args[args.index("-e") + 1]
            if violation == "async" and edges == "features":
                return "cherry-pit-core v0.1.0\nasync-trait v0.1.0\n"
            if violation == "pardosa" and edges == "normal,build":
                return "cherry-pit-core v0.1.0\npardosa v0.5.5\n"
            if violation == "probe-error":
                raise ValueError("probe failed: no clean verdict")
            return "cherry-pit-core v0.1.0\n"
        return cargo

    def test_plant_fail_revert_clean(self):
        for violation, diagnostic in (("async", "async-trait edge"),
                                      ("pardosa", "Pardosa normal/build edge"),
                                      ("probe-error", "no clean verdict")):
            with self.subTest(violation=violation):
                with patch.object(verify, "cargo", self.probe(violation)):
                    with self.assertRaisesRegex(ValueError, diagnostic):
                        verify.graph()
                with patch.object(verify, "cargo", self.probe()):
                    verify.graph()

    def test_missing_adapter_fails(self):
        metadata = json.loads(self.metadata)
        metadata["packages"] = [p for p in metadata["packages"]
                                if p["name"] != "pardosa-cherry-pit-projection"]
        with patch.object(verify, "cargo", return_value=json.dumps(metadata)):
            with self.assertRaisesRegex(ValueError, "outer adapter absent"):
                verify.graph()
        with patch.object(verify, "cargo", self.probe()):
            verify.graph()


class AdmissionTests(unittest.TestCase):
    def test_inactive_unsafe_spellings(self):
        for text in ("/*\n#![forbid(unsafe_code)]\n*/",
                     '#![cfg_attr(any(), forbid(unsafe_code))]',
                     'const TEXT: &str = r#"\n#![forbid(unsafe_code)]\n"#;',
                     '// #![forbid(unsafe_code)]\n'):
            with self.subTest(text=text), self.assertRaises(ValueError):
                verify.unsafe_root(text)
        verify.unsafe_root('//! docs\n\n#![forbid(unsafe_code)]\n')

    def test_intake_failure_precedes_commands(self):
        for group in ("rust", "non-exhaustive"):
            with patch.object(verify, "run") as run, patch.object(
                verify, "intake", side_effect=ValueError("intake identity changed")
            ):
                with self.assertRaisesRegex(ValueError, "intake identity changed"):
                    verify.execute(group)
                run.assert_not_called()

    def test_reviewed_context_uses_direct_compiler_and_sanitized_environment(self):
        env = verify.intake()
        with patch.object(verify, "run", return_value="") as run:
            verify.execute("rust")
        self.assertEqual(run.call_count, 4)
        for call in run.call_args_list:
            self.assertEqual(call.args[0][0], str(verify.Path(env["RUSTC"]).with_name("cargo")))
            self.assertNotIn("+1.98.0", call.args[0])
            self.assertEqual(call.kwargs["env"], env)
        self.assertEqual(run.call_args_list[1].kwargs["timeout"], 900)

    def test_missing_supply_chain_tools_fail(self):
        with patch.object(verify, "intake", return_value={"RUSTUP_TOOLCHAIN": "local", "CARGO_HOME": "/unused"}), \
             patch.object(verify, "run", side_effect=FileNotFoundError("missing tool")) as run:
            with self.assertRaisesRegex(ValueError, "missing tool"):
                verify.execute("supply-chain")
            self.assertEqual(run.call_count, 2)


class ProductionIdentityTests(unittest.TestCase):
    def test_admission_failure_cites_local_invariant(self):
        with patch.object(verify, "admitted_environment", side_effect=OSError("unreadable identity")):
            with self.assertRaisesRegex(ValueError, r"unreadable identity.*CPP-0001:R7"):
                verify.intake()

    def test_production_digest_shapes(self):
        digests = list(verify.LINUX_IDENTITIES.values())
        for asset in verify.ASSETS.values():
            digests.extend((asset[2], asset[5]))
        for digest in digests:
            with self.subTest(digest=digest):
                self.assertRegex(digest, r"\A[0-9a-f]{64}\Z")

    def test_linux_cargo_matches_official_member_evidence(self):
        """ghr-02ld2: official cargo 1.98.0 Linux archive, cargo/bin/cargo."""
        self.assertEqual(verify.LINUX_IDENTITIES["bin/cargo"],
                         "ff3022fcbd13b08434ea7afde9a0ef9d5b3f5c17b5fc7a40031be3ee000a6a24")


class LinuxAdmissionTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = Path(directory.name).resolve()
        (root / "Cargo.lock").write_bytes((verify.ROOT / "Cargo.lock").read_bytes())
        root_mock = patch.object(verify, "ROOT", root)
        root_mock.start()
        self.addCleanup(root_mock.stop)
        self.environment = {
            "CHERRY_INTAKE_CONTEXT": "github-linux-x86_64", "GITHUB_ACTIONS": "true",
            "RUNNER_ENVIRONMENT": "github-hosted", "GITHUB_REPOSITORY": "acje/cherry-pit",
            "GITHUB_WORKSPACE": str(verify.ROOT), "HOME": "/home/runner",
            "CARGO_HOME": "/home/runner/.cargo", "RUSTUP_HOME": "/home/runner/.rustup",
            "RUSTC": "/untrusted/rustc", "RUSTC_WRAPPER": "/untrusted/wrapper",
            "LD_PRELOAD": "/untrusted/loader", "CARGO_ENCODED_RUSTFLAGS": "untrusted",
        }
        for mock in (patch.dict(os.environ, self.environment, clear=True),
                     patch.object(verify.platform, "system", return_value="Linux"),
                     patch.object(verify.platform, "machine", return_value="x86_64")):
            mock.start()
            self.addCleanup(mock.stop)

    def installed(self, corrupt=False):
        original = Path.open
        def opened(path, *args, **kwargs):
            if str(path).startswith("/home/runner/.rustup/"):
                return io.BytesIO(b"corrupt" if corrupt else b"fixture")
            return original(path, *args, **kwargs)
        return patch.object(Path, "open", opened)

    def admitted(self, corrupt=False):
        digest = hashlib.sha256(b"fixture").hexdigest()
        resolve = Path.resolve
        def resolved(path, *args, **kwargs):
            return path if str(path).startswith("/home/runner/") else resolve(path, *args, **kwargs)
        with patch.object(verify, "LINUX_IDENTITIES", {key: digest for key in verify.LINUX_IDENTITIES}), \
             patch.object(Path, "is_file", return_value=True), patch.object(Path, "resolve", resolved), \
             self.installed(corrupt):
            return verify.intake()

    def test_linux_wrong_hash_fail_revert_clean(self):
        with self.assertRaisesRegex(ValueError, "intake identity changed"):
            self.admitted(corrupt=True)
        self.admitted()

    def test_unsupported_context_fail_revert_clean(self):
        for key, value in (("GITHUB_REPOSITORY", "other/repo"),
                           ("CHERRY_INTAKE_CONTEXT", ""), ("RUNNER_ENVIRONMENT", "self-hosted"),
                           ("CARGO_HOME", "/tmp/cargo"), ("GITHUB_WORKSPACE", "/tmp/repo")):
            with self.subTest(key=key), patch.dict(os.environ, {key: value}):
                with self.assertRaisesRegex(ValueError, "unsupported intake context"):
                    self.admitted()
            self.admitted()

    def test_extra_cargo_config_fail_revert_clean(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            (root / "Cargo.lock").write_bytes((verify.ROOT / "Cargo.lock").read_bytes())
            (root / ".cargo").mkdir()
            with patch.object(verify, "ROOT", root), patch.dict(os.environ, {"GITHUB_WORKSPACE": str(root)}):
                for name in ("config", "config.toml"):
                    config = root / ".cargo" / name
                    config.write_text('[build]\nrustc = "/untrusted/rustc"\n')
                    try:
                        with self.assertRaisesRegex(ValueError, "intake config changed"):
                            self.admitted()
                    finally:
                        config.unlink()
                    self.admitted()

    def test_linux_commands_share_sanitized_context(self):
        env = self.admitted()
        self.assertEqual(env["RUSTC"], "/home/runner/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/bin/rustc")
        self.assertEqual(env["RUSTC_WRAPPER"], "")
        self.assertNotIn("LD_PRELOAD", env)
        self.assertNotIn("CARGO_ENCODED_RUSTFLAGS", env)
        with patch.object(verify, "intake", return_value=env), patch.object(verify, "run", return_value="") as run:
            verify.cargo("fetch", "--locked")
            verify.execute("rust")
            verify.execute("non-exhaustive")
        self.assertEqual(run.call_count, 6)
        for call in run.call_args_list:
            self.assertEqual(call.kwargs["env"], env)
            self.assertEqual(call.args[0][0], str(Path(env["RUSTC"]).with_name("cargo")))


class AssetTests(unittest.TestCase):
    def test_archive_and_member_checksum_fail_revert_clean(self):
        binary = b"not an executable; checksum fixture"
        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode="w:gz") as archive:
            entry = tarfile.TarInfo("asset/tool")
            entry.size = len(binary)
            archive.addfile(entry, io.BytesIO(binary))
        payload = buffer.getvalue()
        asset = ("https://example.invalid/asset", len(payload), hashlib.sha256(payload).hexdigest(),
                 "asset/tool", len(binary), hashlib.sha256(binary).hexdigest())
        def downloaded(args, **kwargs):
            Path(args[args.index("--output") + 1]).write_bytes(payload)
            return ""
        for index, message in ((2, "archive checksum changed"), (5, "binary checksum changed")):
            changed = list(asset)
            changed[index] = "0" * 64
            with patch.object(verify, "run", downloaded):
                with patch.dict(verify.ASSETS, {"tool": tuple(changed)}):
                    with self.assertRaisesRegex(ValueError, message):
                        verify.verified_asset("tool", {})
                with patch.dict(verify.ASSETS, {"tool": asset}):
                    self.assertEqual(verify.verified_asset("tool", {}), binary)


if __name__ == "__main__":
    unittest.main()
