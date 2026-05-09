---
name: verify
description: Use after writing or modifying an important artifact (protocol TOML, knowledge-base card, reproduction script, computed result, final run report) to dispatch a high-effort review subagent that audits the artifact against its declared reference. Generic over target — protocol vs primary sources, KB card vs literature, script vs protocol/methodology, result vs paper-reported numbers, close vs protocol/artifacts.
---

# verify

Dispatch a high-effort review subagent (Opus, max effort) to audit an artifact against its declared reference. Generic over target. The skill is the dispatcher; the subagent reads both sources end-to-end and returns a structured diff report. The verifier never modifies anything.

The verifier must be independent from the authoring agent. If the current agent wrote or materially edited the artifact, it may run smoke tests but must still dispatch a separate verifier before the artifact can close a reproduction gate.

## When to activate

- After writing or editing a reproduction protocol TOML.
- After editing a KB card (anchors against literature).
- After writing a reproduction script (quantity / estimator / setup / regime against the paper).
- After producing a result or figure (numerical agreement against paper-reported values).
- After assembling a final run report or consolidated reproduction script.
- Before claiming a reproduction complete; before merging a KB-card edit.
- Pre-compute, to catch a methodological proxy before burning compute.

## Inputs

- `<artifact>` — file path of the thing to verify.
- `--against <reference>` — optional. Auto-inferred when obvious (KB card cites literature folder; script cites paper MD via comment header).
- `--mode <protocol | kb-card | script | result | close>` — optional override; the skill picks from the artifact extension and content.

## Dispatch

Spawn an Agent subagent with `model: "opus"`, `subagent_type: "general-purpose"`. The prompt MUST include:

- The verbatim artifact + the verbatim reference (or precise line range).
- "Think deeply, max effort, do not skim. Read both sources end-to-end before reporting."
- "You are independent from the authoring agent. Do not trust the author's comments, summaries, or stated intent; verify against the declared sources and artifact contents."
- The per-mode axes below.
- Request a structured report (per-row / per-axis status with severity tags).

Read-only on both artifact and reference. Do not downgrade the model or effort for speed — parity-with-doer is the point.

## Per-mode axes

### `kb-card`

Per AGENTS.md provenance discipline (Literal / Analytic / Harness anchor):

- **Literal** → grep the cited literature file for the verbatim phrase or number (modest format tolerance: `0.844` vs `0.84(4)`). ✓ with line number, ✗ unsupported, ⚠ ambiguous near-match.
- **Analytic** → confirm the row states a derivation (definition, limit, closed form). ✓ if stated, ⚠ if unstated.
- **Harness anchor** → confirm the row names a run path under `results/` and a cross-check method. ✓ if both present, ⚠ if either missing.
- *Untagged* → ✗ does not satisfy discipline.

### `protocol`

Compare the protocol TOML against the declared primary sources. Treat KB cards, rendered notes, scripts, and prior artifacts as hints only; they cannot become primary sources by being labeled that way in the protocol. Primary sources are the paper, supplement, official code, or official data. The audit is generic: it verifies source support and internal consistency, not domain truth by built-in type.

Axes:

1. **Source authority** — every claim cites a source; primary-source claims are checked against the paper/supplement/official code/data; KB-sourced claims are marked as hints or assumptions unless confirmed by a primary source.
2. **Claim support** — each claim's statement is supported by the cited source passage or is explicitly recorded as an assumption/deviation.
3. **Check coverage** — each non-assumption claim is covered by at least one declared check or by a justified deviation.
4. **Generic check shape** — checks use mechanical kinds (`source_audit`, `command`, `manifest_fields`, `manifest_consensus`, `numeric_compare`, `freshness`, `verify`) and do not rely on `/reproduce-paper` knowing domain-specific concepts.
5. **Deviation clarity** — deliberate differences from the paper are explicit, scoped, and have their own checks.
6. **Gate completeness** — preflight, compute, assembly, and report gates have enough checks to prevent stale artifacts and unsupported claims from reaching the final report.

Severity tags: `supported`, `unsupported`, `assumption`, `deviation`, `missing-check`, `non-generic`.

### `script`

Compare the script against the protocol TOML plus the cited primary methodology section. Four axes:

