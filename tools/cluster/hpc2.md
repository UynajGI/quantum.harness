# HPC2 Cluster Profile

## Identity

HPC2 — HKUST(GZ) institutional Slurm cluster. CPU-heavy stack with a small GPU annex. Used by the harness for `(L, parameter)` grid sweeps in DMRG / TTN / Pauli-Markov workflows. Cluster-agnostic: the harness reads this card; nothing HPC2-specific is hard-coded into skills.

## Connection

- `ssh.alias` = `hpc2` (entry in `~/.ssh/config`).
- `repo_path_remote` = `~/harness-qmb` (where the harness checkout lives on the cluster login node).

## Scheduler

- `scheduler.type` = `slurm`.
- `scheduler.default_queue` = `i64m512u` (the row tagged `default-cpu` below).
- `scheduler.default_account` = (not required — Slurm picks the user's default automatically; do **not** emit `#SBATCH --account=`).

## Partitions

| Name | Class | Cores | Memory | Max wall | GPU | Notes |
|---|---|---|---|---|---|---|
| `i64m512u` | **default-cpu** | 64 | 512 GB | 7 days | — | Standard CPU partition; harness default. 1024-core partition cap. |
| `i64m512ue` | cpu-exclusive | 64 | 512 GB | 7 days | — | "Exclusive (Medium)" priority, 1024-core cap. |
| `i64m512r` | cpu-storage | 64 | 512 GB | 7 days | — | 6×1.92 TB local storage; 128-core cap. |
| `a128m512u` | high-core | 128 | 512 GB | 7 days | — | AMD EPYC 7763; 256-core cap. |
| `i96m3tu` | high-mem | 192 | 3 TB | 7 days | — | For large-MPS / large-`χ` workloads needing >512 GB. |
| `debug` | debug | varies | varies | 30 min | — | Quick test runs; auto-routed for `< 30 min` jobs. |
| `long_cpu` | long | varies | varies | 14 days | — | Multi-week sweeps. |
| `emergency_cpu` | emergency | varies | varies | varies | — | High-priority, 512-core cap. |
| `i64m1tga800u` | gpu | 64 | 1 TB | 7 days | A800 ×8 | 128-core / 16-GPU cap. |
| `i64m1tga40u` | gpu | 64 | 1 TB | 7 days | A40 ×8 | NVIDIA A40 alternative. |

`/slurm` picks a row by class (`default-cpu`, `gpu`, `high-mem`, …) per the calling skill's resource-class hint. `i64m512u` is the harness default for any CPU job.

## Filesystem

- `home` = `/hpc2hdd/home/<user>`. Standard read/write. The user's checkout of the harness lives here.
- `scratch` = (none) — HPC2 has *no* `/scratch` partition; all workloads run out of `$HOME`. Skills must not assume a scratch directory exists. Watch home-quota for multi-GB tensor / Markov-chain outputs (`du -sh results/<run>`).
- `project_group` = `jinguoliu_team` — files created by harness skills inherit this group; no extra `chgrp` needed.

## Network

- `internet.from_login` = `yes` — `git clone <github-url>`, `arxiv-search`, `download-ref` all work from the login node.
- `internet.from_compute` = `partial` — compute nodes may not see public DNS for some hosts; pre-stage references on the login node before submitting if a job needs them.

## Region

- `region` = `mainland_china` — downstream language skills (`/setup-julia`, etc.) should default to a Chinese mirror for package downloads. The Jinguo-group setup guide (https://scfp.jinguo-group.science/chap1-julia/julia-setup.html) documents the recommended Julia mirror.

## Documentation

- `docs_url` = `https://docs.hpc.hkust-gz.edu.cn/en/docs/hpc12/slurm/queue` — HPC2 (HPC12) official Slurm queue docs. Re-fetch when the profile drifts.

## Bootstrap one-time

Run once per fresh user account / fresh checkout. Idempotent.

```bash
# 1. Repo (skip if already cloned)
git clone git@github.com:fliingelephant/harness-qmb.git ~/harness-qmb 2>/dev/null || true
cd ~/harness-qmb && git fetch origin

# 2. Julia (module is the harness default on HPC2; juliaup is also installed at ~/.juliaup/bin/julia)
module load julia/1.10.9

# 3. Project env (Pkg.instantiate from committed Project.toml + Manifest.toml)
julia --project=julia-env -e 'using Pkg; Pkg.instantiate(); Pkg.precompile()'

# 4. (Optional) Mirror config for mainland China — see /setup-julia for the canonical recipe.
```

`/setup-julia` reads this `bootstrap_one_time` snippet (and the `region` field above) when invoked with `--target remote:hpc2`.

## Sbatch idioms

### Single-cell job

```bash
#!/bin/bash
#SBATCH --job-name=<name>
#SBATCH --partition=i64m512u
#SBATCH --time=1-00:00:00
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --output=results/<run>/cells/<cell_id>/slurm-%j.out

module load julia/1.10.9
cd $SLURM_SUBMIT_DIR
julia --project=julia-env <script>.jl
```

`--cpus-per-task=8` lets ITensors's OpenBLAS use 8 threads for DMRG and MPS contractions. Drop to 1 for pure-Markov-chain workloads that don't benefit from BLAS threading.

### Array job (parameter grid)

```bash
#!/bin/bash
#SBATCH --job-name=<name>-grid
#SBATCH --partition=i64m512u
#SBATCH --time=1-00:00:00
#SBATCH --array=1-<N_cells>
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --output=results/<run>/cells/cell-%a/slurm-%A_%a.out

module load julia/1.10.9
cd $SLURM_SUBMIT_DIR

CELL_ID=$(printf "%03d" $SLURM_ARRAY_TASK_ID)
julia --project=julia-env scripts/<per-cell-script>.jl
```

Array idiom: `#SBATCH --array=N-M` with `$SLURM_ARRAY_TASK_ID` as the cell index. The `/slurm` skill emits exactly this shape.

## Status / queue commands

| Purpose | Command |
|---|---|
| List my queued / running jobs | `squeue -u $USER` |
| Show partition state and capacity | `spartition` (HPC2-local alias for `sinfo` summary) |
| Show one job's full state | `scontrol show job <jobid>` |
| Accounting / completed jobs | `sacct -u $USER --starttime=now-1day --format=JobID,State,ExitCode,MaxRSS,Elapsed` |
| Show partition definitions | `scontrol show partition` |

## Notes

- HPC2-specific facts (partition list, `spartition` alias, filesystem layout) live ONLY in this card. Skills consult `tools/cluster/active.md` (or `HARNESS_CLUSTER_PROFILE`) and never bake HPC2 specifics into their workflow text.
- Default `--time=1-00:00:00` is conservative; bump per method-card runtime estimate. Long sweeps → `long_cpu` partition or split into chunks.
- Group ownership: `jinguoliu_team`. Files created by harness skills inherit this group; no extra `chgrp` needed.
- Modules verified available: `julia/1.10.9`, `julia/1.10.1`, `gcc-13.1.0`, `openmpi-4.1.6`, `mpich-4.3.2`. Use `openmpi` unless there's a reason for `mpich`.
