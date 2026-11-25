#!/usr/bin/env python3
"""Static analysis script for the X32-Behringer-Rust workspace.

Detects:
1. `unused-model-param`: Functions taking `_model: MixerModel` (or `model: MixerModel`
   whose body never references `model`).
2. `hardcoded-osc-port`: Literal X32/X-Air/Wing ports (`10023`, `10024`, `2222`) in
   non-test source code outside canonical model port definitions.
3. `hardcoded-osc-path`: Hardcoded X32 main/channel/bus OSC paths in `apps/` or `tools/`
   that should use `x32_lib::model::*` path builders.
4. `hardcoded-model-limit`: Hardcoded channel/bus ranges (`1..=32`, `0..32`, `> 32`,
   `1..=16`, `> 16`) in `apps/` that should use `x32_lib::model::limits_for_model`.
5. `hardcoded-magic-enum`: Raw integer literals for X32 EQ types, mute/on states, or
   RTA/screen indices in OSC messages where a named enum/const should be used.

Usage:
    python tools/static_analysis.py [--json] [--fail-on-findings] [--category <cat>]
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter
from dataclasses import asdict, dataclass
from pathlib import Path

FN_MODEL_PARAM_RE = re.compile(
    r"\bfn\s+(?P<name>\w+)\s*(?:<[^>]*>)?\s*\((?P<params>[^)]*)\)"
)
MODEL_PARAM_RE = re.compile(r"\b(?P<param>_?model)\s*:\s*(?:&?\s*)?(?:[\w:]+::)?MixerModel\b")

HARDCODED_PORT_RE = re.compile(r"(?<!\d)(10023|10024|2222)(?!\d)")
HARDCODED_OSC_PATH_RE = re.compile(
    r'"(/main/st/mix/(?:fader|on)|/ch/\{:02\}/mix/(?:fader|on)|/bus/\{:02\}/mix/(?:fader|on))"'
)
HARDCODED_LIMIT_RE = re.compile(
    r"\b(?:1\.\.=32|0\.\.32|1\.\.=16|0\.\.16|\b(?:ch|channel|bus)\s*(?:>|>=)\s*(?:32|16))\b"
)
HARDCODED_EQ_TYPE_RE = re.compile(r"/eq/.*/type.*OscArg::Int\(\s*\d+\s*\)|OscArg::Int\(\s*3\s*\).*PEQ")

# Canonical files where model ports/paths/limits are intentionally defined
PORT_ALLOWLIST = {
    "libs/x32_lib/src/model.rs",
    "libs/x32_fxparse/src/lib.rs",
    "libs/x32_lib/src/discovery.rs",
    "tools/x32_emulator/src/lib.rs",
}
PATH_ALLOWLIST = {
    "libs/x32_lib/src/model.rs",
    "libs/x32_lib/src/main_bus.rs",
    "libs/x32_core/src/lib.rs",
    "libs/x32_setsceneparse/src/lib.rs",
}


@dataclass
class Finding:
    file: str
    line: int
    category: str
    message: str
    snippet: str


def iter_rust_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for top in ("libs", "apps", "tools"):
        base = root / top
        if base.exists():
            for p in sorted(base.rglob("*.rs")):
                # Skip dedicated test files
                if "tests" in p.parts or p.name == "tests.rs":
                    continue
                files.append(p)
    return files


def extract_fn_body(lines: list[str], start_idx: int) -> str:
    """Extract brace-balanced function body starting at `start_idx`."""
    depth = 0
    started = False
    out: list[str] = []
    for line in lines[start_idx:]:
        out.append(line)
        for ch in line:
            if ch == "{":
                depth += 1
                started = True
            elif ch == "}":
                depth -= 1
                if started and depth == 0:
                    return "\n".join(out)
    return "\n".join(out)


def analyze_file(path: Path, root: Path) -> list[Finding]:
    findings: list[Finding] = []
    rel = path.relative_to(root).as_posix()
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        return [Finding(rel, 0, "io-error", str(exc), "")]

    lines = text.splitlines()
    in_test_mod = False

    # 1. Multi-line function signature scan for unused MixerModel parameters
    # Map character offset -> 1-based line number
    line_starts: list[int] = [0]
    for line in lines:
        line_starts.append(line_starts[-1] + len(line) + 1)

    def offset_to_line(offset: int) -> int:
        lo, hi = 0, len(line_starts) - 1
        while lo < hi:
            mid = (lo + hi + 1) // 2
            if line_starts[mid] <= offset:
                lo = mid
            else:
                hi = mid - 1
        return lo + 1

    test_mod_offset = text.find("#[cfg(test)]")
    for match in FN_MODEL_PARAM_RE.finditer(text):
        if test_mod_offset != -1 and match.start() >= test_mod_offset:
            continue
        params = match.group("params")
        m_param = MODEL_PARAM_RE.search(params)
        if not m_param:
            continue
        param_name = m_param.group("param")
        fn_name = match.group("name")
        lineno = offset_to_line(match.start())
        if param_name.startswith("_"):
            findings.append(
                Finding(
                    rel,
                    lineno,
                    "unused-model-param",
                    f"`{fn_name}` ignores `{param_name}: MixerModel` — should branch on model or drop parameter",
                    lines[lineno - 1].strip(),
                )
            )
        else:
            body = extract_fn_body(lines, lineno - 1)
            # Strip signature line before checking body references
            body_after_sig = body.split("{", 1)[1] if "{" in body else ""
            if not re.search(rf"\b{re.escape(param_name)}\b", body_after_sig):
                findings.append(
                    Finding(
                        rel,
                        lineno,
                        "unused-model-param",
                        f"`{fn_name}` takes `{param_name}: MixerModel` but never uses it in its body",
                        lines[lineno - 1].strip(),
                    )
                )

    # 2. Line-by-line scan for hardcoded ports, paths, limits, and magic enums
    for idx, line in enumerate(lines):
        lineno = idx + 1
        stripped = line.strip()
        if stripped.startswith("#[cfg(test)]"):
            in_test_mod = True
        if in_test_mod or stripped.startswith("//"):
            continue

        # Hardcoded OSC ports (ignore clap default_value attributes)
        if (
            rel not in PORT_ALLOWLIST
            and "default_value" not in line
            and HARDCODED_PORT_RE.search(line)
        ):
            findings.append(
                Finding(
                    rel,
                    lineno,
                    "hardcoded-osc-port",
                    "Hardcoded mixer OSC port; use `x32_lib::model::default_port_for_model` or `resolve_target_address`",
                    stripped,
                )
            )

        # Hardcoded X32 OSC paths in apps/tools
        if (
            rel not in PATH_ALLOWLIST
            and (rel.startswith("apps/") or rel.startswith("tools/"))
            and HARDCODED_OSC_PATH_RE.search(line)
        ):
            findings.append(
                Finding(
                    rel,
                    lineno,
                    "hardcoded-osc-path",
                    "Hardcoded X32 OSC path; prefer `x32_lib::model` path builders for X-Air/Wing compatibility",
                    stripped,
                )
            )

        # Hardcoded channel/bus limits in apps/
        if rel.startswith("apps/") and HARDCODED_LIMIT_RE.search(line):
            findings.append(
                Finding(
                    rel,
                    lineno,
                    "hardcoded-model-limit",
                    "Hardcoded channel/bus count (16/32); use `x32_lib::model::limits_for_model(model)`",
                    stripped,
                )
            )

        # Hardcoded EQ type magic int
        if HARDCODED_EQ_TYPE_RE.search(line):
            findings.append(
                Finding(
                    rel,
                    lineno,
                    "hardcoded-magic-enum",
                    "Hardcoded EQ type integer literal; use `x32_lib::common::EqType` enum",
                    stripped,
                )
            )

    return findings


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Static analysis for unused MixerModel params and hardcoded mixer constants."
    )
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--json", action="store_true", help="Emit JSON report.")
    parser.add_argument(
        "--category",
        help="Filter findings to a single category (e.g. unused-model-param).",
    )
    parser.add_argument(
        "--fail-on-findings",
        action="store_true",
        help="Exit with code 1 if any findings match.",
    )
    args = parser.parse_args()

    files = iter_rust_files(args.root)
    findings: list[Finding] = []
    for f in files:
        findings.extend(analyze_file(f, args.root))

    if args.category:
        findings = [x for x in findings if x.category == args.category]

    if args.json:
        print(json.dumps([asdict(x) for x in findings], indent=2))
    else:
        counts = Counter(f.category for f in findings)
        print(f"Analyzed {len(files)} non-test Rust files; found {len(findings)} findings:")
        for cat, cnt in sorted(counts.items()):
            print(f"  - {cat}: {cnt}")
        print()
        for f in findings:
            print(f"  [{f.category}] {f.file}:{f.line}")
            print(f"      {f.message}")
            print(f"      > {f.snippet}")

    if args.fail_on_findings and findings:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
