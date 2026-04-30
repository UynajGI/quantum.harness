# Quantum Many-Body Physics Harness

A research harness that works through quantum many-body problems with you. You bring the physics question — the harness handles method selection, conventions, verification, and visualization, then suggests where to go next. You guide the direction; it handles the execution.

## What a session looks like

```
You:    Ground state of 1D Heisenberg, N=20
Harness: E/N = -0.4341 via DMRG (D=200, converged). ED cross-check ✓.
         → [Finite-size extrapolation (Rec.)]  [Correlations plot]  [Done]

You:    [Finite-size extrapolation]
Harness: N=20→80. E/N → -0.44315, matches Bethe ansatz ✓.
         → [Publication figure (Rec.)]  [Try a different model]  [Done]

You:    [Try a different model]
Harness: What problem?

You:    Same but kagome lattice — spin liquid?
Harness: Strong SL candidate. Magnetic order ruled out (consensus).
         Gapped Z₂ vs gapless U(1) debated. Yan-Huse-White 2011.
         → [Run cylinder DMRG (Rec.)]  [Recent arXiv papers]  [Done]
```

Each turn: a result, then concrete options for what to do next. The harness picks methods, checks conventions, runs verification, and generates plots internally — you see outcomes and make decisions.

## What it knows

**Models** — Heisenberg, transverse-field Ising, J1-J2, Hubbard, t-J, t-V, Anderson impurity, multiorbital Hubbard.

**Methods** — DMRG, ED, TEBD, VMC/NQS (NetKet). Literature pointers for spectral functions and finite-T.

**Physics** — criticality, frustration, spin liquids, Mott transitions, Kondo effect.

Coverage: ground-state lattice problems, entry to medium level. Contested cases are flagged with literature context. Dynamics and finite-T are future directions.

## Setup

```bash
make setup && make domain-setup
```

Then start a Claude Code session and describe your problem — or say "where do I start."

## For contributors

Design contract: `AGENTS.md`. Specs and test reports: `docs/`.
