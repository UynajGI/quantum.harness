# Quantum Many-Body Physics Harness

Research harness for solving quantum many-body problems with tensor-network methods. Targets entry/medium-level **ground-state lattice problems** — spin chains, Hubbard / t-J / t-V models, Anderson impurities, frustrated magnets — and the diagnostic physics questions that come with them (criticality, frustration, spin liquids, Mott transitions, Kondo effect).

The harness is **agent-led, user-ratified**: you bring a concrete problem, the agent diagnoses it, recommends a method, sets up the calculation, runs it, and verifies against known limits and benchmarks. Decisions are surfaced only when there's a real branch.

## Quick Start

```bash
# Full setup: Ion + skills + domain stack (Julia + ITensors)
make setup && make domain-setup

# Optional: NetKet for VMC / neural quantum states
make install netket

# Optional: quimb (Python TN alternative)
make install quimb

# See all targets
make help
```

Then ask a concrete problem in your Claude / Codex session — the right problem skill activates automatically. If you're not sure where to start, say "where do I start" or invoke `/onboard`.

## Repository layout

```
tools/skills/problems/
  models/          Hamiltonian-family skills: TFIM, Heisenberg, J1-J2, t-V,
                   Hubbard, t-J, Anderson impurity, multiorbital Hubbard
  physics/         Diagnostic skills: criticality, frustration, spin-liquid,
                   Mott transition, Kondo effect

knowledge-base/    Reference data the skills cite:
  conventions.md           sign and normalization defaults
  limits.md                exact reductions and known limits
  benchmark-numbers.md     reference E/N, gaps, with citations
  symmetry-cheatsheet.md   conserved quantities per lattice
  methods/{ed,dmrg,tebd}.md  per-method notation, code shape, knobs, pitfalls
  2302.04919-variational-benchmarks.md   paper-specific notes

tools/cli/         CLI helpers (`render`, ...)
tools/templates/   HTML render template
docs/              Design specs and design history
Makefile           Bootstrap + on-demand installs
AGENTS.md          Agent-facing design contract (canonical)
Ion.toml           Skill manifest (managed by Ion)
```

## Design principle

**Skills hold problem-level workflow.** Diagnose, decide, branch, verify. They never hardcode reference numbers or method-specific code; they cite the knowledge base for both.

**Knowledge base holds data + method-level reference.** Numbers with citations, conventions, limits, per-method code shape and pitfalls. When a published value gets revised, KB updates and skills don't break.

Two skill shapes, both problem/research-driven:

- **Model skills** drive calculations: Diagnose → Workflow → Method recommendations → Branch table → Verification.
- **Physics skills** evaluate evidence: Diagnose → Evidence to gather → Cross-checks → Interpretation rules.

The harness is not a tutorial. Learning happens as a side effect of watching capable problem solving, not as the product.

## Scope

Current coverage: **ground-state lattice problems with frontier-flagged uncertainty** — entry/medium baselines plus contested cases inside the same domain (kagome ground state, J1-J2 around `J2/J1 ≈ 0.5`, 2D doped Hubbard, …). For contested cases the harness reports literature ranges and residual uncertainty rather than claiming closure.

**Future directions** (out of scope today; added as new skills when real problems demand them): real-time dynamics, finite-T thermodynamics, open quantum systems, topology beyond spin liquids, continuum / field-theory methods, multi-aspect research orchestration.

For the design contract and controlling principles, see `AGENTS.md`. For design history and current spec, see `docs/superpowers/specs/`.
