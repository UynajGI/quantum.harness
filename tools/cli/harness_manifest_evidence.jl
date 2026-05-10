function harness_evidence_float(x, label::AbstractString)
    x isa Bool && error("Expected numeric value for $label, got boolean $(repr(x))")
    x isa Real && return Float64(x)
    return parse(Float64, string(x))
end

function harness_evidence_expected(item::AbstractDict)
    haskey(item, "expected_abs") && return (:expected_abs, harness_evidence_float(item["expected_abs"], "expected_abs"))
    haskey(item, "expected") && return (:expected, harness_evidence_float(item["expected"], "expected"))
    error("Evidence item '$(get(item, "id", "<unnamed>"))' must declare expected or expected_abs")
end

function harness_evidence_diff(item::AbstractDict)
    id = string(get(item, "id", "<unnamed>"))
    value = harness_evidence_float(item["value"], "$id.value")
    tolerance = harness_evidence_float(item["tolerance"], "$id.tolerance")
    isfinite(value) || error("Evidence item '$id' value is not finite")
    isfinite(tolerance) && tolerance >= 0 || error("Evidence item '$id' tolerance must be finite and non-negative")
    mode, expected = harness_evidence_expected(item)
    isfinite(expected) || error("Evidence item '$id' expected value is not finite")
    return mode == :expected_abs ? abs(abs(value) - expected) : abs(value - expected)
end

function harness_evidence_computed_status(item::AbstractDict)
    diff = harness_evidence_diff(item)
    tolerance = harness_evidence_float(item["tolerance"], "$(get(item, "id", "<unnamed>")).tolerance")
    return diff <= tolerance ? "pass" : "fail"
end

function harness_validate_evidence_item(item::AbstractDict)
    id = string(get(item, "id", "<unnamed>"))
    diff = harness_evidence_diff(item)
    tolerance = harness_evidence_float(item["tolerance"], "$id.tolerance")
    computed = diff <= tolerance ? "pass" : "fail"
    declared = lowercase(string(get(item, "status", "")))
    declared == computed ||
        error("Evidence item '$id' declares status='$declared' but computed status='$computed' (diff=$diff tolerance=$tolerance)")
    computed == "pass" ||
        error("Evidence item '$id' failed (diff=$diff tolerance=$tolerance)")
    return true
end

function harness_validate_evidence(evidence; required_ids=String[])
    evidence isa AbstractVector || error("Evidence must be a list")
    ids = Set{String}()
    for item in evidence
        item isa AbstractDict || error("Every evidence item must be an object")
        haskey(item, "id") || error("Every evidence item must have an id")
        id = string(item["id"])
        id in ids && error("Duplicate evidence id '$id'")
        push!(ids, id)
        harness_validate_evidence_item(item)
    end
    for id in required_ids
        string(id) in ids || error("Missing required evidence id '$id'")
    end
    return true
end

function harness_declaration_lookup(declarations)
    declarations isa AbstractVector || error("Evidence declarations must be a list")
    out = Dict{String,Any}()
    for declaration in declarations
        declaration isa AbstractDict || error("Every evidence declaration must be an object")
        haskey(declaration, "id") || error("Every evidence declaration must have an id")
        id = string(declaration["id"])
        haskey(out, id) && error("Duplicate evidence declaration id '$id'")
        out[id] = declaration
    end
    isempty(out) && error("Evidence declarations cannot be empty")
    return out
end

function harness_metadata_matches(evidence::AbstractDict, declaration::AbstractDict)
    id = string(declaration["id"])
    for key in keys(declaration)
        key_s = string(key)
        key_s in ("id", "expected", "expected_abs", "tolerance") && continue
        haskey(evidence, key_s) || error("Evidence item '$id' missing declared metadata '$key_s'")
        evidence[key_s] == declaration[key] ||
            error("Evidence item '$id' metadata '$key_s'=$(evidence[key_s]) does not match declaration $(declaration[key])")
    end
    return true
end

function harness_validate_evidence_against_declarations(evidence, declarations)
    declared = harness_declaration_lookup(declarations)
    evidence isa AbstractVector || error("Evidence must be a list")
    seen = Set{String}()
    for item in evidence
        item isa AbstractDict || error("Every evidence item must be an object")
        haskey(item, "id") || error("Every evidence item must have an id")
        id = string(item["id"])
        id in seen && error("Duplicate evidence id '$id'")
        push!(seen, id)
        haskey(declared, id) || error("Evidence item '$id' has no declaration")
        declaration = declared[id]
        harness_metadata_matches(item, declaration)
        for key in ("expected", "expected_abs", "tolerance")
            haskey(item, key) || continue
            haskey(declaration, key) ||
                error("Evidence item '$id' declares '$key' but the declaration does not")
            item[key] == declaration[key] ||
                error("Evidence item '$id' $key=$(item[key]) does not match declaration $(declaration[key])")
        end
        bound = Dict{String,Any}(string(k) => v for (k, v) in item)
        for key in ("expected", "expected_abs", "tolerance")
            if haskey(declaration, key)
                bound[key] = declaration[key]
            end
        end
        harness_validate_evidence_item(bound)
    end
    for id in keys(declared)
        id in seen || error("Missing required evidence id '$id'")
    end
    return true
end

function harness_manifest_path_tokens(field::AbstractString)
    isempty(field) && error("Manifest field path cannot be empty")
    return split(field, ".")
end

function harness_manifest_lookup(record, field::AbstractString)
    value = record
    for token in harness_manifest_path_tokens(field)
        if value isa AbstractDict && haskey(value, token)
            value = value[token]
        else
            return false, nothing
        end
    end
    return true, value
