---
name: hubbard
description: Use when the user is working on a Hubbard model problem, including half-filled, doped, extended, or next-neighbor hopping variants; Mott physics; correlated electrons; ground-state estimates; or benchmark workflows.
---

# Hubbard

This skill solves Hubbard-model problems as correlated-electron tasks. Doping, extended interactions, `t'`, lattice, and DMFT are workflow choices inside this problem, not separate skill names.

## Problem Form

Base convention:

```text
H = -t sum_<ij>,sigma (c_i,sigma^dag c_j,sigma + h.c.)
  + U sum_i n_i,up n_i,down
```

Ask for lattice, filling or `N_up/N_down`, `U/t`, hopping range (`t'`, `t2`), boundary condition, whether `V1/V2` interactions exist, and the target observable.

## Steering Defaults

- Entry: small ED or DMRG baseline; compute energy, density, double occupancy, spin/charge correlations, and simple gap proxies.
- Intermediate: diagnose half-filled versus doped, 1D versus 2D, sign-problem conditions, finite-size strategy, cross-method checks, and variance/error metrics.

If the user is in a large-`U` doped regime where no-double-occupancy physics is central, offer `t-j` as a faithful handoff, not as a forced replacement.

## Method Guidance

- **ED:** exact references for small clusters, particle-number sectors, double occupancy, and debugging fermion signs.
- **DMRG/MPS:** strong for 1D chains and cylinders; useful for doped systems but sensitive to width and boundary choices.
- **AFQMC/QMC:** powerful when sign-problem-free, especially half-filled bipartite repulsive cases; constrained-path variants are approximate and need bias discussion.
- **DMFT:** appropriate for local self-energy, Mott transition, and impurity-mapped questions; not a top-level problem name.
- **VMC/NQS/tensor networks:** relevant for frustrated/doped 2D regimes and variational benchmarks.

## Software and Setup

For Python ED/TN sketches, prefer `quimb`, `cotengra`, `numpy`, `scipy`, and `matplotlib`; suggest `make install quimb` if missing. If the user asks for TeNPy, Julia, or a specialized QMC/DMFT package, first check that the tool is installable in `Makefile`; otherwise explain the dependency and ask whether to add an install target.

## Outputs and Checks

Return Hamiltonian construction, sector definition, method recommendation, code sketch, diagnostic table, or interpretation. Include checks for particle number, spin balance, fermionic signs, `U = 0` and large-`U` limits, double occupancy trends, and whether the selected method can resolve the requested physics.

## Related Skills

Call `mott-transition` for localization, metal-insulator behavior, double occupancy, local moments, or DMFT-style reasoning. Use `knowledge-base/2302.04919-variational-benchmarks.md` for benchmark definitions and V-score context.
