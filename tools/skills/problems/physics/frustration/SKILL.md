---
name: frustration
description: Use when the user is asking about geometric or interaction frustration, competing interactions, sign-problem-prone regimes, triangular/kagome/pyrochlore lattices, J1-J2 competition, or why a quantum many-body problem is hard.
---

# Frustration

This skill diagnoses frustration as a source of competing low-energy states, sign problems, and method difficulty.

## Inputs to Fix

Identify model, lattice, competing couplings, signs, boundary conditions, filling/doping if fermionic, and the user's target: classification, method choice, order competition, spin-liquid possibility, or benchmark difficulty.

## Steering Defaults

- Entry: explain the obstruction in the specific model and choose basic observables.
- Intermediate: compare candidate orders, finite-size and geometry sensitivity, method limitations, and benchmark hardness.

## Method Guidance

- **ED:** first reference for tiny frustrated clusters and candidate-order competition.
- **DMRG cylinders:** useful but geometry-biased; discuss width, wrapping, and boundary effects.
- **VMC/NQS/tensor networks:** natural for frustrated 2D regimes and spin-liquid candidates.
- **QMC:** do not recommend unless the exact sign-problem condition has been checked.

## Outputs and Checks

Return a frustration diagnosis, viable method list, observable plan, or benchmark interpretation. Check whether frustration is geometric, interaction-based, fermionic, or boundary-induced; these require different evidence.

## Model Hooks

Common callers: `heisenberg`, `j1-j2`, `t-v`, `hubbard`, and `t-j`. Use `knowledge-base/2302.04919-variational-benchmarks.md` for the paper's hard-regime signals.
