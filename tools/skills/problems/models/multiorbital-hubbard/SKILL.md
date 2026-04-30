---
name: multiorbital-hubbard
description: Use when the user is working on a multiorbital Hubbard or Kanamori-interaction problem, including Hund physics, three-band impurity models, orbital degrees of freedom, spin-flip/pair-hopping terms, or material-inspired correlated-electron benchmarks.
---

# Multiorbital Hubbard

This skill solves multiorbital Hubbard and Kanamori-interaction problems. It covers Hund-regime problems without naming the skill after the method or a single coupling.

## Problem Form

Ask for number of orbitals, filling, local one-body terms, interaction convention, `U`, `J`, spin-orbit assumptions, lattice or impurity context, and target observable. For Kanamori interactions, distinguish density-density, spin-flip, and pair-hopping terms.

## Steering Defaults

- Entry: local interaction bookkeeping, small finite-bath or cluster setup, orbital occupancy, spin, and local moment diagnostics.
- Intermediate: full Kanamori terms, symmetry-sector organization, orbital selectivity, impurity/lattice solver feasibility, and benchmark checks.

## Method Guidance

- **Local/cluster ED:** best for validating multiplets, interaction terms, and symmetry sectors.
- **Impurity solver routes:** appropriate for material-inspired or DMFT-derived finite-bath problems.
- **DMFT reasoning:** useful for orbital-selective Mott and Hund-metal questions, but keep DMFT as a method.
- **Tensor-network/variational routes:** plausible only when geometry and Hilbert-space size are controlled; surface cost clearly.

## Software and Setup

For small ED bookkeeping, use `numpy` and `scipy`; for tensor-network sketches, use `quimb` through `make install quimb` if appropriate. Multiorbital problems grow quickly, so always state local Hilbert-space size before recommending a calculation.

## Outputs and Checks

Return interaction-term decomposition, parameter table, basis-sector plan, solver recommendation, or interpretation. Include checks for rotational-invariance assumptions, sign conventions for Hund terms, orbital occupancy, spin-sector consistency, and whether spin-flip/pair-hopping terms are included or intentionally omitted.

## Related Skills

Call `mott-transition` for localization or orbital-selective behavior. Call `kondo-effect` when impurity screening or local moments are central. Use `knowledge-base/2302.04919-variational-benchmarks.md` for the paper's three-band Kanamori impurity context.
