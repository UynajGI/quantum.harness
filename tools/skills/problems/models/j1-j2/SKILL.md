---
name: j1-j2
description: Use when the user is working on a J1-J2 spin model problem, especially frustrated square-lattice or other next-nearest-neighbor Heisenberg variants, competing magnetic orders, spin-liquid candidates, or variational benchmarks.
---

# J1-J2

This skill solves `J1-J2` spin problems, where competing nearest- and next-nearest-neighbor interactions make method choice and diagnostics regime-dependent.

## Problem Form

Use the convention

```text
H = J1 sum_<ij>,a sigma_i^a sigma_j^a
  + J2 sum_<<ij>>,a sigma_i^a sigma_j^a
```

Confirm lattice, `J2/J1`, signs, spin size, boundary condition, cluster/cylinder shape, and target observable.

## Steering Defaults

- Entry: small ED or DMRG baseline; compute energy, correlations, and candidate order indicators.
- Intermediate: frustration-aware method choice, competing order diagnostics, finite-size/cylinder-shape sensitivity, and benchmark/variance checks.

Never treat `J1-J2` as merely "Heisenberg with another parameter" when `J2/J1` places it near a competing-order or spin-liquid regime.

## Method Guidance

- **ED:** best for small clusters, spectra, level crossings, and checking candidate order tendencies.
- **DMRG:** strong for cylinders and quasi-1D cuts; discuss geometry bias and finite-width extrapolation.
- **VMC/NQS:** appropriate for competing variational states and spin-liquid candidate comparisons.
- **PEPS/tensor networks:** useful for 2D states when the user wants a tensor-network route; make bond dimension and extrapolation explicit.
- **QMC:** generally not the default in frustrated regimes; only suggest when the specific formulation is sign-problem-free.

## Software and Setup

For Python sketches, use `quimb`, `cotengra`, `numpy`, `scipy`, and `matplotlib`; suggest `make install quimb` if needed. Use small ED code to verify sign conventions before larger tensor-network or variational runs.

## Outputs and Checks

Return a parameterized Hamiltonian, method plan, diagnostic table, code skeleton, or interpretation. Include checks for energy normalization, boundary/shape effects, candidate order structure factors, and whether the proposed method can distinguish competing phases at the requested size.

## Related Skills

Call `frustration` for regime classification and `spin-liquid` when the task concerns nonmagnetic phases or spin-liquid diagnostics. Use `knowledge-base/2302.04919-variational-benchmarks.md` for benchmark difficulty signals.
