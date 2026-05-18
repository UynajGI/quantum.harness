using Test
using TOML

const REPO_ROOT = normpath(joinpath(@__DIR__, "..", ".."))
const PROTOCOL_TEMPLATE = joinpath(REPO_ROOT, "tools", "templates", "reproduce-paper", "protocol.toml")
const FLOW_TEMPLATE = joinpath(REPO_ROOT, "tools", "flow", "templates", "reproduce-paper.toml")

@testset "protocol checks reference real flow gates" begin
    allowed = Set(string(g["id"]) for g in get(TOML.parsefile(FLOW_TEMPLATE), "gates", Any[]))
    protocol = TOML.parsefile(PROTOCOL_TEMPLATE)
    used = [string(check["gate"]) for check in get(protocol, "checks", Any[]) if haskey(check, "gate")]
    invalid = sort(setdiff(Set(used), allowed) |> collect)

    # The two non-emptiness asserts guard against a vacuous pass when either
    # template happens to declare zero gates / zero checks.
    @test !isempty(allowed)
    @test !isempty(used)
    @test isempty(invalid)
end

@testset "protocol source authority is explicit" begin
    protocol = TOML.parsefile(PROTOCOL_TEMPLATE)
    allowed_authorities = Set(["primary", "trusted_reference", "hint"])

    for source in get(protocol, "sources", Any[])
        @test haskey(source, "id")
        @test haskey(source, "kind")
        @test haskey(source, "authority")
        @test string(source["authority"]) in allowed_authorities
    end
end
