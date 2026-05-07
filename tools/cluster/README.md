# Cluster Profiles

Per-cluster profile cards describing **generic remote-execution conventions** — ssh access, scheduler, partitions, filesystem, internet reach, region — needed to drive `/slurm` (and any other cluster-aware skill) without hard-coding cluster specifics into harness skills. Language-specific setup (Julia, Python, R, …) is *not* in this schema; that's `/setup-julia`, `/setup-python`, etc., which read this profile and apply language-specific recipes downstream.

The harness skills are *cluster-agnostic*: they read this folder when they need to pick a partition, a time limit, a sbatch idiom, or a status command. The HPC2-specific facts live in `hpc2.md`; another cluster (HPC3, internal-lab, AWS Slurm, …) lands its own card here. Skills consult the active profile via either:

- environment variable `HARNESS_CLUSTER_PROFILE=<name>` (e.g., `HARNESS_CLUSTER_PROFILE=hpc2`);
- symlink `tools/cluster/active.md → tools/cluster/<name>.md` (preferred when the user wants the choice persisted across sessions).

A skill that needs cluster information reads `active.md` (or the env-var-picked file) and falls back to a built-in minimal-Slurm default if neither is present.

## Profile schema (every card has these sections)

Each card is markdown; skills parse the headers and tables, not free-form prose. The required sections are:

1. **Identity** — cluster name, purpose, who maintains it. One line.
2. **Connection** — `ssh.alias` (the entry in your `~/.ssh/config`), `repo_path_remote` (where the harness checkout lives on the cluster).
3. **Scheduler** — `scheduler.type` (`slurm` / `pbs` / `lsf` / `none`), default queue / partition for the user.
4. **Partitions** — table of `(name, class, cores, memory, max_wall, GPU)`. The `class` column tags each row (`default-cpu`, `gpu`, `high-mem`, `debug`, `long`, `emergency`); skills pick the row by class, not by name.
5. **Filesystem** — `home`, `scratch` (or absent), project paths; quotas; whether `/scratch` exists.
6. **Network** — `internet.from_login` (yes / no — controls whether `git clone <url>` works on the login node), `internet.from_compute` (whether package install works during a job).
7. **Region** — for downstream language skills to default mirrors (e.g., `mainland_china` → use a Chinese mirror for Julia / pip / conda). Values: `mainland_china`, `none` / blank.
8. **Documentation** — `docs_url` (the cluster's official docs page; cluster-aware skills can re-fetch when the profile drifts).
9. **Bootstrap one-time** — shell snippet describing cluster-specific quirks (group permissions, depot path, license server activation). Run once per fresh checkout. Idempotent.
10. **Sbatch idioms** — single-cell and array-job templates; cluster-specific quirks (array-id env var name, output-file naming).
11. **Status / queue commands** — `squeue` / `sacct` flavor, partition-listing command.
12. **Notes** — anything else (group ownership, network egress, GPU exclusivity rules, etc.).

The schema is *additive*: new fields land as new sections; skills that don't read them ignore them. Language-specific tooling (`julia.provider`, `julia.mirror_url`, `python.distribution`, …) does **not** belong here — those are downstream concerns handled by `/setup-julia`, `/setup-python`, etc., which read this profile's `region` / `internet` / `docs_url` fields and apply their own recipes.

## Picking the active profile

| Situation | Recommended action |
|---|---|
| Single-cluster user with one persistent setup | `ln -s hpc2.md tools/cluster/active.md` once; harness reads it forever after. |
| Multi-cluster user (lab + remote HPC) | Set `HARNESS_CLUSTER_PROFILE=<name>` per-shell or in the project `.envrc`; the env var wins over the symlink. |
| First-time user with no profile yet | The `/onboard` skill's cluster-setup stage walks through profile creation — fetch the cluster's docs URL, extract, ratify. Falls back to ≤4 questions if URL fetch fails. |
| Profile contains secrets | Add the file to `.gitignore` locally; commit only public profiles. HPC2's profile here is public — no secrets. |

## Using a profile from a skill

Cluster-aware skills (currently `/slurm`) read the active profile and consult these sections in order:

1. Connection (`ssh.alias`, `repo_path_remote`) — to ssh / rsync.
2. Scheduler type and default partition — to construct submission.
3. Partitions table — to pick by class (`default-cpu`, `gpu`, `high-mem`).
4. Network reach — to decide ship strategy (git vs rsync vs pre-staged tarball).
5. Bootstrap one-time — to set up the cluster on first use.
6. Sbatch idiom — to construct the per-cell script.
7. Status commands — to poll for job state.

The skill *does not* hard-code anything about HPC2 or any other cluster; it only reads the active profile. If the profile is missing a field the skill needs, the skill emits a minimal default and surfaces a one-line note.

## Authoring a new profile

The recommended path is `/onboard`'s cluster-setup stage — fetch the cluster's docs URL, extract, ratify. For manual authoring:

1. Probe the cluster (run `sinfo`, `squeue`, `scontrol show partition`, `sacctmgr show accounts`, etc.) to fill in the schema.
2. Write `tools/cluster/<name>.md` following the section order above.
3. Test by activating it (`ln -s <name>.md active.md` or set the env var) and running `/slurm` on a tiny job.
4. If the profile is general-interest, commit it; if it contains secrets, add to `.gitignore` and commit only the schema-shaped sections.

## Cards in this folder

- `hpc2.md` — HPC2 cluster (HKUST(GZ)). Slurm, default-CPU partition `i64m512u`, `julia/1.10.9` available via module, `mainland_china` region (mirror config recommended for downstream `/setup-julia`). Public — committed.
