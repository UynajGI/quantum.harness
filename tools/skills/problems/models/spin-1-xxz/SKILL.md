---
name: spin-1-xxz
description: Use when the user is working on a spin-1 XXZ ground-state problem with optional single-ion anisotropy `D` — covers Néel / Haldane SPT / large-`D` phase diagram, Ising / Gaussian transitions between them, and integer-spin chain physics generally.
---

# Spin-1 XXZ

Solve spin-1 XXZ chain ground-state problems with optional single-ion anisotropy. Distinct canonical problem family from the `heisenberg` skill (which targets spin-1/2 by default) because the spin-1 Hilbert space and SPT (Haldane) physics define a separate phase-diagram structure.

## Diagnose

Infer the canonical setup from the user's prompt and propose it for ratification.

**Canonical defaults:** S=1, isotropic `Δ = 1`, `D` from the user's prompt (default `D = 0` — pure Heisenberg spin-1, in the Haldane phase), `S^z_total = 0` sector, OBC, `L = 32`, target `E/N` and a phase-diagnostic observable (e.g., string order, magic, or `L(ρ_AB)`).

**Proposal pattern:** "Going with: 1D chain, S=1, `Δ = 1`, `D = [value]`, `S^z_total = 0`, OBC, `L = 32`, target `E/N` and Haldane-phase indicator. Override any, or pick: `D`-scan across the phase diagram (Néel ↔ Haldane ↔ large-`D`), `Δ`-scan."

Build per `knowledge-base/conventions.md`. The Hamiltonian:

```
H = − Σ_{⟨ij⟩} [ S_i^x S_j^x + S_i^y S_j^y + Δ S_i^z S_j^z ] + D Σ_i (S_i^z)²
```

with `S^a` the spin-1 operators. Default sign-of-coupling matches the methodology reference at `knowledge-base/literature/magic/INDEX.md`; explicit factor-of-2 / sign translations live in `knowledge-base/conventions.md` if the user reports values from a different paper.

## Workflow

1. Set up sites with local dimension 3 and `S^z_total` conservation; choose initial state in the target sector (e.g., a Néel-like product state for finite-size AFM at `D ≪ 0`, or AKLT-like for default Haldane work).
2. Pick method per the table.
3. Short first run; confirm `S^z_total = 0`, lattice translation respected (PBC) or open-boundary effects characterized (OBC).
4. Sweep bond dim until the target observable stabilizes.
5. Verify (next section).
6. If the question becomes phase-diagnostic, hand off via the branch table.

## Method recommendations

| Regime | Method | Card |
|---|---|---|
| 1D chain, ground-state energy + correlations | DMRG | `knowledge-base/methods/dmrg.md` |
| Small cluster (`L ≲ 14`) for exact spectrum, gap, or cross-check | ED pending refreshed references | `knowledge-base/methods/ed.md` |
| 1D ring (PBC) at large `L` with Pauli-string sampling | TTN | `knowledge-base/methods/ttn.md` |
| Imaginary-time route to ground state | TEBD | `knowledge-base/methods/tebd.md` |

## Branch table

| Condition | Action |
|---|---|
| Question is about magic / SRE / long-range magic across the phase diagram | Run the calculation here; hand off to `physics/magic`. Default partition for criticality is `L(ρ_AB)` because full-state magic *misses* both Néel-Haldane and Haldane-large-`D` transitions. See `knowledge-base/magic-benchmarks.md` for reference behavior. |
| Question is about the Néel-Haldane (Ising) or Haldane–large-`D` (Gaussian) transition | Run the calculation here, then call `criticality`. Reference transition values: `D ≈ −0.3` (Néel-Haldane Ising), `D ≈ 0.97` (Haldane–large-`D` Gaussian) at `Δ = 1`. |
| Question is about Haldane phase identification | Compute string order parameter and entanglement spectrum (degeneracy on a cut); document. SPT-phase identification is a runtime computation, not a separate skill. |
| User asks about spin-1/2 (`S = 1/2`) Heisenberg | Switch to `heisenberg`. |
| User asks about dynamics or finite-T | Route to `knowledge-base/methods/spectral.md` (stub) / `finite-t.md` (stub). |

## Verification

Default checks (all auto-run; results aggregated into the report's verification line):

- **Limit checks** via `knowledge-base/limits.md`: at `Δ → ∞` the model is classical Ising-like in `S^z`; at `Δ = 0` it is XY (gapless free-fermion-like in spin-1/2; spin-1 case is more delicate but still computable); at `D → −∞` the ground state is Néel; at `D → +∞` the ground state is the large-`D` trivial product `Π |S^z_i = 0⟩`.
- **Symmetry**: `S^z_total` conservation; lattice translation; reflection symmetry where applicable. Haldane phase is SPT — the entanglement spectrum is doubly degenerate on a periodic cut.
- **Convergence**: bond-dim sweep gives monotonic, asymptoting energy. The Haldane phase has a finite gap → DMRG converges fast; the transition regions are slower (gap closing).
- **Internal consistency**: energy variance small relative to `E²`; string order parameter saturates to a finite value in Haldane, vanishes in trivial phases.
- **Cross-method validation (auto-paired when available)** — use TEBD or another active independent route. Use ED only after `knowledge-base/methods/ed.md` is rebuilt.

Optional check:

- Reference transitions at `Δ = 1`: `D ≈ −0.3` (Néel-Haldane Ising), `D ≈ 0.97` (Haldane-large-`D` Gaussian), per `knowledge-base/magic-benchmarks.md` (transitions cited there from the methodology reference; original DMRG references threaded within).

## Frontier flag

The Haldane phase is SPT-protected by spatial inversion / time reversal / `Z_2 × Z_2`; small symmetry-breaking perturbations can drive crossover-like behavior that is easy to mistake for a phase transition. When the user runs a generic `D`-scan with explicit symmetry-breaking present, surface this and offer:

1. The diagnostic plan above (recommended).
2. A constraint-only report (transitions located but topology not labelled).
3. A pointer to the literature range for the given symmetry sector.

Before interpreting evidence in a frontier symmetry-broken regime, invoke `arxiv-search` with a tailored query.

## Branch table (magic and related)

| Diagnostic | Action |
|---|---|
| `magic` | Hand off to `physics/magic`. Default estimator: `L(ρ_AB)` because full-state `m_n` does not distinguish the Haldane transitions. Two-site Pauli updates preserve `S^z_total` (U(1)) and the lattice symmetry; see `knowledge-base/methods/pauli-markov.md`. The Haldane-phase plateau saturates near the single-qutrit-product upper bound `(2/3) log 4 ≈ 0.92` — see `knowledge-base/magic-benchmarks.md`. |
| `criticality` | Standard finite-size scaling at the two transitions (Ising at `D ≈ −0.3`, Gaussian at `D ≈ 0.97`). |
| `frustration` | Not the canonical framing here unless the user has added next-nearest neighbor coupling that creates competition. |

## Writeup handoff

After verification, if the user wants to communicate the result, consolidate to a runnable script + short run report, then route to `scientific-writing` / `latex-paper-en` / `scientific-visualization`. See AGENTS.md "Writeup handoff".

## Related skills

`heisenberg` (spin-1/2 case; `S = 1/2` is a different canonical family), `criticality`, `magic`.
