# Cached-MPS direct M2 reproduction path for Tarabunga et al. Fig. 4.
#
# This is a stability-oriented cross-method driver: estimate M_2(L) directly
# from the Pauli distribution Xi_P ∝ |<P>|^2, then form
# c_L = 2 M_2(L/2) - M_2(L). It uses the same PBC TFIM Hamiltonian and cached
# MPS Pauli environments as scripts/tfim_fig4_paper_grade.jl, but avoids the
# rare-event ratio in Eq. (24). The Eq. (24) path remains the paper-method
# diagnostic; this script produces the robust Fig. 4 curves for MPS.

using JSON
using Plots
using Printf
using Random
using Statistics

include(joinpath(@__DIR__, "..", "..", "scripts", "tfim_fig4_paper_grade.jl"))

const OUTDIR = joinpath(@__DIR__, "..", "..", "results", "tfim_fig4_cached_direct")

function pauli_markov_M2_direct_cached(cache::CachedMPSPauliExpectation, L::Int;
                                      n_steps=10^5, n_warmup=10^4,
                                      seed::UInt32=UInt32(0xC0FFEE),
                                      progress_every=max(1, n_steps ÷ 10),
                                      proposal::Symbol=:group)
    rng = MersenneTwister(seed)
    set_pauli_string!(cache, zeros(Int, L))
    @assert time_reversal_allowed(cache.p)
    @assert proposal in (:paper, :group)

    cur_v = cached_pauli_value(cache)
    cur_w = abs(cur_v)^2 / 2.0^L
    multiply_Z(old::Int) = old ⊻ 3
    multiply_X(old::Int) = old ⊻ 1
    is_xy(old::Int) = old == 1 || old == 2
    group_two(old::Int) = is_xy(old) ? rand(rng, (0, 3)) : rand(rng, (1, 2))

    accum = 0.0
    n_acc = 0
    block_size = max(1_000, n_steps ÷ 100)
    block_means = Float64[]
    cur_block_sum = 0.0
    cur_block_idx = 0

    for step in 1:(n_warmup + n_steps)
        proposal_kind = rand(rng) < 0.5 ? :single : :two
        if proposal_kind == :single
            i = rand(rng, 1:L)
            new_i = multiply_Z(cache.p[i])
            allowed = time_reversal_allowed_after(cache.p, i, new_i)
            new_v = allowed ? cached_candidate_value(cache, i, new_i) : cur_v
        else
            i = rand(rng, 1:L)
            j = rand(rng, 1:(L - 1))
            j >= i && (j += 1)
            if proposal == :paper
                new_i = multiply_X(cache.p[i])
                new_j = multiply_X(cache.p[j])
            else
                new_i = group_two(cache.p[i])
                new_j = group_two(cache.p[j])
            end
            allowed = time_reversal_allowed_after(cache.p, i, new_i, j, new_j)
            new_v = allowed ? cached_candidate_value(cache, i, new_i, j, new_j) : cur_v
        end

        new_w = abs(new_v)^2 / 2.0^L
        ratio = (cur_w == 0 && new_w == 0) ? 0.0 : (cur_w == 0 ? Inf : new_w / cur_w)
        if allowed && rand(rng) < ratio
            if proposal_kind == :single
                set_cached_pauli!(cache, i, new_i)
            else
                set_cached_paulis!(cache, i, new_i, j, new_j)
            end
            cur_v = new_v
            cur_w = new_w
            n_acc += 1
        end

        if step > n_warmup
            v2 = abs(cur_v)^2
            accum += v2
            cur_block_sum += v2
            cur_block_idx += 1
            if cur_block_idx == block_size
                push!(block_means, cur_block_sum / block_size)
                cur_block_sum = 0.0
                cur_block_idx = 0
            end
            if (step - n_warmup) % progress_every == 0
                mean_v2_now = accum / (step - n_warmup)
                @printf("      [cached direct M2 sample %d/%d] M2=%.6f mean_v2=%.6e accept=%.4f blocks=%d\n",
                        step - n_warmup, n_steps, -log(mean_v2_now), mean_v2_now,
                        n_acc / step, length(block_means))
                flush(stdout)
            end
        end
    end

    mean_v2 = accum / n_steps
    se_v2 = length(block_means) > 1 ? std(block_means) / sqrt(length(block_means)) : NaN
    M2 = -log(mean_v2)
    se_M2 = se_v2 / mean_v2
    return (M2=M2, se=se_M2, mean_v2=mean_v2,
            accept=n_acc / (n_warmup + n_steps),
            method="cached_direct_M2_$(proposal)_proposal", n_recorded=n_steps,
            block_size=block_size)
