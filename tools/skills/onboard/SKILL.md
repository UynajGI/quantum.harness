---
name: onboard
description: Use when the user is new to the harness, asks "where do I start", or opens with an unclear / empty problem. Sets up domain software, optionally configures the user's compute cluster, and routes to the right problem skill.
---

# Onboard

First-touch intake. Set up the domain environment, optionally configure the user's compute cluster, then get the user onto a real problem fast.

## When to activate

- "I'm new here" / "where do I start" / "how do I use this".
- Empty or unclear opening.
- User explicitly invokes `/onboard`.
- First session detected (no `julia-env/` directory).

## Workflow

### 1. Setup — do it, don't ask

Run `make setup && make domain-setup` silently. This installs Ion + skills + the local domain stack (currently Julia + ITensors). The Makefile's `DOMAIN_TOOLS` variable defines what gets installed — if the domain needs change, update the Makefile, not this skill.

Report one line:
- All good: "Domain stack ready."
- Something installed: "Installed [what]. Ready."
- Install failed: say what failed, offer to debug. Don't proceed until the stack works.

### 2. Cluster setup — warm gate, optional

Skip this stage if `tools/cluster/active.md` already exists (user has a profile from a prior session — idempotent).

Otherwise, ask one warm gate via `AskUserQuestion`:

> *"Will you run paper-grade calculations on a remote cluster (SLURM, PBS, plain ssh, ...)? If yes, I'll wire it into the harness now — ship/submit/monitor/fetch all happen from this session, no manual ssh relay. You can also skip and configure later when you actually need it."*

Options:
- "Yes, set it up now (Recommended if you have cluster docs handy)"
- "Skip for now — local-only is fine"

If the user picks "skip", continue to step 3. If "yes", continue inside this stage:

#### 2a. Path to profile

> *"What's the easiest way for you to share your cluster's setup? Either paste your docs URL — I'll pull out the partitions, walltime caps, and how Julia is provided — or run through 4 quick questions. Either way takes about a minute."*

Options:
- Text field for docs URL (the user pastes; the skill `WebFetch`'s it)
- "Walk me through the 4 questions"

#### 2b. From URL — fetch + extract + ratify

`WebFetch` the URL with a prompt that asks for: scheduler type, partitions table (name / class / cores / memory / max_wall / GPU), default queue, filesystem layout, internet reach, region. Propose a `tools/cluster/<short-name>.md` populated per the schema in `tools/cluster/README.md`. Show the user the proposed profile and ratify before write.

If parsing fails twice (e.g., docs page is JS-only or the structure doesn't match), fall through to **2c** (questions).

#### 2c. Walk-through fallback (≤4 questions, each warm)

Pre-amble:
> *"OK, let's walk through 4 things — each one fills in a field of your cluster profile."*

1. *"What's the ssh alias you use to reach the cluster login node? (whatever's in your `~/.ssh/config`)."*
2. AskUserQuestion: *"Which workload manager does the cluster use?"* with options: `Slurm` / `PBS / Torque` / `LSF` / `Plain ssh, no scheduler` / `Not sure — I'll probe`.
3. *"What's your default queue or partition? You can override per job — this is just where jobs go if nothing else is specified."*
4. AskUserQuestion: *"Which region is the cluster in?"* with options: `Mainland China (mirrors will be set up downstream)` / `Outside mainland China (default mirrors)` / `Air-gapped / no internet from login` / `Not sure`.

Write the profile to `tools/cluster/<short-name>.md`, symlink `tools/cluster/active.md → <short-name>.md`. Confirm one line: *"Cluster profile saved at `tools/cluster/<name>.md`. Future jobs will use it automatically."*

Do NOT bootstrap Julia or instantiate environments here — that's `/setup-julia`'s job, dispatched on demand by `/slurm` when the first cluster Julia run happens.

### 3. Problem intake — one question

> *"What problem are you trying to solve?"*

That's it. Don't list models. Don't explain the architecture.

### 4. Route

Infer the model or physics topic from the answer. Hand off to the matched skill. This skill exits.

If ambiguous, use `AskUserQuestion` with 2–3 candidate skills — short labels, one-line tradeoff each, recommended first. Don't list all 13.

If nothing fits: *"That's outside current scope (ground-state lattice problems). Want me to try an off-skill approach, or help you reframe?"*

## What this skill does NOT do

- Lecture about the harness.
- Walk through a tutorial.
- Ask the user to read docs.
- Show a menu of 13 skills.
- Hardcode which software to install (that's the Makefile's job).
- Bootstrap Julia on the cluster (that's `/setup-julia`, dispatched by `/slurm` on first cluster Julia run).
- Pile questions on the user — every gate is one question with a clear *why* and an escape hatch.

## UX rule (applies to every gate in this skill)

Each user-facing question follows the pattern: *frame the why → state the consequence → offer the escape hatch → ask*. No question stands alone without context. Telegraphic prompts ("Cluster?", "URL?") are rude even when short. Warm-clear-concise.

One short setup → one optional cluster gate → one problem question → route. Then exit.
