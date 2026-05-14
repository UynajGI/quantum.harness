#!/usr/bin/env python3
"""Render a /report HTML from a reproduction run.

Usage:
  render.py <run-dir>

Reads:
  <run-dir>/protocol.toml                (required)
  <run-dir>/run-report.md                (optional in dogfood; required for production)
  <run-dir>/cells/<id>/manifest.json     (optional; surfaced in cell drawer)
  <run-dir>/verify/verify_*.md           (optional; backs chip statuses)
  <run-dir>/figs/<id>.{png,json}         (required per [[figures]])
  <run-dir>/editorial.json               (optional; mechanical fallbacks otherwise)
  <run-dir>/progress/state.toml          (optional; flow state for provenance footer)

Writes:
  <run-dir>/report_<run-id>_<YYYY-MM-DD>.html
  <run-dir>/report_latest.html           (symlink, or copy on Windows)

Stdlib only (tomllib, json, base64, datetime, html, re, sys, argparse, pathlib).
"""
import argparse
import base64
import datetime
import html
import json
import re
import sys
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
TEMPLATE_PATH = Path(__file__).resolve().parent.parent / "templates" / "report.html.tmpl"
COLORS = ["#b39c80", "#a87a55", "#c96442", "#7a2a1a"]
SOFT_CAP_BYTES = 1_000_000  # 1MB
HARD_CAP_BYTES = 5_000_000  # 5MB


def load_toml(p: Path) -> dict:
    with open(p, "rb") as f:
        return tomllib.load(f)


def b64_png(p: Path) -> str:
    raw = p.read_bytes()
    if len(raw) > 500_000:
        print(f"[warn] paper PNG {p.name} is {len(raw):,} bytes (> 500KB cap)", file=sys.stderr)
    return "data:image/png;base64," + base64.b64encode(raw).decode("ascii")


def resolve(path_str: str, run_dir: Path) -> Path:
    p = Path(path_str)
    if p.is_absolute():
        return p
    candidate = REPO_ROOT / p
    if candidate.exists():
        return candidate
    return run_dir / p


def load_editorial(run_dir: Path) -> dict:
    p = run_dir / "editorial.json"
    if p.exists():
        return json.loads(p.read_text())
    return {"headline": None, "claims": [], "deviations": [], "figures": [], "glossary": [], "gaps": []}


def chip_for(status: str, label: str, popover: str) -> str:
    return (
        f'<span class="chip {html.escape(status)}" role="button" tabindex="0">'
        f'{html.escape(label)}'
        f'<span class="chip-pop" role="tooltip">{html.escape(popover)}</span>'
        f'</span>'
    )


def claim_status_from_verify(claim_id: str, verify_dir: Path) -> str:
    """ok if claim id appears with ✓ nearby; warn if ✗ or ⚠; muted if no verify mentions it."""
    if not verify_dir.exists():
        return "muted"
    for v in verify_dir.glob("verify_*.md"):
        text = v.read_text()
        if claim_id not in text:
            continue
        # Find nearby status markers (within the same line or next line)
        for line in text.splitlines():
            if claim_id in line:
                if "✗" in line or "✗" in text.split(claim_id)[-1][:200]:
                    return "warn"
                if "⚠" in line:
                    return "warn"
                if "✓" in line:
                    return "ok"
    return "muted"


def status_strip_html(claims: list[dict], deviations: list[dict], editorial: dict, verify_dir: Path) -> str:
    ed_claims = {c["id"]: c for c in editorial.get("claims") or []}
    ed_devs = {d["id"]: d for d in editorial.get("deviations") or []}

    chips = []
    for c in claims[:4]:
        cid = c.get("id", "")
        status = claim_status_from_verify(cid, verify_dir)
        ed = ed_claims.get(cid, {})
        label = ed.get("display_label") or cid
        popover = ed.get("popover") or c.get("statement", "")
        chips.append(chip_for(status, label, popover))
    for d in deviations[:2]:
        did = d.get("id", "")
        ed = ed_devs.get(did, {})
        label = ed.get("display_label") or did
        popover = ed.get("popover") or d.get("statement", "")
        chips.append(chip_for("warn", label, popover))
    chips.append('<span class="chip spacer"></span>')
    chips.append('<span class="chip muted">click any data point for the cell manifest</span>')
    return "\n    ".join(chips)


def contract_html(protocol: dict) -> str:
    artifact = protocol.get("artifact", {})
    sources = protocol.get("sources", [])
    claims = protocol.get("claims", [])
    deviations = protocol.get("deviations", [])
    budgets = protocol.get("budgets", {})

    parts: list[str] = []
    parts.append('<div class="k">Source</div><div class="v">')
    for s in sources:
        parts.append(f'<span class="pill">{html.escape(str(s.get("id") or s.get("path") or ""))}</span>')
    parts.append("</div>")

    parts.append(f'<div class="k">Scope</div><div class="v">{html.escape(artifact.get("description", ""))}</div>')

    parts.append('<div class="k">Claims</div><div class="v">')
    for c in claims:
        parts.append(
            f'<div class="claim-line-c"><span class="id">{html.escape(c.get("id", ""))}</span>'
            f'<span class="stmt">{html.escape(c.get("statement", ""))}</span></div>'
        )
    parts.append("</div>")

    parts.append('<div class="k">Deviations</div><div class="v">')
    for d in deviations:
        parts.append(
            f'<div class="claim-line-c"><span class="id">{html.escape(d.get("id", ""))}</span>'
            f'<span class="stmt">{html.escape(d.get("statement", ""))}</span></div>'
        )
    parts.append("</div>")

    parts.append('<div class="k">Budget</div><div class="v">')
    for k in ("wall_clock", "compute"):
        v = budgets.get(k)
        if v:
            parts.append(f'<span class="pill">{html.escape(str(v))}</span>')
    parts.append("</div>")
    return "".join(parts)


