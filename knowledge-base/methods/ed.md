# ED (Exact Diagonalization)

Direct diagonalization of the Hamiltonian as a sparse matrix in a chosen basis. Exact within the chosen Hilbert subspace; limited by basis size, hence by system size.

## Setup

Canonical stack: `julia-ed` (`tools/software/stacks/julia-ed.toml`).

Install: `make install julia-ed`. The `itensors` stack also installs
KrylovKit, so an existing DMRG environment is sufficient for ED checks.

For very small problems, dense `LinearAlgebra.eigen` works directly on `Matrix{Float64}`.

## Notation

- Full Hilbert space dimension: `2^N` (spin-1/2), `4^N` (electron / Hubbard), `3^N` (t-J projected).
- Sector dimension: typically much smaller after fixing `(N↑, N↓)` or `S^z`.
- Sparse matrix `H` in chosen basis; eigenvalue solver returns `(eigvals, eigvecs)`.

## Code shape (Julia / KrylovKit.jl)

```julia
using LinearAlgebra, KrylovKit, SparseArrays

# 1. Build the basis for the target sector (e.g., S^z = 0 of N spins)
#    — domain-specific construction; for spin-1/2 chains, use bit representation.

# 2. Build sparse H by acting term-by-term on basis states

# 3. Solve for the lowest k eigenvalues
n_states = 4
energies, vectors, info = eigsolve(H, n_states, :SR; tol=1e-12, krylovdim=30)
# :SR  → smallest real eigenvalues
# tol  → convergence tolerance
# krylovdim → Krylov subspace size (raise if not converging)

# 4. Compute observables on the ground-state vector
psi0 = vectors[1]
sz_total = real(psi0' * Sz_total_op * psi0)
```

For ITensors-MPO-based ED (small systems handled as MPS at full bond dimension), the DMRG card pattern works at very small `N`. For genuinely small `N`, build the matrix directly.

## Knobs

| Knob | Effect | Starting point |
|---|---|---|
| Sector / quantum numbers | Block-diagonalize. | Always fix `(N↑, N↓)` for fermions; `S^z_total` for spins. |
| `n_states` | Number of low-lying states to compute. | 1 for ground state; 4–10 for spectrum / gap. |
| `tol` | Convergence tolerance (residual). | `1e-10` to `1e-12` for benchmarks. |
| `krylovdim` | Krylov subspace dimension. | 30 default; raise if convergence fails. |

## Pitfalls

- **Memory blowup**: full Hilbert space for `N = 24` spin-1/2 is `2^24 ≈ 16M`; use sectors aggressively. Hubbard at `N = 12` half-filled has `(12 choose 6)² ≈ 850k`.
- **Fermion sign**: when constructing matrix elements of `c†_i c_j` in occupation-number basis, track Jordan-Wigner sign factors.
- **Wrong sector**: ground state may not be in the sector you constructed; verify by computing in adjacent sectors.
- **Sparse vs dense**: dense diagonalization is fine for `dim < ~3000`; sparse Lanczos for larger.
- **Degeneracy**: gapless ground states have many close-by states; `n_states ≥ 4` and check for degeneracy structure.

## Verification

- **Sector consistency**: compute total `S^z`, particle number on the eigenvector; should match the constructed sector.
- **Limit checks**: at trivial parameter values, the ED spectrum should match analytic results (e.g., `J = 0` Heisenberg → all `E = 0`).
- **Multi-method cross-check**: ED should agree with DMRG at small system sizes (within DMRG's bond-dim accuracy).

## Citations

- Lin, *Phys. Rev. B* **42**, 6561 (1990) — early ED for Hubbard.
- Sandvik, *AIP Conf. Proc.* **1297**, 135 (2010) — ED for spin systems.
- KrylovKit.jl documentation — Julia eigensolver.
