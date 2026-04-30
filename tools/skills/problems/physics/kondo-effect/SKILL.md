---
name: kondo-effect
description: Use when the user is asking about Kondo screening, local moments, impurity-bath coupling, Anderson impurity regimes, Kondo lattice physics, screening scales, or impurity-model diagnostics.
---

# Kondo Effect

This skill diagnoses local-moment formation and screening in impurity or lattice settings.

## Inputs to Fix

Identify impurity degrees of freedom, bath or conduction band, hybridization, local interaction, filling, temperature/ground-state target, and observable. Clarify whether the user wants Anderson-impurity physics, Kondo-limit reasoning, or lattice Kondo behavior.

## Steering Defaults

- Entry: finite-bath model, occupancy, local moment, and impurity-bath spin correlations.
- Intermediate: screening length/scale reasoning, bath discretization sensitivity, symmetry sectors, and comparison across solvers.

## Method Guidance

- **Finite-bath ED:** good first route for local moments and small benchmark problems.
- **MPS impurity solver:** useful after mapping bath to a chain.
- **NRG-style reasoning:** appropriate for scale-separated low-energy screening, even if not implemented directly.
- **DMFT impurity context:** relevant when the impurity comes from a self-consistent lattice problem.

## Outputs and Checks

Return a diagnostic checklist, Hamiltonian reduction, solver plan, or interpretation. Check impurity occupancy, local moment, spin correlations, bath discretization, particle/spin-sector consistency, and whether the finite bath can resolve the claimed Kondo scale.

## Model Hooks

Common callers: `anderson-impurity`, `multiorbital-hubbard`, and `hubbard` when impurity embedding or local moments are central.
