---
name: setup-julia
description: Use when a workflow needs Julia installed and configured — fresh laptop, fresh cluster account, package-mirror change, or Julia-version bump. Generic over target (local laptop or remote cluster via ssh) and over region (mirror auto-defaulted from cluster profile's `region` field). Pairs with `/slurm` (cluster Julia setup) and with `make install julia` (local).
---

# setup-julia

Install and configure Julia for the harness — install via juliaup, configure the package mirror (defaults to Chinese mirror if `region == mainland_china` in the cluster profile), and instantiate the project env (`julia-env/`). Generic over target (local or remote ssh alias). Idempotent — re-running is safe and quick.

This skill is for *Julia-specific configuration*. Cluster-side conventions (ssh, scheduler, partitions) live in `tools/cluster/<active>.md` and are read here for `region` defaults and (when target is remote) the ssh alias and `repo_path_remote`.

Julia package stacks are declared separately in `tools/software/stacks/*.toml`.
This skill only makes Julia itself and `julia-env/` usable locally or remotely;
after that, the selected stack's install command (for example `make install itensors`,
`make install sse`, or `make install pepskit`) installs method packages.

## When to activate

- A workflow about to run Julia code finds `julia` not installed.
- A fresh laptop / fresh cluster account needs the harness's Julia stack.
- Package mirror changes (e.g., the user moves region or the institutional mirror updates).
- Bumping Julia version (`--version 1.11.x`).
- `/slurm`'s pre-submit check sees `julia-env/Manifest.toml` not yet instantiated on the cluster.

## Inputs

- `--target {local | remote:<alias>}` — where to install/configure. Default: `local`. For remote, `<alias>` matches `tools/cluster/<active>.md`'s `ssh.alias` field; the skill reads `repo_path_remote` for where the project lives.
- `--region <name>` (optional) — mirror selection. Defaults from cluster profile's `region` field. Recognized:
  - `mainland_china` → Nanjing University (per Jinguo guide):
    - `JULIAUP_SERVER = https://mirror.nju.edu.cn/julia-releases/`
    - `JULIA_PKG_SERVER = https://mirrors.nju.edu.cn/julia`
  - other / unset → Julia defaults (no mirror config).
