#!/usr/bin/env python3
"""Audit Rust source files for stale, misleading, or inaccurate comments.

Scans `libs/`, `apps/`, and `tools/` for:
1. Stale/placeholder language (`fake`, `dummy`, `simulated`, `placeholder`,
   `omitted`, `WRONG`, `for now`, `TODO`, `FIXME`, `HACK`, `XXX`, `stub`,
   `temporary`, `hardcoded`, `not yet implemented`).
2. Suspicious commented-out assertions or function calls in tests.
3. Model-aware functions that accept `_model: MixerModel` (unused model param).

Usage:
    python tools/audit_comments.py [--fail-on-findings] [--root <path>]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

STALE_PATTERNS: list[tuple[str, re.Pattern[str]]] = [
    (
        "stale-marker",
        re.compile(
            r"//.*\b(TODO|FIXME|HACK|XXX|WRONG|placeholder|omitted|not yet implemented|for now|stub)\b",
            re.IGNORECASE,
        ),
    ),
    (
        "fake-or-dummy",
        re.compile(
            r"//.*\b(fake|dummy|simulated|hardcoded|mocked)\b",
            re.IGNORECASE,
        ),
    ),
    (
        "commented-out-call",
        re.compile(r"^\s*//\s*(run\(|assert|expect\(|unwrap\()"),
    ),
]

UNUSED_MODEL_PARAM = re.compile(r"fn\s+\w+\s*\([^)]*\b_model\s*:\s*MixerModel")


def iter_rust_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for top in ("libs", "apps", "tools"):
        base = root / top
        if base.exists():
            files.extend(sorted(base.rglob("*.rs")))
    return files


def audit_file(path: Path, root: Path) -> list[tuple[str, int, str, str]]:
    findings: list[tuple[str, int, str, str]] = []
    rel = path.relative_to(root).as_posix()
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError as exc:
        findings.append((rel, 0, "io-error", str(exc)))
        return findings

    in_test_mod = False
    for lineno, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("#[cfg(test)]"):
            in_test_mod = True

        # Check unused _model parameter in public/non-test code
        if not in_test_mod and UNUSED_MODEL_PARAM.search(line):
            findings.append((rel, lineno, "unused-model-param", stripped))

        # Only inspect comment lines for comment-quality rules
        if "//" not in line:
            continue

        # Skip intentional CLI help/doc comments explaining --simulate or omitted port
        if stripped.startswith("///") and (
            "--simulate" in line or "port is omitted" in line or "when omitted" in line
        ):
            continue

        for category, pattern in STALE_PATTERNS:
            # Allow test helper comments mentioning "dummy UDP socket for test"
            if category == "fake-or-dummy" and in_test_mod and "dummy" in line.lower():
                continue
            if pattern.search(line):
                findings.append((rel, lineno, category, stripped))
                break

    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit Rust comments for staleness/inaccuracy.")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument(
        "--fail-on-findings",
        action="store_true",
        help="Exit with code 1 if any findings are reported.",
    )
    args = parser.parse_args()

    all_findings: list[tuple[str, int, str, str]] = []
    for rs_file in iter_rust_files(args.root):
        all_findings.extend(audit_file(rs_file, args.root))

    print(f"Scanned {len(iter_rust_files(args.root))} Rust files; found {len(all_findings)} potential comment/stub issues:\n")
    for rel, lineno, category, text in all_findings:
        print(f"  [{category}] {rel}:{lineno}: {text}")

    if args.fail_on_findings and all_findings:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
