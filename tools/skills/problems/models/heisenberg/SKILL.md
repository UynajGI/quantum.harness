---
name: heisenberg
description: Use when the user is working on a Heisenberg spin model problem, including spin chains, square/triangular/kagome/pyrochlore lattices, magnetic phases, frustrated magnets, spin liquids, ground states, or benchmark calculations.
---

# Heisenberg

This skill solves Heisenberg spin-model problems. The lattice and coupling regime often determine the physics, so diagnose them before recommending a method.

## Problem Form

Base convention:

```text
H = J sum_<ij>,a sigma_i^a sigma_j^a
```

Ask for spin size, lattice, coupling signs, anisotropy if any, boundary condition, system size, and target observable. If the user says only "Heisenberg", infer a simple baseline from context but state the assumption.

## Steering Defaults

- Entry: small ED for clusters or DMRG/MPS for chains; compute energy, magnetization, two-point correlations, and simple structure factors.
- Intermediate: choose method by geometry; add finite-size/cylinder checks, structure factor, order diagnostics, and variational accuracy estimates.

Do not collapse all lattices into one difficulty class. Chain, square, triangular, kagome, and pyrochlore Heisenberg problems require different expectations.

## Method Guidance

- **ED:** small clusters, exact spectra, spin quantum numbers, and debugging Hamiltonian conventions.
- **DMRG/MPS:** strong default for chains and cylinders; good for energies, correlations, gaps, and quasi-1D frustrated studies.
- **QMC:** excellent for unfrustrated sign-problem-free cases; avoid promising it for frustrated geometries without checking sign structure.
- **VMC/NQS/tensor networks:** useful for frustrated 2D lattices and spin-liquid candidates where QMC is blocked and DMRG geometry is biased.
- **PEPS/iPEPS:** plausible for 2D thermodynamic-state estimates, but setup is heavier; present as intermediate unless the user already wants it.

## Software and Setup

For entry/intermediate Python work, prefer `quimb`, `cotengra`, `numpy`, `scipy`, and `matplotlib`; suggest `make install quimb` if missing. Use `quimb` for spin Hamiltonians, ED, MPS/DMRG sketches, and tensor-network examples. For Julia/ITensor-style requests, use the `julia` skill and verify available install targets before suggesting commands.

## Outputs and Checks

Produce a Hamiltonian definition, method recommendation, code skeleton, diagnostic checklist, or result interpretation. Include checks against known limits: chain versus 2D expectations, total spin/symmetry if relevant, energy per site conventions, boundary effects, and whether frustration invalidates simple QMC intuition.

## Related Skills

Call `frustration` for triangular, kagome, pyrochlore, competing couplings, or hard variational regimes. Call `spin-liquid` for absence of magnetic order, long-range entanglement, or topological diagnostics. Use `knowledge-base/2302.04919-variational-benchmarks.md` for benchmark and V-score context.
