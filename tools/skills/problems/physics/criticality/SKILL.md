---
name: criticality
description: Use when the user is asking about quantum critical points, scaling, finite-size collapse, critical exponents, gaps, universality, conformal data, fuzzy-sphere regularization, or critical behavior across quantum many-body models.
---

# Criticality

This skill diagnoses quantum critical behavior across models. It should produce evidence, scaling logic, or a calculation plan, not a course on phase transitions.

## Inputs to Fix

Identify the model, control parameter, candidate phases, observable, available system sizes, boundary conditions, and whether the user wants a qualitative diagnosis or quantitative exponent/critical-point estimate.

## Steering Defaults

- Entry: locate candidate critical behavior using gaps, order parameters, susceptibilities, and simple finite-size trends.
- Intermediate: propose scaling forms, collapse variables, universality checks, uncertainty estimates, and method cross-checks.

## Method Guidance

- **ED:** useful for gaps, spectra, and small-size scaling diagnostics.
- **DMRG/MPS:** strong for 1D critical chains; use entanglement entropy and correlation-length scaling where appropriate.
- **QMC:** strong for sign-problem-free critical systems.
- **Tensor networks:** useful for finite-entanglement or 2D variational scaling, with bond-dimension caveats.
- **Fuzzy sphere:** a regularization/platform for critical field-theory questions; use only when it fits the user's problem.

## Outputs and Checks

Return a criticality diagnostic plan, finite-size scaling table, code sketch, or interpretation. Check that the chosen observable actually distinguishes phases, that sizes are sufficient for the claimed scaling, and that nonuniversal conventions are separated from universal quantities.

## Model Hooks

Common callers: `transverse-field-ising`, `heisenberg`, `j1-j2`, `hubbard`, and `t-j`.
