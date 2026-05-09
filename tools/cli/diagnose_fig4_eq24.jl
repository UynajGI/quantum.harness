# Exact small-L diagnostics for the Fig. 4 Eq. (24) estimator.
#
# This script is intentionally independent of the production Fig. 4 driver: it
# enumerates Pauli strings, separates the Z2/time-reversal sector used by the
# paper proposal, and compares the exact ratio expectation against the Markov
# chain on that same sector.

using LinearAlgebra
using Printf
using Random
using Statistics

function ed_groundstate(L, h; pbc::Bool=true)
    dim = 2^L
    H = zeros(Float64, dim, dim)
    for s in 0:(dim - 1)
        d = 0.0
        for i in 0:(L - 1)
            d -= h * (1 - 2 * ((s >> i) & 1))
        end
        H[s + 1, s + 1] = d
    end
    bond_stop = pbc ? L - 1 : L - 2
    for i in 0:bond_stop
        j = (i + 1) % L
        mask = (1 << i) | (1 << j)
        for s in 0:(dim - 1)
            H[(s ⊻ mask) + 1, s + 1] -= 1.0
        end
    end
    F = eigen(Symmetric(H))
    return F.values[1], F.vectors[:, 1]
end

function wht!(x::AbstractVector{Float64})
    n = length(x)
    h = 1
    while h < n
        for i in 1:(2h):n
            @inbounds for j in i:(i + h - 1)
                a = x[j]
                b = x[j + h]
                x[j] = a + b
                x[j + h] = a - b
            end
        end
        h *= 2
    end
    return x
end

@inline function pauli_code(xmask::Int, zmask::Int, i::Int)
    xb = (xmask >> i) & 1
    zb = (zmask >> i) & 1
    return xb == 1 ? (zb == 1 ? 2 : 1) : (zb == 1 ? 3 : 0)
end

@inline function pauli_index_from_masks(xmask::Int, zmask::Int, L::Int)
    idx = 0
    place = 1
    for i in 0:(L - 1)
        idx += pauli_code(xmask, zmask, i) * place
        place *= 4
    end
    return idx
end

@inline function symmetry_allowed_index(idx0::Int, L::Int)
    xparity = 0
    yparity = 0
    x = idx0
    for _ in 1:L
        code = x & 3
        x >>= 2
        xparity ⊻= (code == 1 || code == 2)
        yparity ⊻= (code == 2)
    end
    return xparity == 0 && yparity == 0
end

function pauli_abs4_table(psi::Vector{Float64}, L::Int)
    dim = 2^L
    table = zeros(Float64, 4^L)
    g = Vector{Float64}(undef, dim)
    for xmask in 0:(dim - 1)
        @inbounds for s in 0:(dim - 1)
            g[s + 1] = psi[(s ⊻ xmask) + 1] * psi[s + 1]
        end
        wht!(g)
        @inbounds for zmask in 0:(dim - 1)
            idx0 = pauli_index_from_masks(xmask, zmask, L)
            code_y_parity = isodd(count_ones(xmask & zmask))
            table[idx0 + 1] = code_y_parity ? 0.0 : g[zmask + 1]^4
        end
    end
    return table
end

function split_indices(idx0::Int, L::Int, split_mode::Symbol)
    Lh = L ÷ 2
    left = 0
    right = 0
    lplace = 1
    rplace = 1
    if split_mode == :contiguous
        split = 4^Lh
        return idx0 % split, idx0 ÷ split
    elseif split_mode == :interleaved
        for site in 0:(L - 1)
            code = (idx0 >> (2site)) & 3
            if iseven(site)
                left += code * lplace
                lplace *= 4
            else
                right += code * rplace
                rplace *= 4
            end
        end
        return left, right
    else
        error("unknown split mode: $split_mode")
    end
end

