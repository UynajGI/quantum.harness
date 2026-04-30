---
name: spin-liquid
description: Use when the user is asking about quantum spin liquids, absence of magnetic order, kagome/triangular/frustrated Heisenberg regimes, topological signatures, entanglement diagnostics, or distinguishing spin-liquid candidates from ordered phases.
---

# Spin Liquid

This skill turns a spin-liquid question into diagnostics and evidence. Do not label a state a spin liquid without checking competing explanations.

## Inputs to Fix

Identify model, lattice, spin, coupling regime, geometry/size, boundary condition, candidate competing phases, and available observables or wave functions.

## Steering Defaults

- Entry: rule out obvious magnetic order with correlations and structure factors.
- Intermediate: examine gap behavior, entanglement diagnostics, topological indicators, finite-size/cylinder effects, and competing variational states.

## Method Guidance

- **ED:** useful for spectra, symmetry quantum numbers, and cluster structure factors.
- **DMRG:** strong for kagome/triangular cylinders, but interpret with geometry and finite-width caution.
- **VMC/NQS:** useful for comparing candidate spin-liquid and ordered wave functions.
- **Tensor networks:** useful for topological/entanglement ansatz studies when bond dimension and symmetry choices are explicit.

## Outputs and Checks

Return a diagnostic plan, observable table, code sketch, or interpretation. Check spin correlations, structure factor, gap estimates, entanglement signatures, boundary sensitivity, and whether the evidence only shows "no detected order" rather than a positive spin-liquid identification.

## Model Hooks

Common callers: `heisenberg`, `j1-j2`, and `t-j`. Call `frustration` first if the source of degeneracy or competition is unclear.
