# Regression for exact/batched Pauli-MPS sampling primitives used by Fig. 4.

using Random
using Printf

include(joinpath(@__DIR__, "tfim_fig4_paper_grade.jl"))
include(joinpath(@__DIR__, "..", "tools", "cli", "pauli_mps_sampler.jl"))

function pauli_string_from_index(idx0::Int, L::Int)
    p = zeros(Int, L)
    x = idx0
    for i in 1:L
        p[i] = x & 3
        x >>= 2
    end
    return p
end

function brute_force_squared_norm(Bs)
    L = length(Bs)
    total = 0.0
    for idx0 in 0:(4^L - 1)
        p = pauli_string_from_index(idx0, L)
        total += abs2(pauli_mps_amplitude(Bs, p))^2
    end
    return total
end

function brute_force_born_norm(Bs)
    L = length(Bs)
    total = 0.0
    for idx0 in 0:(4^L - 1)
        p = pauli_string_from_index(idx0, L)
        total += abs2(pauli_mps_amplitude(Bs, p))
    end
    return total
end

function main()
    L = 4
    h = 0.8
    E, psi, sites = dmrg_groundstate(L, h, 6; nsweeps=8, pbc=true)
    Bs = pauli_mps_tensors(psi, sites)

    for p in ([0, 0, 0, 0],
              [3, 0, 3, 0],
              [1, 1, 0, 0],
              [2, 2, 0, 0])
        amp = pauli_mps_amplitude(Bs, collect(p))
        ref = mps_pauli_expectation(psi, collect(p), sites) / sqrt(2.0^L)
        @assert isapprox(amp, ref; atol=1e-10, rtol=1e-10)
    end

    sampler = SquaredPauliMPSSampler(Bs)
    norm_exact = brute_force_squared_norm(Bs)
    norm_sampler = pauli_squared_mps_norm(sampler)
    @assert isapprox(norm_sampler, norm_exact; atol=1e-10, rtol=1e-10)

    born = PauliMPSBornSampler(Bs)
    born_norm_exact = brute_force_born_norm(Bs)
    @assert isapprox(pauli_mps_born_norm(born), born_norm_exact; atol=1e-10, rtol=1e-10)

    Bs_compressed, trunc_err = compress_pauli_mps(Bs, 64, 1e-14)
    compressed_norm = pauli_squared_mps_norm(SquaredPauliMPSSampler(Bs_compressed))
    @assert trunc_err <= 1e-20
    @assert isapprox(compressed_norm, norm_exact; atol=1e-10, rtol=1e-10)

    rng = MersenneTwister(0x515A)
    p = sample_pauli_string(sampler, rng)
    @assert length(p) == L
    @assert all(0 .<= p .<= 3)
    p_born = sample_pauli_string(born, rng)
    @assert length(p_born) == L
    @assert all(0 .<= p_born .<= 3)

    Eh, psih, sitesh = dmrg_groundstate(L ÷ 2, h, 6; nsweeps=8, pbc=true)
    Bhalf = pauli_mps_tensors(psih, sitesh)
    exact_cL = log(pauli_squared_mps_norm(sampler) /
                   pauli_squared_mps_norm(SquaredPauliMPSSampler(Bhalf))^2)
    bridge = pauli_mps_bridge_cL(Bs, Bhalf; n_samples=1_000, seed=0xCACE)
    @assert isapprox(bridge.cL, exact_cL; atol=0.10, rtol=0.0)

    direct = pauli_mps_born_direct_cL(Bs, Bhalf; n_samples=4_000, seed=0xD1EC, n_blocks=10)
    @assert isapprox(direct.cL, exact_cL; atol=max(0.20, 4 * direct.se), rtol=0.0)
    exact_logz4 = log(norm_exact)
    direct_logz4 = pauli_mps_born_direct_logz4(Bs; n_samples=4_000, seed=0x5151, n_blocks=10)
    @assert isapprox(direct_logz4.logz4, exact_logz4; atol=max(0.20, 4 * direct_logz4.se), rtol=0.0)

    @printf("Pauli-MPS sampler regression passed at L=%d h=%.2f E=%.8f norm=%.12e bridge=%.8f direct=%.8f exact=%.8f\n",
            L, h, E, norm_sampler, bridge.cL, direct.cL, exact_cL)
end

main()
