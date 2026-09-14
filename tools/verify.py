#!/usr/bin/env python3
"""Run the supported first-party warning gate without losing target or encoded flags."""
import argparse
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
COMMANDS = {
    "fmt": ["cargo", "fmt", "--all", "--", "--check"],
    "all-targets": ["cargo", "test", "--locked", "--all-targets", "--all-features"],
    "headless": ["cargo", "test", "--locked", "--all-targets", "--no-default-features"],
    "clippy": ["cargo", "clippy", "--locked", "--all-targets", "--all-features", "--", "-D", "warnings"],
    "clippy-headless": ["cargo", "clippy", "--locked", "--all-targets", "--no-default-features", "--", "-D", "warnings"],
    "doctest": ["cargo", "test", "--locked", "--doc", "--all-features"],
    "rustdoc": ["cargo", "doc", "--locked", "--no-deps", "--all-features"],
    "release": ["cargo", "build", "--locked", "--release", "--bin", "system-designer", "--bin", "designer-check"],
    "self-design": ["cargo", "run", "--locked", "--no-default-features", "--bin", "designer-check", "--", "design/system-designer.project.json"],
}

def environment():
    env = os.environ.copy()
    compiler = subprocess.check_output(["rustc", "-vV"], text=True)
    host = next(line.removeprefix("host: ") for line in compiler.splitlines() if line.startswith("host: "))
    config = tomllib.loads((ROOT / ".cargo/config.toml").read_text())
    target = env.get("CARGO_BUILD_TARGET", config.get("build", {}).get("target", host))
    target_flags = config.get("target", {}).get(target, {}).get("rustflags", [])
    for name, configured in [("RUSTFLAGS", target_flags), ("RUSTDOCFLAGS", [])]:
        encoded = "CARGO_ENCODED_" + name
        flags = env[encoded].split("\x1f") if encoded in env else shlex.split(env.get(name, ""))
        flags = [flag for flag in flags if flag]
        # Environment flags override Cargo target configuration; explicitly compose it.
        flags = list(configured) + flags + ["-D", "warnings"]
        env[encoded] = "\x1f".join(flags)
        env.pop(name, None)
    return env, compiler, target

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--only", choices=list(COMMANDS))
    parser.add_argument("--log-dir", type=Path, default=ROOT / "target/qualification")
    args = parser.parse_args()
    env, compiler, target = environment()
    args.log_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"source": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), "compiler": compiler, "target": target, "flags": {key: env[key].split("\x1f") for key in ("CARGO_ENCODED_RUSTFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS")}, "commands": {}}
    for name, command in COMMANDS.items():
        if args.only and name != args.only:
            continue
        print(f"Running {name}: {' '.join(command)}", flush=True)
        # Capture complete stdout/stderr, with no diagnostic filtering or ignored exit codes.
        with (args.log_dir / f"{name}.log").open("wb") as log:
            result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, check=False)
        manifest["commands"][name] = {"command": command, "exit_code": result.returncode}
        (args.log_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        if result.returncode:
            print((args.log_dir / f"{name}.log").read_text(), end="")
            return result.returncode
    print(f"Complete logs: {args.log_dir}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
