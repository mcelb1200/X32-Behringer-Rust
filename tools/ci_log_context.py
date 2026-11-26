#!/usr/bin/env python3
"""Extract GitHub Actions CI logs, filter for high-SNR Rust errors/warnings,
and attach Rust AST enclosing-item + immediately connected component context
for AI agents.

Zero third-party Python dependencies (uses stdlib + `gh` CLI).

Usage:
    python tools/ci_log_context.py                          # Latest run on origin repo
    python tools/ci_log_context.py --run-id 45725713923     # Specific run ID
    python tools/ci_log_context.py --failed-only            # Only failed jobs
    python tools/ci_log_context.py --json                   # Machine-readable JSON
    python tools/ci_log_context.py --max-diagnostics 20     # Cap output size
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ANSI_RE = re.compile(r"(?:\x1b|\^\[)\[[0-9;]*[A-Za-z]")
# Matches Rust compiler diagnostic header: `error[E0428]: ...` or `warning: ...`
RUSTC_DIAG_RE = re.compile(
    r"(?P<sev>error|warning)(?:\[(?P<code>[^\]]+)\])?:\s+(?P<msg>.+)$"
)
# Matches Rust location line: `--> libs/x32_fxparse/src/lib.rs:9:1`
RUSTC_LOC_RE = re.compile(
    r"-->\s+(?P<file>[^\s:]+):(?P<line>\d+)(?::(?P<col>\d+))?"
)
# Matches Rust panic location: `panicked at apps/x32_auto_gain/src/lib.rs:305:9:`
PANIC_RE = re.compile(
    r"thread\s+'(?P<thread>[^']+)'[^p]*panicked at\s+(?P<file>[^\s:]+):(?P<line>\d+):(?P<col>\d+):"
)

# Structural Rust item declarations (AST-lite brace-balanced extractor)
RUST_ITEM_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r"(?P<kind>fn|struct|enum|trait|impl|mod|const|static|type)\s+"
    r"(?:<[^>]*>\s*)?(?P<name>[A-Za-z_][A-Za-z0-9_:]*)"
)
RUST_USE_RE = re.compile(r"^\s*(?:pub\s+)?use\s+([^;]+);")
IDENT_RE = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\b")


@dataclass
class AstItem:
    kind: str
    name: str
    start_line: int
    end_line: int


@dataclass
class ConnectedContext:
    enclosing_item: str | None = None
    enclosing_range: str | None = None
    snippet: list[str] = field(default_factory=list)
    relevant_imports: list[str] = field(default_factory=list)
    outgoing_symbols: list[str] = field(default_factory=list)
    inbound_references: list[str] = field(default_factory=list)


@dataclass
class Diagnostic:
    severity: str
    code: str | None
    message: str
    file: str
    line: int
    col: int | None
    jobs: list[str] = field(default_factory=list)
    context: ConnectedContext | None = None


def run_cmd(cmd: list[str], cwd: Path) -> str:
    proc = subprocess.run(
        cmd,
        cwd=str(cwd),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"Command failed ({proc.returncode}): {' '.join(cmd)}\n{proc.stderr.strip()}"
        )
    return proc.stdout


def detect_repo(root: Path) -> str:
    url = run_cmd(["git", "remote", "get-url", "origin"], root).strip()
    m = re.search(r"github\.com[:/](?P<slug>[^/]+/[^/.]+)(?:\.git)?$", url)
    if not m:
        raise RuntimeError(f"Could not parse GitHub owner/repo from origin URL: {url}")
    return m.group("slug")


def resolve_latest_run_id(repo: str, root: Path, branch: str | None) -> str:
    cmd = [
        "gh",
        "run",
        "list",
        "-R",
        repo,
        "--limit",
        "1",
        "--json",
        "databaseId",
    ]
    if branch:
        cmd.extend(["--branch", branch])
    data = json.loads(run_cmd(cmd, root))
    if not data:
        raise RuntimeError(f"No GitHub Actions runs found for {repo}")
    return str(data[0]["databaseId"])


def fetch_ci_logs(
    repo: str, run_id: str, job_id: str | None, failed_only: bool, root: Path
) -> str:
    cmd = ["gh", "run", "view", run_id, "-R", repo]
    if job_id:
        cmd.extend(["--job", job_id])
    cmd.append("--log-failed" if failed_only else "--log")
    return run_cmd(cmd, root)


def parse_github_url(url: str) -> tuple[str | None, str | None, str | None]:
    m = re.search(
        r"github\.com/(?P<repo>[^/]+/[^/]+)/actions/runs/(?P<run_id>\d+)(?:/job/(?P<job_id>\d+))?",
        url,
    )
    if not m:
        return None, None, None
    return m.group("repo"), m.group("run_id"), m.group("job_id")


def strip_gh_prefix(raw_line: str) -> tuple[str, str]:
    """Splits `job_name\tstep_name\t2026-...Z content` into `(job_name, clean_content)`."""
    clean = ANSI_RE.sub("", raw_line).rstrip("\r\n")
    parts = clean.split("\t", 2)
    if len(parts) == 3:
        job_name = parts[0].strip()
        body = re.sub(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z\s*", "", parts[2])
        body = re.sub(r"^##\[(error|warning)\]", "", body).strip()
        return job_name, body
    return "unknown", clean.strip()


def parse_diagnostics(log_text: str) -> list[Diagnostic]:
    """Extracts and deduplicates high-SNR Rust errors, warnings, and test panics."""
    dedup: dict[tuple[str, str, str, int], Diagnostic] = {}
    pending_by_job: dict[str, tuple[str, str | None, str]] = {}

    for raw_line in log_text.splitlines():
        job, line = strip_gh_prefix(raw_line)
        if not line:
            continue

        # 1. Test panic with file:line:col
        pm = PANIC_RE.search(line)
        if pm:
            file_path = pm.group("file").replace("\\", "/")
            lineno = int(pm.group("line"))
            col = int(pm.group("col"))
            msg = f"test '{pm.group('thread')}' panicked"
            key = ("error", msg, file_path, lineno)
            if key not in dedup:
                dedup[key] = Diagnostic(
                    severity="error",
                    code="panic",
                    message=msg,
                    file=file_path,
                    line=lineno,
                    col=col,
                    jobs=[job],
                )
            elif job not in dedup[key].jobs:
                dedup[key].jobs.append(job)
            continue

        # 2. Rustc error/warning header
        dm = RUSTC_DIAG_RE.match(line)
        if dm:
            sev = dm.group("sev")
            code = dm.group("code")
            msg = dm.group("msg").strip()
            # Filter out cargo summary noise lines like "could not compile ..."
            if msg.startswith("could not compile ") or msg.startswith("build failed,"):
                continue
            pending_by_job[job] = (sev, code, msg)
            continue

        # 3. Rustc location line `--> path:line:col`
        lm = RUSTC_LOC_RE.search(line)
        if lm and job in pending_by_job:
            sev, code, msg = pending_by_job.pop(job)
            file_path = lm.group("file").replace("\\", "/")
            lineno = int(lm.group("line"))
            col = int(lm.group("col")) if lm.group("col") else None
            key = (sev, msg, file_path, lineno)
            if key not in dedup:
                dedup[key] = Diagnostic(
                    severity=sev,
                    code=code,
                    message=msg,
                    file=file_path,
                    line=lineno,
                    col=col,
                    jobs=[job],
                )
            elif job not in dedup[key].jobs:
                dedup[key].jobs.append(job)

    # Sort errors first, then warnings, then by file & line
    order = {"error": 0, "warning": 1}
    return sorted(
        dedup.values(),
        key=lambda d: (order.get(d.severity, 9), d.file, d.line, d.message),
    )


def parse_rust_ast_items(lines: list[str]) -> list[AstItem]:
    """Brace-balanced Rust item extractor returning top-level & nested item spans."""
    items: list[AstItem] = []
    n = len(lines)
    i = 0
    while i < n:
        m = RUST_ITEM_RE.match(lines[i])
        if m:
            kind = m.group("kind")
            name = m.group("name")
            start_line = i + 1
            # Find end of item (semicolon for single-line const/type, or matching brace)
            depth = 0
            found_brace = False
            end_line = start_line
            for j in range(i, n):
                end_line = j + 1
                line = lines[j]
                if not found_brace and ";" in line and "{" not in line:
                    break
                for ch in line:
                    if ch == "{":
                        depth += 1
                        found_brace = True
                    elif ch == "}":
                        depth -= 1
                        if found_brace and depth == 0:
                            break
                if found_brace and depth == 0:
                    break
            items.append(AstItem(kind=kind, name=name, start_line=start_line, end_line=end_line))
        i += 1
    return items


def find_inbound_references(symbol: str, origin_file: str, root: Path, limit: int = 8) -> list[str]:
    if len(symbol) < 3 or symbol in {"new", "run", "main", "default", "from", "fmt", "tests"}:
        return []
    try:
        out = run_cmd(
            ["git", "grep", "-n", "-w", symbol, "--", "libs/", "apps/", "tools/"],
            root,
        )
    except RuntimeError:
        return []
    refs: list[str] = []
    for line in out.splitlines():
        if line.startswith(f"{origin_file}:"):
            continue
        refs.append(line.strip()[:140])
        if len(refs) >= limit:
            break
    return refs


def extract_ast_context(file_rel: str, lineno: int, root: Path) -> ConnectedContext:
    ctx = ConnectedContext()
    path = root / file_rel
    if not path.exists() or not path.is_file():
        return ctx

    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    if not lines:
        return ctx

    # 1. Bounded source snippet around diagnostic line (±4 lines)
    lo = max(1, lineno - 4)
    hi = min(len(lines), lineno + 4)
    for ln in range(lo, hi + 1):
        marker = ">>" if ln == lineno else "  "
        ctx.snippet.append(f"{marker} {ln:4d} | {lines[ln - 1]}")

    if path.suffix != ".rs":
        return ctx

    # 2. Enclosing AST item (prefer narrowest span containing lineno)
    items = parse_rust_ast_items(lines)
    containing = [it for it in items if it.start_line <= lineno <= it.end_line]
    enclosing = min(containing, key=lambda it: it.end_line - it.start_line) if containing else None

    if enclosing:
        ctx.enclosing_item = f"{enclosing.kind} {enclosing.name}"
        ctx.enclosing_range = f"L{enclosing.start_line}-L{enclosing.end_line}"
        body_text = "\n".join(lines[enclosing.start_line - 1 : enclosing.end_line])
        body_idents = set(IDENT_RE.findall(body_text))

        # 3. Relevant `use` imports referenced in enclosing item
        for line in lines:
            um = RUST_USE_RE.match(line)
            if um:
                use_stmt = um.group(1).strip()
                use_idents = set(IDENT_RE.findall(use_stmt))
                if use_idents & body_idents:
                    ctx.relevant_imports.append(f"use {use_stmt};")
                    if len(ctx.relevant_imports) >= 8:
                        break

        # 4. Immediately connected sibling items in the same file referenced by enclosing item
        sibling_names = {
            it.name: f"{it.kind} {it.name} (L{it.start_line})"
            for it in items
            if it.name != enclosing.name
        }
        for sym, label in sibling_names.items():
            if sym in body_idents:
                ctx.outgoing_symbols.append(label)
                if len(ctx.outgoing_symbols) >= 10:
                    break

        # 5. Inbound references across the workspace
        ctx.inbound_references = find_inbound_references(enclosing.name, file_rel, root)

    return ctx


def format_markdown(
    repo: str, run_id: str, job_id: str | None, diagnostics: list[Diagnostic]
) -> str:
    from collections import Counter

    err_cnt = sum(1 for d in diagnostics if d.severity == "error")
    warn_cnt = sum(1 for d in diagnostics if d.severity == "warning")
    url = f"https://github.com/{repo}/actions/runs/{run_id}" + (
        f"/job/{job_id}" if job_id else ""
    )
    out = [
        "# CI High-SNR Diagnostic & AST Context Report",
        f"- **Repository:** `{repo}`",
        f"- **Target:** {url}",
        f"- **Unique Diagnostics:** {len(diagnostics)} ({err_cnt} errors, {warn_cnt} warnings)",
        "",
    ]
    if not diagnostics:
        out.append("No Rust compiler errors, warnings, or test panics detected.")
        return "\n".join(out)

    # High-level summary tables for rapid AI triage
    by_msg = Counter((d.severity.upper(), d.message) for d in diagnostics)
    by_file = Counter(d.file for d in diagnostics)
    out.append("## Summary by Diagnostic Type")
    out.append("| Severity | Message | Count |")
    out.append("| :--- | :--- | :---: |")
    for (sev, msg), cnt in by_msg.most_common():
        out.append(f"| `{sev}` | {msg} | {cnt} |")
    out.append("")

    out.append("## Summary by File")
    out.append("| File | Count |")
    out.append("| :--- | :---: |")
    for fpath, cnt in by_file.most_common():
        out.append(f"| `{fpath}` | {cnt} |")
    out.append("")
    out.append("---")
    out.append("")

    for idx, d in enumerate(diagnostics, start=1):
        code_str = f"[{d.code}]" if d.code else ""
        loc = f"{d.file}:{d.line}" + (f":{d.col}" if d.col else "")
        out.append(f"## {idx}. `{d.severity.upper()}{code_str}` — `{loc}`")
        out.append(f"- **Message:** {d.message}")
        out.append(f"- **Jobs ({len(d.jobs)}):** {', '.join(d.jobs)}")
        if d.context:
            if d.context.enclosing_item:
                out.append(
                    f"- **Enclosing AST Item:** `{d.context.enclosing_item}` ({d.context.enclosing_range})"
                )
            if d.context.relevant_imports:
                out.append(
                    f"- **Connected Imports:** `{'`, `'.join(d.context.relevant_imports)}`"
                )
            if d.context.outgoing_symbols:
                out.append(
                    f"- **Connected Same-File Items:** `{'`, `'.join(d.context.outgoing_symbols)}`"
                )
            if d.context.inbound_references:
                out.append("- **Inbound Workspace Callers:**")
                for ref in d.context.inbound_references:
                    out.append(f"  - `{ref}`")
            if d.context.snippet:
                out.append("```rust")
                out.extend(d.context.snippet)
                out.append("```")
        out.append("")
    return "\n".join(out)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Extract GitHub CI logs, parse high-SNR diagnostics, and attach Rust AST context."
    )
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--url", help="Full GitHub Actions run or job URL.")
    parser.add_argument("--repo", help="GitHub owner/repo (auto-detected from git origin if omitted).")
    parser.add_argument("--run-id", help="GitHub Actions run ID (defaults to latest run).")
    parser.add_argument("--job-id", help="Optional specific GitHub Actions job ID.")
    parser.add_argument("--branch", help="Optional branch filter when resolving latest run.")
    parser.add_argument("--failed-only", action="store_true", help="Fetch only failed job logs.")
    parser.add_argument(
        "--max-diagnostics",
        type=int,
        default=0,
        help="Max diagnostics to report (0 = unlimited).",
    )
    parser.add_argument("--json", action="store_true", help="Emit JSON instead of Markdown.")
    parser.add_argument("--output", type=Path, help="Optional file path to write the report.")
    args = parser.parse_args()

    url_repo, url_run, url_job = parse_github_url(args.url) if args.url else (None, None, None)
    repo = args.repo or url_repo or detect_repo(args.root)
    run_id = args.run_id or url_run or resolve_latest_run_id(repo, args.root, args.branch)
    job_id = args.job_id or url_job
    log_text = fetch_ci_logs(repo, run_id, job_id, args.failed_only, args.root)
    diagnostics = parse_diagnostics(log_text)
    if args.max_diagnostics > 0:
        diagnostics = diagnostics[: args.max_diagnostics]

    for diag in diagnostics:
        diag.context = extract_ast_context(diag.file, diag.line, args.root)

    report = (
        json.dumps(
            {
                "repo": repo,
                "run_id": run_id,
                "job_id": job_id,
                "diagnostics": [asdict(d) for d in diagnostics],
            },
            indent=2,
        )
        if args.json
        else format_markdown(repo, run_id, job_id, diagnostics)
    )

    if args.output:
        args.output.write_text(report, encoding="utf-8")
        print(f"Wrote report ({len(diagnostics)} diagnostics) to {args.output}")
    else:
        print(report)
    return 0


if __name__ == "__main__":
    sys.exit(main())
