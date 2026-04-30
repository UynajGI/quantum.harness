# Quantum Many-Body Physics Harness

Agent-driven problem-solving harness for quantum many-body physics. You bring a concrete problem — the agent sets up the calculation, runs it, verifies the result, and surfaces decisions only when they matter.

## Quick Start

```bash
make setup && make domain-setup    # Ion + skills + Julia/ITensors stack
make install netket                # optional: VMC / neural quantum states
make install quimb                 # optional: Python TN alternative
```

Then describe your problem. Examples:

- "Ground state of 1D Heisenberg, N=20"
- "Is kagome Heisenberg a spin liquid?"
- "Hubbard square lattice, U/t=8, half-filled — Mott phase?"

The harness diagnoses the problem, picks a method, runs, verifies, and reports with a convergence plot.

Say "where do I start" if you don't have a problem yet.

## What it covers

**Models** — transverse-field Ising, Heisenberg, J1-J2, t-V, Hubbard, t-J, Anderson impurity, multiorbital Hubbard.

**Physics questions** — criticality, frustration, spin liquids, Mott transitions, Kondo effect.

**Methods** — DMRG, ED, TEBD, VMC/NQS (NetKet). Stubs for spectral functions and finite-T (literature pointers + off-skill paths).

**Scope** — ground-state lattice problems, entry to medium level. Contested cases (kagome, J1-J2 ≈ 0.5, 2D doped Hubbard) are flagged with literature context rather than overclaimed. Dynamics, finite-T, and open systems are future directions.

## How it works

1. **You describe a problem.** The agent infers the setup from your prompt — no questionnaire.
2. **The agent acts.** Picks a method, runs the calculation, auto-generates a convergence plot. Scripts saved to `scripts/`, results to `results/`.
3. **You see a short report.** ≤3 lines: result, method reasoning, verification status.
4. **You pick what's next.** 2–3 concrete options — visualization, parameter scan, deeper analysis, writeup, or stop. Recommended option first.
5. **Repeat** until you're done.

For frontier or contested problems, the agent leads with a literature summary before offering computation — because computation alone can't close those questions.

## Repository layout

```
tools/skills/
  solve/             Interactive problem-solving loop
  onboard/           First-touch intake + domain setup
  problems/models/   8 Hamiltonian-family skills
  problems/physics/  5 diagnostic physics skills

knowledge-base/
  conventions.md     Sign and normalization defaults
  limits.md          Exact reductions and known limits
  benchmark-numbers.md  Reference values with citations
  symmetry-cheatsheet.md  Conserved quantities per lattice
  methods/           Per-method code shape, knobs, pitfalls
                     (dmrg, ed, tebd, vmc-nqs, anderson-impurity-ed,
                      spectral stub, finite-t stub)

Makefile             Setup + on-demand installs
AGENTS.md            Agent-facing design contract
Ion.toml             Skill manifest
```

## Design

**Skills** hold problem-level workflow — diagnose, act, verify, branch. They never hardcode reference numbers or method-specific code.

**Knowledge base** holds data and method reference — benchmark values, conventions, limits, per-method code shapes. Updating KB never breaks a skill.

Two skill shapes:

- **Model skills** drive calculations: infer setup → run → verify → report → next-steps.
- **Physics skills** evaluate evidence: frame the question → gather indicators → cross-check → interpret.

## For agents

See `AGENTS.md` for the full design contract. Key norms:

- Act first on clear defaults. Offer alternatives after the result, not before.
- Report ≤3 lines + plot. Details on request.
- Use `AskUserQuestion` for all choices — 2–3 options, recommended first, one-line tradeoff each.
- Label off-skill work honestly.
- For frontier regimes, lead with literature via `arxiv-search`.
