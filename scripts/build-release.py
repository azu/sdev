#!/usr/bin/env python3
"""Build one reviewable GitHub Releases executable. Does not publish anything."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parent.parent
TARGETS = {
    "aarch64-apple-darwin": ("macos-arm64", "11.0"),
    "x86_64-apple-darwin": ("macos-x64", "10.12"),
    "aarch64-unknown-linux-gnu": ("linux-arm64", None),
    "x86_64-unknown-linux-gnu": ("linux-x64", None),
    "x86_64-pc-windows-msvc": ("windows-x64", None),
}


def toolchain():
    channel = re.search(r'channel\s*=\s*"([^"]+)"', (ROOT / "rust-toolchain.toml").read_text()).group(1)
    compiler = subprocess.check_output(["rustup", "which", "--toolchain", channel, "rustc"], text=True).strip()
    environment = os.environ.copy()
    environment["RUSTC"] = compiler
    return ["rustup", "run", channel, "cargo"], environment, channel


def package_version(cargo, environment):
    metadata = json.loads(subprocess.check_output([*cargo, "metadata", "--no-deps", "--format-version", "1"], cwd=ROOT, env=environment))
    package = next(item for item in metadata["packages"] if item["name"] == "sdev")
    version = package["version"]
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[A-Za-z0-9.-]+)?", version):
        raise ValueError("Unsupported release version")
    return version


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, choices=TARGETS)
    args = parser.parse_args()
    cargo, environment, channel = toolchain()
    label, minimum = TARGETS[args.target]
    if minimum:
        environment["MACOSX_DEPLOYMENT_TARGET"] = minimum
    version = package_version(cargo, environment)
    subprocess.run(["rustup", "target", "add", "--toolchain", channel, args.target], check=True)
    subprocess.run([*cargo, "build", "--locked", "--release", "--target", args.target], cwd=ROOT, env=environment, check=True)
    extension = ".exe" if "windows" in args.target else ""
    source = ROOT / "target" / args.target / "release" / ("sdev" + extension)
    artifact = ROOT / "dist" / f"sdev-v{version}-{label}{extension}"
    artifact.parent.mkdir(exist_ok=True)
    shutil.copy2(source, artifact)
    digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
    artifact.with_suffix(artifact.suffix + ".sha256").write_text(f"{digest}  {artifact.name}\n")
    notices = artifact.parent / "THIRD-PARTY-NOTICES.html"
    shutil.copy2(ROOT / "licenses/rust-1.96.0-COPYRIGHT-library.html", notices)
    if (ROOT / "LICENSE").is_file():
        shutil.copy2(ROOT / "LICENSE", artifact.parent / "LICENSE")
    relative = artifact.relative_to(ROOT).as_posix()
    info = {"version": version, "target": args.target, "binary": relative, "bytes": artifact.stat().st_size, "sha256": digest, "published": False, "notices": notices.name}
    artifact.with_suffix(artifact.suffix + ".json").write_text(json.dumps(info, indent=2) + "\n")
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
            output.write(f"binary={relative}\nasset={artifact.name}\nversion={version}\n")
    print(json.dumps(info, indent=2))


if __name__ == "__main__":
    main()
