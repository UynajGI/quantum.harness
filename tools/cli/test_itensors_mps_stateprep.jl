using ITensors, ITensorMPS
using Test

include(joinpath(@__DIR__, "itensors_mps_stateprep.jl"))

const TEST_PAULI_MATRICES = [
    ComplexF64[1.0 0.0; 0.0 1.0],
    ComplexF64[0.0 1.0; 1.0 0.0],
    ComplexF64[0.0 -1.0im; 1.0im 0.0],
    ComplexF64[1.0 0.0; 0.0 -1.0],
]

function test_uniform_pauli_expectation(psi::MPS, sites, code::Int)
    Ppsi = copy(psi)
    for i in 1:length(psi)
        Op = ITensor(TEST_PAULI_MATRICES[code + 1], sites[i]', sites[i])
        T = Op * Ppsi[i]
        noprime!(T)
        Ppsi[i] = T
    end
    return real(inner(psi, Ppsi))
end

@testset "ITensors MPS state preparation" begin
    sites = siteinds("S=1/2", 6; conserve_qns=false)
    cat_spec = Dict(
        "terms" => Any[
            Dict("coefficient" => 1.0, "product_state" => Dict("repeat" => "X+")),
            Dict("coefficient" => 1.0, "product_state" => Dict("repeat" => "X-")),
        ],
    )

    psi = itensors_mps_initial_state(sites, cat_spec; default_linkdims=4)
    @test isapprox(test_uniform_pauli_expectation(psi, sites, 3), 1.0; atol=1e-12, rtol=1e-12)

    product_spec = Dict("product_state" => Dict("repeat" => "Z+"))
    product = itensors_mps_initial_state(sites, product_spec; default_linkdims=4)
    @test isapprox(test_uniform_pauli_expectation(product, sites, 3), 1.0; atol=1e-12, rtol=1e-12)

    random_spec = Dict("random" => Dict("linkdims" => 2))
    random = itensors_mps_initial_state(sites, random_spec; default_linkdims=4)
    @test length(random) == length(sites)
    @test_throws ErrorException itensors_mps_initial_state(sites, Dict{String,Any}(); default_linkdims=4)
end
