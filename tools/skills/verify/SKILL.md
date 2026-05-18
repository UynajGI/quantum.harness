---
name: verify
description: Use after writing or modifying an important artifact (protocol TOML, run plan, reproduction script, computed result, final run report, KB card) or when a reproduction gate fails and needs independent mismatch triage.
---

# verify

Dispatch a high-effort review subagent to audit an artifact against its declared reference. Generic over target. The subagent reads both sources end-to-end and returns a structured diff report. Never modifies anything.

The verifier must be a **different actor** from the artifact's author. Flow's `audit` check kind enforces this mechanically; this skill is the dispatcher.

## When to activate

- After writing or editing a `protocol.toml`, KB card, reproduction script, run report, or consolidated script.
- After producing a result, figure, or artifact bundle.
- Before claiming a reproduction complete; before merging a KB-card edit.
- When a flow gate fails (`flow check` shows failures) — use `mismatch` mode.

## Inputs

- `<artifact>` — file path of the artifact under review.
- `--against <reference>` — optional. Auto-inferred when obvious.
- `--mode <protocol | plan | kb-card | script | result | mismatch | close>` — selector; picked from artifact extension + content when absent.

## Dispatch

Spawn the review subagent using the same model id, reasoning/effort, sandbox, approval policy, and tool access as the main agent. **Same actor cannot author and verify** — flow rejects the `audit` check if `--actor` matches the producer's actor.

The verifier brief includes:

- The verbatim artifact + the verbatim reference (or precise line range).
- "Think deeply. Inspect both sources end-to-end before reporting; avoid skim-based conclusions."
- "You are independent from the author. Treat author comments and summaries as claims to check, not evidence."
- The per-mode axes below.
- "Output the per-axis status table. Do not write Action items or fixes — that is the calling skill's job."

Minimum context bundle: the artifact, the primary source (or exact excerpts), the current `protocol.toml` when the artifact is downstream of it, and the exact question (which claim, gate, or mismatch is being audited).

## Per-mode axes

### `kb-card`

Per AGENTS.md provenance discipline:

- **Literal** → grep the cited literature file for the verbatim phrase. ✓ with line number, ✗ unsupported, ⚠ near-match.
- **Analytic** → row states a derivation. ✓ if stated, ⚠ if unstated.
- **Harness anchor** → row names a `results/` path and cross-check method. ✓ if both, ⚠ if either missing.
- *Untagged* → ✗.

### `protocol`

Compare the protocol against declared primary sources.

1. **Source authority** — only `primary` rows support paper-derived claims; `hint` rows require an explicit assumption flag.
2. **Claim support** — each claim cites a primary passage or is marked `assumption=true`.
3. **Check coverage** — each non-assumption claim is covered by ≥1 `[[checks]]` entry or by a justified deviation.
4. **Generic check shape** — checks use one of the six kinds: `audit`, `run`, `exists`, `agree`, `near`, `fresh`. No paper-specific or domain-specific kind names.
5. **Deviation clarity** — declared differences from the paper are scoped, claim-tagged, and have their own checks.
6. **Gate completeness** — every gate the flow template declares has checks sufficient to prevent stale or unsupported artifacts from passing.

Severity tags: `supported`, `unsupported`, `hint-leak`, `assumption`, `deviation`, `missing-check`, `non-generic`.

### `plan`

Compare `reproduce-plan.toml` and `run_spec.json` against the protocol and methodology source.

1. **Claim-to-route coverage** — every non-assumption claim has an executable route, artifact target, and check.
2. **Figure dependency graph** — shared artifacts and edges consistent with the protocol.
3. **Run-spec provenance** — cells carry source ids, claim ids, deviation ids, stack identity, settings.
4. **Trusted-reference reachability** — the chosen reference exercises the production code path at easier scale.
5. **Hint quarantine** — old data, old figures, old plans never serve as evidence.

Severity tags: `covered`, `missing-route`, `plan-gap`, `stale-dependency`, `provenance-gap`, `weak-reference`, `hint-leak`.

### `script`

Compare the script against the protocol and the cited methodology section.

1. **Claim coverage** — script produces evidence for every assigned claim.
2. **Deviation honesty** — every difference from the paper is recorded as a deviation.
3. **Manifest provenance** — script writes the fields required by `exists`/`agree`/`fresh` checks (`evidence_class = "current_run"`, hashes, claim ids, etc.).
4. **Regime adequacy** — budgets and knobs are sufficient for the declared checks, not just plausible-looking numbers.

Severity tags: `match`, `proxy`, `unrecorded-deviation`, `provenance-gap`, `regime-gap`.

### `result`

