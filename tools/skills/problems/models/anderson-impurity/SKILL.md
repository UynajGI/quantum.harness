---
name: anderson-impurity
description: Use when the user is working on an Anderson impurity model problem, including a local interacting impurity coupled to a bath, impurity benchmarks, hybridization functions, bath discretization, or Kondo/local-moment physics.
---

# Anderson Impurity

This skill solves Anderson impurity problems by making the local interacting sector, bath representation, and hybridization explicit.

## Problem Form

Separate the Hamiltonian into local and bath pieces:

```text
H_A = H_loc + H_bath
```

Ask for impurity orbitals/spins, local interaction, bath size or hybridization function, filling/chemical potential, symmetries, and target observable.

## Steering Defaults

- Entry: finite-bath Hamiltonian, ED baseline, occupancy, double occupancy, and local moment.
- Intermediate: bath discretization quality, hybridization checks, symmetry sectors, screening diagnostics, and comparison across solver choices.

## Method Guidance

- **Finite-bath ED:** best first route for small impurity benchmarks and clear Hamiltonian bookkeeping.
- **MPS impurity solvers:** useful for longer bath chains after star-to-chain mapping.
- **NRG-style reasoning:** appropriate for low-energy Kondo screening and scale separation, even if not implemented directly.
- **DMFT embedding:** relevant when the impurity model comes from a lattice self-consistency loop; keep the user-facing problem as impurity or Hubbard depending on intent.

## Software and Setup

For compact ED sketches, use `numpy` and `scipy`; for tensor-network bath chains, use `quimb` if available through `make install quimb`. Do not invent a DMFT solver dependency; if the user wants one, discuss the package and add an install target only after agreement.

## Outputs and Checks

Return local/bath Hamiltonian blocks, bath parameter interpretation, solver plan, observable checklist, or benchmark table. Include checks for bath discretization, impurity occupancy, local moment, particle-number or spin-sector consistency, and whether the finite bath can answer the requested question.

## Related Skills

Call `kondo-effect` for screening, local moments, Kondo scale, or impurity spin compensation. Use `knowledge-base/2302.04919-variational-benchmarks.md` for benchmark impurity definitions.