def discrepancy_html(deviations: list[dict], editorial: dict) -> str:
    ed_devs = {d["id"]: d for d in editorial.get("deviations") or []}
    paragraphs: list[str] = []
    for d in deviations:
        ed = ed_devs.get(d.get("id", ""), {})
        paragraph = ed.get("discrepancy_paragraph") or d.get("statement", "")
        paragraphs.append(f"<p>{html.escape(paragraph)}</p>")
    if not paragraphs:
        paragraphs.append("<p>No deviations declared.</p>")
    return "\n    ".join(paragraphs)


def provenance_html(run_id: str, protocol: dict, flow_state: dict | None, n_cells: int, total_wall_h: float) -> str:
    artifact = protocol.get("artifact", {})
    cluster = ""
    finished = ""
    if flow_state:
        cluster = flow_state.get("cluster") or flow_state.get("default_executor", "")
        finished = flow_state.get("finished_at") or flow_state.get("last_event", "")

    today = datetime.date.today().isoformat()
    return (
        f'<div>'
        f'<div class="label">Run</div>'
        f'<p style="margin: 0;"><code style="font-family: var(--mono); font-size: 12px; color: var(--olive);">results/{html.escape(run_id)}/</code><br>{n_cells} cells · {total_wall_h:.1f} wall-hours</p>'
        f'</div>'
        f'<div>'
        f'<div class="label">Cluster</div>'
        f'<p style="margin: 0;">{html.escape(str(cluster) or "—")}<br>{html.escape(str(finished))}</p>'
        f'</div>'
        f'<div>'
        f'<div class="label">Source</div>'
        f'<p style="margin: 0;"><code style="font-family: var(--mono); font-size: 12px; color: var(--olive);">{html.escape(artifact.get("paper", ""))}</code><br>'
        f'{html.escape(artifact.get("description", "")[:80])}</p>'
        f'</div>'
        f'<div>'
        f'<div class="label">Harness</div>'
        f'<p style="margin: 0;">Report ID: <code style="font-family: var(--mono); font-size: 12px; color: var(--olive);">{html.escape(run_id)}</code> @ {today}<br>Rendered by <code style="font-family: var(--mono); font-size: 12px; color: var(--olive);">/report</code></p>'
        f'</div>'
    )


def count_cells_and_wall(run_dir: Path) -> tuple[int, float]:
    cells_dir = run_dir / "cells"
    if not cells_dir.exists():
        return 0, 0.0
    n = 0
    total = 0.0
    for m in list(cells_dir.glob("**/manifest.json")) + list(cells_dir.glob("manifest_*.json")):
        try:
            d = json.loads(m.read_text())
        except json.JSONDecodeError:
            continue
        n += 1
        total += float(d.get("wall_seconds", 0))
    return n, total / 3600.0