For each numerical claim: ✓ within paper's error bar, ⚠ outside paper but within convergence margin, ✗ disagrees. Confirm each result artifact is `current_run` evidence with matching hashes.

### `mismatch`

Audit a failed gate. Input: the failing check, expected vs observed, protocol, source passages, scripts, manifests, prior repair attempt.

1. **Observed-vs-expected mismatch** — concrete disagreement with cited artifacts.
2. **Mismatch class** — `source_misread`, `unsupported_assumption`, `convention_mismatch`, `plan_gap`, `script_bug`, `stack_or_remote_failure`, `stale_or_provenance_gap`, `insufficient_convergence`, `statistical_noise`, `paper_ambiguity`, `out_of_scope`.
3. **Earliest wrong layer** — source/protocol, trusted reference, plan/run spec, script/aggregator, stack, raw cells, figure/report, or paper scope.
4. **Invalidation scope** — downstream gates that can no longer support claims.

Severity tags: `classified`, `under-evidenced`, `source-layer`, `plan-layer`, `script-layer`, `compute-layer`, `assembly-layer`, `report-layer`, `scope-layer`.

### `close`

Audit the final run report, consolidated script, protocol, verification reports, and manifest set.

1. **Claim boundedness** — every report claim maps to a protocol claim, a result artifact, and a verify report.
2. **Artifact provenance** — manifests carry required hashes, claim ids, deviation ids.
3. **Gate closure** — every flow gate is `passed` (or the run is marked partial / has recorded overrides).
4. **Deviation visibility** — declared differences from the paper appear in the report, linked to protocol deviations.
5. **Reproducibility** — consolidated script + run command sufficient for fresh checkout.
6. **Audience-facing artifact** — when a `report_*.html` is in scope: every editorial sentence in `editorial.json` traces to its `sourced_by` (re-grep the cited file:line); rendered HTML conforms to `docs/DESIGN.md`; mobile rendering at 375×667 has no overflow and ≥ 44×44px tap targets.

Severity tags: `supported`, `unsupported-claim`, `hint-leak`, `stale-artifact`, `provenance-gap`, `open-gate`, `hidden-deviation`, `repro-gap`, `editorial-leak`, `design-drift`.

## Output

Markdown report at `<run-dir>/verify/verify_<artifact-stem>_<date>.md`.

**Citation convention** (mandatory for `result` and `close` modes): every per-axis row carries a `claim ids` column listing the protocol `[[claims]].id` and `[[checks]].id` it backs. `/report` grep-scans these ids to resolve chip status.

```markdown
# /verify report — <artifact> — <date>

**Mode**: protocol | plan | kb-card | script | result | mismatch | close.
**Reference**: <path / lines>.

| Axis / Row | Status | Severity | Claim ids | Notes |
|---|---|---|---|---|
| ... | ✓ / ⚠ / ✗ | ... | `fig4.shape, fig4.minimum` | ... |

## Detailed findings
### Axis N — <name>
... verbatim passage with line number, verbatim script snippet, conclusion ...
```

**No "Action items" section.** Translating findings into next-step options is the *calling skill's* job.

## Discipline

- Inspection-only. Never modify the artifact.
- Same actor cannot author and verify (flow's `audit` check enforces this).
- Conservative on ambiguity: prefer ✗ over rationalized near-matches.
- For Analytic and Harness anchors, the tag is sufficient provenance.
- Generic over artifact type and reference. No artifact-specific logic.
- Subagent matches the main agent's model/effort/sandbox unless the user asks otherwise.

## Composition

- Called by `/reproduce-paper` for protocol, plan, script, result, mismatch, and close audits.
- Called by `/report` for close-mode audit of the rendered HTML.
- Called by KB-card or script authors as a pre-commit gate.
- The subagent surfaces what is wrong; the *calling skill* translates findings into 2–3 user options via the host's native API (`AskUserQuestion` in Claude Code; equivalent in Codex). User ratifies; only then does cleanup happen.

In a flow-backed run, the caller records this audit as an `audit`-kind attempt on the relevant gate, attaches the report path with `--report`, then `flow attempt finish`. Flow's `audit` check verifies actor distinctness and report presence.

## Anti-patterns

- Modifying the artifact from inside this skill.
- Treating KB-sourced protocol claims as primary-source support.
- Treating old scripts, plans, data, or figures as evidence without current hashes.
- Adding paper- or domain-specific kind names to `[[checks]]`.
- Closing a final run report without a `close`-mode review.
- Single-line "looks fine" report — the structured per-axis table is the deliverable.
- Downgrading subagent model or effort.
- Subagent prescribing fixes / writing an `## Action items` section.
