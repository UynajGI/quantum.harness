---
name: mott-transition
description: Use when the user is asking about Mott localization, metal-insulator transitions, Hubbard interaction strength, double occupancy, local moments, DMFT-style reasoning, or correlated-electron regimes.
---

# Mott Transition

This skill diagnoses interaction-driven localization and metal-insulator behavior across Hubbard-like models.

## Inputs to Fix

Identify model, lattice or impurity setting, filling, bandwidth or hopping scale, interaction strength, temperature versus ground state, and observable. Distinguish a true Mott question from ordinary band insulator, Anderson localization, or finite-size gap questions.

## Steering Defaults

- Entry: use density, double occupancy, local moment, charge gap proxy, and simple finite-size trends.
- Intermediate: compare local versus spatial correlations, finite-size effects, hysteresis/transition criteria if relevant, DMFT suitability, and cross-method checks.

## Method Guidance

- **ED/DMRG:** useful for finite systems, chains, and gaps/correlations.
- **QMC/AFQMC:** strong when sign-problem-free; constrained variants require bias discussion.
- **DMFT:** natural for local self-energy and Mott metal-insulator transition questions.
- **Variational/tensor-network methods:** useful in frustrated or doped regimes where unbiased routes are difficult.

## Outputs and Checks

Return a Mott-regime diagnosis, method plan, observable checklist, or interpretation. Check filling, double occupancy trend, charge gap definition, local moment, finite-size artifacts, and whether the method can distinguish metal, band insulator, and Mott insulator.

## Model Hooks

Common callers: `hubbard`, `t-j`, and `multiorbital-hubbard`.
