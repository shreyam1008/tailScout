#!/usr/bin/env python3
"""Fail when the public TailScout site drifts from Cargo's package version."""

from __future__ import annotations

import json
import re
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PACKAGE = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
VERSION = PACKAGE["package"]["version"]
HTML = (ROOT / "docs" / "index.html").read_text(encoding="utf-8")


def fail(message: str) -> None:
    raise SystemExit(f"[check-release-truth] ERROR: {message}")


def text(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require_version(path: str, pattern: str, label: str) -> None:
    match = re.search(pattern, text(path), flags=re.MULTILINE)
    if not match:
        fail(f"{path} has no {label} version")
    if match.group(1) != VERSION:
        fail(f"{path} {label} version {match.group(1)!r} does not match {VERSION!r}")


visible_match = re.search(r'data-release-version="([^"]+)"', HTML)
if not visible_match:
    fail("docs/index.html has no visible data-release-version marker")
if visible_match.group(1) != VERSION:
    fail(f"visible release {visible_match.group(1)!r} does not match Cargo.toml {VERSION!r}")

scripts = re.findall(
    r'<script\s+type="application/ld\+json">\s*(.*?)\s*</script>',
    HTML,
    flags=re.DOTALL,
)
if not scripts:
    fail("docs/index.html has no JSON-LD")

software = None
for source in scripts:
    payload = json.loads(source)
    nodes = payload.get("@graph", [payload])
    software = next((node for node in nodes if node.get("@type") == "SoftwareApplication"), software)

if software is None:
    fail("JSON-LD has no SoftwareApplication node")
if software.get("softwareVersion") != VERSION:
    fail(
        f"JSON-LD softwareVersion {software.get('softwareVersion')!r} "
        f"does not match Cargo.toml {VERSION!r}"
    )

expected_download = f"https://github.com/shreyam1008/tailScout/releases/tag/v{VERSION}"
if software.get("downloadUrl") != expected_download:
    fail(f"JSON-LD downloadUrl must be {expected_download}")

require_version(
    "platform/windows/Directory.Build.props",
    r"<Version>([^<]+)</Version>",
    "Windows",
)
require_version(
    "platform/macos/version.env",
    r"^MARKETING_VERSION=(.+)$",
    "macOS",
)
require_version(
    "packaging/dev.shre.TailScout.metainfo.xml",
    r'<release version="([^"]+)"',
    "AppStream",
)

if f"## [{VERSION}]" not in text("CHANGELOG.md"):
    fail(f"CHANGELOG.md has no {VERSION} release section")

for stale_root in ("packaging", "snap"):
    directory = ROOT / stale_root
    if directory.exists():
        for path in directory.rglob("*"):
            if path.is_file() and "placeholder" in path.read_text(encoding="utf-8", errors="ignore").lower():
                fail(f"{path.relative_to(ROOT)} still contains placeholder text")

print(f"[check-release-truth] OK TailScout v{VERSION}")