function exact_sector_stats(abs4_L::Vector{Float64}, abs4_H::Vector{Float64}, L::Int;
                            split_mode::Symbol=:contiguous)
    Lh = L ÷ 2
    rows = []
    for sector in (:all, :paper)
        Z = 0.0
        mean_num = 0.0
        second_num = 0.0
        log_num = 0.0
        log_second_num = 0.0
        max_R = 0.0
        n = 0
        for idx0 in 0:(length(abs4_L) - 1)
            sector == :paper && !symmetry_allowed_index(idx0, L) && continue
            denom = abs4_L[idx0 + 1]
            denom == 0.0 && continue
            i1, i2 = split_indices(idx0, L, split_mode)
            num = abs4_H[i1 + 1] * abs4_H[i2 + 1]
            R = num / denom
            logR = log(R)
            Z += denom
            mean_num += num
            second_num += num * R
            log_num += denom * logR
            log_second_num += denom * logR^2
            max_R = max(max_R, R)
            n += 1
        end
        mean_R = mean_num / Z
        var_R = second_num / Z - mean_R^2
        mean_logR = log_num / Z
        var_logR = log_second_num / Z - mean_logR^2
        push!(rows, (sector=sector, support=n, Z=Z, mean_R=mean_R,
                    cL=-log(mean_R), sd_R=sqrt(max(var_R, 0.0)),
                    iid_se_cL_1e6=sqrt(max(var_R, 0.0) / 1e6) / mean_R,
                    max_R=max_R, neg_mean_logR=-mean_logR,
                    iid_se_neg_mean_logR_1e6=sqrt(max(var_logR, 0.0) / 1e6)))
    end
    return rows
end

function exact_dual_stats(abs4_L::Vector{Float64}, abs4_H::Vector{Float64}, L::Int;
                          split_mode::Symbol=:contiguous)
    Lh = L ÷ 2
    Zq = 0.0
    mean_num = 0.0
    second_num = 0.0
    max_R = 0.0
    support = 0
    for idx0 in 0:(length(abs4_L) - 1)
        !symmetry_allowed_index(idx0, L) && continue
        i1, i2 = split_indices(idx0, L, split_mode)
        num = abs4_H[i1 + 1] * abs4_H[i2 + 1]
        num == 0.0 && continue
        denom = abs4_L[idx0 + 1]
        R = denom / num
        Zq += num
        mean_num += denom
        second_num += denom * R
        max_R = max(max_R, R)
        support += 1
    end
    mean_R = mean_num / Zq
    var_R = second_num / Zq - mean_R^2
    return (support=support, Z=Zq, mean_R=mean_R, cL=log(mean_R),
            sd_R=sqrt(max(var_R, 0.0)),
            iid_se_cL_1e6=sqrt(max(var_R, 0.0) / 1e6) / mean_R,
            max_R=max_R)
end

function tolerance_sweep(abs4_L::Vector{Float64}, abs4_H::Vector{Float64}, L::Int)
    maxw = maximum(abs4_L)
    rows = []
    for exponent in (-28, -24, -20, -16, -12)
        tol = maxw * 10.0^exponent
        Z = 0.0
        mean_num = 0.0
        second_num = 0.0
        dropped_weight = 0.0
        dropped_num = 0.0
        support = 0
        for idx0 in 0:(length(abs4_L) - 1)
            !symmetry_allowed_index(idx0, L) && continue
            denom = abs4_L[idx0 + 1]
            i1, i2 = split_indices(idx0, L, :contiguous)
            num = abs4_H[i1 + 1] * abs4_H[i2 + 1]
            if denom <= tol
                dropped_weight += denom
                dropped_num += num
                continue
            end
            R = num / denom
            Z += denom
            mean_num += num
            second_num += num * R
            support += 1
        end
        mean_R = mean_num / Z
        var_R = second_num / Z - mean_R^2
        push!(rows, (tol=tol, exponent=exponent, support=support,
                    cL=-log(mean_R), mean_R=mean_R,
                    iid_se_cL_1e6=sqrt(max(var_R, 0.0) / 1e6) / mean_R,
                    dropped_weight=dropped_weight, dropped_num=dropped_num))
    end
    return rows
end

