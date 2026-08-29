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

print(f"[check-release-truth] OK TailScout v{VERSION}")