end

function compute_M2_cell(L::Int, h::Float64; chi=30, n_steps=10^5,
                         n_warmup=max(1_000, n_steps ÷ 10),
                         seed_offset::Int=0, pbc::Bool=true,
                         proposal::Symbol=:group)
    if L <= 8
        E, psi = ed_groundstate(L, h; pbc=pbc)
        expect = q -> ed_pauli_expectation(psi, q, L)
        M2 = exact_sre_M2_from_expect(expect, L)
        return (L=L, h=h, M2=M2, se=0.0, mean_v2=exp(-M2), accept=1.0,
                method="exact_ED_full_Pauli_sum", E=E, n_recorded=4^L,
                block_size=0)
    end
    E, psi, sites = dmrg_groundstate(L, h, chi; pbc=pbc)
    cache = CachedMPSPauliExpectation(psi, sites)
    seed = UInt32(0xD1CE0000) + UInt32(seed_offset & 0xffff)
    res = pauli_markov_M2_direct_cached(cache, L;
                                        n_steps=n_steps, n_warmup=n_warmup,
                                        seed=seed, proposal=proposal)
    return (L=L, h=h, M2=res.M2, se=res.se, mean_v2=res.mean_v2,
            accept=res.accept, method=res.method, E=E,
            n_recorded=res.n_recorded, block_size=res.block_size)
end

function mode_grid()
    mode = get(ENV, "FIG4_DIRECT_MODE", "smoke")
    if mode == "smoke"
        return [8, 16], [1.0], parse(Int, get(ENV, "FIG4_DIRECT_NSTEPS", "5000"))
    elseif mode == "local"
        return [8, 16, 32], [0.80, 0.90, 0.95, 1.00, 1.05, 1.10, 1.20],
               parse(Int, get(ENV, "FIG4_DIRECT_NSTEPS", "100000"))
    elseif mode == "paper"
        return [8, 16, 32, 64, 128], [0.80, 0.90, 0.95, 1.00, 1.05, 1.10, 1.20],
               parse(Int, get(ENV, "FIG4_DIRECT_NSTEPS", "1000000"))
    else
        error("Unknown FIG4_DIRECT_MODE=$mode")
    end
end

