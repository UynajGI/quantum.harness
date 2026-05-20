---
name: parameter-scan
description: Use when the user wants to sweep one or more declared axes for a produced quantity. Generic over axes, payload schema, quantity, and implementation. For cluster execution composes with `/slurm`.
---

# parameter-scan

Sweep a Cartesian product of declared axes, holding other payload fields fixed. Produces a structured grid-indexed result, an auto-generated plot, shape labels, and a brief report.

This primitive subsumes any single-axis or multi-axis scan whose cells can be represented as opaque `params` plus optional `settings`.

## Audience / scope (binding)

<audience>
The caller is a workflow skill (`solve`, a model skill, `/reproduce-paper`, or a physics card) that needs a parameter sweep executed and assembled. The output (table + plot + shape labels) is consumed by that calling workflow; the user reads only the 2–3-line report.

This primitive MUST stay generic over: axis names, payload schema, the produced quantity, and the per-cell implementation. It does NOT know what an axis means physically, does NOT decide ranges, and does NOT interpret shapes.
</audience>

## When to activate

- User asks "how does this depend on `[parameter]`?" — single-axis scan.
- User asks for a two-axis or higher-dimensional grid.
- A calling workflow needs a sweep to characterize a response, locate a feature, or support an extrapolation.
- After any single-point calculation, this is the canonical follow-up offered through `solve`.

## Inputs

- *Calling workflow* and *anchor configuration* — fixed payload fields not on any axis.
- *Axes* — list of named axes with value lists. Axis names and values are opaque to this primitive; the calling workflow or entrypoint resolves them.
- *Quantity* — output field or artifact selected by the caller.
- *Default value lists* — provided by the caller or user. This primitive does not infer domain-specific ranges.
- *Optional* — per-cell settings overrides for any correctness-affecting or budget-affecting payload.

## Workflow

1. **Plan**: enumerate cells (Cartesian product of axes). Plan output: `results/<run>/parameter-scan.plan.json` listing cell ids and their parameter assignments.
2. **Resume detection**: walk **every** subdirectory of `results/<run>/cells/` and identify completed cells via their manifests. Build the *to-run* set.
3. **Execute**:
   - **Local** (laptop, no cluster profile active): loop over cells, dispatching the caller's calculation per cell.
   - **Cluster** (`tools/cluster/active.md` symlink or `HARNESS_CLUSTER_PROFILE` env var present): compose with `/slurm`, handing it the per-cell script and cell map. The slurm skill submits an sbatch array job, monitors, fetches.
4. **Auto-bump controlling settings**: for **every** cell that reports non-convergence AND for which the caller has declared a retry policy, apply the declared bump and re-run that cell. Cells without a declared retry policy are left as-is and surfaced in the report.
5. **Collect**: walk `results/<run>/cells/*/manifest.json` and assemble **every** cell manifest into `results/<run>/parameter-scan.csv` with axis values, selected quantity, uncertainty, controlling settings, runtime, and status. Cover **every** cell directory; do not drop or merge cells silently. Cells without manifests or with failed manifests appear in the assembled CSV with their `status` field set accordingly.
6. **Auto-plot**: emit one or more plots via `scientific-visualization` based on axis arity:
   - 1D: `observable(parameter)` line plot with error bars.
   - 2D: family of `observable(axis_1)` curves indexed by `axis_2` (or vice versa); plus an optional `observable(axis_1, axis_2)` heatmap when both axes are continuous.
7. **Shape detection** (auto labels — see below).
8. **Hand back**: table + plot + shape label list to the calling skill (or `solve`).

## Cell Spec Contract

For cluster or resumable execution, `/parameter-scan` writes `results/<run>/run_spec.json` with opaque cells. The only reusable contract is `cell_id`, `params`, optional per-cell `settings`, shared `settings`, and shared `provenance`:

```json
{
  "run_id": "example-run",
  "run_dir": "results/example-run",
  "settings": {"control_knob": 30, "budget": 1000000},
  "provenance": {"protocol_hash": "...", "sources": ["..."], "claims": ["..."]},
  "cells": [
    {"cell_id": "cell-0001", "params": {"axis_1": 16, "axis_2": 0.8}}
  ]
}
```

The primitive treats `params`, `settings`, and `provenance` as data, not schema. It never knows what a key physically or methodologically means. The entrypoint maps those opaque values into its own code and writes `results/<run>/cells/<cell_id>/manifest.json`. Domain-shaped names are allowed in a specific run spec when the entrypoint expects them, but they are payload keys, not harness-level types.

Setup that affects correctness or uncertainty also belongs in `settings` as opaque payload. The entrypoint must echo the payload it actually used into each manifest and emit machine-readable evidence for any declared constraint it verifies. `/parameter-scan` only checks presence/freshness and assembles cells; it does not hardcode setup types.

If the caller needs stricter assemble gates, `run_spec.json` may carry `assemble.manifest_contract`, `assemble.consensus_fields`, and `assemble.provenance_fields`. The contract is generic over manifest field paths: required/nonempty fields, equality checks, list membership, numeric fields, optional numeric fields, numeric bounds, and evidence-set bindings. Domain requirements are encoded as payload values in the protocol/run spec and echoed by manifests, not as new `/parameter-scan` types.