1. **Claim coverage** — script produces evidence for every protocol claim assigned to it.
2. **Deviation honesty** — any difference from the protocol's paper-derived claims is recorded as a deviation, not silently treated as equivalent.
3. **Manifest provenance** — script writes the fields required by the protocol checks, including protocol hash, source/script identity, assumptions, deviations, and artifact paths where applicable.
4. **Regime adequacy** — budgets and controlling knobs are sufficient for the protocol's declared checks, not merely for producing a plausible number.

Severity tags: `match`, `proxy` (same reported claim through a different path), `unrecorded-deviation`, `provenance-gap`, `regime-gap` (right method, underbudgeted).

### `result`

For each numerical claim (energy, gap, density, exponent, scaling collapse):

- Identify the paper's reported value or range from the source figure / table / text.
- Compare to the script's output. ✓ within paper's error bar, ⚠ outside paper but within convergence margin, ✗ disagrees beyond budget.
- Surface the verification table the paper itself reports (if any) and confirm reproduction.

### `close`

Compare the final run report, consolidated script, protocol TOML, verification reports, and manifest set. This is the global feedback loop: no new domain truth is inferred here; the audit checks that final claims are bounded by already-reviewed evidence.

Axes:

1. **Claim boundedness** — every claim in the run report maps to protocol claims, result artifacts, and verification reports; no unsupported paper-reproduction language appears.
2. **Artifact provenance** — manifests and final artifacts carry protocol hash, script/source identity, assumptions, deviations, and artifact paths required by the protocol.
3. **Gate closure** — preflight, compute, assembly, and report checks are all passed, or the report explicitly marks the run partial/assumptive.
4. **Deviation visibility** — method/backend/estimator/sampling/boundary/budget/error-method differences are visible in the report and linked to protocol deviations.
5. **Reproducibility** — consolidated script and run command are sufficient for a fresh checkout against the declared harness environment.

Severity tags: `supported`, `unsupported-claim`, `provenance-gap`, `open-gate`, `hidden-deviation`, `repro-gap`.

## Output

Markdown report at `results/<run>/verify_<artifact-stem>_<date>.md`. The report stops at the per-axis status table — the audit's deliverable is the diff, not the prescription. Suggested layout:

```markdown
# /verify report — <artifact> — <date>

**Mode**: protocol | kb-card | script | result | close.
**Reference**: <path / lines>.

| Axis / Row | Status | Severity | Notes |
|---|---|---|---|
| ... | ✓ / ⚠ / ✗ | match / proxy / unrecorded-deviation / regime-gap | ... |

## Detailed findings

### Axis N — <name>
... verbatim passage with line number, verbatim script snippet with line number, conclusion ...
```

**No "Action items" section.** Translating findings into next-step options is the *calling skill's* job, not the subagent's (see Composition below).

## Discipline (hard rules)

- Read-only. Never modify the artifact from inside this skill.
- Independent. The artifact author cannot satisfy this review with their own reasoning or smoke tests.
- Conservative on ambiguity: prefer ✗ unsupported over rationalized near-matches.
- For Analytic and Harness anchors, the tag itself is sufficient provenance — do not double-grep against literature.
- Generic over artifact type and reference. No artifact-specific or paper-specific logic.
- In `protocol` mode, do not invent domain-specific validator types. Domain-specific checks must be `kind = "command"` or an explicitly delegated `/verify` audit.
- Subagent dispatched with `model: "opus"` and explicit max-effort framing. No downgrade for speed.

## Composition

- Called by `/reproduce-paper` for the protocol before compute, for each (figure, script, result) triple, and for the final close before claiming reproduction.
- Called by KB-card or script authors as a pre-commit gate.
- The verifier surfaces what is wrong; it does not decide what to do.
- **After the report lands, the calling skill (or the main agent) translates the findings into 2-3 Superpowers-style options** (Recommended first, each with one-line pros / cons) and presents them via `AskUserQuestion`. The user ratifies; only then does cleanup happen. Splitting audit (subagent) from prescription (main-agent fork) preserves the user's steering wheel.

## Anti-patterns (auto-reject)

- Modifying the artifact from inside this skill.
- Inventing a tag for an untagged numerical entry.
- Treating KB-sourced protocol claims as primary-source support.
- Adding paper/domain-specific validator types to the generic protocol layer.
- Closing a final run report without `close`-mode review or an equivalent independent reviewer.
- "Verified" without per-axis status table.
- Downgrading subagent model or effort.
- Single-line "looks fine" report — the structured table is the deliverable.
- Subagent prescribing fixes / writing an `## Action items` section — that is the calling skill's job.
