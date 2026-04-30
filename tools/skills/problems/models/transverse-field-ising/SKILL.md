---
name: transverse-field-ising
description: Use when the user is working on a transverse-field Ising model problem, including ground-state estimates, quantum criticality, finite-size scaling, benchmark setup, or method comparison for Ising lattice Hamiltonians.
---

# Transverse-Field Ising

This skill solves transverse-field Ising model problems. It is not a lesson plan. The agent should diagnose the concrete task, recommend a route, and produce a usable calculation plan, code sketch, result interpretation, or benchmark setup.

## Problem Form

Use the convention

```text
H = J sum_<ij> sigma_i^z sigma_j^z + Gamma sum_i sigma_i^x
```

Confirm sign conventions if the user cares about ferro/antiferromagnetic language. Ask only for missing problem-defining inputs: lattice/dimension, system size, boundary condition, `J`, `Gamma`, target observable, accuracy goal, and compute budget.

## Steering Defaults

Lead with the route that can answer the user's problem with the least fragile machinery:

- Entry: exact diagonalization for small systems, or DMRG/MPS for 1D chains.
- Intermediate: finite-size scaling, gap/order-parameter trends, variance or energy-error checks, and a second method when the conclusion depends on scaling.

If the task has meaningful branches, offer 2-3 real options. Do not offer methods you are not prepared to execute or scaffold.

## Method Guidance

- **ED:** best for tiny clusters, debugging sign conventions, spectra, gaps, and reference values.
- **DMRG/MPS:** natural for 1D chains and quasi-1D strips; use for ground states, gaps, magnetization, and correlations.
- **QMC:** suitable when the formulation is sign-problem-free; good for critical behavior and larger unfrustrated systems.
- **Variational/PQC/NQS:** useful when the user is benchmarking ansatz quality or comparing with the V-score paper.
- **Fuzzy sphere / conformal routes:** only when the user explicitly asks about field-theory-facing criticality; treat as a criticality method, not the model identity.

## Software and Setup

For Python-based ED, DMRG, and tensor-network sketches, prefer `quimb`, `cotengra`, `numpy`, `scipy`, and `matplotlib`. If these are missing, suggest `make install quimb` after checking `make help`. For notebook workflows, use the existing `jupyter-notebook` skill. For Julia-specific requests, use the `julia` skill, but do not invent an install target unless it exists.

## Outputs and Checks

Return one of: Hamiltonian construction, runnable small-system code, finite-size scaling plan, observable table, benchmark comparison, or interpretation. Always include sanity checks: limiting cases (`Gamma = 0`, large `Gamma`), symmetry expectations, system-size trend, and whether the chosen method can resolve the requested observable.

## Related Skills

Call `criticality` for quantum phase transitions, gaps, scaling, exponents, universality, or fuzzy-sphere-style questions. Use `knowledge-base/2302.04919-variational-benchmarks.md` for V-score and benchmark context.