end

function harness_manifest_required(record, field::AbstractString, path::AbstractString)
    found, value = harness_manifest_lookup(record, field)
    found || error("Manifest missing required field '$field': $path")
    return value
end

function harness_manifest_nonempty_value(value)
    value === nothing && return false
    value isa AbstractString && return !isempty(strip(value))
    value isa AbstractVector && return !isempty(value)
    value isa AbstractDict && return !isempty(value)
    return true
end

function harness_manifest_contract_list(contract::AbstractDict, key::AbstractString)
    value = get(contract, key, Any[])
    value isa AbstractVector || error("Manifest contract '$key' must be a list")
    return value
end

function harness_manifest_numeric(record, field::AbstractString, path::AbstractString)
    value = harness_manifest_required(record, field, path)
    number = harness_evidence_float(value, field)
    isfinite(number) || error("Manifest numeric field '$field' is not finite: $path")
    return number
end

function harness_manifest_bound_value(record, bound::AbstractDict, key::AbstractString, path::AbstractString)
    field_key = "$(key)_field"
    if haskey(bound, field_key)
        return harness_manifest_numeric(record, string(bound[field_key]), path)
    end
    haskey(bound, key) || error("Manifest numeric bound '$(get(bound, "id", "<unnamed>"))' missing '$key' or '$field_key'")
    number = harness_evidence_float(bound[key], "$(get(bound, "id", "<unnamed>")).$key")
    isfinite(number) || error("Manifest numeric bound '$(get(bound, "id", "<unnamed>"))' has non-finite '$key'")
    return number
end

function harness_manifest_bound_applies(record, bound::AbstractDict)
    if haskey(bound, "when_present")
        found, value = harness_manifest_lookup(record, string(bound["when_present"]))
        return found && value !== nothing
    end
    return true
end

function harness_manifest_compare(lhs::Float64, op::AbstractString, rhs::Float64)
    op == "<=" && return lhs <= rhs
    op == ">=" && return lhs >= rhs
    op == "<" && return lhs < rhs
    op == ">" && return lhs > rhs
    op == "==" && return lhs == rhs
    error("Unsupported manifest numeric bound op '$op'")
end

function harness_validate_manifest_contract(record::AbstractDict, contract::AbstractDict; path::AbstractString="manifest")
    for field in harness_manifest_contract_list(contract, "required_fields")
        harness_manifest_required(record, string(field), path)
    end

    for field in harness_manifest_contract_list(contract, "nonempty_fields")
        value = harness_manifest_required(record, string(field), path)
        harness_manifest_nonempty_value(value) ||
            error("Manifest field '$(string(field))' is empty: $path")
    end

    for check in harness_manifest_contract_list(contract, "equals")
        check isa AbstractDict || error("Manifest contract equals entries must be objects")
        field = string(check["field"])
        expected = check["value"]
        actual = harness_manifest_required(record, field, path)
        actual == expected ||
            error("Manifest field '$field'=$(repr(actual)) does not equal $(repr(expected)): $path")
    end

    for check in harness_manifest_contract_list(contract, "list_contains")
        check isa AbstractDict || error("Manifest contract list_contains entries must be objects")
        field = string(check["field"])
        values = harness_manifest_required(record, field, path)
        values isa AbstractVector || error("Manifest field '$field' must be a list for list_contains: $path")
        wanted = check["value"]
        any(x -> x == wanted, values) ||
            error("Manifest field '$field' does not contain $(repr(wanted)): $path")
    end

    for field in harness_manifest_contract_list(contract, "numeric_fields")
        harness_manifest_numeric(record, string(field), path)
    end

    for field in harness_manifest_contract_list(contract, "optional_numeric_fields")
        found, value = harness_manifest_lookup(record, string(field))
        (!found || value === nothing) && continue
        number = harness_evidence_float(value, string(field))
        isfinite(number) || error("Manifest optional numeric field '$(string(field))' is not finite: $path")
    end

    for bound in harness_manifest_contract_list(contract, "numeric_bounds")
        bound isa AbstractDict || error("Manifest contract numeric_bounds entries must be objects")
        harness_manifest_bound_applies(record, bound) || continue
        id = string(get(bound, "id", "numeric_bound"))
        lhs = harness_manifest_bound_value(record, bound, "lhs", path)
        rhs = harness_manifest_bound_value(record, bound, "rhs", path)
        op = string(get(bound, "op", "<="))
        harness_manifest_compare(lhs, op, rhs) ||
            error("Manifest numeric bound '$id' failed: $lhs $op $rhs is false: $path")
    end

    for evidence_set in harness_manifest_contract_list(contract, "evidence_sets")
        evidence_set isa AbstractDict || error("Manifest contract evidence_sets entries must be objects")
        evidence_field = string(evidence_set["evidence_field"])
        declarations_field = string(evidence_set["declarations_field"])
        evidence_found, evidence = harness_manifest_lookup(record, evidence_field)
        declarations_found, declarations = harness_manifest_lookup(record, declarations_field)
        required = Bool(get(evidence_set, "required", true))
        if !required && (!evidence_found || !declarations_found ||
                         !harness_manifest_nonempty_value(evidence) ||
                         !harness_manifest_nonempty_value(declarations))
            continue
        end
        evidence_found || error("Manifest missing evidence field '$evidence_field': $path")
        declarations_found || error("Manifest missing evidence declarations field '$declarations_field': $path")
        harness_validate_evidence_against_declarations(evidence, declarations)
    end
    return true
end
