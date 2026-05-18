using Test
using TOML

const REPO_ROOT = normpath(joinpath(@__DIR__, "..", ".."))
const PROTOCOL_TEMPLATE = joinpath(REPO_ROOT, "tools", "templates", "reproduce-paper", "protocol.toml")
const FLOW_TEMPLATE = joinpath(REPO_ROOT, "tools", "flow", "templates", "reproduce-paper.toml")

function gate_ids(template)
    parsed = TOML.parsefile(template)
    return Set(string(gate["id"]) for gate in get(parsed, "gates", Any[]))
end

@testset "reproduce-paper protocol template gates match flow gates" begin
    allowed = gate_ids(FLOW_TEMPLATE)
    protocol = TOML.parsefile(PROTOCOL_TEMPLATE)
    used = [string(check["gate"]) for check in get(protocol, "checks", Any[]) if haskey(check, "gate")]
    invalid = sort(setdiff(Set(used), allowed) |> collect)

    @test !isempty(allowed)
    @test !isempty(used)
    @test isempty(invalid)
end

@testset "reproduce-paper protocol template source authority is explicit" begin
    protocol = TOML.parsefile(PROTOCOL_TEMPLATE)
    allowed_authorities = Set(["primary", "trusted_reference", "hint"])

    for source in get(protocol, "sources", Any[])
        @test haskey(source, "id")
        @test haskey(source, "kind")
        @test haskey(source, "authority")
        @test string(source["authority"]) in allowed_authorities
    end
end
