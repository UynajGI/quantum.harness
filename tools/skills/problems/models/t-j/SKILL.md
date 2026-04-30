---
name: t-j
description: Use when the user is working on a t-J model problem, including doped Mott systems, projected Hilbert spaces with no double occupancy, spin-charge interplay, cuprate-inspired models, or strong-coupling Hubbard reductions.
---

# t-J

This skill solves `t-J` problems as projected spinful-fermion problems. It is related to large-`U` Hubbard, but it deserves separate treatment because the no-double-occupancy constraint changes the Hilbert space and diagnostics.

## Problem Form

Use the convention

```text
H = -t sum_<ij>,sigma P (c_i,sigma^dag c_j,sigma + h.c.) P
  + J sum_<ij> (S_i . S_j - 1/4 n_i n_j)
```

Ask for lattice, hole/electron count, `J/t`, boundary condition, added terms, and whether the goal is comparison to Hubbard or direct projected-model physics.

## Steering Defaults

- Entry: projected finite-system setup, ED or DMRG baseline, density and spin correlations.
- Intermediate: projection-aware basis, competing spin/charge/pairing diagnostics, finite-size effects, and comparison to large-`U` Hubbard when relevant.

## Method Guidance

- **Projected ED:** best for tiny clusters and checking the constraint exactly.
- **DMRG/MPS:** natural for chains and cylinders; good for stripes, hole motion, spin correlations, and pairing tendencies.
- **Projected VMC:** important for 2D variational states and cuprate-inspired questions.
- **Tensor networks/NQS:** useful when the user wants expressive 2D ansatz comparisons.

## Software and Setup

For Python sketches, use `numpy`, `scipy`, `quimb`, and `matplotlib` where useful; suggest `make install quimb` if missing. Be explicit about the projected local basis: empty, spin-up, spin-down; no double occupancy.

## Outputs and Checks

Return projected Hamiltonian construction, basis definition, method plan, observable list, or interpretation. Include checks for no-double-occupancy enforcement, particle/hole count, spin symmetry if relevant, relation `J ~= 4 t^2 / U` only when deriving from Hubbard, and sensitivity to cylinder geometry.

## Related Skills

Call `mott-transition` when connecting to large-`U` Hubbard. Call `frustration` or `spin-liquid` when lattice and doping make those diagnostics relevant.