def main() -> int:
    parser = argparse.ArgumentParser(description="Render a /report HTML for a reproduction run")
    parser.add_argument("run_dir", help="Path to the reproduction run directory")
    args = parser.parse_args()
    run_dir = Path(args.run_dir).resolve()
    if not run_dir.is_dir():
        print(f"error: run-dir does not exist: {run_dir}", file=sys.stderr)
        return 1

    protocol = load_toml(run_dir / "protocol.toml")
    editorial = load_editorial(run_dir)
    artifact = protocol.get("artifact", {})

    figures = protocol.get("figures", [])
    if not figures:
        print("error: protocol.toml has no [[figures]] entries", file=sys.stderr)
        return 1
    featured_id = protocol.get("featured_figure") or figures[0]["id"]
    fig = next((f for f in figures if f["id"] == featured_id), figures[0])

    paper_path = resolve(fig["paper_path"], run_dir)
    data_path = resolve(fig["data_path"], run_dir) if fig.get("data_path") else None
    paper_data_url = b64_png(paper_path)
    data_obj = json.loads(data_path.read_text()) if data_path and data_path.exists() else {"data": [], "axes": {}}

    # Editorial fields with mechanical fallbacks
    headline_text = (editorial.get("headline") or {}).get("text") or (
        protocol.get("claims", [{}])[0].get("statement", "")
    )
    headline_html = html.escape(headline_text)

    ed_figs = {f["label"]: f for f in editorial.get("figures") or []}
    ef = ed_figs.get(featured_id, {})
    paper_panel_title = ef.get("caption_paper") or fig.get("paper_attribution", "Paper")
    ours_panel_title = ef.get("caption_ours") or f"Reproduction · {artifact.get('run_id', run_dir.name)}"

    # Derive run_id, IDs, URLs
    run_id = artifact.get("run_id") or run_dir.name
    paper_id = artifact.get("paper", "")
    arxiv_match = re.search(r"(\d{4}\.\d{5})", paper_id)
    arxiv_id = arxiv_match.group(1) if arxiv_match else paper_id

    # Authors: pull from first source's note (best effort) or leave blank
    authors = ""
    paper_title = ""
    venue = ""
    sources = protocol.get("sources", [])
    if sources:
        note = sources[0].get("note", "")
        # Heuristic: "Authors. Venue (year)." → split on first period
        if "." in note:
            authors_part, _, rest = note.partition(".")
            authors = authors_part.strip()
            venue_match = re.match(r"\s*(.*?)\s*$", rest.strip())
            if venue_match:
                venue = venue_match.group(1).rstrip(".")

    # Glossary JSON (key by data-term value)
    gloss_dict: dict[str, dict] = {}
    for g in editorial.get("glossary") or []:
        sym = g.get("symbol", "")
        # Use a short key derived from the symbol (first letter sequence before any non-letter)
        key_match = re.match(r"[a-zA-Z_]+", sym)
        key = key_match.group(0).lower() if key_match else "term"
        gloss_dict[key] = {
            "name": g.get("name", ""),
            "body": g.get("body", ""),
            "formula": g.get("formula", ""),
        }

    # Flow state (optional)
    flow_state = None
    flow_state_path = run_dir / "progress" / "state.toml"
    if flow_state_path.exists():
        try:
            flow_state = load_toml(flow_state_path)
        except Exception:
            flow_state = None

    n_cells, total_wall_h = count_cells_and_wall(run_dir)

    # Compose substitution map
    subs = {
        "__PAGE_TITLE__": html.escape(f"{paper_id} · {run_id} · /report"),
        "__PAPER_ID__": html.escape(paper_id),
        "__AUTHORS__": html.escape(authors),
        "__PAPER_TITLE__": html.escape(paper_title),
        "__VENUE__": html.escape(venue),
        "__PAPER_URL__": f"https://arxiv.org/abs/{arxiv_id}",
        "__PAPER_URL_DISPLAY__": f"arxiv.org/abs/{arxiv_id}",
        "__RUN_SCOPE__": html.escape(artifact.get("scope", "")),
        "__RUN_META__": html.escape(f"{flow_state.get('cluster', '') if flow_state else ''} · {datetime.date.today().isoformat()}"),
        "__HEADLINE_HTML__": headline_html,
        "__RUN_TAG__": html.escape(f"{n_cells} cells · {total_wall_h:.1f} wall-h"),
        "__PAPER_SOURCE__": html.escape(f"{paper_id} · Fig {fig['id']}"),
        "__OURS_SOURCE__": html.escape(f"{run_id}/figs/{fig['id']}.png"),
        "__PAPER_PANEL_TITLE__": html.escape(paper_panel_title),
        "__OURS_PANEL_TITLE__": html.escape(ours_panel_title),
        "__PAPER_IMG_DATA_URL__": paper_data_url,
        "__STATUS_STRIP_HTML__": status_strip_html(
            protocol.get("claims", []), protocol.get("deviations", []), editorial, run_dir / "verify"
        ),
        "__CONTRACT_HTML__": contract_html(protocol),
        "__DISCREPANCY_HTML__": discrepancy_html(protocol.get("deviations", []), editorial),
        "__PROVENANCE_HTML__": provenance_html(run_id, protocol, flow_state, n_cells, total_wall_h),
        "__RUN_ID__": run_id,
        "__DATA_JSON__": json.dumps(data_obj, separators=(",", ":")),
        "__GLOSS_JSON__": json.dumps(gloss_dict, separators=(",", ":")),
        "__COLORS_JSON__": json.dumps(COLORS),
    }

    template = TEMPLATE_PATH.read_text()
    out_html = template
    for placeholder, value in subs.items():
        out_html = out_html.replace(placeholder, value)

    today = datetime.date.today().isoformat()
    out_filename = f"report_{run_id}_{today}.html"
    out_path = run_dir / out_filename
    out_path.write_text(out_html)

    size = out_path.stat().st_size
    if size > HARD_CAP_BYTES:
        print(f"error: rendered HTML exceeds 5MB hard cap ({size:,} bytes)", file=sys.stderr)
        return 1
    if size > SOFT_CAP_BYTES:
        print(f"[warn] rendered HTML exceeds 1MB soft cap ({size:,} bytes)", file=sys.stderr)

    # Update report_latest symlink (or copy on Windows)
    latest = run_dir / "report_latest.html"
    if latest.exists() or latest.is_symlink():
        latest.unlink()
    try:
        latest.symlink_to(out_filename)
    except OSError:
        latest.write_text(out_html)

    print(f"wrote {out_path}")
    print(f"  size: {size:,} bytes")
    print(f"  symlink: {latest} → {out_filename}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
