using ITensors, ITensorMPS

function itensors_mps_product_state_labels(product_state, L::Int)
    if product_state isa AbstractString
        return fill(string(product_state), L)
    elseif product_state isa AbstractVector
        length(product_state) == L ||
            error("Product-state label list has length $(length(product_state)); expected $L")
        return [string(x) for x in product_state]
    elseif product_state isa AbstractDict
        if haskey(product_state, "repeat")
            return fill(string(product_state["repeat"]), L)
        end
        states = product_state["states"]
        states isa AbstractVector || error("product_state.states must be a list")
        length(states) == L ||
            error("product_state.states has length $(length(states)); expected $L")
        return [string(x) for x in states]
    end
    error("Product state must be a string, label list, or object with repeat/states")
end

function itensors_mps_complex_coefficient(x)
    x isa Real && return ComplexF64(x)
    if x isa AbstractVector
        length(x) == 2 || error("Complex coefficient vector must be [real, imag]")
        return ComplexF64(Float64(x[1]), Float64(x[2]))
    end
    error("Coefficient must be real or [real, imag]")
end

function itensors_mps_initial_state(sites, spec=nothing; default_linkdims::Int=4)
    if spec === nothing
        return randomMPS(sites; linkdims=default_linkdims)
    end
    spec isa AbstractDict || error("Initial-state spec must be an object")
    isempty(spec) && error("Initial-state spec cannot be empty; omit it for a random MPS")

    if haskey(spec, "random")
        random_cfg = spec["random"]
        linkdims = random_cfg isa AbstractDict ? Int(get(random_cfg, "linkdims", default_linkdims)) : default_linkdims
        return randomMPS(sites; linkdims=linkdims)
    end

    if haskey(spec, "product_state")
        return productMPS(sites, itensors_mps_product_state_labels(spec["product_state"], length(sites)))
    end

    terms = spec["terms"]
    terms isa AbstractVector || error("Initial-state terms must be a list")
    isempty(terms) && error("Initial-state terms cannot be empty")
    psi = nothing
    for term in terms
        term isa AbstractDict || error("Each initial-state term must be an object")
        labels = itensors_mps_product_state_labels(term["product_state"], length(sites))
        term_mps = productMPS(sites, labels)
        coeff = itensors_mps_complex_coefficient(get(term, "coefficient", 1.0))
        term_mps[1] = coeff * term_mps[1]
        psi = psi === nothing ? term_mps : psi + term_mps
    end
    normalize!(psi)
    return psi
end
