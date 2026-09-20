#!/usr/bin/env python3
"""Standalone source-derived gates; Python 3.12, no third-party modules."""

import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import signal
import subprocess
import sys
import tarfile
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parent.parent
SOURCE = "c8507377b2748a015148751ce288be2bad9ec708"
TOKEN = r"([A-Z]{2,4}-[0-9]{4}):(R[0-9]+(?:\+R[0-9]+)*)"
LINUX = "x86_64-unknown-linux-gnu"
LINUX_IDENTITIES = {
    "bin/rustc": "3690cc576ede140504698405d5d8fa3826aaadbe71699c6c4ed0a565d6f493e2",
    "bin/cargo": "ff3022fcbd13b08434ea7afde9a0ef9d5b3f5c17b5fc7a40031be3ee000a6a24",
    "lib/librustc_driver-28a98848f7a7c026.so": "56047570f302ba09b1abf3b2996bd1d76560c06f3c95c0fcfa7dc3decd2faa2c",
    "lib/libLLVM.so.22.1-rust-1.98.0-stable": "b2cc5bb9aafd2c8d1fd8ad0e99fd7de09e6831ff319ac99130d737261316596e",
    "lib/libLLVM-22-rust-1.98.0-stable.so": "9816014358f5ca85706c55cbdb44e4da930f33b842782f576b695ba66a48091b",
}
ASSETS = {
    "cargo-audit": (
        "https://github.com/rustsec/rustsec/releases/download/cargo-audit/v0.22.1/cargo-audit-x86_64-unknown-linux-musl-v0.22.1.tgz",
        6402626, "c32506f338bdcdaef5a17fb9f33abb6ecf9561324cfd34237fd335f9283a1eab",
        "cargo-audit-x86_64-unknown-linux-musl-v0.22.1/cargo-audit", 14816000,
        "d7a4f8034d548b36bc12ed6f1f341e2917436748a56faaa3d6a0e2cb0549696e"),
    "cargo-deny": (
        "https://github.com/EmbarkStudios/cargo-deny/releases/download/0.19.4/cargo-deny-0.19.4-x86_64-unknown-linux-musl.tar.gz",
        4965853, "3bd58b784e83715b86ddbc9deac591890372ec77fda5741bb0826970b958506f",
        "cargo-deny-0.19.4-x86_64-unknown-linux-musl/cargo-deny", 8914256,
        "46cd3e14d1f04b42313f368a59835d911c9221343eeca0010c57bd52d98387f1"),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def run(args, timeout=120, env=None):
    """Own one POSIX group; timeout grants 5s TERM grace, then KILL and 5s reap.

    Descendants must retain the group. The direct child is reaped here; parents
    reap their children during grace, otherwise the OS adopts them. Deliberate
    setsid/setpgid escape is outside this verification-command contract.
    """
    require(os.name == "posix", "verification process supervision requires POSIX")
    process = subprocess.Popen(args, cwd=ROOT, text=True, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, env=env, start_new_session=True)
    try:
        stdout, stderr = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        try:
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                process.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                pass
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.communicate(timeout=5)
        except (OSError, subprocess.TimeoutExpired) as error:
            raise RuntimeError(f"process-group cleanup incomplete: {args}; abort verification") from error
        finally:
            process.stdout.close()
            process.stderr.close()
        raise
    require(process.returncode == 0,
            f"probe/command failed ({process.returncode}): {args}\n{stdout}{stderr}; no clean verdict")
    return stdout


def cargo(*args):
    env = intake()
    return run([str(Path(env["RUSTC"]).with_name("cargo")), *args], env=env)


def toml(path):
    return tomllib.loads(path.read_text())


def intake():
    try:
        return admitted_environment()
    except (ValueError, OSError) as error:
        raise ValueError(f"{error} (CPP-0001:R7)") from error


def admitted_environment():
    home = Path("/Users/anders.jensen")
    host = "aarch64-apple-darwin"
    linux = (platform.system(), platform.machine()) == ("Linux", "x86_64")
    if linux:
        require(os.environ.get("CHERRY_INTAKE_CONTEXT") == "github-linux-x86_64"
                and os.environ.get("GITHUB_ACTIONS") == "true"
                and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted"
                and os.environ.get("GITHUB_REPOSITORY") == "acje/cherry-pit"
                and os.environ.get("GITHUB_WORKSPACE") == str(ROOT),
                "unsupported intake context: expected explicit GitHub Linux checkout")
        home = Path("/home/runner")
        host = LINUX
        require(os.environ.get("HOME") == str(home)
                and os.environ.get("CARGO_HOME") == str(home / ".cargo")
                and os.environ.get("RUSTUP_HOME") == str(home / ".rustup"),
                "unsupported intake context: HOME/CARGO_HOME/RUSTUP_HOME")
    else:
        require((platform.system(), platform.machine()) == ("Darwin", "arm64")
                and ROOT == home / "code/cherry-pit"
                and not os.environ.get("CHERRY_INTAKE_CONTEXT"),
                "unsupported intake context: ghr-7wc6p.11.2 local host/checkout")
    toolchain = home / f".rustup/toolchains/1.98.0-{host}"
    identities = {
        ROOT / "Cargo.lock": "8a10bcb0c67834b4b4da113608ca054e344a1cfee4edf1a61f9ef8dbfd1c41f1",
    }
    identities.update({toolchain / path: digest for path, digest in LINUX_IDENTITIES.items()} if linux else {
        home / ".cargo/config.toml": "afc3cdc00f48ced4b8928dd4bbd49a2b7537ff324829de840fc4239fefe1b9ac",
        toolchain / "bin/rustc": "a11618eca0956a8aa4372c2bc898690b513cbdfa2cb9125b2a5301e360ed5b49",
        toolchain / "bin/cargo": "1de2e84c15443b70444eecfa959ff9099dd8c1a5606b6d9ef5bc0ea9c25bc7f9",
        toolchain / "lib/librustc_driver-4031c0ff8e88f5d1.dylib": "275171d3d528b7f78bcad812a84658ec3bd756ce9792a329b6edefdf70884c63",
        toolchain / "lib/libLLVM.dylib": "6da171ecd17bbe20b57b2e2d2e324b8fb2267117b504e864eb8b3012a71a6fec",
    })
    for path, expected in identities.items():
        require(path.resolve() == path and path.is_file(), f"intake nonregular/redirected path: {path}")
        with path.open("rb") as artifact:
            actual = hashlib.file_digest(artifact, "sha256").hexdigest()
        require(actual == expected, f"intake identity changed: {path}; reassess ghr-7wc6p.11.2")
    for directory in (home / ".cargo", *(parent / ".cargo" for parent in (ROOT, *ROOT.parents))):
        for name in ("config", "config.toml"):
            config = directory / name
            require(config in identities or not os.path.lexists(config),
                     f"intake config changed: {config}; reassess ghr-7wc6p.11.2")
    return {
        "HOME": str(home), "CARGO_HOME": str(home / ".cargo"),
        "RUSTUP_HOME": str(home / ".rustup"),
        "RUSTUP_TOOLCHAIN": f"1.98.0-{host}",
        "PATH": f"{toolchain}/bin:/usr/bin:/bin:/usr/sbin:/sbin",
        "RUSTC": str(toolchain / "bin/rustc"),
        "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "",
        **({"LD_LIBRARY_PATH": str(toolchain / "lib")} if linux else
           {"DYLD_FALLBACK_LIBRARY_PATH": f"{toolchain}/lib:/usr/lib"}),
        "CARGO_TERM_PROGRESS_WHEN": "never",
    }


def verified_asset(name, env):
    url, size, digest, member, binary_size, binary_digest = ASSETS[name]
    with tempfile.TemporaryDirectory(prefix="cherry-tools-") as directory:
        archive = Path(directory) / "asset.tar.gz"
        run(["/usr/bin/curl", "--fail", "--silent", "--show-error", "--location",
             "--proto", "=https", "--proto-redir", "=https", "--max-time", "120",
             "--max-filesize", str(size), "--output", str(archive), url], timeout=130, env=env)
        require(archive.stat().st_size == size, f"{name}: archive size changed")
        with archive.open("rb") as stream:
            require(hashlib.file_digest(stream, "sha256").hexdigest() == digest,
                    f"{name}: archive checksum changed")
        with tarfile.open(archive) as stream:
            entry = stream.getmember(member)
            require(entry.isfile() and entry.size == binary_size, f"{name}: invalid binary member")
            binary = stream.extractfile(entry).read(binary_size + 1)
        require(hashlib.sha256(binary).hexdigest() == binary_digest, f"{name}: binary checksum changed")
        return binary


def provision():
    env = intake()
    require(env["RUSTUP_TOOLCHAIN"] == f"1.98.0-{LINUX}", "provision requires admitted Linux context")
    directory = Path(env["CARGO_HOME"]) / "bin"
    require(directory.is_dir() and directory.resolve() == directory, "invalid tool installation directory")
    for name in ASSETS:
        binary = verified_asset(name, env)
        path = directory / name
        require(not os.path.lexists(path), f"refuse to overwrite existing tool: {path}")
        with path.open("xb") as stream:
            stream.write(binary)
        path.chmod(0o755)
        del binary


def toolchain():
    require(toml(ROOT / "rust-toolchain.toml")["toolchain"]["channel"] == "1.98.0"
            and toml(ROOT / "Cargo.toml")["workspace"]["package"]["rust-version"] == "1.98"
            and toml(ROOT / "clippy.toml")["msrv"] == "1.98",
            "toolchain/MSRV drift (gh-report RST-0001:R6)")
    workflow = (ROOT / ".github/workflows/ci.yml").read_text()
    pins = re.findall(r"^\s+toolchain: (\S+)$", workflow, re.M)
    require(pins and set(pins) == {"1.98.0"},
            "workflow toolchain drift (gh-report RST-0001:R6)")


def dead_code_text(text):
    require(not re.search(r"#!\s*\[[^\]]*\b(?:allow|expect)\s*\([^\]]*\bdead_code\b", text, re.S),
            "inner dead_code suppression (gh-report RST-0003:R6)")


def dead_code():
    files = sorted((ROOT / "crates").glob("*/src/**/*.rs"))
    require(files, "zero Rust source files (gh-report RST-0003:R6)")
    for path in files:
        try:
            dead_code_text(path.read_text())
        except ValueError as error:
            raise ValueError(f"{path}: {error}") from error


def deny_policy(data):
    policy = data["advisories"]
    require(policy.get("unmaintained") == "all"
            and policy.get("unused-ignored-advisory") in ("warn", "deny"),
            "advisory posture weakened (gh-report SEC-0013:R2)")
    for entry in policy.get("ignore", []):
        require(isinstance(entry, dict) and isinstance(entry.get("id"), str),
                "ignore must be table form (gh-report SEC-0013:R3)")
        match = re.fullmatch(r"expires=(\d{4}-\d{2}-\d{2}) owner=\S+ class=(unmaintained|notice) -- .+", entry.get("reason", ""))
        require(match is not None, "invalid ignore reason (gh-report SEC-0013:R1+R3)")
        require(datetime.date.fromisoformat(match[1]) >= datetime.date.today(),
                "expired ignore (gh-report SEC-0013:R4)")


def deny_lifecycle():
    deny_policy(toml(ROOT / "deny.toml"))


def adr_collision():
    paths = list((ROOT / "docs/adr").rglob("*.md")) + list((ROOT / "docs/decisions").rglob("*.md"))
    ids = [re.match(r"[A-Z]{2,4}-\d{4}", p.name)[0] for p in paths
           if re.match(r"[A-Z]{2,4}-\d{4}", p.name)]
    require(ids and len(ids) == len(set(ids)),
            "zero ADR ids or duplicate local ADR ids (gh-report COM-0017:R4)")


def citations():
    source = Path(os.environ.get("GH_REPORT_SOURCE", ROOT.parent / "gh-report")).resolve()
    workflow = (ROOT / ".github/workflows/ci.yml").read_text()
    jobs = re.split(r"^  [a-z][a-z0-9-]*:\s*$", workflow.split("jobs:\n", 1)[1], flags=re.M)[1:]
    require(jobs, "zero workflow jobs (gh-report RST-0007:R7)")
    for job in jobs:
        names = re.findall(r"^      - name: (.*)$", job, re.M)
        require(any(re.search(TOKEN, name) for name in names),
                "job lacks step rule citation (gh-report RST-0007:R2+R7)")
    files = run(["git", "-C", str(source), "ls-tree", "-r", "--name-only", SOURCE, "--", "docs/adr"]).splitlines()
    text = workflow + Path(__file__).read_text()
    for adr, rules in sorted(set(re.findall(TOKEN, text))):
        if adr.startswith("CPP-"):
            candidates = list((ROOT / "docs/decisions").glob(adr + "-*.md"))
            require(len(candidates) == 1, f"missing/ambiguous local ADR {adr} (gh-report RST-0007:R7)")
            body = candidates[0].read_text()
            require(re.search(r"^Status: Accepted$", body, re.M),
                    f"unaccepted local ADR {adr} (gh-report RST-0007:R7)")
        else:
            candidates = [p for p in files if Path(p).name.startswith(adr + "-") and "/stale/" not in p]
            require(len(candidates) == 1, f"missing/ambiguous pinned source ADR {adr} (gh-report RST-0007:R7)")
            body = run(["git", "-C", str(source), "show", f"{SOURCE}:{candidates[0]}"])
        for rule in rules.split("+"):
            require(re.search(rf"^{rule} \[", body, re.M),
                    f"missing rule {adr}:{rule} (gh-report RST-0007:R7)")


def unsafe_root(text):
    require(re.match(r"\A(?:\s|//[^\n]*(?:\n|$))*#!\[forbid\(unsafe_code\)\]", text),
            "unsafe prohibition must be the first attribute after whitespace/line comments (gh-report RST-0005:R1)")


def graph():
    meta = json.loads(cargo("metadata", "--locked", "--offline", "--no-deps", "--format-version", "1"))
    packages = [p for p in meta["packages"] if p["id"] in meta["workspace_members"]]
    neutral = [p for p in packages if p["name"].startswith("cherry-pit-")]
    require(len(neutral) >= 8, "neutral member coverage missing (gh-report CHE-0084:R5+R9)")
    require(any(p["name"] == "pardosa-cherry-pit-projection" for p in packages),
            "outer adapter absent (gh-report CHE-0084:R9)")
    for package in neutral:
        name = package["name"]
        tree = cargo("tree", "--locked", "--offline", "--all-features", "--target", "all", "-p", name, "-e", "features")
        require("async-trait" not in tree, f"{name}: async-trait edge (gh-report CHE-0025:R1+R2)")
        tree = cargo("tree", "--locked", "--offline", "--all-features", "--target", "all", "-p", name,
                     "-e", "normal,build", "--prefix", "none", "--format", "{p}")
        require(not re.search(r"^pardosa(?:-|\s)", tree, re.M),
                f"{name}: Pardosa normal/build edge (gh-report CHE-0084:R5+R9)")
    kinds = {"lib", "bin", "cdylib", "rlib", "staticlib", "proc-macro"}
    require(packages, "empty workspace (gh-report RST-0005:R1)")
    for package in packages:
        roots = {t["src_path"] for t in package["targets"] if kinds.intersection(t["kind"])}
        require(roots, f"{package['name']}: no compilation roots (gh-report RST-0005:R1)")
        for path in roots:
            try:
                unsafe_root(Path(path).read_text())
            except ValueError as error:
                raise ValueError(f"{path}: {error}") from error


def static():
    for check in (toolchain, dead_code, deny_lifecycle, adr_collision, citations):
        check()


def execute(kind):
    env = intake()
    commands = {
        "rust": [
            ["cargo", "+1.98.0", "build", "--workspace", "--all-features", "--locked"],
            ["timeout", "900", "cargo", "+1.98.0", "test", "--quiet", "--no-fail-fast", "--workspace", "--all-features", "--locked"],
            ["cargo", "+1.98.0", "clippy", "--quiet", "--workspace", "--all-targets", "--all-features", "--locked", "--", "-D", "warnings"],
            ["cargo", "+1.98.0", "fmt", "--all", "--", "--check"],
        ],
        "non-exhaustive": [["cargo", "+1.98.0", "run", "--locked", "--quiet", "-p", "non-exhaustive-check", "--", str(ROOT)]],
        "supply-chain": [["cargo-audit", "audit"], ["cargo-deny", "deny", "check"]],
    }
    failures = []
    if kind == "supply-chain" and env["RUSTUP_TOOLCHAIN"] == f"1.98.0-{LINUX}":
        for name, asset in ASSETS.items():
            path = Path(env["CARGO_HOME"]) / "bin" / name
            require(path.resolve() == path, f"redirected supply-chain tool: {path}")
            with path.open("rb") as stream:
                require(hashlib.file_digest(stream, "sha256").hexdigest() == asset[-1],
                        f"{name}: installed binary checksum changed")
    for command in commands[kind]:
        try:
            if kind != "supply-chain":
                index = command.index("cargo")
                command[index:index + 2] = [str(Path(env["RUSTC"]).with_name("cargo"))]
                if command[0] == "timeout":
                    command = command[2:]
                    timeout = 900
                else:
                    timeout = 1800
            else:
                command[0] = str(Path(env["CARGO_HOME"]) / "bin" / command[0])
                timeout = 1800
            print(run(command, timeout=timeout, env=env), end="")
        except (ValueError, OSError) as error:
            failures.append(str(error))
    require(not failures, "\n".join(failures) + " (gh-report RST-0003:R4 RST-0004:R3+R4 RST-0006:R3 CHE-0038:R1+R2)")


def main():
    checks = {"static": static, "graph": graph, "intake": intake, "provision": provision,
              "fetch": lambda: print(cargo("fetch", "--locked"), end=""),
              "dead-code": dead_code, "deny-lifecycle": deny_lifecycle,
              "citations": citations, "toolchain": toolchain, "adr-collision": adr_collision}
    require(len(sys.argv) == 2, "usage: python3.12 -B tools/verify.py static|graph|intake|fetch|provision|rust|non-exhaustive|supply-chain|all")
    kind = sys.argv[1]
    if kind == "all":
        static()
        graph()
        for group in ("supply-chain", "rust", "non-exhaustive"):
            execute(group)
    elif kind in checks:
        checks[kind]()
    else:
        require(kind in ("rust", "non-exhaustive", "supply-chain"), "unknown check")
        execute(kind)
    print(f"OK: {kind}")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, IndexError, OSError, subprocess.TimeoutExpired, RuntimeError) as error:
        citation = " (CPP-0001:R7)" if sys.argv[1:2] in (["intake"], ["fetch"], ["provision"], ["supply-chain"]) else ""
        print(f"::error::{error}{citation}", file=sys.stderr)
        sys.exit(1)