function main()
    isdir(OUTDIR) || mkpath(OUTDIR)
    Ls, h_grid, n_steps = mode_grid()
    chi = parse(Int, get(ENV, "FIG4_DIRECT_CHI", "30"))
    proposal = Symbol(get(ENV, "FIG4_DIRECT_PROPOSAL", "group"))
    pbc = true
    cells = Any[]
    M2 = Dict{Tuple{Int,Float64},Float64}()
    M2_err = Dict{Tuple{Int,Float64},Float64}()
    t0 = time()
    total = length(Ls) * length(h_grid)
    idx = 0
    for L in Ls, h in h_grid
        idx += 1
        @printf("\n--- direct cell %d/%d: L=%d h=%.2f ---\n", idx, total, L, h)
        flush(stdout)
        cell_t0 = time()
        res = compute_M2_cell(L, h; chi=chi, n_steps=n_steps,
                              seed_offset=idx, pbc=pbc, proposal=proposal)
        dt = time() - cell_t0
        M2[(L, h)] = res.M2
        M2_err[(L, h)] = res.se
        record = Dict("L"=>L, "h"=>h, "M2"=>res.M2, "m2"=>res.M2 / L,
                      "se_M2"=>res.se, "se_m2"=>res.se / L,
                      "mean_v2"=>res.mean_v2, "accept"=>res.accept,
                      "method"=>res.method, "E"=>res.E, "chi"=>chi,
                      "n_steps"=>n_steps, "n_recorded"=>res.n_recorded,
                      "block_size"=>res.block_size, "pbc"=>pbc,
                      "proposal"=>proposal,
                      "wall_seconds"=>dt)
        push!(cells, record)
        manifest = joinpath(OUTDIR, @sprintf("manifest_L%d_h%.2f.json", L, h))
        open(manifest, "w") do io
            JSON.print(io, record, 2)
        end
        @printf("    M2=%.6f ± %.6f  m2=%.6f  accept=%.3f  %.1fs\n",
                res.M2, res.se, res.M2 / L, res.accept, dt)
        flush(stdout)
    end

    cL = Dict{Tuple{Int,Float64},Float64}()
    cL_err = Dict{Tuple{Int,Float64},Float64}()
    for L in Ls
        L == first(Ls) && continue
        for h in h_grid
            c = 2 * M2[(L ÷ 2, h)] - M2[(L, h)]
            ce = sqrt((2 * M2_err[(L ÷ 2, h)])^2 + M2_err[(L, h)]^2)
            cL[(L, h)] = c
            cL_err[(L, h)] = ce
        end
    end

    data = Dict(
        "model"=>"1D TFIM",
        "method"=>"cached direct M2 with c_L reconstructed as 2M2(L/2)-M2(L)",
        "Ls"=>Ls,
        "h_grid"=>h_grid,
        "chi"=>chi,
        "n_steps"=>n_steps,
        "proposal"=>string(proposal),
        "pbc"=>pbc,
        "cells"=>cells,
        "M_2"=>Dict(string(L)=>[M2[(L, h)] for h in h_grid] for L in Ls),
        "M_2_err"=>Dict(string(L)=>[M2_err[(L, h)] for h in h_grid] for L in Ls),
        "c_L"=>Dict(string(L)=>[cL[(L, h)] for h in h_grid] for L in Ls[2:end]),
        "c_L_err"=>Dict(string(L)=>[cL_err[(L, h)] for h in h_grid] for L in Ls[2:end]),
        "wall_seconds_total"=>time() - t0,
    )
    open(joinpath(OUTDIR, "data.json"), "w") do io
        JSON.print(io, data, 2)
    end

    palette = [:seagreen, :steelblue, :deeppink, :orange, :gray30]
    pa = plot(xlabel="h", ylabel="c_L", title="Fig 4(a) cached MPS direct-M2 reconstruction",
              legend=:topright)
    for (k, L) in enumerate(Ls[2:end])
        plot!(pa, h_grid, [cL[(L, h)] for h in h_grid];
              yerror=[cL_err[(L, h)] for h in h_grid],
              marker=:circle, label="L=$L", c=palette[k])
    end
    savefig(pa, joinpath(OUTDIR, "panel_a_cL_vs_h.png"))

    pb = plot(xlabel="h", ylabel="m_2", title="Fig 4(b) cached MPS direct M2",
              legend=:topleft)
    for (k, L) in enumerate(Ls)
        plot!(pb, h_grid, [M2[(L, h)] / L for h in h_grid];
              yerror=[M2_err[(L, h)] / L for h in h_grid],
              marker=:circle, label="L=$L", c=palette[k])
    end
    savefig(pb, joinpath(OUTDIR, "panel_b_m2_vs_h.png"))
    savefig(plot(pa, pb; layout=(1, 2), size=(1200, 450)),
            joinpath(OUTDIR, "fig4_combined.png"))
    println("\nSaved -> $(joinpath(OUTDIR, "fig4_combined.png"))")
end

main()
