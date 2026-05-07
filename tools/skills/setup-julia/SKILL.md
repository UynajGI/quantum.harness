---
name: setup-julia
description: Use when a workflow needs Julia installed and configured — fresh laptop, fresh cluster account, package-mirror change, or Julia-version bump. Generic over target (local laptop or remote cluster via ssh) and over region (mirror auto-defaulted from cluster profile's `region` field). Pairs with `/slurm` (cluster Julia setup) and with `make install julia` (local).
---

# setup-julia

Install and configure Julia for the harness — install via juliaup, configure the package mirror (defaults to Chinese mirror if `region == mainland_china` in the cluster profile), and instantiate the project env (`julia-env/`). Generic over target (local or remote ssh alias). Idempotent — re-running is safe and quick.

This skill is for *Julia-specific configuration*. Cluster-side conventions (ssh, scheduler, partitions) live in `tools/cluster/<active>.md` and are read here for `region` defaults and (when target is remote) the ssh alias and `repo_path_remote`.

## When to activate

- A workflow about to run Julia code finds `julia` not installed.
- A fresh laptop / fresh cluster account needs the harness's Julia stack.
- Package mirror changes (e.g., the user moves region or the institutional mirror updates).
- Bumping Julia version (`--version 1.11.x`).
- `/slurm`'s pre-submit check sees `julia-env/Manifest.toml` not yet instantiated on the cluster.

## Inputs

- `--target {local | remote:<alias>}` — where to install/configure. Default: `local`. For remote, `<alias>` matches `tools/cluster/<active>.md`'s `ssh.alias` field; the skill reads `repo_path_remote` for where the project lives.
- `--mirror <url>` (optional) — package server URL. If unset, defaults from `region`:
  - `region: mainland_china` → `https://mirrors.tuna.tsinghua.edu.cn/julia` (Tsinghua mirror; Jinguo-group recommended).
  - Other / unset → Julia's default (`https://pkg.julialang.org`).
  - `--mirror none` to skip mirror config entirely.
- `--version <X.Y.Z>` (optional) — Julia version. Default: `release` (juliaup's stable channel). For HPC2 module, this is overridden by the `module load julia/<version>` declaration in the cluster's `bootstrap_one_time` snippet.
- `--instantiate / --no-instantiate` — whether to run `Pkg.instantiate()` after install. Default `--instantiate` (we want the env ready for actual work).

## Workflow

1. **Probe target**: detect if `julia` is reachable (PATH, `module load`, or `~/.juliaup/bin/julia`). For remote: `ssh <alias> 'command -v julia || command -v juliaup'`.
2. **Install Julia (if missing)**:
   - Local: invoke `tools/cli/setup-julia.sh install [--version X.Y.Z]` — runs juliaup (curl-installer if juliaup itself is missing), adds the requested channel, sets default.
   - Remote: same script, dispatched via `ssh <alias>`. If the cluster profile's `bootstrap_one_time` declares a module-loaded Julia (`module load julia/X.Y.Z`), prefer that over juliaup — module-provided Julia is usually what cluster admins expect users to use.
3. **Configure mirror** (per `--mirror` resolution above): write/update `~/.julia/config/startup.jl` with:
   ```julia
   ENV["JULIA_PKG_SERVER"] = "<mirror_url>"
   ```
   Idempotent: replace any existing `JULIA_PKG_SERVER` line, leave the rest of `startup.jl` alone.
4. **Instantiate project env**: in the harness checkout, run `julia --project=julia-env -e 'using Pkg; Pkg.instantiate(); Pkg.precompile()'`. For remote, ssh-execute in `<repo_path_remote>`.
5. **Verify**: run a tiny smoke (`julia --project=julia-env -e 'using ITensors; println("ok")'`); surface failure.
6. **Hand back**: 2-3 line summary (Julia version, mirror url, project env state).

## Output

- For local: Julia installed and on `$PATH`; `~/.julia/config/startup.jl` configured; `julia-env/` instantiated and precompiled.
- For remote: same, on `<alias>:<repo_path_remote>`.
- A 2-3 line report.

## Composition

- Called by `/onboard` if the user signals they will write Julia code (e.g., DMRG / ITensors workflows).
- Called by `/slurm` pre-submit when the remote cluster's `julia-env/Manifest.toml` hasn't been instantiated yet.
- Called by `make install julia` and `make install itensors` recipes (the makefile recipes can dispatch the skill via `${CLAUDE_SKILL_DIR}/setup-julia/...` once registered).
- Pairs with `tools/cluster/<active>.md` for the `region` default mirror and (for remote) the ssh alias.

## Mirror-config note (mainland China)

For users in mainland China, package downloads from `pkg.julialang.org` are typically slow or unreliable. The Jinguo-group setup guide (https://scfp.jinguo-group.science/chap1-julia/julia-setup.html) recommends the Tsinghua mirror:

```
ENV["JULIA_PKG_SERVER"] = "https://mirrors.tuna.tsinghua.edu.cn/julia"
```

This skill applies that automatically when the cluster profile's `region` field is `mainland_china`. For other regions or institutional mirrors, pass `--mirror <url>` explicitly.

## Notes

- The skill does NOT manage `Manifest.toml` content — that's per-project. It only runs `Pkg.instantiate()` to materialize whatever the project's `Project.toml` + `Manifest.toml` declare.
- For the harness specifically, `julia-env/Project.toml` and `julia-env/Manifest.toml` are committed (see repo `.gitignore`); `Pkg.instantiate()` reproduces the locked environment exactly.
- Module-loaded Julia (e.g., `module load julia/1.10.9` on HPC2) is preferred on shared clusters — admins usually optimize the module for the cluster's filesystem and BLAS. juliaup-installed Julia is the fallback when no module is provided.
- This skill does NOT do cluster-level setup (ssh keys, scheduler config, account setup). That's `/onboard`'s cluster-setup stage and the cluster profile's `bootstrap_one_time`.

## Anti-patterns (auto-reject)

- Hardcoding a Chinese mirror for every install — only when `region == mainland_china` (or `--mirror` explicit).
- Reinstalling Julia when it's already on `$PATH` and at the requested version — be idempotent.
- Editing `~/.julia/config/startup.jl` destructively — preserve unrelated lines, replace only the `JULIA_PKG_SERVER` line.
- Bundling cluster bootstrap (clone repo, module load, etc.) into this skill — that's `/onboard` + `bootstrap_one_time`.