function chain_sector(abs4_L::Vector{Float64}, abs4_H::Vector{Float64}, L::Int;
                      n_steps=200_000, n_warmup=10_000, seed=0xBEEF,
                      reject_odd_y=true, split_mode::Symbol=:contiguous,
                      proposal::Symbol=:paper)
    @assert proposal in (:paper, :group)
    Lh = L ÷ 2
    place = [4^i for i in 0:(L - 1)]
    rng = MersenneTwister(UInt32(seed))
    p = zeros(Int, L)
    idx0 = 0
    cur_w = abs4_L[1]
    accum_R = 0.0
    n_R = 0
    n_acc = 0
    for step in 1:(n_warmup + n_steps)
        if rand(rng) < 0.5
            i = rand(rng, 1:L)
            old = p[i]
            new = old ⊻ 3
            new_idx0 = idx0 + (new - old) * place[i]
            if !reject_odd_y || symmetry_allowed_index(new_idx0, L)
                new_w = abs4_L[new_idx0 + 1]
                ratio = cur_w == 0 ? (new_w == 0 ? 0.0 : Inf) : new_w / cur_w
                if rand(rng) < ratio
                    p[i] = new
                    idx0 = new_idx0
                    cur_w = new_w
                    n_acc += 1
                end
            end
        else
            i = rand(rng, 1:L)
            j = rand(rng, 1:(L - 1))
            j >= i && (j += 1)
            oi, oj = p[i], p[j]
            if proposal == :paper
                ni, nj = oi ⊻ 1, oj ⊻ 1
            else
                ni = (oi == 1 || oi == 2) ? rand(rng, (0, 3)) : rand(rng, (1, 2))
                nj = (oj == 1 || oj == 2) ? rand(rng, (0, 3)) : rand(rng, (1, 2))
            end
            new_idx0 = idx0 + (ni - oi) * place[i] + (nj - oj) * place[j]
            if !reject_odd_y || symmetry_allowed_index(new_idx0, L)
                new_w = abs4_L[new_idx0 + 1]
                ratio = cur_w == 0 ? (new_w == 0 ? 0.0 : Inf) : new_w / cur_w
                if rand(rng) < ratio
                    p[i] = ni
                    p[j] = nj
                    idx0 = new_idx0
                    cur_w = new_w
                    n_acc += 1
                end
            end
        end
        if step > n_warmup && cur_w > 0
            i1, i2 = split_indices(idx0, L, split_mode)
            R = abs4_H[i1 + 1] * abs4_H[i2 + 1] / cur_w
            accum_R += R
            n_R += 1
        end
    end
    mean_R = accum_R / n_R
    return (mean_R=mean_R, cL=-log(mean_R), accept=n_acc / (n_steps + n_warmup),
            n_recorded=n_R, reject_odd_y=reject_odd_y)
end

function main()
    L = parse(Int, get(ENV, "FIG4_DIAG_L", "8"))
    h = parse(Float64, get(ENV, "FIG4_DIAG_H", "0.80"))
    n_steps = parse(Int, get(ENV, "FIG4_DIAG_NSTEPS", "200000"))
    @assert iseven(L)
    @printf("[exact-sector] L=%d h=%.2f n_steps=%d\n", L, h, n_steps)
    _, psiL = ed_groundstate(L, h; pbc=true)
    _, psiH = ed_groundstate(L ÷ 2, h; pbc=true)
    abs4_L = pauli_abs4_table(psiL, L)
    abs4_H = pauli_abs4_table(psiH, L ÷ 2)
    for split_mode in (:contiguous, :interleaved)
        for row in exact_sector_stats(abs4_L, abs4_H, L; split_mode=split_mode)
            @printf("  exact %-11s %-5s support=%6d c_L=%+.8f mean_R=%.8g sd_R=%.6g iid_se_cL@1e6=%.6f max_R=%.6g\n",
                    string(split_mode), string(row.sector), row.support, row.cL, row.mean_R,
                    row.sd_R, row.iid_se_cL_1e6, row.max_R)
            @printf("          log-estimator check: -E[log R]=%+.8f iid_se@1e6=%.6f\n",
                    row.neg_mean_logR, row.iid_se_neg_mean_logR_1e6)
        end
    end
    dual = exact_dual_stats(abs4_L, abs4_H, L)
    @printf("  dual half-product support=%6d c_L=%+.8f mean_R=%.8g sd_R=%.6g iid_se_cL@1e6=%.6f max_R=%.6g\n",
            dual.support, dual.cL, dual.mean_R, dual.sd_R,
            dual.iid_se_cL_1e6, dual.max_R)
    for row in tolerance_sweep(abs4_L, abs4_H, L)
        @printf("  tol 1e%+d support=%6d c_L=%+.8f iid_se_cL@1e6=%.6f dropped_w=%.3e dropped_num=%.3e\n",
                row.exponent, row.support, row.cL, row.iid_se_cL_1e6,
                row.dropped_weight, row.dropped_num)
    end
    for reject in (false, true)
        r = chain_sector(abs4_L, abs4_H, L; n_steps=n_steps, reject_odd_y=reject)
        @printf("  chain paper reject_odd_y=%5s c_L=%+.8f mean_R=%.8g accept=%.4f recorded=%d\n",
                string(reject), r.cL, r.mean_R, r.accept, r.n_recorded)
    end
    r = chain_sector(abs4_L, abs4_H, L; n_steps=n_steps, reject_odd_y=true, proposal=:group)
    @printf("  chain group reject_odd_y= true c_L=%+.8f mean_R=%.8g accept=%.4f recorded=%d\n",
            r.cL, r.mean_R, r.accept, r.n_recorded)
end

main()
