---
name: t-v
description: Use when the user is working on a spinless fermion t-V model problem, including hopping plus nearest-neighbor repulsion, charge ordering, fermionic benchmarks, exact diagonalization, or variational ground-state estimates.
---

# t-V

This skill solves spinless fermion `t-V` problems. Keep it separate from `t-j`: `t-V` is density interaction without spin exchange; `t-j` is projected spinful fermions with no double occupancy.

## Problem Form

Use the convention

```text
H = -t sum_<ij> (c_i^dag c_j + h.c.) + V sum_<ij> n_i n_j
```

Ask for lattice, particle number or filling, `V/t`, boundary condition, hopping sign convention if relevant, and target observable.

## Steering Defaults

- Entry: finite spinless-fermion ED baseline and density-density correlations.
- Intermediate: charge structure factor, finite-size checks, sign/geometry issues, and comparison to variational or tensor-network estimates.

## Method Guidance

- **ED:** good for small fixed-particle-number sectors and exact charge correlations.
- **DMRG/MPS:** natural for 1D chains and cylinders; use for ground state, entanglement, and density correlations.
- **QMC:** possible in selected sign-problem-free settings; do not assume it is available for arbitrary geometry.
- **VMC/NQS/tensor networks:** useful for hard geometries, variational benchmarks, or comparisons with the V-score paper.

## Software and Setup

For Python work, use `quimb`, `cotengra`, `numpy`, `scipy`, and `matplotlib`; suggest `make install quimb` if missing. For custom fermion bases, keep fixed-particle-number indexing explicit and validate anticommutation signs.

## Outputs and Checks

Return Hamiltonian construction, fixed-sector basis setup, observable plan, benchmark table, or interpretation. Include checks for particle-number conservation, fermionic signs, energy normalization, density correlations, and known limits such as `V = 0`.

## Related Skills

Call `frustration` if geometry, boundary conditions, or hopping structure create competing kinetic/interacting tendencies. Use `knowledge-base/2302.04919-variational-benchmarks.md` for benchmark context.