<invariants name="assemble">
- Settings are constant only after manifest consensus. The first completed manifest's settings are not global state; every manifest is validated against the merged shared+cell payload from `run_spec.json`.
- Provenance is validated per manifest. Each manifest's declared provenance is checked against the run-spec provenance; mismatches surface, not silently merged.
- Constants vs varying is reported, not assumed. Settings are reported as constant only after they've been checked across all cells; otherwise reported as varying with the per-cell variation surfaced.
- Failed cells stay surfaced. Each cell tagged failed (or missing manifest) appears in the assembled CSV with its `status` field, never silently dropped or merged with successful cells.
</invariants>

Assembler obligations:

<checklist name="assemble">

- The assembler validates **every** manifest's `settings` against the merged shared+cell payload from `run_spec.json`.
- The assembler validates **every** manifest's declared provenance against the run-spec provenance.
- The assembler reports settings as constant vs varying across **all** cells.
- The assembler NEVER treats the first manifest's settings or provenance as global unless `assemble.consensus_fields` declares and passes that invariant.

</checklist>

The protocol should pair this with `cover` plus `producer = "run"` so the observed cell manifests exactly match the declared run-spec cells; `trial` artifacts belong outside that covered path unless explicitly declared.

## Shape detection (auto labels)

The skill labels only the shape of the data. Each entry below splits into a **detection criterion** (binary) and a **recommended downstream label/handoff**.

<checklist name="shapes">

- **Monotone**
  - Detection: observable goes one direction across an axis range.
  - Handoff: surface the direction; no follow-up named.
- **Asymptoting**
  - Detection: successive differences along a declared axis shrink monotonically.
  - Handoff: surface an extrapolation-check follow-up.
- **Power-law-like trend**
  - Detection: log-log slope is approximately stable across a declared axis.
  - Handoff: pass to `/scaling-fit` if the calling skill needs exponents.
- **Drifting / oscillating**
  - Detection: neither asymptoting nor a clean power-law-like trend.
  - Handoff: surface the unresolved shape and the controlling settings.
- **Extremum**
  - Detection: peak or valley at an interior point on an axis.
  - Handoff: surface the location; pass to `/scaling-fit` if exponents on the peak are wanted.
- **Crossing**
  - Detection: curves indexed by one axis cross at one value of another axis.
  - Handoff: surface the crossing point; pass to `/scaling-fit` if the crossing pins a critical parameter.
- **Step-like**
  - Detection: a sharp jump relative to neighboring points.
  - Handoff: flag for follow-up.

</checklist>

Labels are descriptive, not interpretive. The calling workflow decides what the shape means.

## Output

- `results/<run>/parameter-scan.plan.json` — the plan (cell ids + parameter assignments).
- `results/<run>/cells/<cell_id>/manifest.json` — per-cell manifest (one per cell).
- `results/<run>/parameter-scan.csv` — assembled grid table.
- `results/<run>/parameter-scan.png` — auto-generated plot(s).
- A 2-3-line report: shape label(s), recommended next step.
- `scripts/<run>/parameter-scan.jl` — reproducible script.

## Resume semantics

- Re-running on a partial run reuses cells with `success`-tagged manifests; re-submits cells without manifests.
- **Every** cell whose manifest tags the run as `failed` (or whose script exited non-zero with no manifest) is left in place and surfaced in the report — never auto-retried. The user ratifies the retry per-cell or in bulk. (Avoids wasting compute on logic errors.)
- The plan file is immutable per run id; if axes or values change, the user starts a new run id.

## Composition

- For embarrassingly-parallel cluster sweeps, this skill composes with `/slurm` (which submits the sbatch array job + monitors + fetches). The user does not call `/slurm` directly for grids; `/parameter-scan` handles the cell decomposition and delegates.
- Declared scaling grids can feed `/scaling-fit` for collapse or exponent extraction.
- Common follow-ups (offered via `AskUserQuestion`):
  - `/scaling-fit` (Recommended for power-law-like / extremum / crossing labels when the calling skill needs scaling).
  - `/cross-method-check` — independent confirmation at one cell.
  - Done.

## Notes

- The user or calling workflow picks the axes; the primitive does not care what they mean.
- This skill does *not* know the meaning of any axis. It is generic over parameter names and values.
- Retry and convergence logic delegate to the calling workflow and declared check commands.
- For literature-dependent interpretation, the calling workflow should invoke its source-search or source-audit mechanism before interpreting; this primitive only produces data.
- **Every** cell's compute script (declared by the calling workflow) writes its own manifest at `results/<run>/cells/<cell_id>/manifest.json`. This skill does NOT invent a parallel format and does NOT write manifests itself. Per-cell manifests are consumed by `/reproduce-paper` close at session close.

## Anti-patterns (auto-reject)

- Hardcoding domain-specific axis ranges or default values.

<example name="hardcode bad">
bond_dim_axis = [16, 32, 64, 128]  # baked into the skill
</example>

<example name="hardcode good">
# The caller passes the axis range in; this skill never names a bond dim.
axes = caller.axes  # e.g. {"bond_dim": [16, 32, 64, 128]}
</example>

- Bundling sbatch / ssh / rsync logic into this skill — that's `/slurm`'s job.
- Inventing a new manifest format — the per-cell manifest convention is shared with `/reproduce-paper`.
- Silent skip of failed cells — every failed cell must be classified and surfaced.