- `--mirror <url>` (optional, advanced) — override `JULIA_PKG_SERVER` directly with a custom URL.
- `--version <X.Y.Z>` (optional) — Julia version. Default: `release` (juliaup's stable channel).
- `--instantiate / --no-instantiate` — whether to run `Pkg.instantiate()` after install. Default `--instantiate`.

## Workflow

This skill follows the Jinguo-group recipe verbatim (https://book.jinguo-group.science/stable/chap2/julia-setup/). **Mirror first, then install** — juliaup itself respects `JULIAUP_SERVER` if exported before the curl installer runs.

1. **Probe target**: detect if `julia` is reachable (PATH, `module load`, or `~/.juliaup/bin/julia`). For remote: `ssh <alias> 'command -v julia || command -v juliaup'`. Also read the manifest's `julia_version` to verify version compatibility against any pre-existing Julia.
2. **Step 2 of guide — configure mirror first** (only if `--region` resolves to a mirror): `tools/cli/setup-julia.sh mirror --region <name>`. This writes BOTH `JULIAUP_SERVER` and `JULIA_PKG_SERVER` env vars to the user's shell rc (`~/.bashrc` / `~/.zshrc` / `~/.profile`) AND `JULIA_PKG_SERVER` to `~/.julia/config/startup.jl` for julia subprocesses. Idempotent — replaces any prior values.
3. **Step 1 of guide — install Julia (if missing)**: `tools/cli/setup-julia.sh install [--region <name>] [--version X.Y.Z]`. The script:
   - exports `JULIAUP_SERVER` for the current shell (so the curl installer downloads through the mirror);
   - runs `curl -fsSL https://install.julialang.org | sh` to install juliaup;
   - writes the guide's Revise try/catch snippet to `~/.julia/config/startup.jl`;
   - `Pkg.add("Revise")` into the default env.
   Skip step 3 if Julia is already present and the version is compatible.
4. **Instantiate project env**: in the harness checkout, run `tools/cli/setup-julia.sh instantiate <project_dir>` → `Pkg.instantiate(); Pkg.precompile()`. For remote, ssh-execute in `<repo_path_remote>`.
5. **Verify**: `tools/cli/setup-julia.sh verify <project_dir> <package_name>` — exit 0 on success.
6. **Hand back**: 2-3 line summary (Julia version, mirror url, project env state).

## Output

- For local: Julia installed and on `$PATH`; `~/.julia/config/startup.jl` configured; `julia-env/` instantiated and precompiled.
- For remote: same, on `<alias>:<repo_path_remote>`.
- A 2-3 line report.

## Composition

- Called by `/onboard` if the user signals they will write Julia code (e.g., DMRG / ITensors workflows).
- Called by `/slurm` pre-submit when the remote cluster's `julia-env/Manifest.toml` hasn't been instantiated yet.
- Called by `make install julia` and `make install itensors` recipes (the makefile recipes can dispatch the skill via `${CLAUDE_SKILL_DIR}/setup-julia/...` once registered).
- Precedes any `language = "julia"` stack contract in `tools/software/stacks/*.toml`.
- Pairs with `tools/cluster/<active>.md` for the `region` default mirror and (for remote) the ssh alias.

## Mirror-config note (mainland China — Jinguo-group recipe)

For users in mainland China, package downloads from `pkg.julialang.org` are typically slow or unreliable. The Jinguo-group setup guide (https://book.jinguo-group.science/stable/chap2/julia-setup/) configures **two mirror env vars**, both pointing at Nanjing University:

```bash
export JULIAUP_SERVER=https://mirror.nju.edu.cn/julia-releases/    # for the juliaup binary download
export JULIA_PKG_SERVER=https://mirrors.nju.edu.cn/julia           # for Pkg downloads
```

`JULIAUP_SERVER` matters for the *initial Julia install* — without it, juliaup downloads from AWS S3 (slow from mainland China). `JULIA_PKG_SERVER` matters for every subsequent `Pkg.add` / `Pkg.instantiate`. The mirror MUST be set before invoking the curl installer; this skill orders Step 2 before Step 1 when `--region mainland_china`.

This skill applies the NJU mirror automatically when the cluster profile's `region` field is `mainland_china`. For other regions, no mirror is set. For an institutional mirror, pass `--mirror <url>` to override `JULIA_PKG_SERVER`.

## Notes

- **Project.toml only** is the harness rule. `julia-env/Project.toml` is committed (the dependency contract); `julia-env/Manifest.toml` is gitignored and re-generated per machine on first `Pkg.resolve()` + `Pkg.instantiate()`. Manifests are pinned to a specific `julia_version` and don't port across Julia major versions, so committing one would break cross-machine setup. Trade-off: exact dependency-version reproducibility is sacrificed for cross-platform/version portability — acceptable for a research harness running across laptop + cluster + colleagues' machines.
- Module-loaded Julia (e.g., `module load julia/1.10.9`) and juliaup-installed Julia are both fine. Pick whichever the cluster admin prefers; the skill probes both. For Manifest-portability across the local + cluster setup, prefer juliaup so the Julia version is user-controlled (rather than admin-chosen) — easier to keep aligned.
- This skill does NOT do cluster-level setup (ssh keys, scheduler config, account setup). That's `/onboard`'s cluster-setup stage and the cluster profile's `bootstrap_one_time`.

## Anti-patterns (auto-reject)

- Hardcoding a Chinese mirror for every install — only when `region == mainland_china` (or `--mirror` explicit).
- Reinstalling Julia when it's already on `$PATH` and at the requested version — be idempotent.
- Editing `~/.julia/config/startup.jl` destructively — preserve unrelated lines, replace only the `JULIA_PKG_SERVER` line.
- Bundling cluster bootstrap (clone repo, module load, etc.) into this skill — that's `/onboard` + `bootstrap_one_time`.
