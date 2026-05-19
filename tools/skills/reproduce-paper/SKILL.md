---
name: reproduce-paper
description: Use when the user wants to reproduce the figures and main results of a published paper end-to-end. Plans the multi-figure sequence, composes the harness primitives, and closes with a declared entry plus run report. Generic over papers.
---

# reproduce-paper

Thin orchestrator. The discipline lives in `protocol.toml`'s `[[checks]]`, evaluated mechanically by `tools/cli/flow`. This skill only sequences the work and composes the primitives.

## When to activate

- The user types "reproduce paper X", "redo the figures of Y", or names a paper to follow end-to-end.
- A `/solve` session is moving through several figures of the same paper and the user wants the full set.
- After `download-ref` lands a paper, the user wants to put it through the harness as a calibration target.

## Inputs

- A paper identifier (arXiv id, DOI, or a path under `knowledge-base/literature/<method>/`).
- A coverage scope (default: full).
- A wall-clock / compute budget (defaults from cluster profile).

## Spine

```
source → protocol → plan → script → trust → run → assemble → close
```

Each gate's contract lives in `protocol.toml` as `[[checks]]`. `flow` runs the checks on `flow attempt finish` and derives the gate status. The agent never declares pass.

## Workflow

1. **Init the ledger.** `tools/cli/flow init results/<run> --template tools/flow/templates/reproduce-paper.toml`. This is the first run-dir action.

2. **Acquire sources.** Copy `tools/templates/reproduce-paper/protocol.toml` to `results/<run>/protocol.toml`, fill `[artifact]`, `[[sources]]`, and the `source` gate check paths from primary sources, then start a `run` attempt on `source`. Use Markdown as the readable source: if an official or rendered Markdown source exists, place it under `sources/` and cite that; if only a PDF exists, store the PDF under `sources/` and render it with `python3 tools/skills/download-ref/scripts/render.py --pdf <pdf> --out <markdown>`. Finish the attempt, then `flow require <run> source` before protocol audit.

3. **Author the contract.** Fill the rest of `protocol.toml` from the primary source: `[entry]`, `[[claims]]`, `[[checks]]`, `[[figures]]`, optional `[[deviations]]` and `[[pending]]`. Use one-word check kinds: `audit`, `run`, `exists`, `agree`, `near`, `fresh`, `cover`, `support`; keep check ids unique because they are override handles. Use attempt roles `audit`, `trial`, `run`, `report`.

4. **Audit the contract.** Start an `audit`-kind attempt on the `protocol` gate with a verifier subagent (different `--actor` from whoever drafted the protocol). The verifier writes a report; finish the attempt with `--report <path>`. `flow` checks actors differ and the report exists.

5. **Plan the figure graph.** Author `results/<run>/reproduce-plan.toml` (figure ids, categorisations — substantive / methodology / verification / cross-check — and dependency edges). Start and finish the `plan` attempt.

6. **Implement the entry.** One declared `[entry]` runs the reproduction. Start a `run` attempt on `script`; register the entry as an artifact (`flow artifact add`); start a separate `audit` attempt; finish both. The entry's `help` and `dry` commands must exit before evidence IO.

7. **Trust.** Start a `run` attempt on `trust`. For each substantive figure, run the entry at a point where the answer is known (analytic limit, exact small instance, official benchmark), write trust artifacts under `trust/`, register them, and finish the attempt. Declare the check as `kind = "near"`. Failure here blocks `run`.

8. **Run.** Dispatch via `/parameter-scan` + `/slurm` (or the declared primitive). The cell wrappers register manifests as artifacts with `--producer <run-attempt>`. The `run` gate's `[[checks]]` enforce `cover`, `exists`, `agree`, `fresh`, and `producer = "run"`.

9. **Assemble.** Start a `run` attempt on `assemble`. Walk the run manifests, validate consensus/support, and produce each figure as `figs/<id>.png` plus `figs/<id>.json` (interactive plot source). Register assembled artifacts such as `figs/*` with the `run` attempt and finish it.

