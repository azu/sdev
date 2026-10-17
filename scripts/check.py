#!/usr/bin/env python3
"""Run the production CLI checks with its selected Rust compiler."""
import argparse
import subprocess
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path

spec = spec_from_file_location("build_release", Path(__file__).with_name("build-release.py"))
build_release = module_from_spec(spec)
spec.loader.exec_module(build_release)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target")
    args = parser.parse_args()
    cargo, environment, _ = build_release.toolchain()
    subprocess.run([*cargo, "fmt", "--all", "--", "--check"], cwd=build_release.ROOT, env=environment, check=True)
    flags = ["--target", args.target] if args.target else []
    subprocess.run([*cargo, "test", "--locked", *flags], cwd=build_release.ROOT, env=environment, check=True)
    subprocess.run([*cargo, "clippy", "--locked", "--all-targets", *flags, "--", "-D", "warnings"], cwd=build_release.ROOT, env=environment, check=True)


if __name__ == "__main__":
    main()
