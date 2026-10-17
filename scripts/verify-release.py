#!/usr/bin/env python3
"""Check a native release artifact without publishing or requiring a runtime."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def library_checks(binary, target):
    if "apple" in target:
        listing = subprocess.check_output(["otool", "-L", str(binary)], text=True)
        libraries = [line.strip().split(" (")[0] for line in listing.splitlines()[1:]]
        if not all(path.startswith(("/usr/lib/", "/System/Library/")) for path in libraries):
            raise ValueError(f"Non-system library dependency: {libraries}")
        commands = subprocess.check_output(["otool", "-l", str(binary)], text=True)
        minimum = "11.0" if target.startswith("aarch64") else "10.12"
        found = re.search(r"cmd LC_BUILD_VERSION\s+cmdsize \d+\s+platform \d+\s+minos ([\d.]+)", commands)
        if not found:
            found = re.search(r"cmd LC_VERSION_MIN_MACOSX\s+cmdsize \d+\s+version ([\d.]+)", commands)
        if not found or found.group(1) != minimum:
            raise ValueError("Unexpected macOS deployment target")
    elif "linux" in target:
        versions = subprocess.check_output(["readelf", "--version-info", str(binary)], text=True)
        requirements = {tuple(map(int, value.split("."))) for value in re.findall(r"GLIBC_([\d.]+)", versions)}
        if not requirements or max(requirements) > (2, 35):
            raise ValueError(f"Unexpected glibc requirements: {requirements}")
    else:
        installer = Path(os.environ["ProgramFiles(x86)"]) / "Microsoft Visual Studio/Installer/vswhere.exe"
        candidates = subprocess.check_output([str(installer), "-latest", "-products", "*", "-requires", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-find", "VC/Tools/MSVC/**/bin/Hostx64/x64/dumpbin.exe"], text=True).splitlines()
        if not candidates:
            raise ValueError("dumpbin is required to inspect Windows DLL dependencies")
        listing = subprocess.check_output([candidates[0], "/dependents", str(binary)], text=True, errors="replace")
        forbidden = re.findall(r"\b(?:vcruntime|msvcp|msvcr|concrt)[\w.-]*\.dll\b", listing, flags=re.I)
        if forbidden:
            raise ValueError(f"Additional runtime DLLs: {forbidden}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve()
    metadata = json.loads(binary.with_suffix(binary.suffix + ".json").read_text())
    checksum = hashlib.sha256(binary.read_bytes()).hexdigest()
    if checksum != metadata["sha256"] or binary.with_suffix(binary.suffix + ".sha256").read_text().split()[0] != checksum:
        raise ValueError("Release checksum mismatch")
    target = metadata["target"]
    expected_os = "darwin" if "apple" in target else "linux" if "linux" in target else "win32"
    actual_arch = platform.machine().lower()
    expected_arch = ("arm64", "aarch64") if target.startswith("aarch64") else ("amd64", "x86_64")
    if sys.platform != expected_os or actual_arch not in expected_arch:
        raise ValueError("Run this check natively on the target OS and CPU")
    environment = {key: os.environ[key] for key in ("SystemRoot", "WINDIR", "TEMP", "TMP") if key in os.environ}
    environment["PATH"] = ""
    with tempfile.TemporaryDirectory(prefix="sdev-release-") as directory:
        def invoke(*values, status=0):
            result = subprocess.run([str(binary), *values], cwd=directory, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=10)
            if result.returncode != status:
                raise ValueError(f"{values}: {result.stderr.decode(errors='replace')}")
            return result
        assert invoke("--version").stdout.decode() == f"sdev {metadata['version']}\n"
        overview = invoke("--help").stdout.decode()
        assert "sdev help product" in overview
        assert "docs get" not in overview
        topics = re.findall(r"sdev help ([a-z-]+)", overview)
        for topic in topics:
            assert invoke("help", topic).stdout.startswith(b"# ")
        templates = re.findall(r"sdev template ([a-z-]+)", invoke("template").stdout.decode())
        for topic in templates:
            assert invoke("template", topic).stdout == (ROOT / "docs/templates" / (topic + ".md")).read_bytes()
        assert not invoke("help", "nonexistent", status=2).stdout
        assert not invoke("licenses", status=2).stdout
        notices = binary.parent / metadata["notices"]
        assert notices.read_bytes() == (ROOT / "licenses/rust-1.96.0-COPYRIGHT-library.html").read_bytes()
        assert not list(Path(directory).iterdir())
    library_checks(binary, target)
    print(json.dumps({"binary": str(binary.relative_to(ROOT)), "target": target, "guides": len(topics), "templates": len(templates), "verified": "native execution, exact template bodies and guide routing, empty PATH/cwd, checksum, and system-library dependencies"}, indent=2))


if __name__ == "__main__":
    main()