10. **Close.** Start a `report` attempt on `close`; generate `run-report.md`; keep the declared `[entry]` commands runnable with all parameters explicit. Register close-stage artifacts such as `run-report.md` with the `report` attempt; the entry artifact remains producer `run`. Finish the report attempt, then independently audit the close. The close gate runs `[entry].help` and `[entry].dry`; any evidence mutation should be caught by `fresh`.

11. **Render the deliverable.** `/report` consumes the run dir.

## Composition

- **Parameter sweeps** → `/parameter-scan`
- **Critical scaling** → `/scaling-fit`
- **Cross-checks** → `/cross-method-check`
- **Audits** (gate audit attempts in any mode) → spawn a verifier subagent with the host's option API or direct dispatch; pass it the protocol + primary source + artifact under review; have it write `verify/verify_<artifact>_<date>.md`; record the audit attempt with the verifier's actor id.
- **Cluster execution** → `/slurm` (called by `/parameter-scan`).
- **User-facing forks** → host platform's option API (Claude Code: `AskUserQuestion`; Codex: equivalent). Three options max, recommended first.

## Failed checks

When `flow attempt finish` reports a failing check:

1. **Backchain first** — inspect the earliest failed gate and its declared inputs. Do not patch a downstream report to hide an upstream failure.
2. **Repair the evidence** — fix the artifact, run a new attempt.
3. **Record a deviation** — `flow deviate <run> --id <id> --statement "..." --reason "..."` for runtime departures, or declare in `protocol.toml`'s `[[deviations]]` upfront. Both surface as ⚠ in `flow status`.
4. **Record a decision** — at any runtime fork the user hasn't pre-specified, present `AskUserQuestion` then `flow decide <run> --id <id> --question "..." --choice "..."`. Decisions surface in `flow status`.
5. **Override** — `flow override <run> <check-id> --reason "<text>"`. The override is recorded forever; downstream artifacts show ⊘.
6. **Stop** — always a real option.

Never edit the script, the protocol, or the run report to *make* a check pass without changing the underlying evidence.

## Closeout

End every turn with `tools/cli/flow status <run>` (or `flow status <run> --json` for tools). The default projection is terse: current gate, first blocker, next command. Use `flow status <run> --full` only when a human asks for details. Do not author a separate closeout paragraph that upgrades the gate status; the truthful summary always lives in the projection.

Before the agent declares the reproduction complete, the close gate must pass: `tools/cli/flow require <run> close` exits 0. The default protocol template gates per-figure `figs/<id>.{png,json}` in `assemble`, then gates `run-report.md`, the explicit entry artifact, and `[entry].help` / `[entry].dry` in `close` — so completion cannot pass with missing deliverables or unsafe inspection entrypoints. If close fails, repair the evidence or record a deviation; do not stop with an open close gate.

## Resume

The plan file and `progress/events.jsonl` are durable. Re-running this skill on an existing run dir reuses figures already produced (manifest-driven). Failed figures surface with their failure mode; not auto-retried.

## Notes

- Paper-specific words (claim ids, figure ids, deviation labels) live as data values in `protocol.toml`. Never in `flow`'s vocabulary, never in this skill's check-kind names.
- Methodology absorption is a side-effect of running the verification figures the paper declares — they appear alongside the substantive figures because they are also `[[figures]]` entries with `claim_ids`.
- The writeup-handoff close (declared entry + run report) happens in Step 10. Route to `scientific-writing` / `latex-paper-en` / `scientific-visualization` / `jupyter-notebook` for downstream artifacts.

## Related

- `/solve` — single-problem loop. This skill runs atop solve when the user wants the full paper.
- `/report` — renders the run dir as a shareable HTML page.
- `/verify` — audit subagent dispatcher; the audit-kind attempt this skill records is a `/verify` call.
- `/parameter-scan`, `/scaling-fit`, `/cross-method-check`, `/slurm` — primitives this skill orchestrates.
